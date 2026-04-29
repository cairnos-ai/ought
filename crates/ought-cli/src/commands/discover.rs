//! `ought discover` - find source behavior without matching specs.

use std::path::Path;

use ought_gen::{AlignMode, AlignOrchestrator, AlignReport};

use super::{load_config, load_specs};
use crate::{Cli, DiscoverArgs};

pub fn run(cli: &Cli, args: &DiscoverArgs) -> anyhow::Result<()> {
    let (config_path, config) = load_config(&cli.config)?;
    let config_dir = config_path.parent().unwrap_or(Path::new(".")).to_path_buf();
    let specs = load_specs(&config, &config_path)?;
    let specs_root = config
        .specs
        .roots
        .first()
        .cloned()
        .map(|root| config_dir.join(root))
        .unwrap_or_else(|| config_dir.join("ought"));

    let search_paths =
        super::align::resolve_search_paths(&config_dir, &config.context.search_paths, &args.paths);
    super::align::validate_required_search_paths(&search_paths)?;

    let mut candidates = super::align::build_discover_candidates(
        &search_paths,
        &specs,
        &specs_root,
        &config_dir,
        config.context.max_files,
    );
    candidates.sort_by(|a, b| a.target_spec_path.cmp(&b.target_spec_path));

    if candidates.is_empty() {
        let report = AlignReport::from_parts(args.apply, vec![], vec![]);
        if cli.json {
            println!("{}", serde_json::to_string_pretty(&report)?);
        } else {
            eprintln!("No missing specs discovered.");
        }
        return Ok(());
    }

    let parallelism = args
        .parallelism
        .unwrap_or(config.generator.parallelism)
        .max(1);
    let assignments = super::align::build_assignments(
        candidates,
        super::align::AssignmentOptions {
            mode: AlignMode::Discover,
            config_path: &config_path,
            specs_root: &specs_root,
            project_root: &config_dir,
            focus: args.focus.clone(),
            apply: args.apply,
            only: Some(ought_gen::AlignChangeKind::Add),
            parallelism,
        },
    );

    let mut gen_cfg = config.generator.clone();
    if let Some(ref model) = args.model {
        gen_cfg.model = model.clone();
    }
    gen_cfg.parallelism = parallelism;

    if !cli.json {
        eprintln!(
            "{} discovery candidate(s){}",
            assignments
                .iter()
                .map(|a| a.candidates.len())
                .sum::<usize>(),
            if args.apply { " (apply)" } else { "" }
        );
    }

    let orchestrator = AlignOrchestrator::new(gen_cfg, cli.verbose);
    let report = tokio::runtime::Runtime::new()?.block_on(orchestrator.run(assignments))?;

    if cli.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        super::align::render_human(&report, "Discovery report", "No missing specs discovered.");
    }

    if !report.errors.is_empty() {
        anyhow::bail!("discovery failed with {} error(s)", report.errors.len());
    }

    Ok(())
}
