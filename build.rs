fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    // This needs to be in build.rs as it takes too long to run in MIR, and hits the const_eval_limit
    // This also means it doesn't need recalculated every time I compile, which is nice
    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let path = std::path::Path::new(&out_dir).join("hex.rs");
    let lookup_table = format!("{:?}", generate_hex_lookup_table());
    std::fs::write(&path, format!("pub const HEX_LOOKUP: [[u8; u8::MAX as usize + 1]; u8::MAX as usize + 1] = {};", lookup_table)).unwrap();
}

const fn generate_hex_lookup_table() -> [[u8; u8::MAX as usize + 1]; u8::MAX as usize + 1] {
    let mut output = [[0u8; u8::MAX as usize + 1]; u8::MAX as usize + 1];
    let mut input = 0;
    while input < u16::MAX {
        let left = ((input & 0xFF00) >> 8) as u8;
        let right = (input & 0x00FF) as u8;
        output[left as usize][right as usize] = map_hex_char(left) << 4 | map_hex_char(right);
        input += 1;
    }
    output
}

const fn map_hex_char(input: u8) -> u8 {
    if (input) >= b'0' && input <= b'9' {
        input - b'0'
    } else if input >= b'A' && input <= b'F' {
        input - b'A' + 10
    } else if input >= b'a' && input <= b'f' {
        input - b'a' + 10
    } else {
        0
    }
}