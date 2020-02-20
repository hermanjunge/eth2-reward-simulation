////////////////////////////////////////////////////////////////////////////////
//
// The state of the simulation
//   - validators
//   - config variables
//
////////////////////////////////////////////////////////////////////////////////

use super::*;
use integer_sqrt::IntegerSquareRoot;

pub struct State {
    // we keep the config at hand
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

#[cfg(test)]
mod tests {
    use super::*;

    fn get_dummy_validator(balance: u64) -> Validator {
        Validator {
            balance: balance,
            effective_balance: 32_000_000_000,
            is_active: true,
            is_slashed: false,
        }
    }

    #[test]
    fn new_state() {}

    #[test]
    fn new_state_totals() {
        let mut state = State::new();
        state.validators = vec![];
        state.validators.push(get_dummy_validator(1));
        state.validators.push(get_dummy_validator(2));
        state.validators.push(get_dummy_validator(3));
        state.validators.push(get_dummy_validator(4));
        state.validators.push(get_dummy_validator(5));

        let totals = StateTotals::new(&state);

        assert_eq!(totals.staked_balance, 15);
        //
        //
        //
        // Debo todos los valores internos
    }
}
