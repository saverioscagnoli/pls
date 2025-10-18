pub fn keep_ascii_letters_and_whitespace(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_ascii_alphabetic() || c.is_whitespace())
        .collect()
}
