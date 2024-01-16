#![feature(anonymous_lifetime_in_impl_trait)]
#![feature(let_chains)]
#![feature(pattern)]

use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use config::Config;
use model::{Id, WikiPage, WikiTree};
use render::{INDEX_FILE, OUTPUT_EXT};

mod config;
mod log;
mod model;
mod parse;
mod render;

fn create_outdir(outdir: &Path) {
    if std::fs::create_dir_all(outdir).is_err() {
        log::warning(format!(
            "Failed to create output directory: {}",
            outdir.display()
        ));
    }
}

fn create_output_path(outdir: &Path, page: &WikiPage) -> PathBuf {
    let mut output = outdir.to_path_buf();
    for segment in page.url().split('/').filter(|s| !s.is_empty()) {
        output.push(segment);
    }

    if let Some(outdir) = output.parent() {
        create_outdir(outdir);
    }

    output
}

fn render_document(outdir: &Path, config: &Config, tree: &WikiTree, page: &WikiPage) {
    let Ok(html) = render::render_document(config, tree, page) else {
        log::error(format!("Failed to render {}", page.url()));
        return;
    };

    let destination = create_output_path(outdir, page);

    if std::fs::write(&destination, html).is_ok() {
        log::debug(format!(
            "Rendered {} to {}",
            page.url(),
            destination.display()
        ));
    } else {
        log::error(format!("Failed to write file: {}", destination.display()));
    }
}

fn copy_file(media: &WikiPage, outdir: &Path) {
    if let Some(file) = media.file() {
        let destination = create_output_path(outdir, media);
        if let Err(e) = std::fs::copy(file, &destination) {
            log::error(format!("Failed to copy file ({}): {e}", file.display()));
        } else {
            log::debug(format!(
                "Copied {} to {}",
                file.display(),
                destination.display()
            ));
        }
    }
}

fn add_indexes(tree: &mut WikiTree) {
    let directories: Vec<Id> = tree
        .pages()
        .iter()
        .filter(|page| page.is_dir())
        .map(|dir| dir.id())
        .collect();

    for id in directories {
        let children = tree.children(id);
        if !children.is_empty() && !children.iter().any(|child| child.is_index()) {
            let dir = tree.get(id).unwrap();
            let title = dir.title().to_string();
            let document = render::create_index(dir, &children);
            tree.add_index(dir.id(), INDEX_FILE, title, document);
        }
    }
}

fn fail(msg: &str) -> ! {
    log::error(msg);
    panic!();
}

fn main() {
    let Some(arg) = std::env::args().nth(1) else {
        fail("Usage: mdwiki file.md");
    };

    let Ok(metadata) = std::fs::metadata(&arg) else {
        fail("Couldn't read argument file metadata.")
    };

    let config = Config::default();

    if metadata.is_file() {
        let path = PathBuf::from(arg);
        let Some(parent) = path.parent() else {
            fail("Couldn't find parent directory of input file.");
        };

        if let Ok(page) = parse::parse_file(&path) {
            render_document(parent, &config, &WikiTree::new(), &page)
        } else {
            fail("Unable to process file for rendering.");
        }
    } else if metadata.is_dir() {
        let indir = PathBuf::from(arg);
        let Some(Some(dirname)) = indir.file_name().map(OsStr::to_str) else {
            fail("Couldn't find filename of input directory.");
        };

        if let Some(parent) = indir.parent() {
            let outdir = parent.join(format!("{dirname}-{OUTPUT_EXT}"));
            let mut tree = parse::parse_directory(&indir);

            if config.generate_indexes {
                log::info("Generating indexes for directories which don't have them.");
                add_indexes(&mut tree);
            }

            for page in tree.pages() {
                if page.is_doc() || page.is_index() {
                    render_document(&outdir, &config, &tree, page);
                } else if page.is_media() {
                    copy_file(page, &outdir);
                }
            }

            log::info(format!(
                "Successfully rendered {} to {}",
                indir.display(),
                outdir.display()
            ));
        } else {
            fail("Couldn't choose an output directory for files.");
        }
    }
}
