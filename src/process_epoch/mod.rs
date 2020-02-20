////////////////////////////////////////////////////////////////////////////////
//
// Simulates relevant `process_epoch` ops during the state transition
//
////////////////////////////////////////////////////////////////////////////////

mod apply_deltas;
mod get_attestation_deltas;

use integer_sqrt::IntegerSquareRoot;
use std::time::Instant;

use crate::types::*;
use apply_deltas::*;
use get_attestation_deltas::*;

pub fn process_epoch(pre_state: State, epoch_id: i32, output: &mut Output) -> State {
    // start to record
    let mut epoch_report_row = EpochReportRow::new();
    epoch_report_row.epoch_id = epoch_id;
    let epoch_processing_start = Instant::now();

    let mut post_state_validators = vec![];

    // pre-compute some values that remain constant throughout the epoch
    let total_active_balance = pre_state.get_total_active_balance();
    let sqrt_total_active_balance = total_active_balance.integer_sqrt();
    let total_active_validators = pre_state.get_total_active_validators();
    let matching_balance = pre_state.get_matching_balance();

    // pick the 32 block proposers for this epoch
    let mut dice = Dice::new();
    let proposer_indices = dice.pick_epoch_proposers(&pre_state);

    for (validator_index, validator) in pre_state.validators.iter().enumerate() {
        let base_reward = validator.get_base_reward(sqrt_total_active_balance);

        // SPEC: process_rewards_and_penalties.get_attestation_deltas()
        let mut deltas = Deltas::new();
        get_attestation_deltas(
            &validator,
            &validator_index,
            base_reward,
            &pre_state.config,
            total_active_balance,
            total_active_validators,
            matching_balance,
            &proposer_indices,
            &mut deltas,
        );

        // SPEC: process_rewards_and_penalties second half
        let mut new_validator = apply_deltas(&validator, &deltas);

        // SPEC: process_final_updates update balances with hysteriesis
        new_validator.update_effective_balance();

        // your new validator gets added to the post_state
        post_state_validators.push(new_validator);

        // and we aggregate the rewards and penalties in our records
        epoch_report_row.aggregate(&deltas);
    }

    // build the new state and record its new totals
    let post_state = State {
        config: pre_state.config,
        validators: post_state_validators,
    };

    // record and record
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
