use super::{
    find_clause_by_id, find_clause_by_partial_id, load_config, load_specs, primary_test_dir,
};
use crate::{Cli, InspectArgs};

pub fn run(cli: &Cli, args: &InspectArgs) -> anyhow::Result<()> {
    let (config_path, config) = load_config(&cli.config)?;
    let test_dir = primary_test_dir(&config, &config_path);

    if let Ok(specs) = load_specs(&config, &config_path) {
        let clause = find_clause_by_id(&specs, &args.clause)
            .or_else(|| find_clause_by_partial_id(&specs, &args.clause));
        if let Some(clause) = clause {
            println!(
                "// Clause: {} {}",
                ought_gen::keyword_str(clause.keyword),
                clause.text
            );
            if let Some(ref cond) = clause.condition {
                println!("//   GIVEN: {}", cond);
            }
            println!();
        }
    }

    let clause_path = args.clause.replace("::", "/");
    let candidates = [
        test_dir.join(format!("{}_test.rs", clause_path)),
        test_dir.join(format!("{}.rs", clause_path)),
        test_dir.join(format!("{}_test.py", clause_path)),
        test_dir.join(format!("{}.py", clause_path)),
        test_dir.join(format!("{}.test.ts", clause_path)),
        test_dir.join(format!("{}.ts", clause_path)),
        test_dir.join(format!("{}_test.go", clause_path)),
        test_dir.join(format!("{}.go", clause_path)),
    ];

    for candidate in &candidates {
        if candidate.exists() {
            let content = std::fs::read_to_string(candidate)?;
            println!("{}", content);
            return Ok(());
        }
    }

    anyhow::bail!(
        "no generated test found for clause '{}'\nLooked in: {}",
        args.clause,
        test_dir.display()
    );
}
