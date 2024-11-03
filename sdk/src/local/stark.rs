use super::util;
use crate::prover::{ProverInput, ProverResult};
use common::file;
use elf::{endian::AnyEndian, ElfBytes};
use zkm_emulator::state::State;
use zkm_emulator::utils::split_prog_into_segs;

pub fn prove_stark(
    input: &ProverInput,
    storedir: &str,
    result: &mut ProverResult,
) -> anyhow::Result<bool> {
    let seg_path = format!("{}/segments", storedir);
    let seg_size = input.seg_size as usize;
    let file = ElfBytes::<AnyEndian>::minimal_parse(input.elf.as_slice())
        .expect("opening elf file failed");
    let mut state = State::load_elf(&file);
    state.patch_elf(&file);
    state.patch_stack(vec![]);

    state.input_stream.push(input.public_inputstream.clone());
    state.input_stream.push(input.private_inputstream.clone());

    let mut block_path = input.block_path.clone();
    if input.block_no > 0 {
        let input_path = if block_path.ends_with('/') {
            format!("{}input", block_path)
        } else {
            block_path.push_str("/");
            format!("{}input", block_path)
        };

        log::info!("split set input data {}", input_path);
        let input_data = file::new(&input_path).read().unwrap();
        state
            .memory
            .set_memory_range(0x30000000, Box::new(input_data.as_slice()))
            .expect("set memory range failed");
    } else {
        block_path = "".to_string();
    }

    let (_total_steps, seg_num, state) =
        split_prog_into_segs(state, &seg_path, &block_path, seg_size);
    result.output_stream = state.public_values_stream.clone();
    if input.execute_only {
        return Ok(false);
    }
    log::info!("!!!*******seg_num:{}", &seg_num);
    if seg_num == 1 {
        let seg_file = format!("{seg_path}/{}", 0);
        util::prove_single_seg_common(&seg_file, "", "", "")?;
        Ok(false)
    } else {
        util::prove_multi_seg_common(&seg_path, "", "", "", storedir, seg_num, 0)?;
        Ok(true)
    }
}
