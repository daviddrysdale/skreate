#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(input) = std::str::from_utf8(data) else {
        return;
    };
    if !input.is_ascii() {
        return;
    }
    let _result = skreate::generate_with_positions(input);
});
