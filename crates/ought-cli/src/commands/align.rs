//! `ought align` — reconcile `.ought.md` specs with source code.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use ought_gen::{
    AlignAppliedStatus, AlignAssignment, AlignCandidate, AlignChangeKind, AlignMode,
    AlignOrchestrator, AlignReport,
};
use ought_spec::SpecGraph;

use super::{load_config, load_specs};
use crate::{AlignArgs, Cli};

pub fn run(cli: &Cli, args: &AlignArgs) -> anyhow::Result<()> {
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

    let search_paths = resolve_search_paths(&config_dir, &config.context.search_paths, &args.paths);
    validate_existing_search_paths(&search_paths)?;

    let mut candidates = build_align_candidates(&specs, &specs_root, &config_dir, &search_paths);
    candidates.sort_by(|a, b| {
        a.kind
            .cmp(&b.kind)
            .then_with(|| a.target_spec_path.cmp(&b.target_spec_path))
    });

    if candidates.is_empty() {
        let report = AlignReport::from_parts(false, vec![], vec![]);
        if cli.json {
            println!("{}", serde_json::to_string_pretty(&report)?);
        } else {
            eprintln!("No mapped specs need alignment.");
        }
        return Ok(());
    }

    let parallelism = args
        .parallelism
        .unwrap_or(config.generator.parallelism)
        .max(1);
    let assignments = build_assignments(
        candidates,
        AssignmentOptions {
            mode: AlignMode::Align,
            config_path: &config_path,
            specs_root: &specs_root,
            project_root: &config_dir,
            focus: None,
            apply: false,
            only: None,
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
            "{} alignment candidate(s) (report only)",
            assignments
                .iter()
                .map(|a| a.candidates.len())
                .sum::<usize>()
        );
    }

    let orchestrator = AlignOrchestrator::new(gen_cfg, cli.verbose);
    let report = tokio::runtime::Runtime::new()?.block_on(orchestrator.run(assignments))?;

    if cli.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        render_human(
            &report,
            "Alignment report",
            "No mapped specs need alignment.",
        );
    }

    if !report.errors.is_empty() {
        anyhow::bail!("alignment failed with {} error(s)", report.errors.len());
    }

    Ok(())
}

pub(crate) fn render_human(report: &AlignReport, heading: &str, empty_message: &str) {
    if report.changes.is_empty() && report.errors.is_empty() {
        eprintln!("{}", empty_message);
        return;
    }

    eprintln!(
        "\n{}: {} change(s) ({} add, {} update, {} remove)",
        heading,
        report.summary.total,
        report.summary.add,
        report.summary.update,
        report.summary.remove
    );
    if report.applied {
        eprintln!("Applied: {}", report.summary.applied);
    }

    for change in &report.changes {
        eprintln!(
            "\n  [{}] {}",
            change.kind.as_str().to_uppercase(),
            change.target_spec
        );
        eprintln!("    {}", change.summary);
        if !change.source_files.is_empty() {
            eprintln!("    Source: {}", change.source_files.join(", "));
        }
        if let Some(confidence) = change.confidence {
            eprintln!("    Confidence: {:.2}", confidence);
        }
        eprintln!(
            "    Status: {}",
            applied_status_label(&change.applied_status)
        );
        if !change.rationale.is_empty() {
            eprintln!("    Rationale: {}", change.rationale);
        }
    }

    for error in &report.errors {
        eprintln!("\n  error:");
        for line in error.lines() {
            eprintln!("    {}", line);
        }
    }
}

pub(crate) fn applied_status_label(status: &AlignAppliedStatus) -> String {
    match status {
        AlignAppliedStatus::NotApplied => "not applied".to_string(),
        AlignAppliedStatus::Written { path } => format!("written ({})", path),
        AlignAppliedStatus::MarkedStale { path } => format!("marked stale ({})", path),
        AlignAppliedStatus::Rejected { errors } => format!("rejected ({})", errors.join("; ")),
        AlignAppliedStatus::Skipped { reason } => format!("skipped ({})", reason),
        AlignAppliedStatus::Errored { error } => format!("errored ({})", error),
    }
}

pub(crate) fn resolve_search_paths(
    config_dir: &Path,
    configured: &[PathBuf],
    explicit: &[PathBuf],
) -> Vec<PathBuf> {
    let inputs = if explicit.is_empty() {
        configured
    } else {
        explicit
    };
    inputs
        .iter()
        .map(|path| resolve_project_path(config_dir, path))
        .collect()
}

pub(crate) fn validate_existing_search_paths(search_paths: &[PathBuf]) -> anyhow::Result<()> {
    for path in search_paths {
        if !path.exists() {
            anyhow::bail!("source path does not exist: {}", path.display());
        }
    }
    Ok(())
}

pub(crate) fn validate_required_search_paths(search_paths: &[PathBuf]) -> anyhow::Result<()> {
    if search_paths.is_empty() {
        anyhow::bail!(
            "no source paths: pass paths as arguments or set [context].search_paths in ought.toml"
        );
    }
    validate_existing_search_paths(search_paths)
}

pub(crate) fn build_align_candidates(
    specs: &SpecGraph,
    specs_root: &Path,
    project_root: &Path,
    search_paths: &[PathBuf],
) -> Vec<AlignCandidate> {
    let mut by_key: BTreeMap<(AlignChangeKind, String), AlignCandidate> = BTreeMap::new();

    add_spec_metadata_candidates(
        &mut by_key,
        specs,
        specs_root,
        project_root,
        search_paths,
        None,
    );

    by_key.into_values().collect()
}

pub(crate) fn build_discover_candidates(
    search_paths: &[PathBuf],
    specs: &SpecGraph,
    specs_root: &Path,
    project_root: &Path,
    max_files: usize,
) -> Vec<AlignCandidate> {
    let covered_sources = covered_source_paths(specs, project_root);
    let mut by_key: BTreeMap<(AlignChangeKind, String), AlignCandidate> = BTreeMap::new();

    for group in build_source_groups(search_paths, project_root, max_files) {
        if specs_root.join(&group.target_spec_path).exists() {
            continue;
        }

        let source_files = group
            .source_files
            .into_iter()
            .filter(|source| {
                let resolved = project_root.join(source);
                !is_under_any(&resolved, &covered_sources)
            })
            .collect::<Vec<_>>();

        if source_files.is_empty() {
            continue;
        }

        insert_candidate(
            &mut by_key,
            AlignCandidate {
                kind: AlignChangeKind::Add,
                title: group.title,
                target_spec_path: group.target_spec_path,
                source_files,
            },
        );
    }

    by_key.into_values().collect()
}

fn covered_source_paths(specs: &SpecGraph, project_root: &Path) -> Vec<PathBuf> {
    let mut covered = BTreeSet::new();
    for spec in specs.specs() {
        for source in &spec.metadata.sources {
            let resolved = resolve_spec_source(&spec.source_path, project_root, source);
            if resolved.exists() {
                covered.insert(resolved);
            }
        }
    }
    covered.into_iter().collect()
}

fn add_spec_metadata_candidates(
    by_key: &mut BTreeMap<(AlignChangeKind, String), AlignCandidate>,
    specs: &SpecGraph,
    specs_root: &Path,
    project_root: &Path,
    search_paths: &[PathBuf],
    only: Option<AlignChangeKind>,
) {
    for spec in specs.specs() {
        let target_spec_path = relative_display(&spec.source_path, specs_root);
        let mut existing_sources = BTreeSet::new();
        let mut missing_sources = BTreeSet::new();

        for source in &spec.metadata.sources {
            let resolved = resolve_spec_source(&spec.source_path, project_root, source);
            let display = relative_display(&resolved, project_root);
            if resolved.exists() {
                if search_paths.is_empty() || is_under_any(&resolved, search_paths) {
                    existing_sources.insert(display);
                }
            } else {
                missing_sources.insert(display);
            }
        }

        if !existing_sources.is_empty() && kind_allowed(AlignChangeKind::Update, only) {
            insert_candidate(
                by_key,
                AlignCandidate {
                    kind: AlignChangeKind::Update,
                    title: spec.name.clone(),
                    target_spec_path: target_spec_path.clone(),
                    source_files: existing_sources.iter().cloned().collect(),
                },
            );
        }

        if spec.metadata.sources.is_empty() {
            continue;
        }
        if existing_sources.is_empty()
            && !missing_sources.is_empty()
            && kind_allowed(AlignChangeKind::Remove, only)
        {
            insert_candidate(
                by_key,
                AlignCandidate {
                    kind: AlignChangeKind::Remove,
                    title: spec.name.clone(),
                    target_spec_path,
                    source_files: missing_sources.iter().cloned().collect(),
                },
            );
        }
    }
}

fn insert_candidate(
    by_key: &mut BTreeMap<(AlignChangeKind, String), AlignCandidate>,
    candidate: AlignCandidate,
) {
    let key = (candidate.kind, candidate.target_spec_path.clone());
    by_key
        .entry(key)
        .and_modify(|existing| {
            let mut merged = existing
                .source_files
                .iter()
                .cloned()
                .collect::<BTreeSet<_>>();
            merged.extend(candidate.source_files.iter().cloned());
            existing.source_files = merged.into_iter().collect();
        })
        .or_insert(candidate);
}

pub(crate) struct AssignmentOptions<'a> {
    pub mode: AlignMode,
    pub config_path: &'a Path,
    pub specs_root: &'a Path,
    pub project_root: &'a Path,
    pub focus: Option<String>,
    pub apply: bool,
    pub only: Option<AlignChangeKind>,
    pub parallelism: usize,
}

pub(crate) fn build_assignments(
    candidates: Vec<AlignCandidate>,
    options: AssignmentOptions<'_>,
) -> Vec<AlignAssignment> {
    let worker_count = options.parallelism.min(candidates.len()).max(1);
    let mut buckets = vec![Vec::new(); worker_count];
    for (index, candidate) in candidates.into_iter().enumerate() {
        buckets[index % worker_count].push(candidate);
    }

    buckets
        .into_iter()
        .enumerate()
        .filter(|(_, candidates)| !candidates.is_empty())
        .map(|(index, candidates)| AlignAssignment {
            id: format!("{}_{}", options.mode.as_str(), index),
            mode: options.mode,
            project_root: options.project_root.to_string_lossy().into_owned(),
            config_path: options.config_path.to_string_lossy().into_owned(),
            specs_root: options.specs_root.to_string_lossy().into_owned(),
            focus: options.focus.clone(),
            apply: options.apply,
            only: options.only,
            candidates,
        })
        .collect()
}

#[derive(Debug)]
struct SourceGroup {
    title: String,
    target_spec_path: String,
    source_files: Vec<String>,
}

fn build_source_groups(
    search_paths: &[PathBuf],
    project_root: &Path,
    max_files: usize,
) -> Vec<SourceGroup> {
    let mut groups: BTreeMap<String, (String, BTreeSet<String>)> = BTreeMap::new();
    let mut file_count = 0usize;

    for root in search_paths {
        if !root.exists() {
            continue;
        }
        let root_name = root
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("root")
            .to_string();

        walk_source_dir(root, &mut |path| {
            if file_count >= max_files {
                return;
            }
            let rel = match path.strip_prefix(root) {
                Ok(rel) => rel,
                Err(_) => return,
            };
            let first = rel.components().next();
            let (key, title) = match first {
                Some(std::path::Component::Normal(os)) => {
                    let name = os.to_string_lossy().to_string();
                    if rel.components().count() == 1 {
                        (root_name.clone(), root_name.clone())
                    } else {
                        (name.clone(), name)
                    }
                }
                _ => (root_name.clone(), root_name.clone()),
            };
            groups
                .entry(key)
                .or_insert_with(|| (title, BTreeSet::new()))
                .1
                .insert(relative_display(path, project_root));
            file_count += 1;
        });
    }

    if file_count >= max_files {
        eprintln!(
            "  note: stopped at max_files={} (set [context].max_files higher to include more)",
            max_files
        );
    }

    groups
        .into_iter()
        .map(|(key, (title, source_files))| SourceGroup {
            title: pretty_title(&title),
            target_spec_path: format!("{}.ought.md", key.replace(['/', '\\', '.'], "_")),
            source_files: source_files.into_iter().collect(),
        })
        .collect()
}

fn walk_source_dir(dir: &Path, visit: &mut impl FnMut(&Path)) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|name| name.to_str())
                && matches!(
                    name,
                    "target"
                        | "node_modules"
                        | ".git"
                        | "__pycache__"
                        | "vendor"
                        | ".venv"
                        | "venv"
                        | "dist"
                        | "build"
                        | "coverage"
                        | ".claude"
                )
            {
                continue;
            }
            walk_source_dir(&path, visit);
        } else if is_source_file(&path) {
            visit(&path);
        }
    }
}

fn is_source_file(path: &Path) -> bool {
    let ext = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
    matches!(
        ext,
        "rs" | "py"
            | "ts"
            | "tsx"
            | "js"
            | "jsx"
            | "go"
            | "java"
            | "rb"
            | "kt"
            | "swift"
            | "c"
            | "cpp"
            | "h"
            | "hpp"
    )
}

fn kind_allowed(kind: AlignChangeKind, only: Option<AlignChangeKind>) -> bool {
    only.is_none_or(|only| only == kind)
}

fn resolve_project_path(project_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        project_root.join(path)
    }
}

fn resolve_spec_source(spec_path: &Path, project_root: &Path, source: &str) -> PathBuf {
    let source_path = PathBuf::from(source);
    if source_path.is_absolute() {
        return source_path;
    }

    let from_project = project_root.join(&source_path);
    if from_project.exists() {
        return from_project;
    }

    spec_path.parent().unwrap_or(project_root).join(source_path)
}

fn is_under_any(path: &Path, roots: &[PathBuf]) -> bool {
    roots.iter().any(|root| path.starts_with(root))
}

fn relative_display(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn pretty_title(name: &str) -> String {
    name.replace(['_', '-'], " ")
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use ought_gen::AlignChangeKind;
    use ought_spec::parser::{OughtMdParser, Parser as _};
    use tempfile::tempdir;

    fn graph_from(root: &Path, spec_name: &str, content: &str) -> SpecGraph {
        let spec_path = root.join(spec_name);
        std::fs::create_dir_all(spec_path.parent().unwrap()).unwrap();
        std::fs::write(&spec_path, content).unwrap();
        let spec = OughtMdParser.parse_file(&spec_path).unwrap();
        SpecGraph::from_specs(vec![spec]).unwrap()
    }

    #[test]
    fn discover_candidates_add_for_new_source_group() {
        let tmp = tempdir().unwrap();
        let src = tmp.path().join("src/auth");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(src.join("login.rs"), "pub fn login() {}\n").unwrap();
        let specs_root = tmp.path().join("ought");
        std::fs::create_dir_all(&specs_root).unwrap();
        let graph = SpecGraph::from_specs(vec![]).unwrap();

        let candidates = build_discover_candidates(&[src], &graph, &specs_root, tmp.path(), 100);

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].kind, AlignChangeKind::Add);
        assert_eq!(candidates[0].target_spec_path, "auth.ought.md");
    }

    #[test]
    fn discover_candidates_skip_existing_target() {
        let tmp = tempdir().unwrap();
        let src = tmp.path().join("src/auth");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(src.join("login.rs"), "pub fn login() {}\n").unwrap();
        let specs_root = tmp.path().join("ought");
        std::fs::create_dir_all(&specs_root).unwrap();
        std::fs::write(specs_root.join("auth.ought.md"), "# Auth\n").unwrap();
        let graph = SpecGraph::from_specs(vec![]).unwrap();

        let candidates = build_discover_candidates(&[src], &graph, &specs_root, tmp.path(), 100);

        assert!(candidates.is_empty());
    }

    #[test]
    fn align_candidates_update_existing_source_mapping() {
        let tmp = tempdir().unwrap();
        let src = tmp.path().join("src");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(src.join("auth.rs"), "pub fn login() {}\n").unwrap();
        let specs_root = tmp.path().join("ought");
        let graph = graph_from(
            &specs_root,
            "auth.ought.md",
            "# Auth\n\nsource: src/auth.rs\n\n## Behavior\n\n- **MUST** work\n",
        );

        let candidates = build_align_candidates(&graph, &specs_root, tmp.path(), &[src]);

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].kind, AlignChangeKind::Update);
        assert_eq!(candidates[0].target_spec_path, "auth.ought.md");
    }

    #[test]
    fn align_candidates_remove_when_spec_source_missing() {
        let tmp = tempdir().unwrap();
        let specs_root = tmp.path().join("ought");
        let graph = graph_from(
            &specs_root,
            "old.ought.md",
            "# Old\n\nsource: src/old.rs\n\n## Behavior\n\n- **MUST** work\n",
        );

        let candidates = build_align_candidates(&graph, &specs_root, tmp.path(), &[]);

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].kind, AlignChangeKind::Remove);
        assert_eq!(candidates[0].target_spec_path, "old.ought.md");
    }

    #[test]
    fn discover_candidates_skip_mapped_source() {
        let tmp = tempdir().unwrap();
        let src = tmp.path().join("src");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(src.join("auth.rs"), "pub fn login() {}\n").unwrap();
        let specs_root = tmp.path().join("ought");
        let graph = graph_from(
            &specs_root,
            "auth.ought.md",
            "# Auth\n\nsource: src/auth.rs\n\n## Behavior\n\n- **MUST** work\n",
        );

        let candidates = build_discover_candidates(&[src], &graph, &specs_root, tmp.path(), 100);

        assert!(candidates.is_empty());
    }
}
