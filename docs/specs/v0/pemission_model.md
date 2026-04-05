A strong hardening model for agentic systems should use a canonical resource ontology that is **capability-centered, closed over authority surfaces, and separate from tool names**.  [lambda-the-ultimate](http://lambda-the-ultimate.org/node/3930)

## Core idea

The key design move is to stop treating “tool” as the unit of authorization and instead treat authorization as a relation over **subject, action, resource, constraints, and delegation path**.  [lambda-the-ultimate](http://lambda-the-ultimate.org/node/3930) That is how mature systems such as capability-based security and cloud IAM stay systematic: actions are standardized, resources are typed, and conditions further narrow use.  [lambda-the-ultimate](http://lambda-the-ultimate.org/node/3930)

## Canonical shape

A useful canonical model is:

$$
Permission = \langle subject,\ action,\ resource,\ constraints,\ provenance \rangle
$$

This mirrors the logic used in capability systems and resource/action policy systems, where authority is tied to a specific object and operation rather than ambient possession of a broad tool.  [lambda-the-ultimate](http://lambda-the-ultimate.org/node/3930)

## Resource ontology

A practical ontology for agentic systems should have these top-level resource classes:

| Resource class | What it covers |
|---|---|
| Filesystem object | Files, directories, mounts, repositories, temp paths |
| Process/exec object | Shell commands, binaries, interpreters, containers, jobs |
| Network endpoint | Domains, URLs, IPs, ports, protocols, remote APIs |
| Data object | Documents, tables, vector stores, memory entries, transcripts |
| Secret/credential | API keys, tokens, cookies, certificates, SSH keys  [cheatsheetseries.owasp](https://cheatsheetseries.owasp.org/cheatsheets/Secrets_Management_Cheat_Sheet.html) |
| Identity principal | User identity, service account, role, tenant, session  [blog.balancedsec](https://blog.balancedsec.com/p/understanding-cissp-domain-5-identity-3f0) |
| Device/peripheral | Camera, microphone, clipboard, GPU, USB, browser profile |
| External service object | GitHub repo, Slack workspace, Jira project, cloud bucket |
| Model/runtime object | Model endpoint, prompt templates, system policies, agent memory |
| Control-plane object | Tool registry, policy store, approval queue, audit log |
| Communication object | Email account, chat thread, webhook, MCP channel |
| Economic resource | Token budget, spend budget, rate limit, quota  [quali](https://www.quali.com/resource/critical-capabilities-for-ai-agentic-security/) |
| Time/compute resource | CPU time, wall-clock duration, concurrency slots, memory quota  [lambda-the-ultimate](http://lambda-the-ultimate.org/node/3930) |

The point is not that every system must use these exact labels, but that every authority surface should map into one of these resource families.  [quali](https://www.quali.com/resource/critical-capabilities-for-ai-agentic-security/)

## Standard actions

You also need a standardized action vocabulary that is reusable across resource classes.  [docs.aws.amazon](https://docs.aws.amazon.com/service-authorization/latest/reference/reference_policies_actions-resources-contextkeys.html) A good base set is:

- Discover, list, inspect
- Read, copy, export
- Create, modify, append, delete
- Execute, invoke, send
- Connect, bind, listen
- Delegate, approve, escalate
- Store, retain, recall
- Configure, install, enable, disable
- Spend, reserve, exceed
- Observe, log, attest

This follows the same general idea as IAM action/resource separation: tools become merely implementations of actions on resources.  [docs.aws.amazon](https://docs.aws.amazon.com/service-authorization/latest/reference/reference_policies_actions-resources-contextkeys.html)

## Constraints

Without constraints, the ontology is too weak.  [docs.aws.amazon](https://docs.aws.amazon.com/service-authorization/latest/reference/reference_policies_actions-resources-contextkeys.html) Each permission should also carry condition slots such as path prefix, command pattern, domain allowlist, data label, user purpose, max spend, time window, output channel, human approval requirement, sandbox requirement, and non-delegability.  [docs.aws.amazon](https://docs.aws.amazon.com/service-authorization/latest/reference/reference_policies_actions-resources-contextkeys.html)

## Why this is better

This gives you a closed *policy grammar* even if the ecosystem is open-ended.  [lambda-the-ultimate](http://lambda-the-ultimate.org/node/3930) New tools can still be added, but each tool must declare what actions it performs on which resource types under which constraints, instead of inventing ad hoc permission semantics.  [lambda-the-ultimate](http://lambda-the-ultimate.org/node/3930)

## What “exhaustive” should mean

The ontology does **not** need an exhaustive list of all concrete resources in the world.  [docs.aws.amazon](https://docs.aws.amazon.com/service-authorization/latest/reference/reference_policies_actions-resources-contextkeys.html) It needs an exhaustive list of **authority-bearing resource categories**, plus a rule that any new integration must project its operations into that schema before it can be enabled.  [lambda-the-ultimate](http://lambda-the-ultimate.org/node/3930)

## Example mapping

For your earlier examples:
- `write tool` becomes `modify` on `filesystem object` constrained by path prefix and file type.
- `web search` becomes `discover/read` on `network endpoint` through an approved broker, with no raw `connect` or `execute` authority.
- `git push` becomes `modify/send` on `external service object` plus use of a `secret/credential`.
- `run python` becomes `execute` on `process/exec object` plus bounded `filesystem`, `network`, and `time/compute` permissions.  [cheatsheetseries.owasp](https://cheatsheetseries.owasp.org/cheatsheets/Secrets_Management_Cheat_Sheet.html)

## Design rule

If you want to harden systems like OpenClaw systematically, the canonical rule should be: **every tool must ship a machine-readable authority manifest**.  [quali](https://www.quali.com/resource/critical-capabilities-for-ai-agentic-security/) That manifest should declare resource classes touched, actions performed, transitive resources consumed, delegation behavior, approval defaults, and required constraints before the tool can be loaded.  [docs.aws.amazon](https://docs.aws.amazon.com/service-authorization/latest/reference/reference_policies_actions-resources-contextkeys.html)

## Minimal schema

A compact version could look like this:

- `resource_type`
- `resource_selector`
- `action`
- `conditions`
- `delegable`
- `approval_mode`
- `audit_class`
- `transitive_dependencies`
- `revocation_scope`

That gives you something much closer to a proper capability system than a loose tool allowlist.  [lambda-the-ultimate](http://lambda-the-ultimate.org/node/3930)

Next Step: a concrete **agent permission model spec** with YAML examples for files, exec, web, secrets, and MCP tools.