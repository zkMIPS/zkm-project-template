use std::fs::File;
use std::io::BufReader;
use std::time::Duration;

use plonky2::plonk::config::{GenericConfig, PoseidonGoldilocksConfig};
use plonky2::util::timing::TimingTree;

use zkm_prover::all_stark::AllStark;
use zkm_prover::config::StarkConfig;
use zkm_prover::cpu::kernel::assembler::segment_kernel;
use zkm_prover::generation::state::{AssumptionReceipts, Receipt};
use zkm_recursion::{aggregate_proof, create_recursive_circuit, wrap_stark_bn254};

const D: usize = 2;
type C = PoseidonGoldilocksConfig;
type F = <C as GenericConfig<D>>::F;

#[allow(clippy::too_many_arguments)]
pub fn prove_segments(
    seg_dir: &str,
    basedir: &str,
    outdir: &str,
    block: &str,
    file: &str,
    seg_file_number: usize,
    seg_start_id: usize,
    assumptions: AssumptionReceipts<F, C, D>,
) -> anyhow::Result<Receipt<F, C, D>> {

    let total_timing = TimingTree::new("prove total time", log::Level::Info);
    let all_stark = AllStark::<F, D>::default();
    let config = StarkConfig::standard_fast_config();
    // Preprocess all circuits.
    let all_circuits = create_recursive_circuit();


    let seg_file = format!("{}/{}", seg_dir, seg_start_id);
    log::info!("Process segment {}", seg_file);
    let seg_reader = BufReader::new(File::open(seg_file)?);
    let input_first = segment_kernel(basedir, block, file, seg_reader);
    let mut timing = TimingTree::new("prove root first", log::Level::Info);
    let mut agg_receipt = all_circuits.prove_root_with_assumption(
        &all_stark,
        &input_first,
        &config,
        &mut timing,
        assumptions.clone(),
    )?;

    timing.filter(Duration::from_millis(100)).print();
    all_circuits.verify_root(agg_receipt.clone())?;

    let mut base_seg = seg_start_id + 1;
    let mut seg_num = seg_file_number - 1;
    let mut is_agg = false;

    if seg_file_number % 2 == 0 {
        let seg_file = format!("{}/{}", seg_dir, seg_start_id + 1);
        log::info!("Process segment {}", seg_file);
        let seg_reader = BufReader::new(File::open(seg_file)?);
        let input = segment_kernel(basedir, block, file, seg_reader);
        timing = TimingTree::new("prove root second", log::Level::Info);
        let receipt = all_circuits.prove_root_with_assumption(
            &all_stark,
            &input,
            &config,
            &mut timing,
            assumptions,
        )?;
        timing.filter(Duration::from_millis(100)).print();

        all_circuits.verify_root(receipt.clone())?;

        // We can duplicate the proofs here because the state hasn't mutated.
        agg_receipt = aggregate_proof(&all_circuits, agg_receipt, receipt, false, false)?;


        is_agg = true;
        base_seg = seg_start_id + 2;
        seg_num -= 1;
    }

    for i in 0..seg_num / 2 {
        let seg_file = format!("{}/{}", seg_dir, base_seg + (i << 1));
        log::info!("Process segment {}", seg_file);
        let seg_reader = BufReader::new(File::open(&seg_file)?);
        let input_first = segment_kernel(basedir, block, file, seg_reader);
        let mut timing = TimingTree::new("prove root first", log::Level::Info);
        let root_receipt_first =
            all_circuits.prove_root(&all_stark, &input_first, &config, &mut timing)?;

        timing.filter(Duration::from_millis(100)).print();
        all_circuits.verify_root(root_receipt_first.clone())?;

        let seg_file = format!("{}/{}", seg_dir, base_seg + (i << 1) + 1);
        log::info!("Process segment {}", seg_file);
        let seg_reader = BufReader::new(File::open(&seg_file)?);
        let input = segment_kernel(basedir, block, file, seg_reader);
        let mut timing = TimingTree::new("prove root second", log::Level::Info);
        let root_receipt = all_circuits.prove_root(&all_stark, &input, &config, &mut timing)?;
        timing.filter(Duration::from_millis(100)).print();

        all_circuits.verify_root(root_receipt.clone())?;

        // We can duplicate the proofs here because the state hasn't mutated.
        let new_agg_receipt = aggregate_proof(
            &all_circuits,
            root_receipt_first,
            root_receipt,
            false,
            false,
        )?;

        // We can duplicate the proofs here because the state hasn't mutated.
        agg_receipt =
            aggregate_proof(&all_circuits, agg_receipt, new_agg_receipt, is_agg, true)?;
        is_agg = true;

    }

    log::info!(
        "proof size: {:?}",
        serde_json::to_string(&agg_receipt.proof().proof).unwrap().len()
    );

    let final_receipt = if seg_file_number > 1 {
        all_circuits.prove_block(None, &agg_receipt)?
    } else {
        agg_receipt.clone()
    };

    wrap_stark_bn254(&all_circuits, agg_receipt, outdir).unwrap();

    log::info!("build finish");

    total_timing.filter(Duration::from_millis(100)).print();
    Ok(final_receipt)
}
