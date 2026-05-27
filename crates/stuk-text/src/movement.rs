pub(crate) fn char_len(text: &str) -> usize {
    text.chars().count()
}

pub(crate) fn char_to_byte_index(text: &str, char_index: usize) -> usize {
    if char_index == 0 {
        return 0;
    }
    text.char_indices()
        .nth(char_index)
        .map(|(index, _)| index)
        .unwrap_or(text.len())
}

pub(crate) fn previous_word_boundary(text: &str, offset: usize) -> usize {
    let chars = text.chars().collect::<Vec<_>>();
    let mut index = offset.min(chars.len());

    while index > 0 && !is_word_char(chars[index - 1]) {
        index -= 1;
    }
    while index > 0 && is_word_char(chars[index - 1]) {
        index -= 1;
    }
    index
}

pub(crate) fn next_word_boundary(text: &str, offset: usize) -> usize {
    let chars = text.chars().collect::<Vec<_>>();
    let mut index = offset.min(chars.len());

    while index < chars.len() && !is_word_char(chars[index]) {
        index += 1;
    }
    while index < chars.len() && is_word_char(chars[index]) {
        index += 1;
    }
    index
}

pub(crate) fn line_start_boundary(text: &str, offset: usize) -> usize {
    let chars = text.chars().collect::<Vec<_>>();
    let mut index = offset.min(chars.len());

    while index > 0 && chars[index - 1] != '\n' {
        index -= 1;
    }
    index
}

pub(crate) fn line_end_boundary(text: &str, offset: usize) -> usize {
    let chars = text.chars().collect::<Vec<_>>();
    let mut index = offset.min(chars.len());

    while index < chars.len() && chars[index] != '\n' {
        index += 1;
    }
    index
}

fn is_word_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}
