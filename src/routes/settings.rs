use dioxus::prelude::*;

#[allow(non_snake_case)]
pub fn Settings(cx: Scope) -> Element {
    cx.render(rsx! {
        h1 { "Settings" }
    })
}
