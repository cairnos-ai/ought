use std::process;

use super::{count_clauses_in_sections, count_pending_in_sections, load_config, load_specs};
use crate::Cli;

pub fn run(cli: &Cli) -> anyhow::Result<()> {
    let (config_path, config) = load_config(&cli.config)?;

    match load_specs(&config, &config_path) {
        Ok(specs) => {
            let clause_count: usize = specs
                .specs()
                .iter()
                .map(|s| count_clauses_in_sections(&s.sections))
                .sum();
            let pending_count: usize = specs
                .specs()
                .iter()
                .map(|s| count_pending_in_sections(&s.sections))
                .sum();
            if pending_count > 0 {
                eprintln!(
                    "All specs valid: {} files, {} clauses ({} pending)",
                    specs.specs().len(),
                    clause_count,
                    pending_count
                );
            } else {
                eprintln!(
                    "All specs valid: {} files, {} clauses",
                    specs.specs().len(),
                    clause_count
                );
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    }
}
