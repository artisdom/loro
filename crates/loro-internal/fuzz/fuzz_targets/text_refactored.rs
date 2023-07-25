#![no_main]
use libfuzzer_sys::fuzz_target;
use loro_internal::fuzz::{test_multi_sites_refactored, Action};

fuzz_target!(|actions: Vec<Action>| { test_multi_sites_refactored(8, &mut actions.clone()) });