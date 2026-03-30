use ought_spec::Clause;

use crate::context::GenerationContext;
use crate::generator::{ClauseGroup, GeneratedTest, Generator};

use super::{
    build_batch_prompt, build_prompt, derive_file_path, exec_cli, exec_cli_verbose,
    parse_batch_response,
};

/// Generates tests by exec-ing the `ollama` CLI for local models.
pub struct OllamaGenerator {
    model: String,
}

impl OllamaGenerator {
    pub fn new(model: String) -> Self {
        Self { model }
    }

    fn exec_prompt(&self, prompt: &str, verbose: bool) -> anyhow::Result<String> {
        if verbose {
            exec_cli_verbose("ollama", &["run", &self.model], Some(prompt))
        } else {
            exec_cli("ollama", &["run", &self.model], prompt)
        }
    }
}

impl Generator for OllamaGenerator {
    fn generate(
        &self,
        clause: &Clause,
        context: &GenerationContext,
    ) -> anyhow::Result<GeneratedTest> {
        let prompt = build_prompt(clause, context);
        let code = self.exec_prompt(&prompt, context.verbose)?;
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
        if group.clauses.len() == 1 {
            return Ok(vec![self.generate(group.clauses[0], context)?]);
        }

        let prompt = build_batch_prompt(group, context);
        let response = self.exec_prompt(&prompt, context.verbose)?;
        Ok(parse_batch_response(&response, group, context.target_language))
    }
}
