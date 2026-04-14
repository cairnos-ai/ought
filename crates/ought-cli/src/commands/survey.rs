use std::path::PathBuf;

use super::{load_config, load_specs};
use crate::{Cli, SurveyArgs};

pub fn run(cli: &Cli, args: &SurveyArgs) -> anyhow::Result<()> {
    let (config_path, config) = load_config(&cli.config)?;
    let specs = load_specs(&config, &config_path)?;

    let paths: Vec<PathBuf> = if let Some(ref path) = args.path {
        vec![path.clone()]
    } else {
        let config_dir = config_path
            .parent()
            .unwrap_or(std::path::Path::new("."))
            .to_path_buf();
        config
            .context
            .search_paths
            .iter()
            .map(|p| config_dir.join(p))
            .collect()
    };

    let result = ought_analysis::survey::survey(&specs, &paths)?;

    if cli.json {
        println!("{{");
        println!("  \"uncovered\": [");
        for (i, item) in result.uncovered.iter().enumerate() {
            let comma = if i + 1 < result.uncovered.len() {
                ","
            } else {
                ""
            };
            println!(
                "    {{\"file\": {:?}, \"line\": {}, \"description\": {:?}, \"suggested_clause\": {:?}, \"suggested_spec\": {:?}}}{}",
                item.file.display().to_string(),
                item.line,
                item.description,
                item.suggested_clause,
                item.suggested_spec.display().to_string(),
                comma
            );
        }
        println!("  ]");
        println!("}}");
    } else if result.uncovered.is_empty() {
        eprintln!("No uncovered behaviors found.");
    } else {
        eprintln!("Found {} uncovered behaviors:\n", result.uncovered.len());
        let mut current_spec: Option<&std::path::Path> = None;
        for item in &result.uncovered {
            if current_spec != Some(&item.suggested_spec) {
                eprintln!("  \x1b[1m{}\x1b[0m", item.suggested_spec.display());
                current_spec = Some(&item.suggested_spec);
            }
            eprintln!(
                "    {}:{} - {}",
                item.file.display(),
                item.line,
                item.description
            );
            eprintln!("      Suggested: \x1b[33m{}\x1b[0m", item.suggested_clause);
        }
    }

    Ok(())
}
