-- ============================================================================
-- 0001_initial.surql
--
-- Initial SurrealDB schema for baby-phi v0.1 / milestone M1.
--
-- Landscape:
--   * 5 utility tables  — migrations meta, bootstrap credentials, secrets
--                         vault, audit event log, resources catalogue.
--   * 37 node tables    — one per NodeKind in `domain::model::nodes`.
--   * 66 edge relations — one per variant in `domain::model::edges::Edge`.
--
-- Conventions:
--   * Every node table uses `SCHEMAFULL` and declares at minimum its `id` +
--     `created_at` fields. Load-bearing M1 nodes (Agent, AgentProfile, Grant,
--     AuthRequest, Template, Consent, User, Organization, Channel,
--     InboxObject, OutboxObject, Memory, ToolAuthorityManifest) declare the
--     full M1 field shape. Later milestones can `ALTER FIELD` to flesh out
--     the remaining 24 scaffolded tables.
--   * Every edge is declared `TYPE RELATION FROM <src> TO <dst>` so Surreal
--     can enforce endpoint types. Where the ontology lists a single edge
--     name with multiple source/target type pairs (HOLDS_GRANT, CONNECTS_TO,
--     PROVIDES_TOOL, OWNED_BY), we declare one relation per concrete pair,
--     matching the 66-variant `Edge` enum in domain.
--   * Indexes: `id` (every table, unique by default via SurrealDB's
--     implicit record id), `owning_org` on agent / grant / auth_request
--     (query-critical for Step 2 of the Permission Check), and a compound
--     index on `audit_events (org_scope, timestamp)` for hash-chain walks.
--
-- Run inside a SurrealDB transaction managed by the migration runner.
-- ============================================================================

-- ----------------------------------------------------------------------------
-- Utility tables (4) — the fifth utility table, `_migrations`, is created
-- by the runner's `bootstrap_ledger_table` function before this script runs,
-- so we intentionally do NOT re-declare it here (DEFINE TABLE is
-- non-idempotent without IF NOT EXISTS, and the ledger row for this
-- migration will have already been absent on first entry).
-- ----------------------------------------------------------------------------

-- Bootstrap credentials. One-shot secrets generated at install time by the
-- `--bootstrap-init` command. The credential digest is stored as an
-- argon2id hash; the plaintext is never persisted. `consumed_at` flips from
-- NONE to a timestamp on successful claim.
DEFINE TABLE bootstrap_credentials SCHEMAFULL;
DEFINE FIELD digest       ON bootstrap_credentials TYPE string;
DEFINE FIELD created_at   ON bootstrap_credentials TYPE string;
DEFINE FIELD consumed_at  ON bootstrap_credentials TYPE option<string>;
DEFINE INDEX bootstrap_credentials_digest ON bootstrap_credentials FIELDS digest UNIQUE;

-- Secrets vault. Every row's `value_ciphertext` is AES-GCM-sealed by the
-- `store::crypto` layer using the master key from `BABY_PHI_MASTER_KEY`.
-- Plaintext never touches disk.
--
-- Ciphertext + nonce are stored as base64 (standard, no-padding) strings
-- rather than native `bytes` columns. Rationale: `.bind(Vec<u8>)` in the
-- SurrealDB Rust driver serializes as a JSON array of numbers, which does
-- not coerce into the `bytes` field type. Base64 strings are wire-safe and
-- human-debuggable.
-- Columns:
--   slug            — human-readable identifier (e.g. "anthropic-api-key"),
--                     stable across rotations; paired with the domain-layer
--                     SecretId UUID for referential integrity.
--   custodian_id    — AgentId (UUID string) of the agent currently
--                     authorized to rotate + reveal. Set at creation;
--                     mutable only via the custody-reassign flow (M2/P4).
--   sensitive       — opt-in masking for list / audit-diff views. The
--                     vault always masks plaintext regardless.
--   value_ciphertext_b64, nonce_b64 — AES-GCM-sealed material.
--   last_rotated_at — set on every successful rotate; null before the
--                     first rotation.
DEFINE TABLE secrets_vault SCHEMAFULL;
DEFINE FIELD slug               ON secrets_vault TYPE string;
DEFINE FIELD custodian_id       ON secrets_vault TYPE string;
DEFINE FIELD sensitive          ON secrets_vault TYPE bool DEFAULT false;
DEFINE FIELD value_ciphertext_b64 ON secrets_vault TYPE string;
DEFINE FIELD nonce_b64          ON secrets_vault TYPE string;
DEFINE FIELD created_at         ON secrets_vault TYPE string;
DEFINE FIELD last_rotated_at    ON secrets_vault TYPE option<string>;
DEFINE INDEX secrets_vault_slug ON secrets_vault FIELDS slug UNIQUE;

-- Audit event log. One row per emitted event. The `prev_event_hash_b64`
-- column forms the hash chain scoped per `org_scope` (see `domain::audit`).
-- The hash is stored base64-encoded (same rationale as secrets_vault).
DEFINE TABLE audit_events SCHEMAFULL;
DEFINE FIELD event_type                  ON audit_events TYPE string;
DEFINE FIELD actor_agent_id              ON audit_events TYPE option<string>;
DEFINE FIELD target_entity_id            ON audit_events TYPE option<string>;
DEFINE FIELD timestamp                   ON audit_events TYPE string;
DEFINE FIELD diff                        ON audit_events FLEXIBLE TYPE object;
DEFINE FIELD audit_class                 ON audit_events TYPE string
    ASSERT $value INSIDE ["silent", "logged", "alerted"];
DEFINE FIELD provenance_auth_request_id  ON audit_events TYPE option<string>;
DEFINE FIELD org_scope                   ON audit_events TYPE option<string>;
DEFINE FIELD prev_event_hash_b64         ON audit_events TYPE option<string>;
DEFINE INDEX audit_events_org_timestamp ON audit_events FIELDS org_scope, timestamp;

-- Resources catalogue. The platform-level and per-org enumeration of
-- resource instances. Permission Check Step 0 queries this; bootstrap seeds
-- the platform-level row at install time.
DEFINE TABLE resources_catalogue SCHEMAFULL;
DEFINE FIELD owning_org   ON resources_catalogue TYPE option<string>;
DEFINE FIELD resource_uri ON resources_catalogue TYPE string;
DEFINE FIELD kind         ON resources_catalogue TYPE string;
DEFINE FIELD added_at     ON resources_catalogue TYPE string;
DEFINE INDEX resources_catalogue_owning_uri ON resources_catalogue FIELDS owning_org, resource_uri UNIQUE;

-- ----------------------------------------------------------------------------
-- Node tables (37) — grouped to match `docs/specs/v0/concepts/ontology.md`
-- ----------------------------------------------------------------------------

-- Core Identity (4)

DEFINE TABLE agent SCHEMAFULL;
DEFINE FIELD kind          ON agent TYPE string
    ASSERT $value INSIDE ["human", "llm"];
DEFINE FIELD display_name  ON agent TYPE string;
DEFINE FIELD owning_org    ON agent TYPE option<string>;
DEFINE FIELD created_at    ON agent TYPE string;
DEFINE INDEX agent_owning_org ON agent FIELDS owning_org;

-- AgentProfile wraps phi-core's AgentProfile (per concepts/phi-core-mapping.md).
-- `blueprint` is the full phi-core struct serialized as a flexible object
-- so SurrealDB doesn't strip fields as phi-core evolves. baby-phi-only
-- governance fields (`agent_id`, `parallelize`) sit alongside the wrap.
DEFINE TABLE agent_profile SCHEMAFULL;
DEFINE FIELD agent_id      ON agent_profile TYPE string;
DEFINE FIELD parallelize   ON agent_profile TYPE int DEFAULT 1
    ASSERT $value >= 1;
DEFINE FIELD blueprint     ON agent_profile FLEXIBLE TYPE object;
DEFINE FIELD created_at    ON agent_profile TYPE string;
DEFINE INDEX agent_profile_agent_id ON agent_profile FIELDS agent_id UNIQUE;

DEFINE TABLE user SCHEMAFULL;
DEFINE FIELD display_name  ON user TYPE string;
DEFINE FIELD created_at    ON user TYPE string;

DEFINE TABLE identity SCHEMAFULL;
DEFINE FIELD created_at    ON identity TYPE string;

-- Execution History (5)

DEFINE TABLE session SCHEMAFULL;
DEFINE FIELD created_at ON session TYPE string;

DEFINE TABLE loop SCHEMAFULL;
DEFINE FIELD created_at ON loop TYPE string;

DEFINE TABLE turn SCHEMAFULL;
DEFINE FIELD created_at ON turn TYPE string;

DEFINE TABLE message SCHEMAFULL;
DEFINE FIELD created_at ON message TYPE string;

DEFINE TABLE event SCHEMAFULL;
DEFINE FIELD created_at ON event TYPE string;

-- Capability (8)

DEFINE TABLE model_config SCHEMAFULL;
DEFINE FIELD created_at ON model_config TYPE string;

DEFINE TABLE tool_definition SCHEMAFULL;
DEFINE FIELD created_at ON tool_definition TYPE string;

DEFINE TABLE tool_implementation SCHEMAFULL;
DEFINE FIELD created_at ON tool_implementation TYPE string;

DEFINE TABLE skill SCHEMAFULL;
DEFINE FIELD created_at ON skill TYPE string;

DEFINE TABLE mcp_server SCHEMAFULL;
DEFINE FIELD created_at ON mcp_server TYPE string;

DEFINE TABLE open_api_spec SCHEMAFULL;
DEFINE FIELD created_at ON open_api_spec TYPE string;

DEFINE TABLE system_prompt SCHEMAFULL;
DEFINE FIELD created_at ON system_prompt TYPE string;

DEFINE TABLE evaluation_strategy SCHEMAFULL;
DEFINE FIELD created_at ON evaluation_strategy TYPE string;

-- Governance (9)

DEFINE TABLE execution_limits SCHEMAFULL;
DEFINE FIELD created_at ON execution_limits TYPE string;

DEFINE TABLE grant SCHEMAFULL;
DEFINE FIELD holder_kind    ON grant TYPE string;
DEFINE FIELD holder_id      ON grant TYPE string;
DEFINE FIELD action         ON grant TYPE array<string>;
DEFINE FIELD resource_uri   ON grant TYPE string;
-- Explicit fundamentals list (M2/P4.5 — G19 / D17). Empty array preserves
-- legacy URI-derivation semantics in the engine's `resolve_grant` Case C.
-- Populated by handlers issuing instance-URI grants (e.g. `secret:<slug>`)
-- so the engine can resolve fundamentals without a URI-scheme parser.
DEFINE FIELD fundamentals   ON grant TYPE array<string> DEFAULT [];
DEFINE FIELD descends_from  ON grant TYPE option<string>;
DEFINE FIELD delegable      ON grant TYPE bool DEFAULT false;
DEFINE FIELD issued_at      ON grant TYPE string;
DEFINE FIELD revoked_at     ON grant TYPE option<string>;
DEFINE INDEX grant_holder ON grant FIELDS holder_kind, holder_id;
DEFINE INDEX grant_resource ON grant FIELDS resource_uri;

DEFINE TABLE auth_request SCHEMAFULL;
DEFINE FIELD requestor_kind             ON auth_request TYPE string;
DEFINE FIELD requestor_id               ON auth_request TYPE string;
DEFINE FIELD kinds                      ON auth_request TYPE array<string>;
DEFINE FIELD scope                      ON auth_request TYPE array<string>;
DEFINE FIELD state                      ON auth_request TYPE string
    ASSERT $value INSIDE [
        "draft", "pending", "in_progress", "approved", "denied",
        "partial", "expired", "revoked", "cancelled"
    ];
DEFINE FIELD valid_until                ON auth_request TYPE option<string>;
DEFINE FIELD submitted_at               ON auth_request TYPE string;
DEFINE FIELD resource_slots             ON auth_request FLEXIBLE TYPE array<object>;
DEFINE FIELD justification              ON auth_request TYPE option<string>;
DEFINE FIELD audit_class                ON auth_request TYPE string
    ASSERT $value INSIDE ["silent", "logged", "alerted"];
DEFINE FIELD terminal_state_entered_at  ON auth_request TYPE option<string>;
DEFINE FIELD archived                   ON auth_request TYPE bool DEFAULT false;
DEFINE FIELD active_window_days         ON auth_request TYPE int DEFAULT 90;
DEFINE FIELD provenance_template        ON auth_request TYPE option<string>;
DEFINE INDEX auth_request_state ON auth_request FIELDS state;
DEFINE INDEX auth_request_archived ON auth_request FIELDS archived;

DEFINE TABLE template SCHEMAFULL;
DEFINE FIELD name       ON template TYPE string;
DEFINE FIELD created_at ON template TYPE string;
DEFINE INDEX template_name ON template FIELDS name UNIQUE;

DEFINE TABLE tool_authority_manifest SCHEMAFULL;
DEFINE FIELD tool_name     ON tool_authority_manifest TYPE string;
DEFINE FIELD resource      ON tool_authority_manifest TYPE array<string>;
DEFINE FIELD transitive    ON tool_authority_manifest TYPE array<string>;
DEFINE FIELD actions       ON tool_authority_manifest TYPE array<string>;
DEFINE FIELD constraints   ON tool_authority_manifest TYPE array<string>;
DEFINE FIELD kinds         ON tool_authority_manifest TYPE array<string>;
DEFINE FIELD target_kinds  ON tool_authority_manifest TYPE array<string>;
DEFINE FIELD delegable     ON tool_authority_manifest TYPE bool DEFAULT false;
DEFINE FIELD approval      ON tool_authority_manifest TYPE string;
DEFINE INDEX tool_authority_manifest_tool ON tool_authority_manifest FIELDS tool_name UNIQUE;

DEFINE TABLE consent SCHEMAFULL;
DEFINE FIELD subordinate   ON consent TYPE string;
DEFINE FIELD scoped_to     ON consent TYPE string;
DEFINE FIELD granted_at    ON consent TYPE string;
DEFINE FIELD revoked_at    ON consent TYPE option<string>;
DEFINE INDEX consent_subordinate_org ON consent FIELDS subordinate, scoped_to;

DEFINE TABLE compaction_policy SCHEMAFULL;
DEFINE FIELD created_at ON compaction_policy TYPE string;

DEFINE TABLE retry_policy SCHEMAFULL;
DEFINE FIELD created_at ON retry_policy TYPE string;

DEFINE TABLE cache_policy SCHEMAFULL;
DEFINE FIELD created_at ON cache_policy TYPE string;

-- Social Structure (11)

DEFINE TABLE project SCHEMAFULL;
DEFINE FIELD created_at ON project TYPE string;

DEFINE TABLE task SCHEMAFULL;
DEFINE FIELD created_at ON task TYPE string;

DEFINE TABLE bid SCHEMAFULL;
DEFINE FIELD created_at ON bid TYPE string;

DEFINE TABLE rating SCHEMAFULL;
DEFINE FIELD created_at ON rating TYPE string;

DEFINE TABLE organization SCHEMAFULL;
DEFINE FIELD display_name ON organization TYPE string;
DEFINE FIELD created_at   ON organization TYPE string;

DEFINE TABLE channel SCHEMAFULL;
DEFINE FIELD agent_id   ON channel TYPE string;
DEFINE FIELD kind       ON channel TYPE string
    ASSERT $value INSIDE ["slack", "email", "web"];
DEFINE FIELD handle     ON channel TYPE string;
DEFINE FIELD created_at ON channel TYPE string;
DEFINE INDEX channel_agent ON channel FIELDS agent_id;

DEFINE TABLE memory SCHEMAFULL;
DEFINE FIELD owning_agent ON memory TYPE string;
DEFINE FIELD tags         ON memory TYPE array<string>;
DEFINE FIELD created_at   ON memory TYPE string;
DEFINE INDEX memory_owning_agent ON memory FIELDS owning_agent;

DEFINE TABLE agent_config SCHEMAFULL;
DEFINE FIELD created_at ON agent_config TYPE string;

DEFINE TABLE prompt_block SCHEMAFULL;
DEFINE FIELD created_at ON prompt_block TYPE string;

DEFINE TABLE inbox_object SCHEMAFULL;
DEFINE FIELD agent_id   ON inbox_object TYPE string;
DEFINE FIELD created_at ON inbox_object TYPE string;
DEFINE INDEX inbox_object_agent ON inbox_object FIELDS agent_id UNIQUE;

DEFINE TABLE outbox_object SCHEMAFULL;
DEFINE FIELD agent_id   ON outbox_object TYPE string;
DEFINE FIELD created_at ON outbox_object TYPE string;
DEFINE INDEX outbox_object_agent ON outbox_object FIELDS agent_id UNIQUE;

-- ----------------------------------------------------------------------------
-- Edge relations (66) — one per `domain::model::edges::Edge` variant
-- ----------------------------------------------------------------------------

-- Agent-Centric (22)
DEFINE TABLE has_profile TYPE RELATION FROM agent TO agent_profile;
DEFINE TABLE uses_model TYPE RELATION FROM agent TO model_config;
DEFINE TABLE has_tool TYPE RELATION FROM agent TO tool_definition;
DEFINE TABLE has_skill TYPE RELATION FROM agent TO skill;
DEFINE TABLE agent_holds_grant TYPE RELATION FROM agent TO grant;
DEFINE TABLE governed_by TYPE RELATION FROM agent TO execution_limits;
DEFINE TABLE uses_compaction TYPE RELATION FROM agent TO compaction_policy;
DEFINE TABLE uses_retry TYPE RELATION FROM agent TO retry_policy;
DEFINE TABLE uses_cache TYPE RELATION FROM agent TO cache_policy;
DEFINE TABLE uses_evaluation TYPE RELATION FROM agent TO evaluation_strategy;
DEFINE TABLE has_system_prompt TYPE RELATION FROM agent TO system_prompt;
DEFINE TABLE agent_connects_to_mcp_server TYPE RELATION FROM agent TO mcp_server;
DEFINE TABLE agent_connects_to_open_api_spec TYPE RELATION FROM agent TO open_api_spec;
DEFINE TABLE runs_session TYPE RELATION FROM agent TO session;
DEFINE TABLE delegates_to TYPE RELATION FROM agent TO agent;
DEFINE TABLE agent_owned_by_user TYPE RELATION FROM agent TO user;
DEFINE TABLE has_memory TYPE RELATION FROM agent TO memory;
DEFINE TABLE has_inbox TYPE RELATION FROM agent TO inbox_object;
DEFINE TABLE has_outbox TYPE RELATION FROM agent TO outbox_object;
DEFINE TABLE has_channel TYPE RELATION FROM agent TO channel;
DEFINE TABLE loaded_from TYPE RELATION FROM agent TO agent_config;
DEFINE TABLE member_of TYPE RELATION FROM agent TO organization;

-- Execution Chain (7)
DEFINE TABLE contains_loop TYPE RELATION FROM session TO loop;
DEFINE TABLE continues_from TYPE RELATION FROM loop TO loop;
DEFINE TABLE contains_turn TYPE RELATION FROM loop TO turn;
DEFINE TABLE configured_with TYPE RELATION FROM loop TO agent_config;
DEFINE TABLE emits TYPE RELATION FROM loop TO event;
DEFINE TABLE produces TYPE RELATION FROM turn TO message;
DEFINE TABLE executes_tool TYPE RELATION FROM turn TO tool_definition;

-- Cross-Agent (3)
DEFINE TABLE spawned_from TYPE RELATION FROM session TO session;
DEFINE TABLE spawned_child TYPE RELATION FROM loop TO loop;
DEFINE TABLE parallel_with TYPE RELATION FROM loop TO loop;

-- Capability Wiring (5)
DEFINE TABLE implemented_by TYPE RELATION FROM tool_definition TO tool_implementation;
DEFINE TABLE has_manifest TYPE RELATION FROM tool_definition TO tool_authority_manifest;
DEFINE TABLE mcp_provides_tool TYPE RELATION FROM mcp_server TO tool_definition;
DEFINE TABLE open_api_provides_tool TYPE RELATION FROM open_api_spec TO tool_definition;
DEFINE TABLE contains_block TYPE RELATION FROM system_prompt TO prompt_block;

-- Social Structure (14)
DEFINE TABLE has_board TYPE RELATION FROM organization TO agent;
DEFINE TABLE has_ceo TYPE RELATION FROM organization TO agent;
DEFINE TABLE has_project TYPE RELATION FROM organization TO project;
DEFINE TABLE has_member TYPE RELATION FROM organization TO agent;
DEFINE TABLE has_suborganization TYPE RELATION FROM organization TO organization;
DEFINE TABLE has_sponsor TYPE RELATION FROM project TO agent;
DEFINE TABLE has_agent TYPE RELATION FROM project TO agent;
DEFINE TABLE has_task TYPE RELATION FROM project TO task;
DEFINE TABLE belongs_to TYPE RELATION FROM project TO organization;
DEFINE TABLE assigned_to TYPE RELATION FROM task TO agent;
DEFINE TABLE has_bid TYPE RELATION FROM task TO bid;
DEFINE TABLE bid_submitted_by TYPE RELATION FROM bid TO agent;
DEFINE TABLE rates TYPE RELATION FROM rating TO agent;
DEFINE TABLE given_by TYPE RELATION FROM rating TO agent;

-- Governance — Ownership (3)
-- Resource and Principal are type unions, not single tables. SurrealDB
-- relations require concrete endpoint tables, so we declare these three
-- edges with broad acceptance via the `ANY` placeholder. At query time the
-- domain layer enforces the Resource / Principal constraint in Rust.
DEFINE TABLE owned_by TYPE RELATION;
DEFINE TABLE created TYPE RELATION;
DEFINE TABLE allocated_to TYPE RELATION;

-- Governance — Grant + Auth Request (10)
DEFINE TABLE issued_grant TYPE RELATION FROM user TO grant;
DEFINE TABLE descends_from_request TYPE RELATION FROM grant TO auth_request;
DEFINE TABLE applies_to TYPE RELATION FROM grant TO agent;
DEFINE TABLE project_holds_grant TYPE RELATION FROM project TO grant;
DEFINE TABLE org_holds_grant TYPE RELATION FROM organization TO grant;
DEFINE TABLE requests_on TYPE RELATION;
DEFINE TABLE approved_by TYPE RELATION;
DEFINE TABLE auth_request_submitted_by TYPE RELATION;
DEFINE TABLE emitted_by TYPE RELATION FROM auth_request TO template;
-- Governance-section HOLDS_GRANT(Agent→Grant) — deliberate dedupe partner
-- of agent_holds_grant above; kept as its own relation to mirror the
-- concept doc's two listings (Agent-Centric + Governance Wiring).
DEFINE TABLE agent_holds_grant_dedupe TYPE RELATION FROM agent TO grant;

-- Consent (2)
DEFINE TABLE has_consent TYPE RELATION FROM agent TO consent;
DEFINE TABLE scoped_to TYPE RELATION FROM consent TO organization;
