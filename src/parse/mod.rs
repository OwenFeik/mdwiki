use std::path::Path;

use crate::model::{WikiPage, WikiTree};

mod fs;
mod md;

#[cfg(test)]
pub use self::md::parse_document;

pub use self::fs::IMAGE_EXTS;

pub fn parse_file(path: &Path) -> Result<WikiPage, ()> {
    let mut tree = WikiTree::new();
    fs::process_document(&mut tree, WikiTree::ROOT, path);
    if let Some(page) = tree.into_pages().into_iter().find(|p| !p.is_root()) {
        Ok(page)
    } else {
        Err(())
    }
}

pub fn parse_directory(path: &Path) -> WikiTree {
    let mut tree = WikiTree::new();
    fs::process_directory(
        &mut tree,
        true,
        WikiTree::ROOT,
        path,
        &std::path::PathBuf::from("value"),
    );
    tree
}
