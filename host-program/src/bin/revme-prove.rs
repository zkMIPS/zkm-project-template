use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;
use std::io::prelude::*;
use zkm_emulator::utils::{load_elf_with_patch, split_prog_into_segs};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum DataId {
    TYPE1,
    TYPE2,
    TYPE3,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Data {
    pub input1: [u8; 10],
    pub input2: u8,
    pub input3: i8,
    pub input4: u16,
    pub input5: i16,
    pub input6: u32,
    pub input7: i32,
    pub input8: u64,
    pub input9: i64,
    pub input10: Vec<u8>,
    pub input11: DataId,
    pub input12: String,
}

impl Default for Data {
    fn default() -> Self {
        Self::new()
    }
}

impl Data {
    pub fn new() -> Self {
        let array = [1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8, 8u8, 9u8, 10u8];
        Self {
            input1: array,
            input2: 0x11u8,
            input3: -1i8,
            input4: 0x1122u16,
            input5: -1i16,
            input6: 0x112233u32,
            input7: -1i32,
            input8: 0x1122334455u64,
            input9: -1i64,
            input10: array[1..3].to_vec(),
            input11: DataId::TYPE3,
            input12: "hello".to_string(),
        }
    }
}

fn main() {
    env_logger::try_init().unwrap_or_default();
    // 1. split ELF into segs
    let elf_path = env::var("ELF_PATH")
        .unwrap_or("guest-program/mips-elf/zkm-mips-elf-revme-rust".to_string());
    let seg_path = env::var("SEG_OUTPUT").expect("Segment output path is missing");
    let json_path =
        env::var("JSON_PATH").unwrap_or("host-program/test-vectors/244.json".to_string());
    let seg_size = env::var("SEG_SIZE").unwrap_or("131072".to_string());
    let seg_size = seg_size.parse::<_>().unwrap_or(0);
    let mut f = File::open(json_path).unwrap();
    let mut data = vec![];
    f.read_to_end(&mut data).unwrap();
    let mut state = load_elf_with_patch(&elf_path, vec![]);
    // load input
    state.add_input_stream(&data);

    let (total_steps, mut _state) = split_prog_into_segs(state, &seg_path, "", seg_size);

    let mut seg_num = 1usize;
    if seg_size != 0 {
        seg_num = (total_steps + seg_size - 1) / seg_size;
    }

    if seg_num == 1 {
        let seg_file = format!("{seg_path}/{}", 0);
        zkm_sdk::local::util::prove_single_seg_common(&seg_file, "", "", "", total_steps)
    } else {
        let outdir = "verifier/data/test_circuit/".to_string();
        zkm_sdk::local::util::prove_multi_seg_common(
            &seg_path, "", "", "", &outdir, seg_size, seg_num, 0,
        )
        .unwrap()
    }
}
