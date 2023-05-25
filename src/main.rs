#![feature(pattern)]

mod model;
mod parse;
mod render;

#[cfg(test)]
mod test;

fn main() {
    let file = std::env::args().nth(1).expect(
        "Usage: mdwiki file.md"
    );

    let markdown = std::fs::read_to_string(&file).expect(
        "Failed to read file"
    );

    let document = parse::parse_document(&markdown);
    let html = render::render_document(&document);

    let output = file.replace(".md", ".html");
    std::fs::write(output, html).expect(
        "Failed to write file"
    );
}
