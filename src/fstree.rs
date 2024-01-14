#[derive(Debug)]
pub enum NodeType {
    File,
    Directory,
    Index,
}

#[derive(Debug)]
pub struct FsNode {
    ty: NodeType,
    path: Vec<usize>,
    title: String,
    url: String,
}

impl FsNode {
    pub fn id(&self) -> usize {
        *self.path.last().unwrap() // Path must never be non-empty.
    }

    pub fn parent(&self) -> Option<usize> {
        let n = self.path.len();
        if n >= 2 {
            self.path.get(n - 2).copied()
        } else {
            None
        }
    }

    pub fn is_descendent_of(&self, ancestor: usize) -> bool {
        self.path.contains(&ancestor)
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn path(&self) -> &[usize] {
        &self.path
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn is_dir(&self) -> bool {
        matches!(self.ty, NodeType::Directory)
    }

    pub fn is_root(&self) -> bool {
        self.id() == FsTree::ROOT
    }

    pub fn is_index(&self) -> bool {
        matches!(self.ty, NodeType::Index)
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
                path: vec![Self::ROOT],
                title: "Index".to_string(),
                url: "".to_string(),
            }],
        }
    }

    fn add<D: std::fmt::Display, S: ToString>(
        &mut self,
        ty: NodeType,
        parent: usize,
        filename: D,
        title: S,
    ) -> usize {
        let id = self.nodes.len();
        let parent = if let Some(parent) = self.get(parent) {
            parent
        } else {
            self.get(Self::ROOT).unwrap()
        };
        let mut path = parent.path.clone();
        path.push(id);

        self.nodes.push(FsNode {
            ty,
            path,
            title: title.to_string(),
            url: format!("{}/{}", parent.url(), filename),
        });

        id
    }

    pub fn add_file<D: std::fmt::Display, S: ToString>(
        &mut self,
        parent: usize,
        filename: D,
        title: S,
    ) -> usize {
        self.add(NodeType::File, parent, filename, title)
    }

    pub fn add_dir<D: std::fmt::Display>(&mut self, parent: usize, filename: D) -> usize {
        let title = filename.to_string();
        self.add(NodeType::Directory, parent, filename, title)
    }

    pub fn add_index<D: std::fmt::Display, S: ToString>(
        &mut self,
        parent: usize,
        filename: D,
        title: S,
    ) -> usize {
        self.add(NodeType::Index, parent, filename, title)
    }

    pub fn get(&self, id: usize) -> Option<&FsNode> {
        self.nodes.get(id)
    }

    pub fn get_parent(&self, node: &FsNode) -> Option<&FsNode> {
        node.parent().and_then(|id| self.get(id))
    }

    pub fn children(&self, id: usize) -> Vec<&FsNode> {
        self.nodes
            .iter()
            .filter(|n| n.parent() == Some(id))
            .collect()
    }

    pub fn nodes(&self) -> &[FsNode] {
        &self.nodes
    }
}
