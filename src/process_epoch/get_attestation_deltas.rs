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
    config: &Config,
    total_active_balance: u64,
    total_active_validators: u64,
    matching_balance: u64,
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
        || !dice.throw_dice(config.probability_online)
        || !dice.throw_dice(config.probability_honest)
    {
        deltas.head_ffg_penalty = 3 * base_reward;
    } else {
        // HACK: avoid integer overflows by "shaving" both balances
        // NOTE: this issue has been reported as of 2020.02.10
        let mb = matching_balance / 1000;
        let tab = total_active_balance / 1000;
        deltas.head_ffg_reward = 3 * base_reward * mb / tab;

        // inclusion rewards - proposer
        let proposer_reward_amount = base_reward / config::PROPOSER_REWARD_QUOTIENT;
        if proposer_indices.contains(validator_index) {
            let number_of_attesters = total_active_validators / 32;
            let number_of_attestations = (number_of_attesters as f32
                * config.probability_online
                * config.probability_honest)
                .floor() as u64;
            deltas.proposer_reward = proposer_reward_amount * number_of_attestations;
        }

        // inclusion rewards - attester
        let maximum_attester_reward = base_reward - proposer_reward_amount;
        deltas.attester_reward =
            (maximum_attester_reward as f32 * config.exp_value_inclusion_prob).floor() as u64;
    }
}

// TODO: Tests
// - attester reward
// - non elegible validator
// - slashed validator
// - FFG rewards

#[cfg(test)]
mod tests {
    use super::*;
    use integer_sqrt::IntegerSquareRoot;

    #[test]
    fn proposer_reward_validator_is_proposer() {
        let mut state = State::new();
        let mut deltas = Deltas::new();
        let mut dice = Dice::new();
        let mut proposer_indices = dice.pick_epoch_proposers(&state);

        // modify so as to be one of the proposers
        proposer_indices.sort();
        proposer_indices[0] = 0;

        // arrange for our validator to be always online and honest
        state.config.probability_online = 1.0;
        state.config.probability_honest = 1.0;

        // call get_attestation_deltas on your validators
        get_attestation_deltas(
            &state.validators[0],
            &(0 as usize),
            state.validators[0].get_base_reward(state.get_total_active_balance().integer_sqrt()),
            &state.config,
            state.get_total_active_balance(),
            state.get_total_active_validators(),
            state.get_matching_balance(),
            &proposer_indices,
            &mut deltas,
        );

        // the actual test
        assert_eq!(1_396_656, deltas.proposer_reward);
    }

    #[test]
    fn proposer_reward_validator_is_not_proposer() {
        let mut state = State::new();
        let mut deltas = Deltas::new();
        let mut dice = Dice::new();
        let mut proposer_indices = dice.pick_epoch_proposers(&state);

        // modify so as NOT to be one of the proposers
        proposer_indices.sort();
        if proposer_indices[0] == 0 {
            proposer_indices[0] = 1;
        }

        // arrange for our validator to be always online and honest
        state.config.probability_online = 1.0;
        state.config.probability_honest = 1.0;

        // call get_attestation_deltas on your validators
        get_attestation_deltas(
            &state.validators[0],
            &(0 as usize),
            state.validators[0].get_base_reward(state.get_total_active_balance().integer_sqrt()),
            &state.config,
            state.get_total_active_balance(),
            state.get_total_active_validators(),
            state.get_matching_balance(),
            &proposer_indices,
            &mut deltas,
        );

        // the actual test
        assert_eq!(0, deltas.proposer_reward);
    }

    #[test]
    fn attester_reward() {
        let mut state = State::new();
        let mut deltas = Deltas::new();
        let mut dice = Dice::new();

        // arrange for our validator to be always online and honest
        state.config.probability_online = 1.0;
        state.config.probability_honest = 1.0;
        state.config.exp_value_inclusion_prob = 1.0;

        // call get_attestation_deltas on your validators
        get_attestation_deltas(
            &state.validators[0],
            &(0 as usize),
            state.validators[0].get_base_reward(state.get_total_active_balance().integer_sqrt()),
            &state.config,
            state.get_total_active_balance(),
            state.get_total_active_validators(),
            state.get_matching_balance(),
            &dice.pick_epoch_proposers(&state),
            &mut deltas,
        );

        // the actual test
        assert_eq!(20_035, deltas.attester_reward);
    }
}
