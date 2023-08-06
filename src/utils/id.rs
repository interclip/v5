use rand::Rng;

/// Generate an alphanumeric ID, n letters long
pub fn gen_id(length: usize) -> String {
    let mut code = String::new();
    let mut rng = rand::thread_rng();
    let chars: Vec<char> = "abcdefghijklmnopqrstuvwxyz0123456789".chars().collect();

    for _ in 0..length {
        let random_char = rng.gen_range(0..chars.len());
        code.push(chars[random_char]);
    }

    code
}
