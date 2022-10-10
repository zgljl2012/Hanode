use std::convert::TryInto;

pub fn read_be_i32(input: &[u8]) -> i32 {
    i32::from_be_bytes(input.try_into().expect("Read from input buffer is failed"))
}
