pub fn wrap_paragraph(paragraph: &str, width: usize) -> String {
    let mut result = Vec::new();
    let mut current_line = String::new();

    for word in paragraph.split_whitespace() {
        let potential_length = if current_line.is_empty() {
            word.len()
        } else {
            current_line.len() + 1 + word.len() // +1 for the space
        };

        if potential_length <= width {
            // Word fits on current line
            if !current_line.is_empty() {
                current_line.push(' ');
            }
            current_line.push_str(word);
        } else {
            // Word doesn't fit, start a new line
            if !current_line.is_empty() {
                result.push(current_line.clone());
                current_line.clear();
            }

            // Handle words longer than the width
            if word.len() > width {
                // Split the word
                let mut remaining = word;
                while remaining.len() > width {
                    result.push(remaining[..width].to_string());
                    remaining = &remaining[width..];
                }
                if !remaining.is_empty() {
                    current_line = remaining.to_string();
                }
            } else {
                current_line = word.to_string();
            }
        }
    }

    if !current_line.is_empty() {
        result.push(current_line);
    }

    result.join("\n")
}

pub fn wrap_text(text: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }

    let mut result = Vec::new();

    let paragraphs: Vec<&str> = text.split("\n\n").collect();

    for paragraph in paragraphs {
        let wrapped_paragraph = wrap_paragraph(paragraph, width);
        result.push(wrapped_paragraph);
    }

    result.join("\n\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_text() {
        let text = "This is a sample text that should be wrapped at a specific width";
        let wrapped = wrap_text(text, 20);
        let expected = "This is a sample\ntext that should be\nwrapped at a\nspecific width";
        assert_eq!(wrapped, expected);
    }

    #[test]
    fn test_wrap_text_short() {
        let text = "Short";
        let wrapped = wrap_text(text, 20);
        assert_eq!(wrapped, "Short");
    }

    #[test]
    fn test_wrap_text_long_word() {
        let text = "Supercalifragilisticexpialidocious word";
        let wrapped = wrap_text(text, 10);
        let expected = "Supercalif\nragilistic\nexpialidoc\nious word";
        assert_eq!(wrapped, expected);
    }

    #[test]
    fn test_wrap_text_zero_width() {
        let text = "Some text";
        let wrapped = wrap_text(text, 0);
        assert_eq!(wrapped, "");
    }

    #[test]
    fn test_wrap_text_with_spaces() {
        let text = "Some text\n\nwith multiple\n\nnew lines";
        let wrapped = wrap_text(text, 10);
        let expected = "Some text\n\nwith\nmultiple\n\nnew lines";
        assert_eq!(wrapped, expected);
    }

    #[test]
    fn test_wrap_paragraph() {
        let paragraph = "This is a test paragraph to check the wrapping functionality.\nThis is a second line.";
        let wrapped = wrap_paragraph(paragraph, 72);
        let expected = "This is a test paragraph to check the wrapping functionality. This is a\nsecond line.";
        assert_eq!(wrapped, expected);
    }

    #[test]
    fn test_complete() {
        let text = "refactor: migrate git config to new settings format\n\
        \n\
        The change moves git configuration from legacy format (userName, userEmail, \
        extraConfig) to the new consolidated settings format while preserving the same functionality.";
        let wrapped = wrap_text(text, 72);
        let expected = "refactor: migrate git config to new settings format\n\n\
        The change moves git configuration from legacy format (userName,\n\
        userEmail, extraConfig) to the new consolidated settings format while\n\
        preserving the same functionality.";
        assert_eq!(wrapped, expected);
    }
}
