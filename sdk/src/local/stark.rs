use super::util;
use crate::prover::{ProverInput, ProverResult};
use elf::{endian::AnyEndian, ElfBytes};
use util::{C, D, F};
use zkm_emulator::state::State;
use zkm_emulator::utils::split_prog_into_segs;
use zkm_prover::generation::state::{AssumptionReceipts, Receipt};

#[allow(clippy::type_complexity)]
pub fn prove_stark(
    input: &ProverInput,
    storedir: &str,
    result: &mut ProverResult,
) -> anyhow::Result<(bool, Option<Vec<u8>>, Option<Vec<u8>>)> {
    let seg_path = format!("{}/segments", storedir);
    let seg_size = input.seg_size as usize;
    let file = ElfBytes::<AnyEndian>::minimal_parse(input.elf.as_slice())
        .expect("Opening elf file failed");
    let mut state = State::load_elf(&file);
    state.patch_elf(&file);
    state.patch_stack(vec![]);

    state.input_stream.push(input.public_inputstream.clone());
    state.input_stream.push(input.private_inputstream.clone());
    for receipt_input in input.receipt_inputs.iter() {
        state.input_stream.push(receipt_input.clone());
    }

    let split_start_time = std::time::Instant::now();
    let (total_steps, seg_num, state) = split_prog_into_segs(state, &seg_path, "", seg_size);
    result.output_stream = state.public_values_stream.clone();
    result.total_steps = total_steps as u64;
    result.split_cost = split_start_time.elapsed().as_millis() as u64;
    if input.execute_only {
        return Ok((false, None, None));
    }
    log::info!("The seg_num is:{}", &seg_num);
    let mut receipts: AssumptionReceipts<F, C, D> = vec![];
    for receipt_data in input.receipts.iter() {
        let receipt: Receipt<F, C, D> =
            bincode::deserialize(receipt_data).map_err(|e| anyhow::anyhow!(e))?;
        receipts.push(receipt.into());
    }
    let receipt = util::prove_segments(&seg_path, "", storedir, "", "", seg_num, 0, receipts)?;
    let receipt_data = bincode::serialize(&receipt).map_err(|e| anyhow::anyhow!(e))?;
    Ok((seg_num > 1, Some(receipt_data), Some(receipt.claim().elf_id)))
}
