mod api;

use c11ity_ui::App;
use sycamore::prelude::*;

fn main() {
    #[cfg(feature = "client")]
    {
        console_error_panic_hook::set_once();
        console_log::init_with_level(log::Level::Debug).unwrap();
    }

    sycamore::render(|cx| {
        view! { cx, App(path=None) }
    });
}
