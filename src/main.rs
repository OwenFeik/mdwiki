#![feature(let_else)]
#![feature(pattern)]

use std::path::{Path, PathBuf};

mod log;
mod model;
mod parse;
mod render;

#[cfg(test)]
mod test;

fn process_file(file: &Path, outdir: &Path) {
    let Some(Some(name)) = file.file_name().map(std::ffi::OsStr::to_str) else {
        log::error(
            &format!("Couldn't find file name for file: {}", file.display())
        );
        return;
    };

    let Ok(markdown) = std::fs::read_to_string(file) else {
        log::error(&format!("Failed to read input file: {}", file.display()));
        return;
    };

    let document = parse::parse_document(&markdown);
    let html = render::render_document(&document);

    if std::fs::create_dir_all(outdir).is_err() {
        log::warning(
            &format!("Failed to create output directory: {}", outdir.display())
        );
    }

    let output = outdir.join(name.replace(".md", ".html"));
    if std::fs::write(&output, html).is_ok() {
        log::info(&format!("Rendered {} to {}", file.display(), output.display()));
    } else {
        log::error(&format!("Failed to write file: {}", output.display()));
    }
}

fn process_directory(indir: &Path, outdir: &Path) {
    let Ok(dir) = std::fs::read_dir(indir) else {
        log::error(&format!("Couldn't read directory: {}", indir.display()));
        return;
    };

    log::info(
        &format!("Rendering {} to {}", indir.display(), outdir.display())
    );

    for entry in dir {
        if let Ok(entry) = entry {
            if let Ok(filetype) = entry.file_type() {
                if filetype.is_dir() {
                    process_directory(
                        &indir.join(entry.file_name()),
                        &outdir.join(entry.file_name())
                    );
                }
                else if filetype.is_file() {
                    process_file(&entry.path(), outdir);
                }
            }
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
        fail("Couldn't read argument file metadata")
    };

    if metadata.is_file() {
        let path = PathBuf::from(arg);
        let Some(parent) = path.parent() else {
            fail("Couldn't find parent directory of input file.");
        };
        process_file(&path, parent);
    } else if metadata.is_dir() {
        let path = PathBuf::from(arg);
        let indir = path.clone();
        let Some(outdir) = &indir.parent().map(|p| p.join("build")) else {
            fail("Couldn't choose an output directory for files.");
        };
        process_directory(&indir, outdir);
    }
}
