use ought_spec::Clause;

use crate::context::GenerationContext;
use crate::generator::{GeneratedTest, Generator};

use super::{build_prompt, derive_file_path, exec_cli};

/// Generates tests by exec-ing the `claude` CLI.
pub struct ClaudeGenerator {
    model: Option<String>,
}

impl ClaudeGenerator {
    pub fn new(model: Option<String>) -> Self {
        Self { model }
    }
}

impl Generator for ClaudeGenerator {
    fn generate(
        &self,
        clause: &Clause,
        context: &GenerationContext,
    ) -> anyhow::Result<GeneratedTest> {
        let prompt = build_prompt(clause, context);

        let mut args = vec!["--print", "-p"];
        if let Some(ref model) = self.model {
            args.push("--model");
            args.push(model.as_str());
        }

        let code = exec_cli("claude", &args, &prompt)?;
        let file_path = derive_file_path(clause, context.target_language);

        Ok(GeneratedTest {
            clause_id: clause.id.clone(),
            code,
            language: context.target_language,
            file_path,
        })
    }
}
