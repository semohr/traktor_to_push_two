use std::time::Instant;

use super::{DISPLAY_HEIGHT, DISPLAY_WIDTH, FRAME_SIZE, LINE_SIZE};

pub fn padding(buffer: &[u16]) -> Vec<u8> {
    let mut padded: Vec<u8> = vec![0;FRAME_SIZE];
    let buffer = u16_to_le_bytes(buffer);

    // Pixels are encoded in 16 bit
    for (i, chunk) in buffer.chunks_exact(DISPLAY_WIDTH * 2).enumerate() {
        let padding_start = i * LINE_SIZE * 2;
        padded[padding_start..padding_start + DISPLAY_WIDTH * 2].copy_from_slice(chunk);
    }
    padded
}

pub fn padding_org(buffer: &[u16]) -> [u8; FRAME_SIZE] {
    let mut padded: [u8; FRAME_SIZE] = [0; FRAME_SIZE];

    for r in 0..DISPLAY_HEIGHT {
        for c in 0..DISPLAY_WIDTH {
            let i = r * DISPLAY_WIDTH + c;
            let b: [u8; 2] = buffer[i].to_le_bytes();
            let di = r * LINE_SIZE * 2 + c * 2;
            padded[di] = b[0];
            padded[di + 1] = b[1];
        }
    }
    padded
}

fn u16_to_le_bytes(input: &[u16]) -> &[u8] {
    // Safety: Ensure that the input slice is properly aligned for u16 access
    assert_eq!(input.as_ptr() as usize % std::mem::align_of::<u16>(), 0);

    // Convert each u16 to little endian u8 and reinterpret the slice
    let len = input.len() * std::mem::size_of::<u16>();
    let ptr = input.as_ptr() as *const u8;
    unsafe { std::slice::from_raw_parts(ptr, len) }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_padding() {
        let buffer: Vec<u16> = vec![std::u16::MAX; DISPLAY_HEIGHT * DISPLAY_WIDTH]; // Example buffer with all elements set to std::u16::MAX

        let padded = padding(&buffer);

        // 1. Length test
        assert_eq!(padded.len(), FRAME_SIZE);

        // 2. Content validation (for simplicity, just check some known values)
        for c in padded.chunks_exact(LINE_SIZE * 2) {
            assert!(c[0..DISPLAY_WIDTH * 2].iter().all(|&x| x == std::u8::MAX));
            assert!(c[DISPLAY_WIDTH * 2..LINE_SIZE * 2]
                .iter()
                .all(|&x| x == 0u8));
        }
    }

    #[test]
    fn test_padding_org() {
        let buffer: Vec<u16> = vec![std::u16::MAX; DISPLAY_HEIGHT * DISPLAY_WIDTH]; // Example buffer with all elements set to std::u16::MAX

        let padded = padding_org(&buffer);

        // 1. Length test
        assert_eq!(padded.len(), FRAME_SIZE);

        // 2. Content validation (for simplicity, just check some known values)
        for c in padded.chunks_exact(LINE_SIZE * 2) {
            assert!(c[0..DISPLAY_WIDTH * 2].iter().all(|&x| x == std::u8::MAX));
            assert!(c[DISPLAY_WIDTH * 2..LINE_SIZE * 2]
                .iter()
                .all(|&x| x == 0u8));
        }
    }

    #[test]
    fn test_padding_equal() {
        let buffer: Vec<u16> = vec![std::u16::MAX; DISPLAY_HEIGHT * DISPLAY_WIDTH]; // Example buffer with all elements set to std::u16::MAX

        let buffer2: Vec<u16> = vec![std::u16::MAX; DISPLAY_HEIGHT * DISPLAY_WIDTH];
        let padded1 = padding(&buffer2);
        let padded2 = padding_org(&buffer);

        // Check if both padded arrays are equal
        assert_eq!(padded1.len(), padded2.len());
        assert_eq!(&padded1[..], &padded2[..]);
    }

    #[test]
    fn test_perf() {
        fn test() {
            let buffer: Vec<u16> = vec![std::u16::MAX; DISPLAY_HEIGHT * DISPLAY_WIDTH]; // Example buffer with all elements set to std::u16::MAX
            padding(&buffer);
        }

        measure_execution_time(test, 1000);

        fn test2() {
            let buffer: Vec<u16> = vec![std::u16::MAX; DISPLAY_HEIGHT * DISPLAY_WIDTH]; // Example buffer with all elements set to std::u16::MAX
            padding_org(&buffer);
        }
        measure_execution_time(test2, 1000);
    }
}

// Function to run another function n times and record the execution times
pub fn measure_execution_time<F>(func: F, n: usize)
where
    F: Fn(),
{
    let mut durations: Vec<u128> = Vec::with_capacity(n);

    for _ in 0..n {
        let start = Instant::now();
        func();
        durations.push(start.elapsed().as_nanos());
    }

    // Calculate mean
    let total_duration: u128 = durations.iter().sum();
    let mean_duration = total_duration / (n as u128);

    // Calculate median
    durations.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let median_duration = if n % 2 == 0 {
        (durations[n / 2 - 1] + durations[n / 2]) / 2
    } else {
        durations[n / 2]
    };

    println!("Mean execution time: {:?}", mean_duration / 1000);
    println!("Median execution time: {:?}", median_duration / 1000);
}
