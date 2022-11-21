mod api;

use c11ity_client::{wasm::client, Account, Client};
use c11ity_common::api as api_models;
use gloo_net::websocket::futures::WebSocket;
use sycamore::{futures::spawn_local_scoped, prelude::*};

#[component]
fn App<G: Html>(cx: Scope) -> View<G> {
    let id = create_signal::<Option<String>>(cx, None);
    let name = create_signal::<Option<String>>(cx, None);

    spawn_local_scoped(cx, async move {
        let ws = WebSocket::open("ws://localhost:3000/api/v1/rpc").unwrap();
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

    view! { cx,
        p {
            "Hello World!"
        }
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

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    sycamore::render(|cx| {
        view! { cx, App {} }
    });
}
