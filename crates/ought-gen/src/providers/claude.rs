use ought_spec::Clause;

use crate::context::GenerationContext;
use crate::generator::{ClauseGroup, GeneratedTest, Generator};

use super::{build_batch_prompt, build_prompt, derive_file_path, exec_cli_with_arg, parse_batch_response};

/// Generates tests by exec-ing the `claude` CLI.
pub struct ClaudeGenerator {
    model: Option<String>,
}

impl ClaudeGenerator {
    pub fn new(model: Option<String>) -> Self {
        Self { model }
    }

    fn build_args(&self, prompt: String) -> Vec<String> {
        let mut args: Vec<String> = vec!["-p".into()];
        if let Some(ref model) = self.model {
            args.push("--model".into());
            args.push(model.clone());
        }
        args.push(prompt);
        args
    }
}

impl Generator for ClaudeGenerator {
    fn generate(
        &self,
        clause: &Clause,
        context: &GenerationContext,
    ) -> anyhow::Result<GeneratedTest> {
        let prompt = build_prompt(clause, context);
        let args = self.build_args(prompt);
        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let code = exec_cli_with_arg("claude", &args_ref)?;
        let file_path = derive_file_path(clause, context.target_language);

        Ok(GeneratedTest {
            clause_id: clause.id.clone(),
            code,
            language: context.target_language,
            file_path,
        })
    }

    fn generate_batch(
        &self,
        group: &ClauseGroup<'_>,
        context: &GenerationContext,
    ) -> anyhow::Result<Vec<GeneratedTest>> {
        if group.clauses.is_empty() {
            return Ok(vec![]);
        }

        // For single clause, use the simpler per-clause path
        if group.clauses.len() == 1 {
            let test = self.generate(group.clauses[0], context)?;
            return Ok(vec![test]);
        }

        let prompt = build_batch_prompt(group, context);
        let args = self.build_args(prompt);
        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let response = exec_cli_with_arg("claude", &args_ref)?;

        Ok(parse_batch_response(&response, group, context.target_language))
    }
}
