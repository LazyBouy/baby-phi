<!-- Last verified: 2026-04-21 by Claude Code -->

# Architecture — platform catalogue seeding

**Status: [EXISTS]**

The `resources_catalogue` table (M1) is the Permission-Check engine's
**Step 0 precondition**: every target URI a caller names must be
registered there before the engine will let a call proceed. M2 admin
writes seed catalogue entries as part of the transaction that creates
a new resource, so follow-up reads + reveals pass Step 0 cleanly.

## How Step 0 consumes the catalogue

[`domain::permissions::engine::step_0_catalogue`][1] calls
`ctx.catalogue.contains(owning_org, &ctx.call.target_uri)`:

- **Empty `target_uri`** (class-level invocations like `allocate`
  against a class rather than an instance) — Step 0 skips.
- **Non-empty URI** — must be present in the catalogue for the
  owning-org scope, else `Decision::Denied { FailedStep::Catalogue }`
  with a `CatalogueMiss { resource_uri }` reason.

The catalogue is keyed by `(owning_org, resource_uri) → kind` tuples:

- Platform-level resources (`system:root`, `secret:<slug>`,
  `provider:<id>`, `mcp:<id>`, `platform-defaults:root`) live under
  `owning_org = None` — the "root" scope.
- Org-scoped resources (filesystem paths, memory stores) key under
  `owning_org = Some(org_id)`.

## Repository surface

Two methods on the `Repository` trait:

1. [`seed_catalogue_entry(owning_org, resource_uri, kind)`][2] — the
   raw primitive. Takes an arbitrary kind string.
2. [`seed_catalogue_entry_for_composite(owning_org, resource_uri,
   composite)`][3] — thin convenience wrapper over #1 that pulls the
   kind name off `Composite::kind_name()`. Used by M2 admin writes
   that persist a [`domain::model::Composite`] variant.

Both are idempotent: a re-seed of the same `(owning_org, uri)` pair
updates the kind in place (SurrealDB `CREATE … CONTENT` on a
composite key). Callers don't need to check for existence first.

## M2 seeding pattern

Every M2 admin write that creates a new named resource follows the
same three-step pattern:

1. Build a Template E Auth Request (see [`template-e-auto-approve.md`](template-e-auto-approve.md)).
2. Persist the composite-instance row.
3. Seed the catalogue entry for the URI that row is known by.

**Page 04 (credentials vault) — the working reference:**

```rust
// modules/crates/server/src/platform/secrets/add.rs
let uri = secret_uri(input.slug);                  // "secret:<slug>"
repo.create_auth_request(&ar).await?;              // Step 1
repo.put_secret(&credential, &sealed_blob).await?; // Step 2
repo.seed_catalogue_entry(None, &uri, "secret_credential").await?; // Step 3
repo.create_grant(&grant).await?;                  // instance-URI grant
```

**Note on the kind string.** `secret_credential` is an
ontology-tagged **fundamental bundle** rather than a
`domain::model::Composite` enum variant (M2 plan §1.5 G1); page 04
uses the raw `seed_catalogue_entry` primitive with the explicit
string. Pages 02 (model providers) and 03 (MCP servers) use the
typed wrapper because their composites *are* `Composite` variants
(`ModelRuntimeObject`, `ExternalServiceObject`).

## Interaction with the P4.5 grant shape

The catalogue is what scopes an instance-URI grant to a specific
resource. Without it, a class-wide grant on `secret_credential` with
an `Any` selector would admit any target URI. With it:

- Catalogue miss → Step 0 denies with `CATALOGUE_MISS`.
- Catalogue hit → engine proceeds to Step 1 and the grant's
  selector (parsed from `secret:<slug>`) only matches exact URIs.

This is why the P4.5 grant shape (per-instance URI + explicit
`fundamentals`) is safe even though the admin holds the grant
class-wide-adjacent: the catalogue is the authoritative presence
check, the grant's selector is the per-instance scope check, and
the manifest's `constraint_requirements` is the purpose check.
Three independent layers — any one saying "no" stops the call.

## References

- [`../../m1/architecture/permission-check-engine.md`](../../m1/architecture/permission-check-engine.md) — Step 0 pipeline position.
- [`../../m1/architecture/graph-model.md`](../../m1/architecture/graph-model.md) — `resources_catalogue` node shape.
- [`../../../concepts/permissions/01-resource-ontology.md`](../../../concepts/permissions/01-resource-ontology.md) — fundamentals + composites + the "tagged fundamental bundle" pattern.
- [Vault encryption](vault-encryption.md) — the seeding pattern in action.

[1]: ../../../../../../modules/crates/domain/src/permissions/engine.rs
[2]: ../../../../../../modules/crates/domain/src/repository.rs
[3]: ../../../../../../modules/crates/domain/src/repository.rs
