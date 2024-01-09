pub struct Config {
    pub nav_tree: bool,
    pub page_heading: bool,
    pub path: bool,
}

impl Config {
    pub fn none() -> Self {
        Self {
            nav_tree: false,
            page_heading: false,
            path: false,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            nav_tree: true,
            page_heading: false,
            path: true,
        }
    }
}
