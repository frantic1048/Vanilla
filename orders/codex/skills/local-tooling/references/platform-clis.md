# Platform CLIs

## Preference

Prefer platform CLIs when installed and configured, because they usually give more accurate live state than manual web or filesystem inference.

Common CLIs:

- `gh` for GitHub
- `glab` for GitLab
- `twg` for Atlassian/Jira/Confluence/internal work graph tasks
- `pup` for Datadog
- `sentry` for Sentry

## Verification

These CLIs may not always be installed or authenticated on every machine. Before promising live operations, run a small non-mutating check:

- `command -v <tool>`
- a read-only auth/status command when available
- a small read against the relevant host/project when auth status is known to be unreliable

## Failure Handling

If a platform CLI exists but is unauthenticated or configured for the wrong host, report that directly and avoid inferring live state from stale local context.

For GitLab in particular, auth status and API behavior can diverge in some shells. Prefer proving a small authenticated read before planning a larger metadata refresh.
