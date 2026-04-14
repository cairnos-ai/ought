use super::{load_config, load_specs, resolve_primary_runner};
use crate::{Cli, BisectArgs};

pub fn run(cli: &Cli, args: &BisectArgs) -> anyhow::Result<()> {
    let (config_path, config) = load_config(&cli.config)?;
    let specs = load_specs(&config, &config_path)?;

    let (_runner_name, runner, _resolved, _test_dir) =
        resolve_primary_runner(&config, &config_path)?;

    if !runner.is_available() {
        anyhow::bail!(
            "test runner '{}' is not available -- is the toolchain installed?",
            runner.name()
        );
    }

    let clause_id = ought_spec::ClauseId(args.clause.clone());
    let options = ought_analysis::bisect::BisectOptions {
        range: args.range.clone(),
        regenerate: args.regenerate,
    };

    eprintln!("Bisecting clause {}...", clause_id);

    let result = ought_analysis::bisect::bisect(&clause_id, &specs, runner.as_ref(), &options)?;

    if cli.json {
        println!(
            "{{\"clause_id\": {:?}, \"breaking_commit\": {{\"hash\": {:?}, \"message\": {:?}, \"author\": {:?}}}, \"diff_summary\": {:?}}}",
            result.clause_id.0,
            result.breaking_commit.hash,
            result.breaking_commit.message,
            result.breaking_commit.author,
            result.diff_summary
        );
    } else {
        eprintln!("\x1b[1mBisect result for {}\x1b[0m\n", result.clause_id);
        eprintln!(
            "Breaking commit: \x1b[31m{}\x1b[0m",
            &result.breaking_commit.hash[..7.min(result.breaking_commit.hash.len())]
        );
        eprintln!("  Message: {}", result.breaking_commit.message);
        eprintln!("  Author:  {}", result.breaking_commit.author);
        eprintln!("  Date:    {}", result.breaking_commit.date);
        if !result.diff_summary.is_empty() {
            eprintln!("\nDiff summary:\n{}", result.diff_summary);
        }
    }

    Ok(())
}
