#![no_std]
#![no_main]

mod memory_allocations;
zkm_runtime::entrypoint!(main);
use memory_allocations::{alloc_vec, push_vec, pop_vec};

pub fn main() {
    let mut vvec = alloc_vec();

    push_vec(&mut vvec);

    pop_vec(&mut vvec);
}
