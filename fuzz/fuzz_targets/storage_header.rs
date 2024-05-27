#![no_main]

extern crate modality_dlt;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
   let _ = modality_dlt::consume_dlt_storage_header_sync(data);
});
