////////////////////////////////////////////////////////////////////////////////
//
// Simulates relevant `process_rewards_and_penalties` ops during the state transition
//
////////////////////////////////////////////////////////////////////////////////

use crate::types::*;

pub fn get_attestation_deltas(
    validator: &Validator,
    validator_index: &usize,
    base_reward: u64,
    state: &State,
    state_totals: &StateTotals,
    proposer_indices: &Vec<usize>,
    deltas: &mut Deltas,
) {
    // load our random component
    let mut dice = Dice::new();

    // eligibility check
    if !validator.is_active {
        return;
    }

    // head and FFG incentives (and penalties)
    if validator.is_slashed
        || !dice.throw_dice(state.config.probability_online)
        || !dice.throw_dice(state.config.probability_honest)
    {
        deltas.head_ffg_penalty = 3 * base_reward;
    } else {
        // HACK: avoid integer overflows by "shaving" both balances
        // NOTE: this issue has been reported as of 2020.02.10
        let mb = (state_totals.matching_balance as f32 * state.config.probability_online).floor()
            as u64
            / 1000;
        let tab = state_totals.active_balance / 1000;
        deltas.head_ffg_reward = 3 * base_reward * mb / tab;

        // inclusion rewards - proposer
        let proposer_reward_amount = base_reward / config::PROPOSER_REWARD_QUOTIENT;
        if proposer_indices.contains(validator_index) {
            let number_of_attesters = state_totals.active_validators / 32;
            let number_of_attestations = (number_of_attesters as f32
                * state.config.probability_online
                * state.config.probability_honest)
                .floor() as u64;
            deltas.proposer_reward = proposer_reward_amount * number_of_attestations;
        }

        // inclusion rewards - attester
        let maximum_attester_reward = base_reward - proposer_reward_amount;
        deltas.attester_reward =
            (maximum_attester_reward as f32 * state.config.exp_value_inclusion_prob).floor() as u64;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_eligible_validator() {
        let mut state = State::new();
        let state_totals = StateTotals::new(&state);
        let mut deltas = Deltas::new();
        let mut dice = Dice::new();

        // our validator was not active last epoch
        state.validators[0].is_active = false;

        get_attestation_deltas(
            &state.validators[0],
            &(0 as usize),
            state.validators[0].get_base_reward(state_totals.sqrt_active_balance),
            &state,
            &state_totals,
            &dice.pick_epoch_proposers(&state),
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
        let mut dice = Dice::new();

        // our validator has the slashed status
        state.validators[0].is_slashed = true;
        let base_reward = state.validators[0].get_base_reward(state_totals.sqrt_active_balance);

        get_attestation_deltas(
            &state.validators[0],
            &(0 as usize),
            base_reward,
            &state,
            &state_totals,
            &dice.pick_epoch_proposers(&state),
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
        let mut dice = Dice::new();

        state.validators[0].is_active = true;
        state.validators[0].is_slashed = false;
        state.config.probability_online = 1.0;
        state.config.probability_honest = 1.0;
        let base_reward = state.validators[0].get_base_reward(state_totals.sqrt_active_balance);

        get_attestation_deltas(
            &state.validators[0],
            &(0 as usize),
            base_reward,
            &state,
            &state_totals,
            &dice.pick_epoch_proposers(&state),
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
        let mut dice = Dice::new();

        let mut proposer_indices = dice.pick_epoch_proposers(&state);
        proposer_indices.sort();
        proposer_indices[0] = 0;
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

        assert_eq!(1_396_656, deltas.proposer_reward);
    }

    #[test]
    fn proposer_reward_validator_is_not_proposer() {
        let mut state = State::new();
        let state_totals = StateTotals::new(&state);
        let mut deltas = Deltas::new();
        let mut dice = Dice::new();

        let mut proposer_indices = dice.pick_epoch_proposers(&state);
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
        let mut dice = Dice::new();

        state.config.probability_online = 1.0;
        state.config.probability_honest = 1.0;
        state.config.exp_value_inclusion_prob = 1.0;
        let base_reward = state.validators[0].get_base_reward(state_totals.sqrt_active_balance);

        get_attestation_deltas(
            &state.validators[0],
            &(0 as usize),
            base_reward,
            &state,
            &state_totals,
            &dice.pick_epoch_proposers(&state),
            &mut deltas,
        );

        assert_eq!(20_035, deltas.attester_reward);
    }
}
