////////////////////////////////////////////////////////////////////////////////
//
// Specs constants and simulation variables
//
////////////////////////////////////////////////////////////////////////////////

pub const MAX_EFFECTIVE_BALANCE: u64 = 32_000_000_000;
pub const BASE_REWARD_FACTOR: u64 = 64;
pub const BASE_REWARDS_PER_EPOCH: u64 = 4;
pub const PROPOSER_REWARD_QUOTIENT: u64 = 8;
pub const EFFECTIVE_BALANCE_INCREMENT: u64 = 1_000_000_000;

pub struct Config {
    // how many epochs we want to run?
    pub epochs: i32,

    // how much ETH we want to start with?
    pub total_at_stake_initial: u64,

    // probabilities of any validator
    pub probability_online: f32,
    pub probability_honest: f32,

    // pre-computation
    pub exp_value_inclusion_prob: f32,
}

impl Config {
    pub fn new() -> Config {
        // we want to get these values from the command line
        let total_at_stake_initial = 500_000 * 1_000_000_000; // Default 500,000 ETH
        let epochs = 10; // 81_125 = (60 * 60 * 24 * 365)/(12 * 32) // Default 10
        let probability_online: f32 = 0.99; // Default 0.99
        let probability_honest: f32 = 1.0; // Default 1.00

        // pre-computation
        let exp_value_inclusion_prob = Config::get_exp_value_inclusion_prob(probability_online);

        Config {
            epochs: epochs,
            total_at_stake_initial: total_at_stake_initial,
            probability_online: probability_online,
            probability_honest: probability_honest,
            exp_value_inclusion_prob: exp_value_inclusion_prob,
        }
    }

    fn get_exp_value_inclusion_prob(p: f32) -> f32 {
        p * p.ln() / (p - 1.00)
    }
}

// TODO
// - CLI options to fill config variables
// - Fill up with defaults otherwise
// - A control variable for printing output
// - Tests
//   - edge cases for get_exp_value_inclusion_prob() (0, 1, values outside the interval)
//   - Config::new()
