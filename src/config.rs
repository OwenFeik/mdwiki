pub struct Config {
    pub directory_index: bool,
    pub page_heading: bool,
    pub path: bool,
}

impl Config {
    pub fn none() -> Self {
        Self {
            directory_index: false,
            page_heading: false,
            path: false,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            directory_index: true,
            page_heading: false,
            path: true,
        }
    }
}
