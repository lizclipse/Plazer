mod api;

use c11ity_client::{wasm::client, Account, Client};
use c11ity_common::api as api_models;
#[cfg(feature = "client")]
use gloo_net::websocket::futures::WebSocket;
#[cfg(feature = "client")]
use gloo_utils::window;
use sycamore::{futures::spawn_local_scoped, prelude::*};

#[derive(Prop)]
pub struct Props {
    path: Option<String>,
}

#[component]
pub fn App<G: Html>(cx: Scope, Props { path }: Props) -> View<G> {
    let id = create_signal::<Option<String>>(cx, None);
    let name = create_signal::<Option<String>>(cx, None);

    if G::IS_BROWSER {
        spawn_local_scoped(cx, async move {
            let host = window().location().host().unwrap();
            let ws = WebSocket::open(&format!("ws://{}/api/v1/rpc", host)).unwrap();
            let client = client(ws);
            let account = client.account();

            let acc = account
                .login(api_models::account::LoginReq {
                    uname: "test",
                    pword: "test",
                })
                .await
                .unwrap()
                .unwrap();

            id.set(Some(acc.id));
            name.set(acc.name);
        });
    }

    view! { cx,
        p {
            "Hello World!"
        }
        p { "At: " (match &path {
            Some(path) => path.to_owned(),
            None => "".to_owned(),
        }) }
        (match id.get().as_ref().as_ref().map(|id| id.to_owned()) {
            Some(id) => view! { cx,
                p { "ID: " (id) }
            },
            None => view! { cx, }
        })
        (match name.get().as_ref().as_ref().map(|name| name.to_owned()) {
            Some(name) => view! { cx,
                p { "Name: " (name) }
            },
            None => view! { cx, }
        })
    }
}

pub async fn render(path: String) -> String {
    sycamore::web::render_to_string_await_suspense(move |cx| {
        let path = path;
        view! { cx, App(path=Some(path)) }
    }).await
}
