//! Exact original-layout oracle for the allocation-free bounded wrapper.
use super::*;

fn original_wrap(text: &str, columns: usize) -> Vec<String> {
    let text: String = text.chars().take(256).collect();
    let mut lines = Vec::new();
    for paragraph in text.split('\n') {
        let mut line = String::new();
        for word in paragraph.split_whitespace() {
            let chars: Vec<_> = word.chars().collect();
            if !line.is_empty() && line.chars().count() + 1 + chars.len() > columns {
                lines.push(std::mem::take(&mut line));
            }
            for chunk in chars.chunks(columns) {
                if !line.is_empty() {
                    line.push(' ');
                }
                line.extend(chunk);
                if line.chars().count() >= columns {
                    lines.push(std::mem::take(&mut line));
                }
            }
        }
        if !line.is_empty() || paragraph.is_empty() {
            lines.push(line);
        }
    }
    lines
}

fn compare(text: &str, columns: usize) {
    let expected = original_wrap(text, columns);
    let actual = wrap(text, columns);
    let lines: Vec<String> = actual.iter().map(|line| line.chars().collect()).collect();
    assert_eq!(lines, expected, "columns={columns}, text={text:?}");
    assert!(actual.used <= TEXT_CHARS);
    assert!(actual.lines <= TEXT_LINES);
}

#[test]
fn original_unicode_whitespace_empty_paragraphs_long_words_and_prefix_limits_match() {
    let texts = [
        "".to_string(),
        "  \t\r  ".to_string(),
        "\n".to_string(),
        "\n\n A \n\n".to_string(),
        "ONE TWO THREE LONGWORD".to_string(),
        "A\u{00a0}B\u{2003}C\u{202f}D\u{0085}E\u{2028}F".to_string(),
        "é😀Ω日本語 X\n Z".to_string(),
        "#".repeat(300),
        "\n".repeat(300),
        "longword".repeat(40),
        "é ".repeat(180),
        "a\r\nb\r\n\r\nc".to_string(),
    ];
    for text in texts {
        for columns in [1, 2, 3, 5, 7, 11, 25, 128, 256, 1024] {
            compare(&text, columns);
        }
    }
    let worst = wrap(&"\n".repeat(256), 1);
    assert_eq!(worst.lines, 257);
    assert_eq!(worst.used, 0);
}

#[test]
fn exhaustive_short_whitespace_sequences_match_the_actual_original_wrapper() {
    let alphabet = ['A', 'B', ' ', '\t', '\n', '\r', '\u{00a0}'];
    for length in 0..=5 {
        for mut number in 0..alphabet.len().pow(length) {
            let mut text = String::new();
            for _ in 0..length {
                text.push(alphabet[number % alphabet.len()]);
                number /= alphabet.len();
            }
            for columns in 1..=5 {
                compare(&text, columns);
            }
        }
    }
}

#[test]
fn seeded_full_prefix_unicode_corpus_preserves_exact_line_content_and_boundaries() {
    let alphabet = ['A', ' ', '\t', '\n', '\r', '\u{2003}', 'Ω', '😀', '#'];
    let mut state = 0x542ea931u32;
    for case in 0..300 {
        let mut text = String::new();
        for _ in 0..(240 + case % 61) {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            text.push(alphabet[state as usize % alphabet.len()]);
        }
        for columns in [1, 2, 5, 11, 25, 63, 256] {
            compare(&text, columns);
        }
    }
}

#[test]
fn bounded_storage_handles_maximum_character_and_line_prefixes() {
    let words = wrap(&"I ".repeat(128), 1);
    assert_eq!(words.used, 128);
    assert_eq!(words.lines, 128);
    let maximum = wrap(&"Ω".repeat(300), 1);
    assert_eq!(maximum.used, 256);
    assert_eq!(maximum.lines, 256);
    assert!(maximum.iter().all(|line| line.len() == 1));
    assert_eq!(std::mem::size_of::<[char; TEXT_CHARS]>(), 1024);
    assert!(std::mem::size_of::<WrappedLines>() <= 2176);
    println!(
        "Native bounded WrappedLines={} bytes; raw256-char input={} bytes. These are object sizes, not measured stack high-water/WASM acceptance.",
        std::mem::size_of::<WrappedLines>(),
        std::mem::size_of::<[char; TEXT_CHARS]>()
    );
}
