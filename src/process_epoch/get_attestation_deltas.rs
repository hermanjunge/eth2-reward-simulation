////////////////////////////////////////////////////////////////////////////////
//
// Simulates `process_rewards_and_penalties` ops during the state transition
//
////////////////////////////////////////////////////////////////////////////////

use crate::types::*;

pub fn get_attestation_deltas(
    validator: &Validator,
    base_reward: u64,
    state: &State,
    state_totals: &StateTotals,
    deltas: &mut Deltas,
) {
    if !validator.is_active {
        return;
    }

    if !validator.has_matched_source {
        assign_ffg_penalty(deltas, base_reward);
    } else {
        assign_ffg_reward(
            deltas,
            state_totals.matching_balance,
            state_totals.active_balance,
            state.config.probability_online,
            state.config.probability_honest,
            base_reward,
        );

        if validator.is_proposer {
            assign_proposer_incentive(
                deltas,
                state_totals.active_validators,
                state.config.probability_online,
                state.config.probability_honest,
                base_reward,
            );
        }

        assign_attester_incentive(deltas, state.config.exp_value_inclusion_prob, base_reward);
    }
}

fn assign_ffg_reward(
    deltas: &mut Deltas,
    matching_balance: u64,
    active_balance: u64,
    probability_online: f32,
    probability_honest: f32,
    base_reward: u64,
) {
    // HACK: avoid integer overflows by "shaving" both balances
    // NOTE: this issue has been reported as of 2020.02.10
    let mb =
        (matching_balance as f32 * probability_online * probability_honest).floor() as u64 / 1000;
    let tab = active_balance / 1000;

    deltas.head_ffg_reward = 3 * base_reward * mb / tab;
}

fn assign_ffg_penalty(deltas: &mut Deltas, base_reward: u64) {
    deltas.head_ffg_penalty = 3 * base_reward;
}

fn assign_proposer_incentive(
    deltas: &mut Deltas,
    active_validators: u64,
    probability_online: f32,
    probability_honest: f32,
    base_reward: u64,
) {
    let proposer_reward_amount = base_reward / config::PROPOSER_REWARD_QUOTIENT;
    let number_of_attesters = active_validators / 32;
    let number_of_attestations =
        (number_of_attesters as f32 * probability_online * probability_honest).floor() as u64;

    deltas.proposer_reward = proposer_reward_amount * number_of_attestations;
}

fn assign_attester_incentive(deltas: &mut Deltas, magic_number: f32, base_reward: u64) {
    let proposer_reward_amount = base_reward / config::PROPOSER_REWARD_QUOTIENT;
    let maximum_attester_reward = base_reward - proposer_reward_amount;

    deltas.attester_reward = (maximum_attester_reward as f32 * magic_number).floor() as u64;
}

/*
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_eligible_validator() {
        let mut state = State::new();
        let state_totals = StateTotals::new(&state);
        let mut deltas = Deltas::new();

        // our validator was not active last epoch
        state.validators[0].is_active = false;

        get_attestation_deltas(
            &state.validators[0],
            &(0 as usize),
            state.validators[0].get_base_reward(state_totals.sqrt_active_balance),
            &state,
            &state_totals,
            &state.pick_epoch_proposers(),
            &mut deltas,
        );

        assert_eq!(0, deltas.head_ffg_reward);
        assert_eq!(0, deltas.head_ffg_penalty);
        assert_eq!(0, deltas.proposer_reward);
        assert_eq!(0, deltas.attester_reward);
    }

    #[test]
    fn slashed_validator() {
        let mut state = State::new();
        let state_totals = StateTotals::new(&state);
        let mut deltas = Deltas::new();

        // our validator has the slashed status
        state.validators[0].is_slashed = true;
        let base_reward = state.validators[0].get_base_reward(state_totals.sqrt_active_balance);

        get_attestation_deltas(
            &state.validators[0],
            &(0 as usize),
            base_reward,
            &state,
            &state_totals,
            &state.pick_epoch_proposers(),
            &mut deltas,
        );

        assert_eq!(0, deltas.head_ffg_reward);
        assert_eq!(68691, deltas.head_ffg_penalty);
        assert_eq!(3 * base_reward, deltas.head_ffg_penalty);
        assert_eq!(0, deltas.proposer_reward);
        assert_eq!(0, deltas.attester_reward);
    }

    #[test]
    fn ffg_rewards_1() {
        let mut state = State::new();
        let state_totals = StateTotals::new(&state);
        let mut deltas = Deltas::new();

        state.config.probability_online = 1.0;
        state.validators[0].is_active = true;
        state.validators[0].has_matched_source = true;
        let base_reward = state.validators[0].get_base_reward(state_totals.sqrt_active_balance);

        get_attestation_deltas(
            &state.validators[0],
            &(0 as usize),
            base_reward,
            &state,
            &state_totals,
            &state.pick_epoch_proposers(),
            &mut deltas,
        );

        assert_eq!(68690, deltas.head_ffg_reward / 10 * 10);
        assert_eq!(3 * base_reward / 10 * 10, deltas.head_ffg_reward / 10 * 10);
        assert_eq!(0, deltas.head_ffg_penalty);
    }

    #[test]
    fn proposer_reward_validator_is_proposer() {
        let mut state = State::new();
        let state_totals = StateTotals::new(&state);
        let mut deltas = Deltas::new();

        let mut proposer_indices = state.pick_epoch_proposers();
        proposer_indices.sort();
        proposer_indices[0] = 0;
        state.config.probability_online = 1.0;
        state.validators[0].has_matched_source = true;
        let base_reward = state.validators[0].get_base_reward(state_totals.sqrt_active_balance);

        get_attestation_deltas(
            &state.validators[0],
            &(0 as usize),
            base_reward,
            &state,
            &state_totals,
            &proposer_indices,
            &mut deltas,
        );

        assert_eq!(1_396_656, deltas.proposer_reward);
    }

    #[test]
    fn proposer_reward_validator_is_not_proposer() {
        let mut state = State::new();
        let state_totals = StateTotals::new(&state);
        let mut deltas = Deltas::new();

        let mut proposer_indices = state.pick_epoch_proposers();
        // modify so as NOT to be one of the proposers
        proposer_indices.sort();
        if proposer_indices[0] == 0 {
            proposer_indices[0] = 1;
        }
        state.config.probability_online = 1.0;
        state.config.probability_honest = 1.0;
        let base_reward = state.validators[0].get_base_reward(state_totals.sqrt_active_balance);

        get_attestation_deltas(
            &state.validators[0],
            &(0 as usize),
            base_reward,
            &state,
            &state_totals,
            &proposer_indices,
            &mut deltas,
        );

        assert_eq!(0, deltas.proposer_reward);
    }

    #[test]
    fn attester_reward() {
        let mut state = State::new();
        let state_totals = StateTotals::new(&state);
        let mut deltas = Deltas::new();

        state.config.probability_online = 1.0;
        state.config.exp_value_inclusion_prob = 1.0;
        state.validators[0].has_matched_source = true;
        let base_reward = state.validators[0].get_base_reward(state_totals.sqrt_active_balance);

        get_attestation_deltas(
            &state.validators[0],
            &(0 as usize),
            base_reward,
            &state,
            &state_totals,
            &state.pick_epoch_proposers(),
            &mut deltas,
        );

        assert_eq!(20_035, deltas.attester_reward);
    }
}

*/
