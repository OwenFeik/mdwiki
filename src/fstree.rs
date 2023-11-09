pub struct FsNode {
    id: usize,
    path: Vec<String>,
    children: Vec<usize>,
}

impl FsNode {
    pub fn path(&self) -> &[String] {
        &self.path
    }
}

pub struct FsTree {
    nodes: Vec<FsNode>
}

impl FsTree {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new()
        }
    }

    pub fn add<S: ToString>(&mut self, name: S, parent: usize) -> usize {
        let id = self.nodes.len();

        let path = if let Some(parent) = self.nodes.get_mut(parent) {
            parent.children.push(id);
            let mut path: Vec<String> = parent.path().into();
            path.push(name.to_string());
            path
        } else {
            vec![name.to_string()]
        };
        self.nodes.push(FsNode { id, path, children: Vec::new() });        
        id
    }

    pub fn get(&self, id: usize) -> Option<&FsNode> {
        self.nodes.get(id)
    }
}
