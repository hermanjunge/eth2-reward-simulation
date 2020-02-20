////////////////////////////////////////////////////////////////////////////////
//
// A simplified Eth2 validator
//
////////////////////////////////////////////////////////////////////////////////

use super::config;
use std::cmp;

pub struct Validator {
    pub balance: u64,
    pub effective_balance: u64,
    pub is_active: bool,
    pub is_slashed: bool,
    pub has_matched_source: bool,
    pub has_matched_head: bool,
    pub has_matched_target: bool,
    pub was_proposer: bool,
}

impl Validator {
    pub fn get_base_reward(&self, sqrt_total_active_balance: u64) -> u64 {
        self.effective_balance * config::BASE_REWARD_FACTOR
            / sqrt_total_active_balance
            / config::BASE_REWARDS_PER_EPOCH
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
            was_proposer: false,
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
                was_proposer: false,
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
