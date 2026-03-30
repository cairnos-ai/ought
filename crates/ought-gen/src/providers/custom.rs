use std::path::PathBuf;

use ought_spec::Clause;

use crate::context::GenerationContext;
use crate::generator::{ClauseGroup, GeneratedTest, Generator};

use super::{build_batch_prompt, build_prompt, derive_file_path, exec_cli, parse_batch_response};

/// Generates tests by exec-ing an arbitrary user-specified executable.
pub struct CustomGenerator {
    executable: PathBuf,
}

impl CustomGenerator {
    pub fn new(executable: PathBuf) -> Self {
        Self { executable }
    }

    fn exec_prompt(&self, prompt: &str) -> anyhow::Result<String> {
        let exe_str = self
            .executable
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("executable path is not valid UTF-8"))?;
        exec_cli(exe_str, &[], prompt)
    }
}

impl Generator for CustomGenerator {
    fn generate(
        &self,
        clause: &Clause,
        context: &GenerationContext,
    ) -> anyhow::Result<GeneratedTest> {
        let prompt = build_prompt(clause, context);
        let code = self.exec_prompt(&prompt)?;
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
        let response = self.exec_prompt(&prompt)?;
        Ok(parse_batch_response(&response, group, context.target_language))
    }
}
