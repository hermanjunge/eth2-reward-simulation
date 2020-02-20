////////////////////////////////////////////////////////////////////////////////
//
// The state of the simulation
//   - validators
//   - config variables
//
////////////////////////////////////////////////////////////////////////////////

use super::*;
use integer_sqrt::IntegerSquareRoot;
use rand::prelude::*;

pub struct State {
    pub config: config::Config,
    pub validators: Vec<Validator>,
}

impl State {
    pub fn new() -> State {
        let config = config::Config::new();

        let number_of_validators = config.total_at_stake_initial / config::MAX_EFFECTIVE_BALANCE;
        let mut validators = vec![];

        for _ in 0..number_of_validators {
            validators.push(Validator {
                balance: config::MAX_EFFECTIVE_BALANCE,
                effective_balance: config::MAX_EFFECTIVE_BALANCE,
                is_active: true,
                is_slashed: false,
                has_matched_source: false,
                has_matched_head: false,
                has_matched_target: false,
                is_proposer: false,
            });
        }

        State {
            config: config,
            validators: validators,
        }
    }

    pub fn get_total_staked_balance(&self) -> u64 {
        self.validators.iter().map(|v: &Validator| v.balance).sum()
    }

    pub fn get_total_active_balance(&self) -> u64 {
        self.validators
            .iter()
            .map(
                |v: &Validator| {
                    if v.is_active {
                        v.effective_balance
                    } else {
                        0
                    }
                },
            )
            .sum()
    }

    pub fn get_total_active_validators(&self) -> u64 {
        self.validators
            .iter()
            .map(|v: &Validator| if v.is_active { 1 } else { 0 })
            .sum()
    }

    pub fn get_matching_balance(&self) -> u64 {
        self.validators
            .iter()
            .map(|v: &Validator| {
                if v.is_active && !v.is_slashed {
                    v.effective_balance
                } else {
                    0
                }
            })
            .sum()
    }

    pub fn get_max_balance(&self) -> u64 {
        self.validators
            .iter()
            .map(|v: &Validator| v.balance)
            .fold(0, std::cmp::max)
    }

    pub fn get_min_balance(&self) -> u64 {
        self.validators
            .iter()
            .map(|v: &Validator| v.balance)
            .fold(std::u64::MAX, std::cmp::min)
    }

    pub fn pick_epoch_proposers(&self) -> Vec<usize> {
        let mut rng = thread_rng();

        let mut proposer_indices = vec![];

        let n = self.validators.len();
        let proposers_per_epoch = 32;
        let max_effective_balance = 32_000_000_000;
        let max_random_byte = 255;

        if self.get_total_active_validators() < 32 {
            panic!("not enough active validators");
        }

        loop {
            if proposer_indices.len() == proposers_per_epoch {
                break;
            }

            let candidate_index = rng.gen_range(0, n);

            if self.validators[candidate_index].is_slashed
                || !self.validators[candidate_index].is_active
                || proposer_indices.contains(&candidate_index)
            {
                continue;
            }

            // effective balance bias on proposer election
            let random_byte = rng.gen_range(0, 255);
            if self.validators[candidate_index].effective_balance * max_random_byte
                >= random_byte * max_effective_balance
            {
                proposer_indices.push(candidate_index);
            } else {
                continue;
            }
        }

        proposer_indices
    }
}

pub struct StateTotals {
    pub staked_balance: u64,
    pub active_balance: u64,
    pub sqrt_active_balance: u64,
    pub matching_balance: u64,
    pub max_balance: u64,
    pub min_balance: u64,
    pub active_validators: u64,
}

impl StateTotals {
    pub fn new(state: &State) -> StateTotals {
        let total_active_balance = state.get_total_active_balance();

        StateTotals {
            staked_balance: state.get_total_staked_balance(),
            active_balance: state.get_total_active_balance(),
            sqrt_active_balance: total_active_balance.integer_sqrt(),
            active_validators: state.get_total_active_validators(),
            matching_balance: state.get_matching_balance(),
            max_balance: state.get_max_balance(),
            min_balance: state.get_min_balance(),
        }
    }
}

// TODO: Test
// - self::pick_epoch_proposers()

#[cfg(test)]
mod tests {
    use super::*;

    fn get_dummy_validator(
        balance: u64,
        effective_balance: u64,
        is_active: bool,
        is_slashed: bool,
    ) -> Validator {
        Validator {
            balance: balance,
            effective_balance: effective_balance,
            is_active: is_active,
            is_slashed: is_slashed,
            has_matched_source: false,
            has_matched_head: false,
            has_matched_target: false,
            is_proposer: false,
        }
    }

    #[test]
    fn new_state() {
        let state = State::new();

        assert_eq!(state.config.total_at_stake_initial, 500000000000000);
        assert_eq!(state.validators.len(), 15625);
        assert_eq!(state.get_total_active_balance(), 500000000000000);
        assert_eq!(state.get_max_balance(), 32000000000);
        assert_eq!(state.get_min_balance(), 32000000000);
    }

    #[test]
    fn new_state_totals() {
        let mut state = State::new();
        state.validators = vec![];
        state
            .validators
            .push(get_dummy_validator(100, 16, true, true));
        state
            .validators
            .push(get_dummy_validator(200, 18, true, false));
        state
            .validators
            .push(get_dummy_validator(300, 30, true, false));
        state
            .validators
            .push(get_dummy_validator(400, 40, false, false));

        let totals = StateTotals::new(&state);

        assert_eq!(totals.staked_balance, 1000);
        assert_eq!(totals.active_balance, 64);
        assert_eq!(totals.sqrt_active_balance, 8);
        assert_eq!(totals.active_validators, 3);
        assert_eq!(totals.matching_balance, 48);
        assert_eq!(totals.max_balance, 400);
        assert_eq!(totals.min_balance, 100);
    }
}
