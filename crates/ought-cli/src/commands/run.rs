use std::process;

use ought_gen::manifest::Manifest;
use ought_report::types::ReportOptions;

use super::{
    collect_all_clause_info, collect_generated_tests, load_config, load_specs,
    resolve_primary_runner,
};
use crate::{Cli, RunArgs};

pub fn run(cli: &Cli, args: &RunArgs) -> anyhow::Result<()> {
    let (config_path, config) = load_config(&cli.config)?;
    let specs = load_specs(&config, &config_path)?;

    let (runner_name, runner, resolved, test_dir) = resolve_primary_runner(&config, &config_path)?;

    if !runner.is_available() {
        anyhow::bail!(
            "test runner '{}' is not available — is the toolchain installed?",
            runner.name()
        );
    }

    let manifest_path = test_dir.join("manifest.toml");
    let _manifest = Manifest::load(&manifest_path).unwrap_or_default();

    let _ = runner_name;
    let generated_tests = collect_generated_tests(&test_dir, &resolved.file_extensions)?;

    if generated_tests.is_empty() {
        eprintln!("No generated tests found. Run `ought generate` first.");
        let empty_results = ought_run::RunResult {
            results: vec![],
            total_duration: std::time::Duration::ZERO,
        };
        if let Some(junit_path) = &cli.junit {
            ought_report::junit::report(&empty_results, specs.specs(), junit_path)?;
        }
        if cli.json {
            let json = ought_report::json::report(&empty_results, specs.specs())?;
            println!("{}", json);
        }
        return Ok(());
    }

    let results = runner.run(&generated_tests, &test_dir)?;

    let report_opts = ReportOptions {
        quiet: cli.quiet,
        color: cli.color.to_report_color(),
    };

    if cli.json {
        let json = ought_report::json::report(&results, specs.specs())?;
        println!("{}", json);
    } else {
        ought_report::terminal::report(&results, specs.specs(), &report_opts)?;
    }

    if let Some(junit_path) = &cli.junit {
        ought_report::junit::report(&results, specs.specs(), junit_path)?;
    }

    // Exit code logic: exit 1 if any Required-severity (MUST/MUST NOT) test
    // failed or errored. Also exit 1 if --fail-on-should and any SHOULD test failed.
    // A failed MUST clause is forgiven if an OTHERWISE clause in its chain passed.
    let clause_info = collect_all_clause_info(&specs);
    let result_map: std::collections::HashMap<&str, &ought_run::TestResult> = results
        .results
        .iter()
        .map(|r| (r.clause_id.0.as_str(), r))
        .collect();

    let has_hard_failure = results.results.iter().any(|r| {
        let is_failure =
            r.status == ought_run::TestStatus::Failed || r.status == ought_run::TestStatus::Errored;
        if !is_failure {
            return false;
        }
        let info = clause_info
            .get(r.clause_id.0.as_str())
            .or_else(|| {
                let needle = r.clause_id.0.as_str();
                clause_info
                    .iter()
                    .find(|(k, _)| needle.ends_with(k.as_str()) || k.ends_with(needle))
                    .map(|(_, v)| v)
            });

        let severity = info
            .map(|i| i.severity)
            .unwrap_or(ought_spec::Severity::Required);

        if let Some(info) = info
            && !info.otherwise_ids.is_empty()
        {
            let otherwise_passed = info.otherwise_ids.iter().any(|ow_id| {
                let ow_suffix = ow_id.rsplit("::").next().unwrap_or(ow_id.as_str());
                result_map
                    .get(ow_id.as_str())
                    .or_else(|| {
                        result_map
                            .iter()
                            .find(|(k, _)| k.ends_with(ow_suffix))
                            .map(|(_, v)| v)
                    })
                    .map(|tr| tr.status == ought_run::TestStatus::Passed)
                    .unwrap_or(false)
            });
            if otherwise_passed {
                return false;
            }
        }

        match severity {
            ought_spec::Severity::Required => true,
            ought_spec::Severity::Recommended | ought_spec::Severity::Optional => {
                args.fail_on_should
            }
            ought_spec::Severity::NegativeConfirmation => false,
        }
    });

    if has_hard_failure {
        process::exit(1);
    }

    Ok(())
}
