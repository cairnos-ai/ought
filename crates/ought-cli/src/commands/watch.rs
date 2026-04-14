use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

use notify::{RecursiveMode, Watcher};

use ought_cli::config::Config;
use ought_report::types::ReportOptions;

use super::{collect_generated_tests, load_config, load_specs, resolve_primary_runner};
use crate::Cli;

pub fn run(cli: &Cli) -> anyhow::Result<()> {
    let (config_path, config) = load_config(&cli.config)?;
    let config_dir = config_path
        .parent()
        .unwrap_or(std::path::Path::new("."))
        .to_path_buf();

    let spec_roots: Vec<PathBuf> = config
        .specs
        .roots
        .iter()
        .map(|r| config_dir.join(r))
        .collect();

    let source_paths: Vec<PathBuf> = config
        .context
        .search_paths
        .iter()
        .map(|p| config_dir.join(p))
        .collect();

    fn run_cycle(cli: &Cli, config_path: &std::path::Path, config: &Config) {
        eprint!("\x1b[2J\x1b[H");

        let specs = match load_specs(config, config_path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("error loading specs: {}", e);
                return;
            }
        };

        eprintln!(" ought watch: checking {} spec(s)...", specs.specs().len());
        for spec in specs.specs() {
            if let Some(name) = spec.source_path.file_name() {
                eprintln!("  {}", name.to_string_lossy());
            }
        }

        let (_runner_name, runner, resolved, test_dir) =
            match resolve_primary_runner(config, config_path) {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("error creating runner: {}", e);
                    return;
                }
            };

        if !runner.is_available() {
            eprintln!("runner '{}' is not available", runner.name());
            return;
        }

        let generated_tests = match collect_generated_tests(&test_dir, &resolved.file_extensions) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("error collecting tests: {}", e);
                return;
            }
        };

        if generated_tests.is_empty() {
            eprintln!("No generated tests found. Run `ought generate` first.");
            return;
        }

        let results = match runner.run(&generated_tests, &test_dir) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("error running tests: {}", e);
                return;
            }
        };

        let report_opts = ReportOptions {
            quiet: cli.quiet,
            color: cli.color.to_report_color(),
        };

        if cli.json {
            if let Ok(json) = ought_report::json::report(&results, specs.specs()) {
                println!("{}", json);
            }
        } else {
            let _ = ought_report::terminal::report(&results, specs.specs(), &report_opts);
        }
    }

    eprintln!("ought watch: running initial cycle...");
    run_cycle(cli, &config_path, &config);

    let (tx, rx) = mpsc::channel();
    let source_paths_for_filter: Vec<PathBuf> = source_paths
        .iter()
        .map(|p| p.canonicalize().unwrap_or_else(|_| p.clone()))
        .collect();
    let mut watcher = notify::recommended_watcher(move |res: Result<notify::Event, _>| {
        if let Ok(event) = res {
            let dominated = matches!(
                event.kind,
                notify::EventKind::Modify(_)
                    | notify::EventKind::Create(_)
                    | notify::EventKind::Remove(_)
            );
            if !dominated {
                return;
            }
            let is_relevant = event.paths.iter().any(|p| {
                if p.to_str().is_some_and(|s| s.ends_with(".ought.md")) {
                    return true;
                }
                source_paths_for_filter.iter().any(|src| p.starts_with(src))
            });
            if is_relevant {
                let _ = tx.send(());
            }
        }
    })?;

    for root in &spec_roots {
        if root.exists() {
            watcher.watch(root, RecursiveMode::Recursive)?;
        }
    }
    for path in &source_paths {
        let p: &std::path::Path = path.as_ref();
        if p.exists() {
            watcher.watch(p, RecursiveMode::Recursive)?;
        }
    }

    eprintln!("ought watch: watching for changes...");

    let debounce = Duration::from_millis(500);

    while let Ok(()) = rx.recv() {
        loop {
            match rx.recv_timeout(debounce) {
                Ok(()) => {}
                Err(mpsc::RecvTimeoutError::Timeout) => break,
                Err(mpsc::RecvTimeoutError::Disconnected) => return Ok(()),
            }
        }

        while rx.try_recv().is_ok() {}

        let config = match Config::load(&config_path) {
            Ok(c) => c,
            Err(e) => {
                eprint!("\x1b[2J\x1b[H");
                eprintln!("error reloading config: {}", e);
                continue;
            }
        };

        run_cycle(cli, &config_path, &config);
    }

    Ok(())
}
