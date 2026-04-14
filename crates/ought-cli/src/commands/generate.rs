use std::process;

use ought_gen::manifest::Manifest;

use super::{
    build_agent_assignments, collect_all_testable_ids, collect_section_groups, load_config,
    load_specs, primary_test_dir,
};
use crate::{Cli, GenerateArgs};

pub fn run(cli: &Cli, args: &GenerateArgs) -> anyhow::Result<()> {
    let (config_path, config) = load_config(&cli.config)?;
    let specs = load_specs(&config, &config_path)?;

    let test_dir = primary_test_dir(&config, &config_path);

    std::fs::create_dir_all(&test_dir)?;

    let manifest_path = test_dir.join("manifest.toml");
    let mut manifest = Manifest::load(&manifest_path).unwrap_or_default();

    let groups = collect_section_groups(&specs);

    let mut generated_count = 0;
    let mut error_count = 0;
    let mut stale_count = 0;

    if args.check {
        for group in &groups {
            for clause in &group.testable_clauses {
                if clause.pending {
                    continue;
                }
                if args.force || manifest.is_stale(&clause.id, &clause.content_hash, "") {
                    eprintln!("  stale: {}", clause.id);
                    stale_count += 1;
                }
            }
        }
    } else {
        let assignments = build_agent_assignments(
            &groups,
            &manifest,
            &config,
            &config_path,
            &test_dir,
            config.generator.parallelism.max(1),
            args.force,
        );

        if assignments.is_empty() {
            eprintln!("All tests up to date, nothing to generate.");
        } else {
            let total_clauses: usize = assignments
                .iter()
                .map(|a| a.groups.iter().map(|g| g.clauses.len()).sum::<usize>())
                .sum();
            eprintln!(
                "{} assignments, {} clauses to generate",
                assignments.len(),
                total_clauses
            );

            let orchestrator = ought_gen::Orchestrator::new(&config.generator, cli.verbose);
            let reports = orchestrator.run(assignments)?;

            for report in &reports {
                generated_count += report.generated;
                for err in &report.errors {
                    eprintln!("  error: {}", err);
                    error_count += 1;
                }
            }
        }
    }

    let all_ids = collect_all_testable_ids(&specs);
    let id_refs: Vec<&ought_spec::ClauseId> = all_ids.iter().collect();
    manifest.remove_orphans(&id_refs);

    manifest.save(&manifest_path)?;

    eprintln!("\n{} generated, {} errors", generated_count, error_count);

    if args.check && stale_count > 0 {
        eprintln!("{} stale clauses", stale_count);
        process::exit(1);
    }

    Ok(())
}
