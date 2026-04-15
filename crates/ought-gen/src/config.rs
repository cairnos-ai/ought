use serde::{Deserialize, Serialize};

/// Configuration for the LLM test generator.
///
/// Composed into the aggregate `ought.toml` config by the CLI crate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratorConfig {
    /// Which upstream LLM provider to talk to.
    #[serde(default = "default_provider")]
    pub provider: Provider,

    /// Provider-specific model identifier (e.g. `"claude-sonnet-4-6"`).
    #[serde(default = "default_model")]
    pub model: String,

    /// Maximum number of model turns per assignment before giving up.
    #[serde(default = "default_max_turns")]
    pub max_turns: u32,

    /// Token cap on each individual model response.
    #[serde(default = "default_max_tokens_per_response")]
    pub max_tokens_per_response: u32,

    /// Optional sampling temperature.
    #[serde(default)]
    pub temperature: Option<f32>,

    #[serde(default)]
    pub tolerance: ToleranceConfig,

    /// Number of assignments to run concurrently.
    #[serde(default = "default_parallelism")]
    pub parallelism: usize,

    /// Anthropic-provider settings. Used when `provider = "anthropic"`.
    #[serde(default)]
    pub anthropic: AnthropicConfig,

    /// OpenAI-provider settings. Used when `provider = "openai"`.
    #[serde(default)]
    pub openai: OpenAiConfig,

    /// OpenRouter-provider settings. Used when `provider = "openrouter"`.
    #[serde(default)]
    pub openrouter: OpenRouterConfig,

    /// Ollama-provider settings. Used when `provider = "ollama"`.
    #[serde(default)]
    pub ollama: OllamaConfig,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            provider: Provider::Anthropic,
            model: "claude-sonnet-4-6".to_string(),
            max_turns: default_max_turns(),
            max_tokens_per_response: default_max_tokens_per_response(),
            temperature: None,
            tolerance: ToleranceConfig::default(),
            parallelism: default_parallelism(),
            anthropic: AnthropicConfig::default(),
            openai: OpenAiConfig::default(),
            openrouter: OpenRouterConfig::default(),
            ollama: OllamaConfig::default(),
        }
    }
}

/// Which upstream LLM provider to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    Anthropic,
    Openai,
    Openrouter,
    Ollama,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicConfig {
    /// Name of the env var to read the API key from.
    #[serde(default = "default_anthropic_key_env")]
    pub api_key_env: String,
    /// Override the API base URL (proxies, gateways, etc.).
    #[serde(default)]
    pub base_url: Option<String>,
}

impl Default for AnthropicConfig {
    fn default() -> Self {
        Self {
            api_key_env: default_anthropic_key_env(),
            base_url: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiConfig {
    #[serde(default = "default_openai_key_env")]
    pub api_key_env: String,
    #[serde(default)]
    pub base_url: Option<String>,
}

impl Default for OpenAiConfig {
    fn default() -> Self {
        Self {
            api_key_env: default_openai_key_env(),
            base_url: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRouterConfig {
    #[serde(default = "default_openrouter_key_env")]
    pub api_key_env: String,
    /// Optional `HTTP-Referer` header value (your project URL).
    #[serde(default)]
    pub app_url: Option<String>,
    /// Optional `X-Title` header value (your project name).
    #[serde(default)]
    pub app_title: Option<String>,
}

impl Default for OpenRouterConfig {
    fn default() -> Self {
        Self {
            api_key_env: default_openrouter_key_env(),
            app_url: None,
            app_title: None,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OllamaConfig {
    /// Base URL of the Ollama server (defaults to `http://localhost:11434/v1`).
    #[serde(default)]
    pub base_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToleranceConfig {
    #[serde(default = "default_multiplier")]
    pub must_by_multiplier: f64,
}

impl Default for ToleranceConfig {
    fn default() -> Self {
        Self {
            must_by_multiplier: default_multiplier(),
        }
    }
}

fn default_multiplier() -> f64 {
    1.0
}
fn default_parallelism() -> usize {
    1
}
fn default_max_turns() -> u32 {
    50
}
fn default_max_tokens_per_response() -> u32 {
    8192
}
fn default_anthropic_key_env() -> String {
    "ANTHROPIC_API_KEY".to_string()
}
fn default_provider() -> Provider {
    Provider::Anthropic
}
fn default_model() -> String {
    "claude-sonnet-4-6".to_string()
}
fn default_openai_key_env() -> String {
    "OPENAI_API_KEY".to_string()
}
fn default_openrouter_key_env() -> String {
    "OPENROUTER_API_KEY".to_string()
}
