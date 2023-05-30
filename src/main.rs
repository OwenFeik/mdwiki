#![feature(pattern)]

use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

mod log;
mod model;
mod parse;
mod render;

#[cfg(test)]
mod test;

const INPUT_EXT: &str = "md";
const OUTPUT_EXT: &str = "html";

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
        log::warning(&format!(
            "Failed to create output directory: {}",
            outdir.display()
        ));
    }

    let output = outdir.join(name.replace(&format!(".{INPUT_EXT}"), &format!(".{OUTPUT_EXT}")));
    if std::fs::write(&output, html).is_ok() {
        log::info(&format!(
            "Rendered {} to {}",
            file.display(),
            output.display()
        ));
    } else {
        log::error(&format!("Failed to write file: {}", output.display()));
    }
}

fn process_directory(indir: &Path, outdir: &Path) {
    let Ok(dir) = std::fs::read_dir(indir) else {
        log::error(&format!("Couldn't read directory: {}", indir.display()));
        return;
    };

    log::info(&format!(
        "Rendering {} to {}",
        indir.display(),
        outdir.display()
    ));

    for entry in dir {
        if let Ok(entry) = entry {
            if let Ok(filetype) = entry.file_type() {
                if filetype.is_dir() {
                    process_directory(
                        &indir.join(entry.file_name()),
                        &outdir.join(entry.file_name()),
                    );
                } else if filetype.is_file() {
                    let path = entry.path();
                    if let Some(Some(ext)) = path.extension().map(OsStr::to_str) {
                        if ext == INPUT_EXT {
                            process_file(&entry.path(), outdir);
                        }
                    }
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

        let Some(Some(dirname)) = indir.file_name().map(OsStr::to_str) else {
            fail("Couldn't find filename of input directory.");
        };

        if let Some(parent) = indir.parent() {
            let outdir = parent.join(format!("{dirname}-{OUTPUT_EXT}"));
            process_directory(&indir, &outdir);
        } else {
            fail("Couldn't choose an output directory for files.");
        }
    }
}
