use super::{load_config, load_specs};
use crate::Cli;

pub fn run(cli: &Cli) -> anyhow::Result<()> {
    let (config_path, config) = load_config(&cli.config)?;
    let specs = load_specs(&config, &config_path)?;

    let result = ought_analysis::audit::audit(&specs)?;

    if cli.json {
        println!("{{");
        println!("  \"findings\": [");
        for (i, finding) in result.findings.iter().enumerate() {
            let comma = if i + 1 < result.findings.len() {
                ","
            } else {
                ""
            };
            let kind = match finding.kind {
                ought_analysis::AuditFindingKind::Contradiction => "contradiction",
                ought_analysis::AuditFindingKind::Gap => "gap",
                ought_analysis::AuditFindingKind::Ambiguity => "ambiguity",
                ought_analysis::AuditFindingKind::Redundancy => "redundancy",
            };
            let clauses_json: Vec<String> =
                finding.clauses.iter().map(|c| format!("{:?}", c.0)).collect();
            println!(
                "    {{\"kind\": {:?}, \"description\": {:?}, \"clauses\": [{}], \"suggestion\": {:?}, \"confidence\": {:?}}}{}",
                kind,
                finding.description,
                clauses_json.join(", "),
                finding.suggestion,
                finding.confidence,
                comma
            );
        }
        println!("  ]");
        println!("}}");
    } else if result.findings.is_empty() {
        eprintln!("No issues found. Specs are coherent.");
    } else {
        eprintln!("Found {} issues:\n", result.findings.len());
        for finding in &result.findings {
            let kind_str = match finding.kind {
                ought_analysis::AuditFindingKind::Contradiction => {
                    "\x1b[31mCONTRADICTION\x1b[0m"
                }
                ought_analysis::AuditFindingKind::Gap => "\x1b[33mGAP\x1b[0m",
                ought_analysis::AuditFindingKind::Ambiguity => "\x1b[34mAMBIGUITY\x1b[0m",
                ought_analysis::AuditFindingKind::Redundancy => "\x1b[36mREDUNDANCY\x1b[0m",
            };
            eprintln!("  [{}] {}", kind_str, finding.description);
            if !finding.clauses.is_empty() {
                eprintln!(
                    "    Clauses: {}",
                    finding
                        .clauses
                        .iter()
                        .map(|c| c.0.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
            if let Some(ref suggestion) = finding.suggestion {
                eprintln!("    Suggestion: {}", suggestion);
            }
            if let Some(confidence) = finding.confidence {
                eprintln!("    Confidence: {:.0}%", confidence * 100.0);
            }
            eprintln!();
        }
    }

    Ok(())
}
