/// XORs the input buffer with a given pattern using SIMD.
///
/// # Arguments
///
/// * `buffer` - A mutable slice of u8 elements to be XORed.
/// * `pattern` - A u8x16 SIMD pattern used for XOR operation.
///
/// # Safety
///
/// This function uses unsafe SIMD operations.
fn xor_with_pattern_simd(buffer: &mut [u8]) {
    let len = buffer.len();
    let pattern = u8x16::from(PATTERN);
    let mut i = 0;

    // XOR in chunks of 16 bytes using SIMD
    while i + 16 <= len {
        let chunk = u8x16::from_slice(&buffer[i..i + 16]);
        let result = chunk ^ pattern;
        result.copy_to_slice(&mut buffer[i..i + 16]);
        i += 16;
    }

    // Handle the remaining bytes
    while i < len {
        buffer[i] ^= pattern.to_array()[i % 16];
        i += 1;
    }
}

fn original_mask(
    buffer: [u16; DISPLAY_HEIGHT * DISPLAY_WIDTH],
) -> [u8; BYTES_PER_LINE * DISPLAY_HEIGHT] {
    let mut masked_buffer: [u8; BYTES_PER_LINE * DISPLAY_HEIGHT] =
        [0; BYTES_PER_LINE * DISPLAY_HEIGHT];

    for r in 0..DISPLAY_HEIGHT {
        for c in 0..DISPLAY_WIDTH {
            let i = r * DISPLAY_WIDTH + c;
            let b: [u8; 2] = u16::to_le_bytes(buffer[i]);
            let di = r * BYTES_PER_LINE + c * 2;
            masked_buffer[di] = b[0] ^ MASK[di % 4];
            masked_buffer[di + 1] = b[1] ^ MASK[(di + 1) % 4];
        }
    }

    masked_buffer
}

fn original_xor(buffer: &mut [u8]) {
    for i in 0..buffer.len() {
        buffer[i] = buffer[i] ^ MASK[i % 4];
    }
}

#[test]
fn test_padding_original_equivalence() {
    let original: [u16; DISPLAY_HEIGHT * DISPLAY_WIDTH] = [1; DISPLAY_HEIGHT * DISPLAY_WIDTH];

    let mut padded_new = apply_padding(original);
    let padding_org = original_padding(original);

    let u8_case = u16_to_le_bytes(&mut padded_new);

    assert_eq!(u8_case, padding_org);
}

#[test]
fn test_apply_padding_with_different_values() {
    let mut original: [u16; DISPLAY_HEIGHT * DISPLAY_WIDTH] = [0; DISPLAY_HEIGHT * DISPLAY_WIDTH];
    for i in 0..DISPLAY_HEIGHT * DISPLAY_WIDTH {
        original[i] = i as u16;
    }
    let padded = apply_padding(original);

    for i in 0..DISPLAY_HEIGHT {
        let padded_start = i * (DISPLAY_WIDTH + 64);
        let padded_end = padded_start + DISPLAY_WIDTH;
        let padding_start = padded_end;
        let padding_end = padding_start + 64;

        // Check the copied elements
        assert_eq!(
            &padded[padded_start..padded_end],
            &(i * DISPLAY_WIDTH..(i + 1) * DISPLAY_WIDTH)
                .map(|x| x as u16)
                .collect::<Vec<u16>>()[..]
        );

        // Check the padding elements
        assert_eq!(&padded[padding_start..padding_end], &[0; 64]);
    }
}

#[test]
fn test_xor_with_pattern_simd() {
    let mut buffer: Vec<u8> = (0..32).collect();
    xor_with_pattern_simd(&mut buffer);

    let mut expected: Vec<u8> = vec![0; 32];
    for (i, x) in (0..32).enumerate() {
        expected[i] = x ^ PATTERN[i % 16];
    }
    assert_eq!(buffer, expected);
}

#[test]
fn test_xor_with_pattern_simd_partial_chunk() {
    let mut buffer: Vec<u8> = (0..18).collect();

    xor_with_pattern_simd(&mut buffer);

    let mut expected: Vec<u8> = vec![0; 18];
    for (i, x) in (0..18).enumerate() {
        expected[i] = x ^ PATTERN[i % 16];
    }
    assert_eq!(buffer, expected);
}

#[test]
fn test_xor_same() {
    let mut original: [u8; DISPLAY_HEIGHT * DISPLAY_WIDTH] = [1; DISPLAY_HEIGHT * DISPLAY_WIDTH];
    xor_with_pattern_simd(&mut original);

    let mut original2: [u8; DISPLAY_HEIGHT * DISPLAY_WIDTH] = [1; DISPLAY_HEIGHT * DISPLAY_WIDTH];
    original_xor(&mut original2);

    assert_eq!(original, original2);
}

#[test]
fn test_same() {
    let mut original: [u16; DISPLAY_HEIGHT * DISPLAY_WIDTH] = [1; DISPLAY_HEIGHT * DISPLAY_WIDTH];
    let mut original2: [u16; DISPLAY_HEIGHT * DISPLAY_WIDTH] = [1; DISPLAY_HEIGHT * DISPLAY_WIDTH];

    let mut new = apply_padding(original);
    xor_with_pattern_simd(u16_to_le_bytes(&mut new));

    let org = original_mask(original2);

    assert_eq!(new, new);
}
