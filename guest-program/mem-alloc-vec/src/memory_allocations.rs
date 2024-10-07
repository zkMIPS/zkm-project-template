// copied from https://github.com/vacp2p/nescience-zkvm-testing/blob/main/shared/memory_allocations/src/lib.rs
extern crate alloc;
use alloc::vec::Vec;

pub const CAP_LIM: usize = 10000;
pub const TO_INSERT_ITEM: usize = 145;

macro_rules! write_tests_maps {
    ($t:ty, $fn_name_1:ident, $fn_name_2:ident, $fn_name_3:ident, $fn_name_4:ident) => {
        pub fn $fn_name_1() -> $t {
            <$t>::with_capacity(CAP_LIM)
        }

        pub fn $fn_name_2(mmap: &mut $t) {
            for i in 1..CAP_LIM {
                mmap.insert(TO_INSERT_ITEM + i, TO_INSERT_ITEM);
            }
        }

        pub fn $fn_name_3(mmap: &mut $t) {
            for i in 0..(CAP_LIM + 1) {
                mmap.insert(TO_INSERT_ITEM + i, TO_INSERT_ITEM);
            }
        }

        pub fn $fn_name_4(mmap: &mut $t) {
            for i in 1..CAP_LIM {
                mmap.remove(&(TO_INSERT_ITEM + i));
            }
        }
    };
}

macro_rules! write_tests_sets {
    ($t:ty, $fn_name_1:ident, $fn_name_2:ident, $fn_name_3:ident, $fn_name_4:ident) => {
        pub fn $fn_name_1() -> $t {
            <$t>::with_capacity(CAP_LIM)
        }

        pub fn $fn_name_2(mmap: &mut $t) {
            for i in 1..CAP_LIM {
                mmap.insert(TO_INSERT_ITEM + i);
            }
        }

        pub fn $fn_name_3(mmap: &mut $t) {
            for i in 0..(CAP_LIM + 1) {
                mmap.insert(TO_INSERT_ITEM + i);
            }
        }

        pub fn $fn_name_4(mmap: &mut $t) {
            for i in 1..CAP_LIM {
                mmap.remove(&(TO_INSERT_ITEM + i));
            }
        }
    };
}

macro_rules! write_tests_vec {
    ($t:ty, $fn_name_1:ident, $fn_name_2:ident, $fn_name_3:ident, $fn_name_4:ident) => {
        pub fn $fn_name_1() -> $t {
            <$t>::with_capacity(CAP_LIM)
        }

        pub fn $fn_name_2(vecc: &mut $t) {
            for _ in 1..CAP_LIM {
                vecc.push(TO_INSERT_ITEM);
            }
        }

        pub fn $fn_name_3(vecc: &mut $t) {
            for _ in 0..(CAP_LIM + 1) {
                vecc.push(TO_INSERT_ITEM);
            }
        }

        pub fn $fn_name_4(vecc: &mut $t) {
            for _ in 1..CAP_LIM {
                vecc.pop();
            }
        }
    };
}

write_tests_vec!(Vec<usize>, alloc_vec, push_vec, dyn_alloc_vec, pop_vec);
