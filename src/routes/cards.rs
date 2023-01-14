use std::collections::HashSet;

use dioxus::prelude::*;
use dioxus_router::use_router;

use crate::{
    components::{MarkdownView, Tags},
    hooks::use_database,
};

#[allow(non_snake_case)]
pub fn Cards(cx: Scope) -> Element {
    let db = use_database(&cx);
    let router = use_router(&cx);

    let cards = use_state(&cx, || Vec::new());
    let selected_tags = use_ref(&cx, || HashSet::new());
    let show_tagless = use_state(&cx, || false);

    let db_clone = db.clone();
    let cards_clone = cards.clone();

    use_effect(
        &cx,
        (selected_tags, show_tagless),
        |(selected, show)| async move {
            let cards = if *show.current() {
                db_clone.borrow().get_cards_without_tags()
            } else {
                let ids = selected.read().iter().copied().collect::<Box<[_]>>();
                db_clone.borrow().get_cards_with_tags(ids)
            };
            cards_clone.set(cards);
        },
    );

    cx.render(rsx! {
        h1 { "Cards" }

        h2 { "Tags" }
        button {
            onclick: |_| {
                show_tagless.set(!*show_tagless.current());
            },
            "Tagless"
        }
        button {
            onclick: |_| {
                if **show_tagless {
                    show_tagless.set(false);
                }
                if !selected_tags.read().is_empty() {
                    selected_tags.write().clear();
                }
            },
            "Reset"
        }
        br {}
        Tags {
            selected: selected_tags,
        }

        cards.iter().map(|c| rsx! {
            div {
                MarkdownView {
                    key: "{c.id}",
                    text: "{c.content}",
                }
                button {
                    onclick: |_| {
                        router.push_route(&format!("/edit_card/{}", c.id), None, None);
                    },
                    "Edit"
                }
            }
        })
    })
}
