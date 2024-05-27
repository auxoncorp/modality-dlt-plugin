#![no_main]

extern crate modality_dlt;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
   let _ = modality_dlt::read_dlt_message_sync(data);
});

