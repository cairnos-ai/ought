use ought_gen::manifest::Manifest;

use super::{load_config, load_specs, primary_test_dir};
use crate::Cli;

pub fn run(cli: &Cli) -> anyhow::Result<()> {
    let (config_path, config) = load_config(&cli.config)?;
    let specs = load_specs(&config, &config_path)?;

    let test_dir = primary_test_dir(&config, &config_path);

    let manifest_path = test_dir.join("manifest.toml");
    let manifest = Manifest::load(&manifest_path).unwrap_or_default();

    struct StaleClause {
        id: String,
        keyword: ought_spec::Keyword,
        text: String,
        reason: String,
    }

    struct SpecDiff {
        spec_file: String,
        stale: Vec<StaleClause>,
        total: usize,
    }

    let mut diffs: Vec<SpecDiff> = Vec::new();

    for spec in specs.specs() {
        let spec_file = spec
            .source_path
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| spec.name.clone());

        let mut stale_clauses = Vec::new();
        let mut total = 0;

        fn collect_stale(
            sections: &[ought_spec::Section],
            manifest: &Manifest,
            stale_clauses: &mut Vec<StaleClause>,
            total: &mut usize,
        ) {
            for section in sections {
                for clause in &section.clauses {
                    if clause.keyword == ought_spec::Keyword::Given {
                        continue;
                    }
                    if clause.pending {
                        continue;
                    }
                    *total += 1;
                    if manifest.is_stale(&clause.id, &clause.content_hash, "") {
                        let reason = match manifest.entries.get(&clause.id.0) {
                            Some(entry) => {
                                if entry.clause_hash != clause.content_hash {
                                    "clause changed".to_string()
                                } else {
                                    "source changed".to_string()
                                }
                            }
                            None => "new clause".to_string(),
                        };
                        stale_clauses.push(StaleClause {
                            id: clause.id.0.clone(),
                            keyword: clause.keyword,
                            text: clause.text.clone(),
                            reason,
                        });
                    }
                }
                collect_stale(&section.subsections, manifest, stale_clauses, total);
            }
        }

        collect_stale(&spec.sections, &manifest, &mut stale_clauses, &mut total);
        diffs.push(SpecDiff {
            spec_file,
            stale: stale_clauses,
            total,
        });
    }

    let mut any_stale = false;
    for diff in &diffs {
        if diff.stale.is_empty() {
            continue;
        }
        any_stale = true;
        println!("--- {}", diff.spec_file);
        println!("+++ {} (pending)", diff.spec_file);
        println!("@@ {}/{} clauses stale @@", diff.stale.len(), diff.total);
        for sc in &diff.stale {
            let kw = ought_gen::keyword_str(sc.keyword);
            println!("  M {}  ({}, {} {})", sc.id, sc.reason, kw, sc.text);
        }
        println!();
    }

    if !any_stale {
        println!("All generated tests are up to date.");
    }

    Ok(())
}
