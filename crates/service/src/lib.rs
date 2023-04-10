mod account;
mod error;
mod persist;
mod schema;

use std::{net::SocketAddr, path::Path, sync::Arc};

use anyhow::Context as _;
#[cfg(feature = "graphiql")]
use async_graphql::http::GraphiQLSource;
use async_graphql::{http::ALL_WEBSOCKET_PROTOCOLS, Data, ResultExt as _};
use async_graphql_axum::{GraphQLProtocol, GraphQLRequest, GraphQLResponse, GraphQLWebSocket};
use axum::{
    extract::{FromRef, State, WebSocketUpgrade},
    headers::{authorization::Bearer, Authorization},
    response::{self, IntoResponse},
    routing::{get, post},
    Router, Server, TypedHeader,
};
use ring::signature::{self, KeyPair as _};
use thiserror::Error;
use tracing::Level;

use account::authenticate;
use error::ErrorResponse;
pub use schema::schema;
use schema::ServiceSchema;

pub struct ServeConfig {
    data_dir: String,
    jwt_enc_key: jsonwebtoken::EncodingKey,
    jwt_dec_key: jsonwebtoken::DecodingKey,
    host: Option<String>,
    port: Option<u16>,
}

impl ServeConfig {
    pub fn new(
        data_dir: String,
        jwt_enc_key: jsonwebtoken::EncodingKey,
        jwt_dec_key: jsonwebtoken::DecodingKey,
    ) -> Self {
        Self {
            data_dir,
            jwt_enc_key,
            jwt_dec_key,
            host: None,
            port: None,
        }
    }

    pub fn host(self, host: String) -> Self {
        self.set_host(Some(host))
    }

    pub fn set_host(mut self, host: Option<String>) -> Self {
        self.host = host;
        self
    }

    pub fn port(self, port: u16) -> Self {
        self.set_port(Some(port))
    }

    pub fn set_port(mut self, port: Option<u16>) -> Self {
        self.port = port;
        self
    }
}

pub async fn serve(config: ServeConfig) -> Result<(), ServeError> {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    let persist = persist::Persist::new();
    let schema = schema(move |s| s.data(persist));

    let state = ServiceState::new(schema, config.jwt_enc_key, config.jwt_dec_key);

    let router = Router::new();
    #[cfg(feature = "graphiql")]
    let router = router.route("/", get(graphiql));
    let app = router
        .route("/api/graphql", post(graphql_handler))
        .route("/api/graphql/ws", get(graphql_ws_handler))
        .with_state(state);

    let addr = SocketAddr::new(
        config
            .host
            .unwrap_or_else(|| "0.0.0.0".to_owned())
            .parse()?,
        config.port.unwrap_or(8080),
    );
    log::info!("Listening on {}", addr);
    #[cfg(feature = "graphiql")]
    log::info!("GraphQL Playground: http://localhost:{}/", addr.port());
    Server::try_bind(&addr)?
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

#[derive(Error, Debug)]
pub enum ServeError {
    #[error("Invalid host: {0}")]
    InvalidHost(#[from] std::net::AddrParseError),
    #[error("Failed to start server: {0}")]
    ServeError(#[from] hyper::Error),
}

pub async fn read_key(
    path: impl AsRef<Path>,
) -> anyhow::Result<(jsonwebtoken::EncodingKey, jsonwebtoken::DecodingKey)> {
    let pem = tokio::fs::read_to_string(path)
        .await
        .context("Unable to locate private key")?;
    let (_, doc) = pkcs8::Document::from_pem(&pem)
        .map_err(|err| anyhow::anyhow!("Failed to parse private key: {:?}", err))?;
    let key_pair = signature::Ed25519KeyPair::from_pkcs8(doc.as_ref())?;
    let enc_key =
        jsonwebtoken::EncodingKey::from_ed_pem(pem.as_bytes()).context("Private key is invalid")?;
    let dec_key = jsonwebtoken::DecodingKey::from_ed_der(key_pair.public_key().as_ref());

    Ok((enc_key, dec_key))
}

async fn graphql_handler(
    State(schema): State<ServiceSchema>,
    State(dec_key): State<DecodingKey>,
    auth_header: Option<TypedHeader<Authorization<Bearer>>>,
    req: GraphQLRequest,
) -> Result<GraphQLResponse, ErrorResponse> {
    let current = authenticate(auth_header, &dec_key).await?;
    Ok(schema.execute(req.into_inner().data(current)).await.into())
}

async fn graphql_ws_handler(
    State(schema): State<ServiceSchema>,
    State(dec_key): State<DecodingKey>,
    protocol: GraphQLProtocol,
    upgrade: WebSocketUpgrade,
) -> impl IntoResponse {
    upgrade
        .protocols(ALL_WEBSOCKET_PROTOCOLS)
        .on_upgrade(move |stream| {
            GraphQLWebSocket::new(stream, schema, protocol)
                .on_connection_init(|init| async move {
                    let mut data = Data::default();
                    let current = authenticate(init, &dec_key).await.extend()?;
                    data.insert(current);
                    Ok(data)
                })
                .serve()
        })
}

#[cfg(feature = "graphiql")]
async fn graphiql() -> impl IntoResponse {
    response::Html(
        GraphiQLSource::build()
            .endpoint("/api/graphql")
            .subscription_endpoint("/api/graphql/ws")
            .finish(),
    )
}

type EncodingKey = Arc<jsonwebtoken::EncodingKey>;
type DecodingKey = Arc<jsonwebtoken::DecodingKey>;

#[derive(Clone)]
struct ServiceState {
    schema: ServiceSchema,
    jwt_enc_key: EncodingKey,
    jwt_dec_key: DecodingKey,
}

impl ServiceState {
    fn new(
        schema: ServiceSchema,
        jwt_enc_key: jsonwebtoken::EncodingKey,
        jwt_dec_key: jsonwebtoken::DecodingKey,
    ) -> Self {
        Self {
            schema,
            jwt_enc_key: Arc::new(jwt_enc_key),
            jwt_dec_key: Arc::new(jwt_dec_key),
        }
    }
}

impl FromRef<ServiceState> for ServiceSchema {
    fn from_ref(state: &ServiceState) -> Self {
        state.schema.clone()
    }
}

impl FromRef<ServiceState> for EncodingKey {
    fn from_ref(state: &ServiceState) -> Self {
        state.jwt_enc_key.clone()
    }
}

impl FromRef<ServiceState> for DecodingKey {
    fn from_ref(state: &ServiceState) -> Self {
        state.jwt_dec_key.clone()
    }
}
