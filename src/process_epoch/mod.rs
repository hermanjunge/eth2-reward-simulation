////////////////////////////////////////////////////////////////////////////////
//
// Simulates relevant `process_epoch` ops during the state transition
//
////////////////////////////////////////////////////////////////////////////////

mod apply_deltas;
mod get_attestation_deltas;

use crate::types::*;
use apply_deltas::*;
use get_attestation_deltas::*;

pub fn process_epoch(
    pre_state: State,
    state_totals: &mut StateTotals,
    epoch_id: i32,
    output: &mut Output,
) -> State {
    let mut epoch_report_row = EpochReportRow::open(epoch_id);

    let mut post_state_validators = vec![];
    let proposer_bitmap = pre_state.pick_epoch_proposers();

    for (validator_index, pre_state_validator) in pre_state.validators.iter().enumerate() {
        // SPEC: process_rewards_and_penalties.get_attestation_deltas()
        let mut deltas = Deltas::new();
        let validator = pre_state_validator.update_previous_epoch_activity(
            &pre_state,
            &proposer_bitmap,
            validator_index,
        );
        let base_reward = validator.get_base_reward(state_totals.sqrt_active_balance);

        get_attestation_deltas(
            &validator,
            base_reward,
            &pre_state,
            &state_totals,
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

    epoch_report_row.close(&post_state, state_totals);
    output.push(epoch_report_row);

    post_state
}
