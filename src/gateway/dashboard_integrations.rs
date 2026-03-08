use crate::config::{default_model_fallback_for_provider, Config};
use crate::integrations::{IntegrationCategory, IntegrationEntry, IntegrationStatus};
use serde::{Deserialize, Serialize};

pub(super) const MASKED_SECRET: &str = "***MASKED***";
const REVISION: &str = "v1";

const OPENROUTER_MODEL_OPTIONS: &[&str] = &[
    "anthropic/claude-sonnet-4-6",
    "openai/gpt-5.2",
    "google/gemini-3.1-pro",
    "deepseek/deepseek-reasoner",
    "x-ai/grok-4",
];
const ANTHROPIC_MODEL_OPTIONS: &[&str] = &["claude-sonnet-4-6", "claude-opus-4-6"];
const OPENAI_MODEL_OPTIONS: &[&str] = &["gpt-5.2", "gpt-5.2-codex", "gpt-4o"];
const OLLAMA_MODEL_OPTIONS: &[&str] = &["llama3.2"];

#[derive(Debug, Clone, Copy)]
struct DashboardProviderDescriptor {
    provider_key: &'static str,
    integration_name: &'static str,
    integration_id: &'static str,
    provider_aliases: &'static [&'static str],
    model_options: &'static [&'static str],
    supports_api_key: bool,
}

impl DashboardProviderDescriptor {
    fn matches_provider(self, provider: &str) -> bool {
        self.provider_key == provider
            || self.provider_aliases.iter().any(|alias| *alias == provider)
    }

    fn field_options(self) -> Vec<String> {
        if self.model_options.is_empty() {
            vec![default_model_fallback_for_provider(Some(self.provider_key)).to_string()]
        } else {
            self.model_options
                .iter()
                .map(|value| (*value).to_string())
                .collect()
        }
    }
}

const DASHBOARD_PROVIDER_DESCRIPTORS: &[DashboardProviderDescriptor] = &[
    DashboardProviderDescriptor {
        provider_key: "openrouter",
        integration_name: "OpenRouter",
        integration_id: "openrouter",
        provider_aliases: &[],
        model_options: OPENROUTER_MODEL_OPTIONS,
        supports_api_key: true,
    },
    DashboardProviderDescriptor {
        provider_key: "anthropic",
        integration_name: "Anthropic",
        integration_id: "anthropic",
        provider_aliases: &[],
        model_options: ANTHROPIC_MODEL_OPTIONS,
        supports_api_key: true,
    },
    DashboardProviderDescriptor {
        provider_key: "openai",
        integration_name: "OpenAI",
        integration_id: "openai",
        provider_aliases: &[],
        model_options: OPENAI_MODEL_OPTIONS,
        supports_api_key: true,
    },
    DashboardProviderDescriptor {
        provider_key: crate::providers::inception::CANONICAL_NAME,
        integration_name: crate::providers::inception::DASHBOARD_INTEGRATION_NAME,
        integration_id: crate::providers::inception::CANONICAL_NAME,
        provider_aliases: crate::providers::inception::ALIASES,
        model_options: crate::providers::inception::DASHBOARD_MODEL_OPTIONS,
        supports_api_key: true,
    },
    DashboardProviderDescriptor {
        provider_key: "google",
        integration_name: "Google",
        integration_id: "google",
        provider_aliases: &["gemini", "google-gemini", "vertex"],
        model_options: &[],
        supports_api_key: true,
    },
    DashboardProviderDescriptor {
        provider_key: "deepseek",
        integration_name: "DeepSeek",
        integration_id: "deepseek",
        provider_aliases: &[],
        model_options: &[],
        supports_api_key: true,
    },
    DashboardProviderDescriptor {
        provider_key: "xai",
        integration_name: "xAI",
        integration_id: "xai",
        provider_aliases: &["x-ai", "grok"],
        model_options: &[],
        supports_api_key: true,
    },
    DashboardProviderDescriptor {
        provider_key: "mistral",
        integration_name: "Mistral",
        integration_id: "mistral",
        provider_aliases: &[],
        model_options: &[],
        supports_api_key: true,
    },
    DashboardProviderDescriptor {
        provider_key: "perplexity",
        integration_name: "Perplexity",
        integration_id: "perplexity",
        provider_aliases: &[],
        model_options: &[],
        supports_api_key: true,
    },
    DashboardProviderDescriptor {
        provider_key: "vercel",
        integration_name: "Vercel AI",
        integration_id: "vercel-ai",
        provider_aliases: &["vercel-ai"],
        model_options: &[],
        supports_api_key: true,
    },
    DashboardProviderDescriptor {
        provider_key: "bedrock",
        integration_name: "Amazon Bedrock",
        integration_id: "amazon-bedrock",
        provider_aliases: &["aws-bedrock"],
        model_options: &[],
        supports_api_key: true,
    },
    DashboardProviderDescriptor {
        provider_key: "groq",
        integration_name: "Groq",
        integration_id: "groq",
        provider_aliases: &[],
        model_options: &[],
        supports_api_key: true,
    },
    DashboardProviderDescriptor {
        provider_key: "together",
        integration_name: "Together AI",
        integration_id: "together-ai",
        provider_aliases: &["together-ai"],
        model_options: &[],
        supports_api_key: true,
    },
    DashboardProviderDescriptor {
        provider_key: "cohere",
        integration_name: "Cohere",
        integration_id: "cohere",
        provider_aliases: &[],
        model_options: &[],
        supports_api_key: true,
    },
    DashboardProviderDescriptor {
        provider_key: "fireworks",
        integration_name: "Fireworks AI",
        integration_id: "fireworks-ai",
        provider_aliases: &["fireworks-ai"],
        model_options: &[],
        supports_api_key: true,
    },
    DashboardProviderDescriptor {
        provider_key: "venice",
        integration_name: "Venice",
        integration_id: "venice",
        provider_aliases: &[],
        model_options: &[],
        supports_api_key: true,
    },
    DashboardProviderDescriptor {
        provider_key: "moonshot",
        integration_name: "Moonshot",
        integration_id: "moonshot",
        provider_aliases: &["moonshot-cn", "moonshot-intl", "kimi", "kimi-cn"],
        model_options: &[],
        supports_api_key: true,
    },
    DashboardProviderDescriptor {
        provider_key: "stepfun",
        integration_name: "StepFun",
        integration_id: "stepfun",
        provider_aliases: &["step", "step-ai", "step_ai"],
        model_options: &[],
        supports_api_key: true,
    },
    DashboardProviderDescriptor {
        provider_key: "synthetic",
        integration_name: "Synthetic",
        integration_id: "synthetic",
        provider_aliases: &[],
        model_options: &[],
        supports_api_key: true,
    },
    DashboardProviderDescriptor {
        provider_key: "opencode",
        integration_name: "OpenCode Zen",
        integration_id: "opencode-zen",
        provider_aliases: &["opencode-zen"],
        model_options: &[],
        supports_api_key: true,
    },
    DashboardProviderDescriptor {
        provider_key: "zai",
        integration_name: "Z.AI",
        integration_id: "z-ai",
        provider_aliases: &["z.ai", "zai-cn", "z.ai-cn", "zai-intl", "z.ai-global"],
        model_options: &[],
        supports_api_key: true,
    },
    DashboardProviderDescriptor {
        provider_key: "glm",
        integration_name: "GLM",
        integration_id: "glm",
        provider_aliases: &[
            "glm-cn",
            "glm-intl",
            "zhipu",
            "zhipu-cn",
            "zhipu-global",
            "bigmodel",
        ],
        model_options: &[],
        supports_api_key: true,
    },
    DashboardProviderDescriptor {
        provider_key: "minimax",
        integration_name: "MiniMax",
        integration_id: "minimax",
        provider_aliases: &[
            "minimax-cn",
            "minimax-intl",
            "minimax-io",
            "minimax-global",
            "minimax-oauth",
            "minimax-oauth-cn",
            "minimax-portal",
            "minimax-portal-cn",
            "minimax-oauth-global",
            "minimax-portal-global",
        ],
        model_options: &[],
        supports_api_key: true,
    },
    DashboardProviderDescriptor {
        provider_key: "qwen",
        integration_name: "Qwen",
        integration_id: "qwen",
        provider_aliases: &["qwen-cn", "qwen-intl", "dashscope"],
        model_options: &[],
        supports_api_key: true,
    },
    DashboardProviderDescriptor {
        provider_key: "qianfan",
        integration_name: "Qianfan",
        integration_id: "qianfan",
        provider_aliases: &["baidu"],
        model_options: &[],
        supports_api_key: true,
    },
    DashboardProviderDescriptor {
        provider_key: "ark",
        integration_name: "Volcengine ARK",
        integration_id: "volcengine-ark",
        provider_aliases: &["doubao", "volcengine", "doubao-cn"],
        model_options: &[],
        supports_api_key: true,
    },
    DashboardProviderDescriptor {
        provider_key: "siliconflow",
        integration_name: "SiliconFlow",
        integration_id: "siliconflow",
        provider_aliases: &["silicon-cloud", "siliconcloud"],
        model_options: &[],
        supports_api_key: true,
    },
    DashboardProviderDescriptor {
        provider_key: "ollama",
        integration_name: "Ollama",
        integration_id: "ollama",
        provider_aliases: &[],
        model_options: OLLAMA_MODEL_OPTIONS,
        supports_api_key: false,
    },
];

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct IntegrationCredentialsUpdateRequest {
    #[serde(default)]
    pub fields: IntegrationCredentialFields,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct IntegrationCredentialFields {
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub default_model: Option<String>,
}

#[derive(Debug, Serialize)]
pub(super) struct IntegrationsResponse {
    integrations: Vec<IntegrationSummary>,
}

#[derive(Debug, Serialize)]
struct IntegrationSummary {
    name: String,
    description: String,
    category: IntegrationCategory,
    status: IntegrationStatus,
}

#[derive(Debug, Serialize)]
pub(super) struct IntegrationSettingsResponse {
    revision: &'static str,
    active_default_provider_integration_id: Option<String>,
    integrations: Vec<IntegrationSettingsEntry>,
}

#[derive(Debug, Serialize)]
struct IntegrationSettingsEntry {
    id: String,
    name: String,
    description: String,
    category: IntegrationCategory,
    status: IntegrationStatus,
    configured: bool,
    activates_default_provider: bool,
    fields: Vec<IntegrationField>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum IntegrationFieldInputType {
    Secret,
    Select,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct IntegrationField {
    key: &'static str,
    label: &'static str,
    required: bool,
    has_value: bool,
    input_type: IntegrationFieldInputType,
    options: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    masked_value: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    current_value: Option<String>,
}

impl IntegrationField {
    fn secret(has_value: bool) -> Self {
        Self {
            key: "api_key",
            label: "API Key",
            required: true,
            has_value,
            input_type: IntegrationFieldInputType::Secret,
            options: Vec::new(),
            masked_value: has_value.then_some(MASKED_SECRET),
            current_value: None,
        }
    }

    fn default_model(current_value: Option<&str>, options: Vec<String>) -> Self {
        Self {
            key: "default_model",
            label: "Default Model",
            required: false,
            has_value: current_value.is_some_and(|value| !value.is_empty()),
            input_type: IntegrationFieldInputType::Select,
            options,
            masked_value: None,
            current_value: current_value.map(ToOwned::to_owned),
        }
    }
}

pub(super) fn build_integrations_response(
    config: &Config,
    entries: &[IntegrationEntry],
) -> IntegrationsResponse {
    IntegrationsResponse {
        integrations: entries
            .iter()
            .map(|entry| IntegrationSummary {
                name: entry.name.to_string(),
                description: entry.description.to_string(),
                category: entry.category,
                status: (entry.status_fn)(config),
            })
            .collect(),
    }
}

pub(super) fn build_integration_settings_response(
    config: &Config,
    entries: &[IntegrationEntry],
) -> IntegrationSettingsResponse {
    let active_default_provider_integration_id = config
        .default_provider
        .as_deref()
        .and_then(integration_id_from_provider);

    IntegrationSettingsResponse {
        revision: REVISION,
        active_default_provider_integration_id,
        integrations: entries
            .iter()
            .map(|entry| {
                let status = (entry.status_fn)(config);
                let (configured, fields) = integration_settings_fields(config, entry.name);
                IntegrationSettingsEntry {
                    id: integration_id_for_entry(entry.name),
                    name: entry.name.to_string(),
                    description: entry.description.to_string(),
                    category: entry.category,
                    status,
                    configured,
                    activates_default_provider: is_ai_provider(entry.name),
                    fields,
                }
            })
            .collect(),
    }
}

pub(super) fn apply_credentials_update(
    config: &mut Config,
    integration_id: &str,
    body: &IntegrationCredentialsUpdateRequest,
) -> Result<(), String> {
    let Some(provider_key) = provider_key_from_integration_id(integration_id) else {
        return Err(format!(
            "Integration '{}' does not support credential updates via this endpoint",
            integration_id
        ));
    };
    let descriptor = provider_descriptor_by_provider(provider_key)
        .expect("dashboard integration id should always resolve to a known provider descriptor");

    if descriptor.supports_api_key {
        if let Some(api_key) = body
            .fields
            .api_key
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty() && *value != MASKED_SECRET)
        {
            config.api_key = Some(api_key.to_string());
        }
    }

    if let Some(default_model) = body
        .fields
        .default_model
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        config.default_model = Some(default_model.to_string());
    }

    config.default_provider = Some(descriptor.provider_key.to_string());
    Ok(())
}

pub(super) fn provider_key_from_integration_id(id: &str) -> Option<&'static str> {
    provider_descriptor_by_integration_id(id).map(|descriptor| descriptor.provider_key)
}

pub(super) fn integration_id_from_provider(provider: &str) -> Option<String> {
    provider_descriptor_by_provider(provider)
        .map(|descriptor| descriptor.integration_id.to_string())
}

fn is_ai_provider(name: &str) -> bool {
    provider_descriptor_by_integration_name(name).is_some()
}

fn integration_id_for_entry(name: &str) -> String {
    provider_descriptor_by_integration_name(name)
        .map(|descriptor| descriptor.integration_id.to_string())
        .unwrap_or_else(|| integration_name_to_id(name))
}

fn integration_name_to_id(name: &str) -> String {
    name.to_lowercase()
        .replace(' ', "-")
        .replace(['/', '.'], "-")
}

fn integration_settings_fields(config: &Config, name: &str) -> (bool, Vec<IntegrationField>) {
    let Some(descriptor) = provider_descriptor_by_integration_name(name) else {
        return (false, Vec::new());
    };

    let mut fields = Vec::new();
    let has_key = descriptor.supports_api_key && config.api_key.is_some();
    if descriptor.supports_api_key {
        fields.push(IntegrationField::secret(has_key));
    }

    fields.push(IntegrationField::default_model(
        config.default_model.as_deref(),
        descriptor.field_options(),
    ));

    let configured = if descriptor.supports_api_key {
        has_key
    } else {
        config
            .default_provider
            .as_deref()
            .is_some_and(|provider| descriptor.matches_provider(provider))
    };

    (configured, fields)
}

fn provider_descriptor_by_integration_name(
    name: &str,
) -> Option<&'static DashboardProviderDescriptor> {
    DASHBOARD_PROVIDER_DESCRIPTORS
        .iter()
        .find(|descriptor| descriptor.integration_name == name)
}

fn provider_descriptor_by_integration_id(id: &str) -> Option<&'static DashboardProviderDescriptor> {
    DASHBOARD_PROVIDER_DESCRIPTORS
        .iter()
        .find(|descriptor| descriptor.integration_id == id)
}

fn provider_descriptor_by_provider(provider: &str) -> Option<&'static DashboardProviderDescriptor> {
    DASHBOARD_PROVIDER_DESCRIPTORS
        .iter()
        .find(|descriptor| descriptor.matches_provider(provider))
}
