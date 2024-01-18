use std::path::{Path, PathBuf};

use crate::render::capitalise;

use super::Doc;

pub type Id = usize;

#[derive(Debug)]
enum WikiPageInner {
    Document(Doc),
    Directory,
    Index(Doc),
    Media(PathBuf),
}

#[derive(Debug)]
pub struct WikiPage {
    inner: WikiPageInner,
    path: Vec<Id>,
    title: String,
    url: String,
}

impl WikiPage {
    pub fn id(&self) -> Id {
        *self.path.last().unwrap() // Path must never be non-empty.
    }

    pub fn parent(&self) -> Option<Id> {
        let n = self.path.len();
        if n >= 2 {
            self.path.get(n - 2).copied()
        } else {
            None
        }
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn filename(&self) -> &str {
        if let Some((i, _)) = self.url.rmatch_indices('/').next() {
            &self.url[(i + 1)..]
        } else {
            ""
        }
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn file(&self) -> Option<&Path> {
        if let WikiPageInner::Media(path) = &self.inner {
            Some(path)
        } else {
            None
        }
    }

    pub fn document(&self) -> Option<&Doc> {
        if let WikiPageInner::Document(doc) | WikiPageInner::Index(doc) = &self.inner {
            Some(doc)
        } else {
            None
        }
    }

    pub fn is_descendent_of(&self, ancestor: Id) -> bool {
        self.path.contains(&ancestor)
    }

    pub fn is_root(&self) -> bool {
        self.id() == WikiTree::ROOT
    }

    pub fn is_doc(&self) -> bool {
        matches!(self.inner, WikiPageInner::Document(..))
    }

    pub fn is_dir(&self) -> bool {
        matches!(self.inner, WikiPageInner::Directory)
    }

    pub fn is_index(&self) -> bool {
        matches!(self.inner, WikiPageInner::Index(..))
    }

    pub fn is_media(&self) -> bool {
        matches!(self.inner, WikiPageInner::Media(..))
    }
}

pub struct WikiTree {
    nodes: Vec<WikiPage>,
}

impl WikiTree {
    pub const ROOT: Id = 0;

    pub fn new() -> Self {
        Self {
            nodes: vec![WikiPage {
                inner: WikiPageInner::Directory,
                path: vec![Self::ROOT],
                title: "Index".to_string(),
                url: "".to_string(),
            }],
        }
    }

    fn add<D: std::fmt::Display, S: ToString>(
        &mut self,
        inner: WikiPageInner,
        parent: Id,
        filename: D,
        title: S,
    ) -> Id {
        let id = self.nodes.len();
        let parent = if let Some(parent) = self.get(parent) {
            parent
        } else {
            self.get(Self::ROOT).unwrap()
        };
        let mut path = parent.path.clone();
        path.push(id);

        self.nodes.push(WikiPage {
            inner,
            path,
            title: title.to_string(),
            url: format!("{}/{}", parent.url(), filename),
        });

        id
    }

    pub fn add_doc<D: std::fmt::Display, S: ToString>(
        &mut self,
        parent: Id,
        filename: D,
        title: S,
        document: Doc,
    ) -> Id {
        self.add(WikiPageInner::Document(document), parent, filename, title)
    }

    pub fn add_dir<D: std::fmt::Display>(&mut self, parent: Id, filename: D) -> Id {
        let title = capitalise(&filename.to_string().replace('-', " "));
        self.add(WikiPageInner::Directory, parent, filename, title)
    }

    pub fn add_index<D: std::fmt::Display, S: ToString>(
        &mut self,
        parent: Id,
        filename: D,
        title: S,
        document: Doc,
    ) -> Id {
        self.add(WikiPageInner::Index(document), parent, filename, title)
    }

    pub fn add_media<D: std::fmt::Display, S: ToString, P: Into<PathBuf>>(
        &mut self,
        parent: Id,
        filename: D,
        title: S,
        path: P,
    ) -> Id {
        self.add(WikiPageInner::Media(path.into()), parent, filename, title)
    }

    pub fn get(&self, id: Id) -> Option<&WikiPage> {
        self.nodes.get(id)
    }

    pub fn get_parent(&self, node: &WikiPage) -> Option<&WikiPage> {
        node.parent().and_then(|id| self.get(id))
    }

    pub fn children(&self, id: Id) -> Vec<&WikiPage> {
        self.nodes
            .iter()
            .filter(|n| n.parent() == Some(id))
            .collect()
    }

    pub fn pages(&self) -> &[WikiPage] {
        &self.nodes
    }

    pub fn into_pages(self) -> Vec<WikiPage> {
        self.nodes
    }

    fn link_target_filename(title: &str, ext: &str) -> String {
        let mut rectified_name = title.to_lowercase().replace(' ', "-");
        rectified_name.push('.');
        rectified_name.push_str(ext);
        rectified_name
    }

    pub fn find_link_target(&self, title: &str, ext: &str, from: &WikiPage) -> Option<&WikiPage> {
        let filename = &Self::link_target_filename(title, ext);

        let mut frontier = self.children(from.id());
        let mut i = 0;
        while let Some(descendent) = frontier.get(i) {
            if descendent.filename() == filename || descendent.title().eq_ignore_ascii_case(title) {
                return Some(descendent);
            }

            frontier.extend(self.children(descendent.id()));
            i += 1;
        }

        self.get_parent(from)
            .and_then(|p| self.find_link_target(title, ext, p))
    }

    #[cfg(debug)]
    fn print_subtree(&self, node: &WikiPage, indent: usize) {
        println!("{}{}", "  ".repeat(indent), node.filename());
        for child in self.children(node.id()) {
            self.print_subtree(child, indent + 1);
        }
    }

    #[cfg(debug)]
    pub fn print(&self) {
        self.print_subtree(self.get(Self::ROOT).unwrap(), 0);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_filename() {
        let node = WikiPage {
            inner: WikiPageInner::Document(Doc::empty()),
            path: vec![0, 1],
            title: "Title".into(),
            url: "/rootdir/title.html".into(),
        };

        assert_eq!(node.filename(), "title.html");
    }

    #[test]
    fn test_find_link_target() {
        let mut tree = WikiTree::new();
        let rootdir = tree.add_dir(WikiTree::ROOT, "rootdir");
        let rootdoc = tree.add_doc(rootdir, "file.html", "File", Doc::empty());
        let childdir = tree.add_dir(rootdir, "childdir");

        let title = "File";
        let name = "file.html";

        {
            let root = tree.get(rootdir).unwrap();
            let child = tree.get(childdir).unwrap();

            assert_eq!(
                tree.find_link_target(title, name, root).unwrap().id(),
                rootdoc
            );
            assert_eq!(
                tree.find_link_target(title, name, child).unwrap().id(),
                rootdoc
            );
        }

        let childdoc = tree.add_doc(childdir, "file.html", "File", Doc::empty());

        {
            let root = tree.get(rootdir).unwrap();
            let child = tree.get(childdir).unwrap();
            assert_eq!(
                tree.find_link_target(title, name, root).unwrap().id(),
                rootdoc
            );
            assert_eq!(
                tree.find_link_target(title, name, child).unwrap().id(),
                childdoc
            );
        }
    }
}
