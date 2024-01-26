use std::collections::HashMap;

use crate::model::Tag;

pub struct Config {
    /// Whether to support empty links by attempting to rewrite them. For
    /// example:
    /// `[City]()` would be rendered as `<a href="/path/city.html">City</a>`
    /// if that file exists in the tree. The nearest relative of the page the
    /// link is in will be used. The title is converted to kebab case to find
    /// an appropriate file.
    pub empty_links: bool,

    /// Whether to generate index.html for directories in which it doesn't
    /// exist. If true a simple index of the directory will be generated.
    pub generate_indexes: bool,

    /// Whether to include a nav tree of all wiki content in generated pages.
    /// If true, all pages will have a tree reflecting the directory structure
    /// on the left hand side, with directories not part of the current files
    /// ancestry collapsed.
    pub nav_tree: bool,

    /// Whether to add a heading to pages. If true a heading will be added using
    /// the capitalised filename with extension omitted.
    pub page_heading: bool,

    /// Whether to add a series of breadcrumbs with links for all ancestors of
    /// the current directory above the first node.
    pub add_breadcrumbs: bool,

    /// Passwords associated with tags. Including a password here will cause
    /// all nodes with the given tag to be AES encoded with the given password.
    pub tag_keys: HashMap<Tag, String>,
}

impl Config {
    #[cfg(test)]
    pub fn none() -> Self {
        Self {
            empty_links: false,
            generate_indexes: false,
            nav_tree: false,
            page_heading: false,
            add_breadcrumbs: false,
            tag_keys: HashMap::new(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        let mut tag_passwords = HashMap::new();
        tag_passwords.insert("dm".into(), "skoraeus".to_string());
        Self {
            empty_links: true,
            generate_indexes: true,
            nav_tree: true,
            page_heading: false,
            add_breadcrumbs: true,
            tag_keys: tag_passwords,
        }
    }
}
