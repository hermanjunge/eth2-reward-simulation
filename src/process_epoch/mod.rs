////////////////////////////////////////////////////////////////////////////////
//
// Simulates relevant `process_epoch` ops during the state transition
//
////////////////////////////////////////////////////////////////////////////////

mod apply_deltas;
mod get_attestation_deltas;

use std::time::Instant;

use crate::types::*;
use apply_deltas::*;
use get_attestation_deltas::*;

pub fn process_epoch(pre_state: State, epoch_id: i32, output: &mut Output) -> State {
    let mut epoch_report_row = EpochReportRow::new();
    epoch_report_row.epoch_id = epoch_id;
    let epoch_processing_start = Instant::now();

    let mut post_state_validators = vec![];
    let pre_state_totals = StateTotals::new(&pre_state);
    let mut dice = Dice::new();
    let proposer_indices = dice.pick_epoch_proposers(&pre_state);

    for (validator_index, validator) in pre_state.validators.iter().enumerate() {
        // SPEC: process_rewards_and_penalties.get_attestation_deltas()
        let mut deltas = Deltas::new();
        let base_reward = validator.get_base_reward(pre_state_totals.sqrt_active_balance);
        // <- Get validator activity from previous epoch

        get_attestation_deltas(
            &validator,
            &validator_index,
            base_reward,
            &pre_state,
            &pre_state_totals,
            &proposer_indices,
            &mut deltas,
        );

        // SPEC: process_rewards_and_penalties second half
        let mut new_validator = apply_deltas(&validator, &deltas);

        // SPEC: process_final_updates update balances with hysteriesis
        new_validator.update_effective_balance();

        post_state_validators.push(new_validator);
        epoch_report_row.aggregate(&deltas);
    }

    let post_state = State {
        config: pre_state.config,
        validators: post_state_validators,
    };

    epoch_report_row.total_staked_balance = post_state.get_total_staked_balance();
    epoch_report_row.total_effective_balance = post_state.get_total_active_balance();
    epoch_report_row.max_balance = post_state.get_max_balance();
    epoch_report_row.min_balance = post_state.get_min_balance();
    epoch_report_row.total_validators = post_state.validators.len() as u64;
    epoch_report_row.total_active_validators = post_state.get_total_active_validators();
    epoch_report_row.time_elapsed = epoch_processing_start.elapsed().as_micros();
    output.push(epoch_report_row);

    post_state
}

// TODO: Test
// - process_epoch()
