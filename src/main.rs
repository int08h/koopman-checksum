use koopman_checksum::{koopman16, koopman8};

// counterexamples from TethysSvensson
fn main() {
    // These two messages are 4095 bytes and have a hamming distance of 2
    // The crate claims to be able to detect hamming distances of up to 3
    // for messages up to 4096 bytes using this checksum
    let mut a = [0; 4092];
    a[0] = 0x80;
    let mut b = [0; 4092];
    b[4091] = 1;
    // assert_eq!(koopman16(&a, 0), koopman16(&b, 0));

    // These two messages are 2 bytes and have a hamming distance of 3
    // The crate claims to be able to detect hamming distances of up to 3
    // for messages up to 13 bytes using this checksum
    let a = [1, 0];
    let b = [0, 3];
    for i in 0..=255 {
        if koopman8(&a, i) == koopman8(&b, i) {
            println!("failure with seed {i:b}");
        }
    }

}
