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
    let proposer_indices = pre_state.pick_epoch_proposers();

    for (validator_index, pre_state_validator) in pre_state.validators.iter().enumerate() {
        // SPEC: process_rewards_and_penalties.get_attestation_deltas()
        let mut deltas = Deltas::new();
        let validator = pre_state_validator.update_previous_epoch_activity(&pre_state.config);
        let base_reward = validator.get_base_reward(pre_state_totals.sqrt_active_balance);

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

// TODO
// Refactor
// - init of the epoch_report_row should only take a single line
// - replace epoch_report_row updating with a single function
// Test
// - process_epoch()
