pub struct Config {
    pub generate_indexes: bool,
    pub nav_tree: bool,
    pub page_heading: bool,
    pub path: bool,
}

impl Config {
    #[cfg(test)]
    pub fn none() -> Self {
        Self {
            generate_indexes: false,
            nav_tree: false,
            page_heading: false,
            path: false,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            generate_indexes: true,
            nav_tree: true,
            page_heading: false,
            path: true,
        }
    }
}
