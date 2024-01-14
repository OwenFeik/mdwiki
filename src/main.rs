#![feature(let_chains)]
#![feature(pattern)]

use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use config::Config;
use fstree::{FsNode, FsTree};
use model::File;

mod config;
mod fstree;
mod log;
mod model;
mod parse;
mod render;

const INPUT_EXT: &str = "md";
const RESOURCE_EXTS: &[&str] = &["jpg", "jpeg", "png"];
const OUTPUT_EXT: &str = "html";
const INDEX_FILE: &str = "index.html";

fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|name| name.starts_with('.'))
        .unwrap_or(false)
}

fn create_outdir(outdir: &Path) {
    if std::fs::create_dir_all(outdir).is_err() {
        log::warning(format!(
            "Failed to create output directory: {}",
            outdir.display()
        ));
    }
}

fn document_title(nodes: &[model::Node], filename: &str) -> Option<String> {
    for node in nodes {
        if let model::El::Heading(1, children) = node.el() {
            for node in children {
                if let model::El::Text(text) = node.el() {
                    return Some(text.clone());
                }
            }
        }
    }

    filename.split('.').next().map(|s| s.to_string())
}

fn process_file(
    tree: &mut FsTree,
    parent: usize,
    file: &Path,
    outdir: &Path,
) -> Option<(File, PathBuf)> {
    let Some(Some(name)) = file.file_name().map(std::ffi::OsStr::to_str) else {
        log::error(format!(
            "Couldn't find file name for file: {}",
            file.display()
        ));
        return None;
    };

    let Ok(markdown) = std::fs::read_to_string(file) else {
        log::error(format!("Failed to read input file: {}", file.display()));
        return None;
    };

    let filename = name.replace(&format!(".{INPUT_EXT}"), &format!(".{OUTPUT_EXT}"));
    let document = parse::parse_document(&markdown);
    let title = document_title(&document, &filename).unwrap_or_else(|| filename.clone());

    let file = File::new(tree, parent, &filename, title, document);

    create_outdir(outdir);
    let destination = outdir.join(filename);

    Some((file, destination))
}

fn copy_file(file: &Path, outdir: &Path) {
    let Some(name) = file.file_name() else {
        log::error(format!(
            "Couldn't find file name for file: {}",
            file.display()
        ));
        return;
    };

    create_outdir(outdir);
    let output = outdir.join(name);
    if let Err(e) = std::fs::copy(file, &output) {
        log::error(format!("Failed to copy file ({}): {e}", file.display()));
    } else {
        log::info(format!("Copied {} to {}", file.display(), output.display()));
    }
}

fn process_directory(
    tree: &mut FsTree,
    tree_exclude: bool,
    parent: usize,
    indir: &Path,
    outdir: &Path,
) -> Vec<(File, PathBuf)> {
    let Ok(dir) = std::fs::read_dir(indir) else {
        log::error(format!("Couldn't read directory: {}", indir.display()));
        return Vec::new();
    };

    log::info(format!(
        "Rendering {} to {}",
        indir.display(),
        outdir.display()
    ));

    let Some(name) = indir.file_name() else {
        fail(&format!("Couldn't read file name of {}", indir.display()));
    };

    let node = if tree_exclude {
        FsTree::ROOT
    } else {
        tree.add_dir(parent, name.to_string_lossy())
    };

    let mut documents = Vec::new();
    for entry in dir.flatten() {
        if let Ok(filetype) = entry.file_type() {
            let file_path = entry.path();
            if is_hidden(&file_path) {
            } else if filetype.is_dir() {
                let name = entry.file_name();
                let docs =
                    process_directory(tree, false, node, &indir.join(&name), &outdir.join(&name));
                documents.extend(docs);
            } else if filetype.is_file() {
                if let Some(Some(ext)) = file_path.extension().map(OsStr::to_str) {
                    if ext == INPUT_EXT {
                        if let Some(doc) = process_file(tree, node, &file_path, outdir) {
                            documents.push(doc);
                        }
                    } else if RESOURCE_EXTS.contains(&ext) {
                        copy_file(&file_path, outdir);
                    }
                }
            }
        }
    }
    documents
}

fn render_document(config: &Config, tree: &FsTree, file: &File, destination: &Path) {
    let html = render::render_document(config, tree, file);
    if std::fs::write(destination, html).is_ok() {
        if let Some(fsnode) = tree.get(file.fsnode()) {
            log::info(format!(
                "Rendered {} to {}",
                fsnode.url(),
                destination.display()
            ));
        }
    } else {
        log::error(format!("Failed to write file: {}", destination.display()));
    }
}

fn create_directory_index(
    tree: &FsTree,
    fsnode: &FsNode,
    outdir: &Path,
) -> Option<(usize, PathBuf, Vec<model::Node>)> {
    let children = tree.children(fsnode.id());
    if !children.is_empty() && !children.iter().any(|child| child.is_index()) {
        let document = render::create_index(fsnode, &children);

        let mut output = outdir.to_path_buf();
        for segment in fsnode.url().split('/').filter(|s| !s.is_empty()) {
            output.push(segment);
        }
        output.push(INDEX_FILE);

        Some((fsnode.id(), output, document))
    } else {
        None
    }
}

fn add_indexes(tree: &mut FsTree, outdir: &Path) -> Vec<(File, PathBuf)> {
    let indexes_to_add: Vec<(usize, PathBuf, Vec<model::Node>)> = tree
        .nodes()
        .iter()
        .filter(|node| node.is_dir())
        .filter_map(|fsnode| create_directory_index(tree, fsnode, outdir))
        .collect();

    indexes_to_add
        .into_iter()
        .map(|(parent, output, document)| {
            let title = tree.get(parent).map(|n| n.title()).unwrap().to_string();
            (
                File::new_index(tree, parent, INDEX_FILE, title, document),
                output,
            )
        })
        .collect()
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
    let mut tree = FsTree::new();

    if metadata.is_file() {
        let path = PathBuf::from(arg);
        let Some(parent) = path.parent() else {
            fail("Couldn't find parent directory of input file.");
        };

        if let Some((file, destination)) = process_file(&mut tree, 0, &path, parent) {
            render_document(&config, &tree, &file, &destination)
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
            let mut files = process_directory(&mut tree, true, 0, &indir, &outdir);

            if config.generate_indexes {
                log::info("Generating indexes for directories which don't have them.");
                files.extend(add_indexes(&mut tree, &outdir));
            }

            for (file, destination) in files {
                render_document(&config, &tree, &file, &destination);
            }
        } else {
            fail("Couldn't choose an output directory for files.");
        }
    }
}
