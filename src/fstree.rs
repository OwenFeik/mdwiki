use crate::INDEX_FILE;

#[derive(Debug)]
pub enum NodeType {
    File,
    Directory,
}

#[derive(Debug)]
pub struct FsNode {
    ty: NodeType,
    pub id: usize,
    path: Vec<String>,
    pub parent: Option<usize>,
    title: String,
}

impl FsNode {
    pub fn path(&self) -> &[String] {
        &self.path
    }

    pub fn name(&self) -> Option<&String> {
        self.path().last()
    }

    pub fn url(&self) -> String {
        format!("/{}", self.path.join("/"))
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn is_dir(&self) -> bool {
        matches!(self.ty, NodeType::Directory)
    }

    pub fn is_root(&self) -> bool {
        self.id == FsTree::ROOT
    }

    pub fn is_index_file(&self) -> bool {
        !self.is_dir() && self.name().map(String::as_str) == Some(INDEX_FILE)
    }
}

pub struct FsTree {
    nodes: Vec<FsNode>,
}

impl FsTree {
    pub const ROOT: usize = 0;

    pub fn new() -> Self {
        Self {
            nodes: vec![FsNode {
                ty: NodeType::Directory,
                id: Self::ROOT,
                path: Vec::new(),
                parent: None,
                title: "Index".to_string(),
            }],
        }
    }

    fn path<S: ToString>(&self, name: S, parent: usize) -> (Vec<String>, Option<usize>) {
        if let Some(node) = self.nodes.get(parent) {
            let mut path: Vec<String> = node.path().into();
            path.push(name.to_string());
            (path, Some(parent))
        } else {
            (vec![name.to_string()], None)
        }
    }

    pub fn add<S: ToString>(&mut self, name: S, parent: usize, title: Option<String>) -> usize {
        let id = self.nodes.len();
        let name = name.to_string();
        let (path, parent) = self.path(name.clone(), parent);
        self.nodes.push(FsNode {
            ty: NodeType::File,
            id,
            path,
            parent,
            title: title.map(|s| s.to_string()).unwrap_or(name),
        });
        id
    }

    pub fn add_dir<S: ToString>(&mut self, name: S, parent: usize) -> usize {
        let id = self.nodes.len();
        let title = name.to_string();
        let (path, parent) = self.path(name, parent);
        self.nodes.push(FsNode {
            ty: NodeType::Directory,
            id,
            path,
            parent,
            title,
        });
        id
    }

    pub fn get(&self, id: usize) -> Option<&FsNode> {
        self.nodes.get(id)
    }

    pub fn get_parent(&self, node: &FsNode) -> Option<&FsNode> {
        node.parent.and_then(|id| self.get(id))
    }

    pub fn children(&self, id: usize) -> Vec<&FsNode> {
        self.nodes.iter().filter(|n| n.parent == Some(id)).collect()
    }

    pub fn nodes(&self) -> &[FsNode] {
        &self.nodes
    }
}
