pub fn pack_string<const N: usize>(input: &str) -> [u8; N] {
    let mut slot: [u8; N] = [0u8; N];
    let input_bytes = input.as_bytes();
    let input_size = input_bytes.len();
    let len = std::cmp::min(input_size, N);
    slot[..len].copy_from_slice(&input_bytes[..len]);

    slot
}
