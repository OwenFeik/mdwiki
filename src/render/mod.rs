mod html;
mod nav;

#[cfg(test)]
mod test;

pub use self::html::render_document;
pub use self::nav::create_index;

pub const INDEX_FILE: &str = "index.html";
pub const OUTPUT_EXT: &str = "html";

fn capitalise_word(word: &str) -> String {
    match word {
        "a" | "and" | "at" | "is" | "of" | "to" => word.to_string(),
        _ if !word.is_empty() => format!(
            "{}{}",
            word.chars().next().unwrap().to_uppercase(),
            &word[1..]
        ),
        _ => word.to_string(),
    }
}

pub fn capitalise(title: &str) -> String {
    title
        .split(|c| c == ' ' || c == '-' || c == '_')
        .map(capitalise_word)
        .collect::<Vec<String>>()
        .join(" ")
}
