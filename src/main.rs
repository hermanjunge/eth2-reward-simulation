////////////////////////////////////////////////////////////////////////////////
//
// simple simulator of rewards and penalties for Phase 0
//
////////////////////////////////////////////////////////////////////////////////

mod process_epoch;
mod types;

use process_epoch::process_epoch;
use types::*;

fn main() {
    let mut state = State::new();
    let mut state_totals = StateTotals::new(&state);
    let mut output = Output::new();

    for i in 0..state.config.epochs {
        state = process_epoch(state, &mut state_totals, i, &mut output);
    }

    if state.config.printing_output == "monthly" {
        output.print_monthly_report(&state.config);
    } else if state.config.printing_output == "epoch" {
        output.print_epoch_report("csv");
    }
}
