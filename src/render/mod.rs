mod aes;
mod html;
mod nav;

#[cfg(test)]
mod test;

use crate::config::Config;
use crate::model::{Tag, WikiPage, WikiTree};

pub use self::html::{render_document, CSS_CLASS_ATTR, CSS_ID_ATTR};
pub use self::nav::create_index;

pub const INDEX_FILE: &str = "index.html";
pub const OUTPUT_EXT: &str = "html";

struct RenderState<'a> {
    tree: &'a WikiTree,
    page: &'a WikiPage,
    config: &'a Config,
    html: &'a mut html::Html,
}

impl<'a> std::ops::Deref for RenderState<'a> {
    type Target = html::Html;

    fn deref(&self) -> &Self::Target {
        self.html
    }
}

impl<'a> std::ops::DerefMut for RenderState<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.html
    }
}

fn capitalise_word(word: &str) -> String {
    match word {
        "a" | "and" | "at" | "is" | "of" | "to" => word.to_string(),
        _ if !word.is_empty() => format!(
            "{}{}",
            word.chars().next().unwrap().to_uppercase(),
            &word[1..]
        ),
        _ => word.to_string(),
    }
}

fn encryption_pairs<'a>(
    state: &'a RenderState,
    tags: &[Tag],
) -> Option<Vec<(&'a Tag, &'a String)>> {
    if tags.is_empty() {
        None
    } else {
        let pairs: Vec<(&Tag, &String)> = state
            .config
            .tag_keys
            .iter()
            .filter(|(k, _)| tags.contains(k))
            .collect();
        if pairs.is_empty() {
            None
        } else {
            Some(pairs)
        }
    }
}

pub fn capitalise(title: &str) -> String {
    title
        .split(|c| c == ' ' || c == '-' || c == '_')
        .map(capitalise_word)
        .collect::<Vec<String>>()
        .join(" ")
}

#[cfg(test)]
fn render_node(node: &crate::model::Node) -> String {
    let mut tree = WikiTree::new();
    let page = tree.add_doc(
        WikiTree::ROOT,
        "document.html",
        "Document",
        crate::model::Doc::empty(),
    );
    let page = tree.get(page).unwrap();
    let config = crate::config::Config::none();
    html::render_node_only(&config, &tree, page, node)
}
