////////////////////////////////////////////////////////////////////////////////
//
// A simplified Eth2 validator
//
////////////////////////////////////////////////////////////////////////////////

use super::*;
use rand::prelude::*;
use std::cmp;

#[derive(Debug)]
pub struct Validator {
    pub balance: u64,
    pub effective_balance: u64,
    pub is_active: bool,
    pub is_slashed: bool,
    pub has_matched_source: bool,
    pub has_matched_target: bool,
    pub has_matched_head: bool,
    pub is_proposer: bool,
}

impl Validator {
    pub fn get_base_reward(&self, sqrt_total_active_balance: u64) -> u64 {
        self.effective_balance * config::BASE_REWARD_FACTOR
            / sqrt_total_active_balance
            / config::BASE_REWARDS_PER_EPOCH
    }

    pub fn update_previous_epoch_activity(
        &self,
        state: &State,
        proposer_indices: &Vec<usize>,
        validator_index: usize,
    ) -> Validator {
        let mut rng = thread_rng();
        let has_been_online = state.config.probability_online > rng.gen();
        let has_been_honest = state.config.probability_honest > rng.gen();
        let has_matched_source = !self.is_slashed && has_been_online && has_been_honest;

        Validator {
            balance: self.balance,
            effective_balance: self.effective_balance,
            is_active: self.is_active,
            is_slashed: self.is_slashed,
            has_matched_source: has_matched_source,
            has_matched_target: has_matched_source,
            has_matched_head: has_matched_source,
            is_proposer: proposer_indices.contains(&validator_index),
        }
    }

    pub fn update_effective_balance(&mut self) {
        let half_increment = config::EFFECTIVE_BALANCE_INCREMENT / 2;

        if self.balance < self.effective_balance
            || self.effective_balance + 3 * half_increment < self.balance
        {
            self.effective_balance = cmp::min(
                self.balance - self.balance % config::EFFECTIVE_BALANCE_INCREMENT,
                config::MAX_EFFECTIVE_BALANCE,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestCaseHasSource {
        state: State,
        validator: Validator,
        expected_result: bool,
    }

    fn prepare_test_has_source(
        is_slashed: bool,
        probability_online: f32,
        probability_honest: f32,
        expected_result: bool,
    ) -> TestCaseHasSource {
        let mut state = State::new();
        let validator = Validator {
            balance: 32_000_000_000,
            effective_balance: 32_000_000_000,
            is_active: true,
            is_slashed: is_slashed,
            has_matched_source: false,
            has_matched_head: false,
            has_matched_target: false,
            is_proposer: false,
        };

        state.config.probability_online = probability_online;
        state.config.probability_honest = probability_honest;

        TestCaseHasSource {
            state: state,
            validator: validator,
            expected_result: expected_result,
        }
    }

    #[test]
    fn update_previous_epoch_activity_has_matched_source() {
        let mut cases = vec![];

        // is_slashed true should always fail
        cases.push(prepare_test_has_source(true, 1.0, 1.0, false));
        cases.push(prepare_test_has_source(true, 1.0, 0.5, false));
        cases.push(prepare_test_has_source(true, 1.0, 0.0, false));
        cases.push(prepare_test_has_source(true, 0.0, 1.0, false));
        cases.push(prepare_test_has_source(true, 0.0, 0.5, false));
        cases.push(prepare_test_has_source(true, 0.0, 0.0, false));

        // the "always good" case
        cases.push(prepare_test_has_source(false, 1.0, 1.0, true));

        // a 0.0 in one of the probabilities will always fail
        cases.push(prepare_test_has_source(false, 0.0, 1.0, false));
        cases.push(prepare_test_has_source(false, 1.0, 0.0, false));

        let dummy_vec = vec![];

        for mut case in cases {
            case.validator =
                case.validator
                    .update_previous_epoch_activity(&case.state, &dummy_vec, 0);
            assert_eq!(case.expected_result, case.validator.has_matched_source);
        }
    }

    struct TestCaseProposer {
        state: State,
        validator: Validator,
        proposer_indices: Vec<usize>,
        validator_index: usize,
        expected_result: bool,
    }

    fn prepare_test_proposer(validator_index: usize, expected_result: bool) -> TestCaseProposer {
        let state = State::new();
        let validator = Validator {
            balance: 32_000_000_000,
            effective_balance: 32_000_000_000,
            is_active: true,
            is_slashed: false,
            has_matched_source: false,
            has_matched_head: false,
            has_matched_target: false,
            is_proposer: false,
        };

        let mut proposer_indices = vec![];
        proposer_indices.push(4usize);
        proposer_indices.push(8usize);
        proposer_indices.push(15usize);
        proposer_indices.push(16usize);
        proposer_indices.push(23usize);
        proposer_indices.push(42usize);

        TestCaseProposer {
            state: state,
            validator: validator,
            proposer_indices: proposer_indices,
            validator_index: validator_index,
            expected_result: expected_result,
        }
    }

    #[test]
    fn update_previous_epoch_activity_proposer() {
        let mut cases = vec![];

        cases.push(prepare_test_proposer(0, false));
        cases.push(prepare_test_proposer(8, true));
        cases.push(prepare_test_proposer(19, false));
        cases.push(prepare_test_proposer(42, true));

        for mut case in cases {
            case.validator = case.validator.update_previous_epoch_activity(
                &case.state,
                &case.proposer_indices,
                case.validator_index,
            );

            assert_eq!(case.expected_result, case.validator.is_proposer);
        }
    }

    #[test]
    fn get_base_reward() {
        let validator = Validator {
            balance: 32_000_000_000,
            effective_balance: 32_000_000_000,
            is_active: true,
            is_slashed: false,
            has_matched_source: false,
            has_matched_head: false,
            has_matched_target: false,
            is_proposer: false,
        };

        // we pick sqrt of 500,000 ETH
        let sqrt_total_active_balance: u64 = 22_360_679;

        assert_eq!(22_897, validator.get_base_reward(sqrt_total_active_balance));
    }

    struct TestCaseUpdateBalance {
        validator: Validator,
        expected_result: u64,
    }

    fn eth_to_gwei(eth_number: f64) -> u64 {
        (eth_number * 1_000_000_000 as f64) as u64
    }

    fn prepare_test_case_update_balance(
        balance: f64,
        effective_balance: f64,
        expected_result: f64,
    ) -> TestCaseUpdateBalance {
        TestCaseUpdateBalance {
            validator: Validator {
                balance: eth_to_gwei(balance),
                effective_balance: eth_to_gwei(effective_balance),
                is_active: true,
                is_slashed: false,
                has_matched_source: false,
                has_matched_head: false,
                has_matched_target: false,
                is_proposer: false,
            },
            expected_result: eth_to_gwei(expected_result),
        }
    }

    #[test]
    fn update_effective_balance() {
        let mut cases = vec![];

        // balance below (or equal to) 24. effective balance 24
        cases.push(prepare_test_case_update_balance(23.0, 24.0, 23.0));
        cases.push(prepare_test_case_update_balance(23.5, 24.0, 23.0));
        cases.push(prepare_test_case_update_balance(24.0, 24.0, 24.0));

        // balance above 24. effective balance 24
        cases.push(prepare_test_case_update_balance(24.5, 24.0, 24.0));
        cases.push(prepare_test_case_update_balance(25.0, 24.0, 24.0));
        cases.push(prepare_test_case_update_balance(25.5, 24.0, 24.0));
        cases.push(prepare_test_case_update_balance(25.500001, 24.0, 25.0));
        cases.push(prepare_test_case_update_balance(26.0, 24.0, 26.0));

        // balance below (or equal to) 32. effective balance 32
        cases.push(prepare_test_case_update_balance(31.0, 32.0, 31.0));
        cases.push(prepare_test_case_update_balance(31.5, 32.0, 31.0));
        cases.push(prepare_test_case_update_balance(32.0, 32.0, 32.0));

        // balance above 32. effective balance 32
        cases.push(prepare_test_case_update_balance(32.5, 32.0, 32.0));
        cases.push(prepare_test_case_update_balance(33.0, 32.0, 32.0));
        cases.push(prepare_test_case_update_balance(33.5, 32.0, 32.0));
        cases.push(prepare_test_case_update_balance(34.0, 32.0, 32.0));

        // effective balance 31. balance above 31
        cases.push(prepare_test_case_update_balance(31.5, 31.0, 31.0));
        cases.push(prepare_test_case_update_balance(32.0, 31.0, 31.0));
        cases.push(prepare_test_case_update_balance(32.5, 31.0, 31.0));
        cases.push(prepare_test_case_update_balance(32.500001, 31.0, 32.0));

        for mut case in cases {
            case.validator.update_effective_balance();
            assert_eq!(case.expected_result, case.validator.effective_balance);
        }
    }
}
