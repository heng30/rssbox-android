use md5;

pub fn md5_hex(text: &str) -> String {
    format!("{:X}", md5::compute(text))
}
