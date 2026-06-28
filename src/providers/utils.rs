pub(super) fn extract_first_number(input: &str) -> String {
    let mut digits = String::new();
    let mut in_match = false;

    for character in input.chars() {
        if character.is_ascii_digit() {
            digits.push(character);
            in_match = true;
        } else if in_match {
            break;
        }
    }

    digits
}

pub(super) fn digits_only(input: &str) -> String {
    input
        .chars()
        .filter(|character| character.is_ascii_digit())
        .collect()
}

pub(super) fn encode_path_segment(input: &str) -> String {
    input.replace('%', "%25").replace('#', "%23")
}
