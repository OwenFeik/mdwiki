use std::{ffi::OsStr, path::Path};

use crate::{
    log,
    model::WikiTree,
    render::{capitalise, INDEX_FILE, OUTPUT_EXT},
};

const INPUT_EXT: &str = "md";
pub const IMAGE_EXTS: &[&str] = &["jpg", "jpeg", "png"];

fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|name| name.starts_with('.'))
        .unwrap_or(false)
}

fn title_from_filename(filename: &str) -> String {
    filename
        .split('.')
        .next()
        .map(capitalise)
        .unwrap_or_else(|| capitalise(filename))
}

pub fn process_document(tree: &mut WikiTree, parent: usize, file: &Path) {
    let Some(Some(name)) = file.file_name().map(std::ffi::OsStr::to_str) else {
        log::error(format!(
            "Couldn't find file name for file: {}",
            file.display()
        ));
        return;
    };

    let Ok(markdown) = std::fs::read_to_string(file) else {
        log::error(format!("Failed to read input file: {}", file.display()));
        return;
    };

    let filename = name.replace(&format!(".{INPUT_EXT}"), &format!(".{OUTPUT_EXT}"));
    let document = super::md::parse_document(&markdown);
    let title = document
        .title_from_heading()
        .unwrap_or_else(|| title_from_filename(&filename));

    if filename == INDEX_FILE {
        tree.add_index(parent, filename, title, document);
    } else {
        tree.add_doc(parent, filename, title, document);
    }
}

pub fn process_directory(
    tree: &mut WikiTree,
    tree_exclude: bool,
    parent: usize,
    indir: &Path,
    outdir: &Path,
) {
    let Ok(dir) = std::fs::read_dir(indir) else {
        log::error(format!("Couldn't read directory: {}", indir.display()));
        return;
    };

    let Some(name) = indir.file_name() else {
        log::error(format!("Couldn't read file name of {}", indir.display()));
        return;
    };

    let node = if tree_exclude {
        WikiTree::ROOT
    } else {
        tree.add_dir(parent, name.to_string_lossy())
    };

    for entry in dir.flatten() {
        if let Ok(filetype) = entry.file_type() {
            let file_path = entry.path();
            if is_hidden(&file_path) {
            } else if filetype.is_dir() {
                let name = entry.file_name();
                process_directory(tree, false, node, &indir.join(&name), &outdir.join(&name));
            } else if filetype.is_file() {
                if let Some(Some(ext)) = file_path.extension().map(OsStr::to_str) {
                    if ext == INPUT_EXT {
                        process_document(tree, node, &file_path);
                    } else if IMAGE_EXTS.contains(&ext) {
                        if let Some(name) = file_path
                            .file_name()
                            .and_then(OsStr::to_str)
                            .map(|s| s.to_string())
                        {
                            let title = title_from_filename(&name);
                            tree.add_media(node, name, title, file_path);
                        }
                    }
                }
            }
        }
    }
}
