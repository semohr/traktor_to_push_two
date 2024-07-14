use std::{simd::*, time::Instant};

const MASK: [u8; 4] = [0xe7, 0xf3, 0xe7, 0xff];

#[rustfmt::skip]
const PATTERN :[u8;16] = [
   0xe7, 0xf3, 0xe7, 0xff, 
   0xe7, 0xf3, 0xe7, 0xff, 
   0xe7, 0xf3, 0xe7, 0xff, 
   0xe7, 0xf3, 0xe7, 0xff, 
];

pub fn xor(buffer: &mut [u8]) {
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

pub fn xor_org(buffer: &mut [u8]) {
    for i in 0..buffer.len() {
        buffer[i] = buffer[i] ^ MASK[i % 4];
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_xor() {
        let mut buffer: Vec<u8> = (0..32).collect(); // Example buffer with all elements set to std::u8::MAX

        xor(&mut buffer);

        let mut expected: Vec<u8> = vec![0; 32];
        for (i, x) in (0..32).enumerate() {
            expected[i] = x ^ PATTERN[i % 16];
        }
        assert_eq!(buffer, expected);
    }

    #[test]
    fn test_xor_org() {
        let mut buffer: Vec<u8> = (0..32).collect(); // Example buffer with all elements set to std::u8::MAX

        xor_org(&mut buffer);

        let mut expected: Vec<u8> = vec![0; 32];
        for (i, x) in (0..32).enumerate() {
            expected[i] = x ^ PATTERN[i % 16];
        }
        assert_eq!(buffer, expected);
    }

    #[test]
    fn test_xor_equal() {
        let mut original: [u8; 160 * 920] = [1; 160 * 920];
        xor(&mut original);

        let mut original2: [u8; 160 * 920] = [1; 160 * 920];
        xor_org(&mut original2);

        assert_eq!(original, original2);
    }

    #[test]
    fn test_perf() {
        fn test() {
            let mut buffer: Vec<u8> = vec![std::u8::MAX; 160 * 920]; // Example buffer with all elements set to std::u8::MAX
            xor(&mut buffer);
        }

        measure_execution_time(test, 1000);

        fn test2() {
            let mut buffer: Vec<u8> = vec![std::u8::MAX; 160 * 920]; // Example buffer with all elements set to std::u16::MAX
            xor_org(&mut buffer);
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
