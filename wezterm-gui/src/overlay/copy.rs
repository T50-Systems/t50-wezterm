include!("copy/types.rs");
include!("copy/copy_overlay.rs");
include!("copy/renderable.rs");
include!("copy/pane.rs");
include!("copy/writer.rs");

fn is_whitespace_word(word: &str) -> bool {
    word.chars()
        .next()
        .map(char::is_whitespace)
        .unwrap_or(false)
}
