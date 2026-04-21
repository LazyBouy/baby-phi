//! `list_provider_kinds` — the set of `ApiProtocol` variants
//! baby-phi currently supports.
//!
//! **phi-core is the single source of truth.** The response enumerates
//! [`phi_core::provider::registry::ProviderRegistry::default().protocols()`]
//! — baby-phi never hard-codes a list of its own. When phi-core adds a
//! new provider (e.g. a future `OpenAiBatchApi`), this endpoint picks
//! it up at the next `cargo update` without any baby-phi code change.
//!
//! Return shape: a `Vec<ApiProtocol>` serialised via phi-core's own
//! serde impl (`#[serde(rename_all = "snake_case")]`), so wire values
//! like `"anthropic_messages"` / `"openai_completions"` match what the
//! POST `/platform/model-providers` endpoint accepts verbatim.

use phi_core::provider::model::ApiProtocol;
use phi_core::provider::registry::ProviderRegistry;

/// List every `ApiProtocol` variant registered in phi-core's default
/// registry — i.e. the supported provider kinds as of the currently
/// pinned phi-core version.
pub fn list_provider_kinds() -> Vec<ApiProtocol> {
    ProviderRegistry::default().protocols()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_registry_contains_anthropic() {
        // Sanity check — if the default registry ever drops Anthropic,
        // something upstream changed and baby-phi should react
        // intentionally rather than silently.
        let kinds = list_provider_kinds();
        assert!(
            kinds.contains(&ApiProtocol::AnthropicMessages),
            "default registry must include AnthropicMessages; got {:?}",
            kinds
        );
    }

    #[test]
    fn kinds_serialise_as_snake_case() {
        let kinds = list_provider_kinds();
        // Pick a variant we know the wire shape for — phi-core's
        // `#[serde(rename_all = "snake_case")]` + its `Display` impl
        // both produce the same string.
        let json = serde_json::to_string(&ApiProtocol::AnthropicMessages).unwrap();
        assert_eq!(json, "\"anthropic_messages\"");
        assert!(!kinds.is_empty(), "default registry must expose ≥1 kind");
    }
}
