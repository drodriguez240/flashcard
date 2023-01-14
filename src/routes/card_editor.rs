use std::{collections::HashSet, path::Path};

use dioxus::prelude::*;
use dioxus_router::{use_route, use_router};

use database::*;
use sqlite::{params, SqliteId};

use crate::{
    components::{MarkdownEditor, Tags},
    hooks::use_database,
};

#[allow(non_snake_case)]
pub fn AddCard(cx: Scope) -> Element {
    let db = use_database(&cx);
    let content = use_state(&cx, || String::new());
    let selected_tags = use_ref(&cx, || HashSet::new());

    let html = markdown::to_html(&content);

    cx.render(rsx! {
        h1 { "Add Card" }
        MarkdownEditor {
            text: content,
        }
        div {
            dangerous_inner_html: "{html}",
        }
        Tags {
            selected: selected_tags,
        }
        button {
            onclick: move |_| {
                let db = db.borrow();
                add_card(&content.current(), &selected_tags.read(), &db);
                store_assets(&html, &db);
                content.set(String::new());
            },
            "Add"
        }
    })
}

fn add_card(content: &str, tags: &HashSet<SqliteId>, db: &Database) {
    db.execute_one("INSERT INTO cards (content) VALUES (?)", [content])
        .unwrap();

    let card_id = db.last_insert_rowid();

    tags.iter().copied().for_each(|tag_id| {
        db.execute_one(
            "INSERT INTO card_tag (card_id, tag_id) VALUES (?, ?)",
            [card_id, tag_id],
        )
        .unwrap();
    });
}

#[allow(non_snake_case)]
pub fn EditCard(cx: Scope) -> Element {
    let id = use_route(&cx)
        .segment("id")
        .unwrap()
        .parse::<SqliteId>()
        .unwrap();
    let db = use_database(&cx);
    let router = use_router(&cx);
    let content = use_state(&cx, || {
        db.borrow()
            .fetch_one::<String>("SELECT id, content FROM cards WHERE id = ?", [id], |row| {
                row.get(1)
            })
            .unwrap()
    });
    let current_tags = use_ref(&cx, || {
        let mut tags = HashSet::new();
        db.borrow()
            .fetch_with::<SqliteId>(
                "SELECT card_id, tag_id FROM card_tag WHERE card_id = ?",
                [id],
                |row| {
                    tags.insert(row.get(1).unwrap());
                },
            )
            .unwrap();
        tags
    });
    let selected_tags = use_ref(&cx, || current_tags.read().clone());

    let html = markdown::to_html(&content);

    cx.render(rsx! {
        h1 { "Edit Card ({id})" }
        MarkdownEditor {
            text: content,
        }
        div {
            dangerous_inner_html: "{html}",
        }
        Tags {
            selected: selected_tags,
        }
        button {
            onclick: move |_| {
                let db = db.borrow();
                edit_card(id, &content.current(), &current_tags.read(), &selected_tags.read(), &db);
                store_assets(&html, &db);
                router.pop_route();
            },
            "Save"
        }
    })
}

fn edit_card(
    card_id: SqliteId,
    content: &str,
    current_tags: &HashSet<SqliteId>,
    selected_tags: &HashSet<SqliteId>,
    db: &Database,
) {
    db.execute_one(
        "UPDATE cards SET content = ? WHERE id = ?",
        (content, card_id),
    )
    .unwrap();

    let to_add = selected_tags.difference(current_tags);
    let to_remove = current_tags.difference(selected_tags);

    to_add.copied().for_each(|tag_id| {
        db.execute_one(
            "INSERT INTO card_tag (card_id, tag_id) VALUES (?, ?)",
            [card_id, tag_id],
        )
        .unwrap();
    });

    to_remove.copied().for_each(|tag_id| {
        db.execute_one(
            "DELETE FROM card_tag WHERE card_id = ? AND tag_id = ?",
            [card_id, tag_id],
        )
        .unwrap();
    });
}

fn store_assets(html: &str, db: &Database) {
    // const ASSET_REGEX: &str = const_format::concatcp!(r#"src\s*=\s*["|']"#, config::ASSETS_DIR, r#"/(.+?)["|']"#);
    const ASSET_REGEX: &str = const_format::concatcp!(config::ASSETS_DIR, r"/[0-9]*.[\w]*");

    let regex = regex::Regex::new(ASSET_REGEX).unwrap();
    for caps in regex.captures_iter(html) {
        let asset_path = Path::new(&caps[0]);

        let Some(file_stem) = asset_path.file_stem() else {
            return;
        };
        let Some(ext) = asset_path.extension() else {
            return;
        };
        let Ok(hash) = file_stem.to_string_lossy().parse::<Seahash>() else {
            return;
        };

        if db
            .fetch_one::<Seahash>(
                "SELECT seahash FROM assets WHERE seahash = ?",
                [hash],
                |row| row.get(0),
            )
            .is_err()
        {
            let bytes = std::fs::read(asset_path).unwrap();
            db.execute_one(
                "INSERT INTO assets (seahash, bytes, extension) VALUES (?, ?, ?)",
                params![hash, bytes, ext.to_string_lossy()],
            )
            .unwrap();
        }
    }
}
