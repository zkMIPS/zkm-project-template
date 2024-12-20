use super::util;
use crate::prover::{ProverInput, ProverResult};
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
        .expect("Opening elf file failed");
    let mut state = State::load_elf(&file);
    state.patch_elf(&file);
    state.patch_stack(vec![]);

    state.input_stream.push(input.public_inputstream.clone());
    state.input_stream.push(input.private_inputstream.clone());

    let (total_steps, seg_num, state) = split_prog_into_segs(state, &seg_path, "", seg_size);
    result.output_stream = state.public_values_stream.clone();
    result.total_steps = total_steps as u64;
    if input.execute_only {
        return Ok(false);
    }
    log::info!("[The seg_num is:{} ]", &seg_num);
    util::prove_segments(&seg_path, "", storedir, "", "", seg_num, 0, vec![])?;
    Ok(seg_num > 1)
}
