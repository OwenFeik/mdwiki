pub struct FsNode {
    id: usize,
    path: Vec<String>,
    parent: Option<usize>,
}

impl FsNode {
    pub fn path(&self) -> &[String] {
        &self.path
    }

    pub fn url(&self) -> String {
        format!("/{}", self.path.join("/"))
    }

    pub fn name(&self) -> Option<&str> {
        self.path.last().map(|x| x.as_str())
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
                id: Self::ROOT,
                path: Vec::new(),
                parent: None,
            }],
        }
    }

    pub fn add<S: ToString>(&mut self, name: S, parent: usize) -> usize {
        let id = self.nodes.len();

        let (path, parent) = if let Some(node) = self.nodes.get(parent) {
            let mut path: Vec<String> = node.path().into();
            path.push(name.to_string());
            (path, Some(parent))
        } else {
            (vec![name.to_string()], None)
        };

        self.nodes.push(FsNode { id, path, parent });
        id
    }

    pub fn get(&self, id: usize) -> Option<&FsNode> {
        self.nodes.get(id)
    }

    pub fn children(&self, id: usize) -> Vec<usize> {
        self.nodes
            .iter()
            .filter(|n| n.parent == Some(id))
            .map(|n| n.id)
            .collect()
    }
}
