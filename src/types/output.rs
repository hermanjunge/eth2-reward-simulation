////////////////////////////////////////////////////////////////////////////////
//
// Output stores the outcomes from the simulation of an epoch
//
////////////////////////////////////////////////////////////////////////////////
use super::*;
use integer_sqrt::IntegerSquareRoot;
use std::time::Instant;

const MONTHS_PER_YEAR: i32 = 12;

pub struct Output {
    pub rows: Vec<EpochReportRow>,
}

impl Output {
    pub fn new() -> Output {
        let rows = vec![];

        Output { rows: rows }
    }

    pub fn push(&mut self, row: EpochReportRow) {
        self.rows.push(row);
    }

    pub fn print_epoch_report(&self, mode: &str) {
        if mode == "csv" {
            println!(
                "{},{},{},{},{},{},{},{},{},{},{},{}",
                "epoch number".to_string(),
                "FFG rewards".to_string(),
                "FFG penalties".to_string(),
                "proposer rewards".to_string(),
                "attester rewards".to_string(),
                "total staked balance".to_string(),
                "total effective balance".to_string(),
                "max balance".to_string(),
                "min balance".to_string(),
                "total validators".to_string(),
                "total active validatos".to_string(),
                "time μs".to_string(),
            );

            for row in &self.rows {
                println!(
                    "{},{},{},{},{},{},{},{},{},{},{},{}",
                    row.epoch_id,
                    row.deltas_head_ffg_rewards,
                    row.deltas_head_ffg_penalties,
                    row.deltas_proposer_rewards,
                    row.deltas_attester_rewards,
                    row.total_staked_balance,
                    row.total_effective_balance,
                    row.max_balance,
                    row.min_balance,
                    row.total_validators,
                    row.total_active_validators,
                    row.time_elapsed,
                );
            }
        }
    }

    pub fn print_monthly_report(&self, config: &Config) {
        let epochs_per_year = config.epochs;
        let epochs_per_month = epochs_per_year / MONTHS_PER_YEAR;

        let mut monthly_report: Vec<MonthlyReportRow> = Vec::new();
        let mut items_to_get = vec![];

        for epoch in (epochs_per_month..epochs_per_year).step_by(epochs_per_month as usize) {
            items_to_get.push(epoch)
        }

        for (index, item) in items_to_get.iter().enumerate() {
            let current_item = &self.rows[*item as usize];
            let network_percentage_rewards = Output::get_variation_percentage(
                current_item.total_staked_balance,
                config.total_at_stake_initial,
            );
            let network_percentage_penalties = Output::get_variation_percentage(
                current_item.deltas_head_ffg_penalties,
                config.total_at_stake_initial,
            );
            let network_percentage_net_rewards =
                network_percentage_rewards - network_percentage_penalties;

            monthly_report.push(MonthlyReportRow {
                month_number: index as u32 + 1u32,
                network_percentage_rewards: network_percentage_rewards,
                network_percentage_penalties: network_percentage_penalties,
                network_percentage_net_rewards: network_percentage_net_rewards,
            });
        }

        for record in monthly_report {
            println!(
                "Month number: {}, Total Network Rewards {}",
                record.month_number, record.network_percentage_net_rewards
            );
        }
    }

    fn get_variation_percentage(new_value: u64, old_value: u64) -> f64 {
        ((new_value as f64 - old_value as f64) / old_value as f64) * 100.0
    }
}

#[derive(Copy, Clone)]
pub struct MonthlyReportRow {
    pub month_number: u32,
    pub network_percentage_rewards: f64,
    pub network_percentage_penalties: f64,
    pub network_percentage_net_rewards: f64,
}

pub struct EpochReportRow {
    pub epoch_id: i32,

    pub deltas_head_ffg_rewards: u64,
    pub deltas_head_ffg_penalties: u64,
    pub deltas_proposer_rewards: u64,
    pub deltas_attester_rewards: u64,

    pub total_staked_balance: u64,
    pub total_effective_balance: u64,
    pub max_balance: u64,
    pub min_balance: u64,
    pub total_validators: u64,
    pub total_active_validators: u64,

    pub time_started: Instant,
    pub time_elapsed: u128,
}

impl EpochReportRow {
    pub fn open(id: i32) -> EpochReportRow {
        EpochReportRow {
            epoch_id: id,

            deltas_head_ffg_rewards: 0,
            deltas_head_ffg_penalties: 0,
            deltas_proposer_rewards: 0,
            deltas_attester_rewards: 0,

            total_staked_balance: 0,
            total_effective_balance: 0,
            max_balance: 0,
            min_balance: 0,
            total_validators: 0,
            total_active_validators: 0,

            time_started: Instant::now(),
            time_elapsed: 0,
        }
    }

    pub fn aggregate(&mut self, deltas: &Deltas) {
        self.deltas_head_ffg_rewards += deltas.head_ffg_reward;
        self.deltas_head_ffg_penalties += deltas.head_ffg_penalty;
        self.deltas_proposer_rewards += deltas.proposer_reward;
        self.deltas_attester_rewards += deltas.attester_reward;
    }

    pub fn close(&mut self, state: &State, state_totals: &mut StateTotals) {
        state_totals.staked_balance = state.get_total_staked_balance();
        state_totals.active_balance = state.get_total_active_balance();
        state_totals.sqrt_active_balance = state_totals.active_balance.integer_sqrt();
        state_totals.active_validators = state.get_total_active_validators();
        state_totals.matching_balance = state.get_matching_balance();
        state_totals.max_balance = state.get_max_balance();
        state_totals.min_balance = state.get_min_balance();

        self.total_staked_balance = state_totals.staked_balance;
        self.total_effective_balance = state_totals.active_balance;
        self.max_balance = state_totals.max_balance;
        self.min_balance = state_totals.min_balance;
        self.total_validators = state.validators.len() as u64;
        self.total_active_validators = state_totals.active_validators;
        self.time_elapsed = self.time_started.elapsed().as_micros();
    }
}

// TODO: Tests
// - Output::new()
// - Output::push()
// - Output::print_epoch_report()
// - Output::print_monthly_report()
// - EpochReportRow::open()
// - EpochReportRow::aggregate()
// - EpochReportRow::close()
