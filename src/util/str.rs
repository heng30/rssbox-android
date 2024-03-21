pub fn split_string_to_fixed_length_parts(input: &str, length: usize) -> Vec<String> {
    input
        .chars()
        .collect::<Vec<_>>()
        .chunks(length)
        .map(|chunk| chunk.iter().collect::<String>())
        .collect()
}
