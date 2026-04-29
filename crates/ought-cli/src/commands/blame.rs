use super::{collect_generated_tests, load_config, load_specs, resolve_primary_runner};
use crate::{BlameArgs, Cli};

pub fn run(cli: &Cli, args: &BlameArgs) -> anyhow::Result<()> {
    let (config_path, config) = load_config(&cli.config)?;
    let specs = load_specs(&config, &config_path)?;

    let (_runner_name, runner, resolved, test_dir) = resolve_primary_runner(&config, &config_path)?;

    let generated_tests = collect_generated_tests(&test_dir, &resolved.file_extensions)?;
    let results = if !generated_tests.is_empty() && runner.is_available() {
        runner.run(&generated_tests, &test_dir)?
    } else {
        ought_run::RunResult {
            results: vec![],
            total_duration: std::time::Duration::ZERO,
        }
    };

    let clause_id = ought_spec::ClauseId(args.clause.clone());
    let result = ought_analysis::blame::blame(&clause_id, &specs, &results)?;

    if cli.json {
        let commit_json = if let Some(ref c) = result.likely_commit {
            format!(
                "{{\"hash\": {:?}, \"message\": {:?}, \"author\": {:?}}}",
                c.hash, c.message, c.author
            )
        } else {
            "null".to_string()
        };
        println!(
            "{{\"clause_id\": {:?}, \"narrative\": {:?}, \"likely_commit\": {}, \"suggested_fix\": {:?}}}",
            result.clause_id.0, result.narrative, commit_json, result.suggested_fix
        );
    } else {
        eprintln!("\x1b[1mBlame: {}\x1b[0m\n", result.clause_id);
        eprintln!("{}", result.narrative);
        if let Some(ref commit) = result.likely_commit {
            eprintln!(
                "\nLikely commit: \x1b[33m{}\x1b[0m {} ({})",
                &commit.hash[..7.min(commit.hash.len())],
                commit.message,
                commit.author
            );
        }
        if let Some(ref fix) = result.suggested_fix {
            eprintln!("\nSuggested fix: {}", fix);
        }
    }

    Ok(())
}
