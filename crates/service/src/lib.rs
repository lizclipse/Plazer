#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::enum_glob_use)]
#![allow(clippy::match_wildcard_for_single_variants)]
#![allow(clippy::missing_errors_doc)]
#![forbid(unsafe_code)]

mod account;
mod board;
pub mod config;
mod conv;
mod error;
mod macros;
mod migration;
mod persist;
mod post;
mod prelude;
mod query;
mod schema;

use std::{io, net::SocketAddr, sync::Arc};

#[cfg(feature = "graphiql")]
use async_graphql::http::GraphiQLSource;
use async_graphql::{http::ALL_WEBSOCKET_PROTOCOLS, Data, ResultExt as _};
use async_graphql_axum::{GraphQLBatchRequest, GraphQLProtocol, GraphQLResponse, GraphQLWebSocket};
use axum::{
    extract::{FromRef, State, WebSocketUpgrade},
    headers::{authorization::Bearer, Authorization},
    routing::{get, post},
    Router, Server, TypedHeader,
};
use config::LogConfig;
use ring::rand::{SecureRandom as _, SystemRandom};
use thiserror::Error;
use tokio::signal;
use tracing::{debug, error, info, instrument, metadata::LevelFilter, trace};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, layer::SubscriberExt as _, Layer as _};

pub use crate::schema::schema;
use crate::{
    account::authenticate, config::ServeConfig, error::ErrorResponse, migration::Migrations,
    schema::ServiceSchema,
};

/// Initialise logging.
///
/// # Panics
///
/// Panics if the global subscriber cannot be set.
pub fn init_logging(
    LogConfig {
        dir: path,
        level_stdout,
        level_file,
    }: LogConfig,
) -> WorkerGuard {
    let file_appender = tracing_appender::rolling::hourly(path, "service.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let collector = tracing_subscriber::registry()
        .with(
            fmt::Layer::new()
                .with_writer(io::stdout)
                .pretty()
                .with_filter(LevelFilter::from_level(level_stdout)),
        )
        .with(
            fmt::Layer::new()
                .with_writer(non_blocking)
                .json()
                .with_filter(LevelFilter::from_level(level_file)),
        );
    tracing::subscriber::set_global_default(collector).expect("Unable to set a global subscriber");

    trace!(?level_stdout, ?level_file, "Logging initialised");

    guard
}

#[instrument(skip(jwt_enc_key, jwt_dec_key))]
pub async fn serve(
    ServeConfig {
        address,
        namespace,
        database,
        jwt_enc_key,
        jwt_dec_key,
        host,
        port,
    }: ServeConfig,
) -> Result<(), ServeError> {
    debug!("Initialising RNG");
    // Call fill once before starting to initialize the RNG.
    let csrng = SystemRandom::new();
    let mut rng_buf = [0u8; 1];
    csrng.fill(&mut rng_buf)?;

    let jwt_enc_key = Arc::new(jwt_enc_key);
    let jwt_dec_key = Arc::new(jwt_dec_key);
    let persist = persist::Persist::new(address, namespace, database).await?;

    info!("Configuring database...");
    if let Err(err) = Migrations::run(&persist).await {
        error!(
            error = ?err,
            "Failed to complete configuration, database may be corrupt"
        );
        return Err(err.into());
    }
    info!("Database configuration complete");

    let schema = schema(|s| {
        s.data(persist)
            .data(csrng)
            .data(jwt_enc_key.clone())
            .data(jwt_dec_key.clone())
    });

    let state = ServiceState::new(schema, jwt_enc_key, jwt_dec_key);

    let router = Router::new();
    #[cfg(feature = "graphiql")]
    let router = router.route("/", get(graphiql));
    let app = router
        .route("/api/graphql", post(graphql_handler))
        .route("/api/graphql/ws", get(graphql_ws_handler))
        .with_state(state);

    let addr = SocketAddr::new(host, port);
    info!("Listening on {}", addr);
    #[cfg(feature = "graphiql")]
    info!("GraphQL Playground: http://localhost:{}/", addr.port());
    Server::try_bind(&addr)?
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }

    info!("Shutting down");
}

#[derive(Error, Debug)]
pub enum ServeError {
    #[error("Invalid host: {0}")]
    InvalidHost(#[from] std::net::AddrParseError),
    #[error("Failed to start server: {0}")]
    ServeError(#[from] hyper::Error),
    #[error("Failed to initialise database: {0}")]
    PersistError(#[from] surrealdb::Error),
    #[error("Failed to initialise cryptography")]
    CryptoError(#[from] ring::error::Unspecified),
}

#[instrument(skip_all)]
async fn graphql_handler(
    State(schema): State<ServiceSchema>,
    State(dec_key): State<DecodingKey>,
    auth_header: Option<TypedHeader<Authorization<Bearer>>>,
    req: GraphQLBatchRequest,
) -> Result<GraphQLResponse, ErrorResponse> {
    let current = authenticate(auth_header, &dec_key)?;
    Ok(schema
        .execute_batch(req.into_inner().data(Arc::new(current)))
        .await
        .into())
}

#[instrument(skip_all)]
async fn graphql_ws_handler(
    State(schema): State<ServiceSchema>,
    State(dec_key): State<DecodingKey>,
    protocol: GraphQLProtocol,
    upgrade: WebSocketUpgrade,
) -> axum::response::Response {
    upgrade
        .protocols(ALL_WEBSOCKET_PROTOCOLS)
        .on_upgrade(move |stream| {
            GraphQLWebSocket::new(stream, schema, protocol)
                .on_connection_init(|init| async move {
                    let mut data = Data::default();
                    let current = authenticate(init, &dec_key).extend()?;
                    data.insert(current);
                    Ok(data)
                })
                .serve()
        })
}

#[cfg(feature = "graphiql")]
#[instrument(skip_all)]
async fn graphiql() -> axum::response::Html<String> {
    axum::response::Html(
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
        jwt_enc_key: impl Into<EncodingKey>,
        jwt_dec_key: impl Into<DecodingKey>,
    ) -> Self {
        Self {
            schema,
            jwt_enc_key: jwt_enc_key.into(),
            jwt_dec_key: jwt_dec_key.into(),
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
