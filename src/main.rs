#![feature(pattern)]

use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use config::Config;

mod config;
mod log;
mod model;
mod parse;
mod render;

const INPUT_EXT: &str = "md";
const RESOURCE_EXTS: &[&str] = &["jpg", "jpeg", "png"];
const OUTPUT_EXT: &str = "html";

fn create_outdir(outdir: &Path) {
    if std::fs::create_dir_all(outdir).is_err() {
        log::warning(&format!(
            "Failed to create output directory: {}",
            outdir.display()
        ));
    }
}

fn process_file(config: &Config, path: &[String], file: &Path, outdir: &Path) {
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
    let html = render::render_document(config, path, &document);

    create_outdir(outdir);
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

fn copy_file(file: &Path, outdir: &Path) {
    let Some(name) = file.file_name() else {
        log::error(
            &format!("Couldn't find file name for file: {}", file.display())
        );
        return;
    };

    create_outdir(outdir);
    let output = outdir.join(name);
    if let Err(e) = std::fs::copy(file, &output) {
        log::error(&format!("Failed to copy file ({}): {e}", file.display()));
    } else {
        log::info(&format!(
            "Copied {} to {}",
            file.display(),
            output.display()
        ));
    }
}

fn process_directory(config: &Config, path: Vec<String>, indir: &Path, outdir: &Path) {
    let Ok(dir) = std::fs::read_dir(indir) else {
        log::error(&format!("Couldn't read directory: {}", indir.display()));
        return;
    };

    log::info(&format!(
        "Rendering {} to {}",
        indir.display(),
        outdir.display()
    ));

    for entry in dir.flatten() {
        if let Ok(filetype) = entry.file_type() {
            if filetype.is_dir() {
                let name = entry.file_name();
                let mut path = path.clone();
                path.push(name.to_string_lossy().into_owned());
                process_directory(config, path, &indir.join(&name), &outdir.join(&name));
            } else if filetype.is_file() {
                let file_path = entry.path();
                if let Some(Some(ext)) = file_path.extension().map(OsStr::to_str) {
                    if ext == INPUT_EXT {
                        process_file(config, &path, &file_path, outdir);
                    } else if RESOURCE_EXTS.contains(&ext) {
                        copy_file(&file_path, outdir);
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
        fail("Couldn't read argument file metadata.")
    };

    let config = Config::default();

    if metadata.is_file() {
        let path = PathBuf::from(arg);
        let Some(parent) = path.parent() else {
            fail("Couldn't find parent directory of input file.");
        };
        process_file(&config, &[], &path, parent);
    } else if metadata.is_dir() {
        let indir = PathBuf::from(arg);
        let Some(Some(dirname)) = indir.file_name().map(OsStr::to_str) else {
            fail("Couldn't find filename of input directory.");
        };

        if let Some(parent) = indir.parent() {
            let outdir = parent.join(format!("{dirname}-{OUTPUT_EXT}"));
            process_directory(&config, Vec::new(), &indir, &outdir);
        } else {
            fail("Couldn't choose an output directory for files.");
        }
    }
}
