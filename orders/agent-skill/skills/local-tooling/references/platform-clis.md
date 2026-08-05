# Platform CLIs

## Preference

Prefer platform CLIs when installed and configured, because they usually give more accurate live state than manual web or filesystem inference.

Common CLIs:

- `gh` for GitHub
- `glab` for GitLab
- `twg` for Atlassian/Jira/Confluence/internal work graph tasks
- `pup` for Datadog
- `sentry` for Sentry

Product-specific skills still own command grammar and write safety. This
reference decides where to execute an already appropriate command; it does not
bypass product approval, metadata, or readback requirements.

## Choose The Execution Context

Classify the operation before invoking it:

| Operation | Default execution context |
| --- | --- |
| Local introspection such as `--help`, `version`, command discovery, config parsing, or `doctor` | Sandbox |
| Authenticated remote read | Sandbox first; retry the identical read outside the sandbox if secure-store, network, or host-access restrictions are plausible |
| User-authorized remote mutation such as updating a ticket, MR, PR, comment, release, or deployment | Narrowly scoped out-of-sandbox execution using existing authentication, followed by an authoritative readback |
| Host-bound local operation that inherently needs Keychain/keyring, cache/state writes, Git LFS temp files, local sockets, listeners, or daemons | Out of sandbox when known in advance; otherwise retry there after a characteristic access failure |

When the agent host exposes an escalation mechanism, request the narrowest
useful command or subcommand scope and explain the required host capability.
Do not request a broad shell or language-runtime approval.

## Verification

These CLIs may not always be installed or authenticated on every machine. Before promising live operations, run a small non-mutating check:

- `command -v <tool>`
- a read-only auth/status command when available
- a small read against the relevant host/project when auth status is known to be unreliable

`doctor` and auth-status commands prove configuration and credential presence,
not that every sandboxed GraphQL, REST, upload, or mutation path is reachable.
A successful status check can coexist with `Unable to connect`, DNS, timeout,
or permission failures on the real operation.

## Credential Storage

On the user's personal macOS and Linux machines, prefer CLI-native authentication backed by macOS Keychain or a Linux Secret Service provider. Keep credential material out of plaintext configuration files and environment variables; non-secret host and CLI settings may remain in configuration files.

Do not work around an authentication failure by exporting a token, writing one to disk, passing one on the command line, or replacing secure-store authentication. If a CLI cannot use an available secure store, report that limitation instead of weakening credential storage.

## Classify Failures Before Retrying

Preserve the exact command and error, then distinguish:

- **Secure-store/auth access:** missing Keychain/keyring access, missing OAuth
  refresh token, or a token-required error that conflicts with healthy host
  auth.
- **Remote reachability:** `Unable to connect`, DNS resolution, timeout, TLS,
  or connection-refused errors.
- **Host-resource isolation:** `EPERM`, `operation not permitted`, blocked cache
  or state writes, Git LFS temp failures, or blocked socket/listener creation.
- **Product/contract failure:** an authenticated 4xx response, ACL denial,
  invalid field, validation error, unknown flag, or unsupported command.

Retry the same authoritative operation once outside the sandbox for the first
three classes. Do not change credentials, payload, host, or tool while testing
the execution boundary. If the same failure repeats outside the sandbox, stop
and report it. Escalation does not repair product/contract failures; handle
those through the product workflow.

## Authentication Failures

A sandbox may be able to run a CLI and read its non-secret configuration while being unable to access the host credential store. In that case, an auth-status failure does not prove that the host CLI is logged out.

When a platform CLI reports an authentication error:

1. Preserve and report the exact command and error message.
2. If the failure may be caused by unavailable Keychain or Secret Service access, retry the same non-mutating operation through an allowed host or out-of-sandbox execution path, using only the existing authentication.
3. If that retry is unavailable or also fails, report the remaining verification gap and do not infer live platform state from stale local context.

Never respond to an authentication error by opening a browser, starting an interactive or device authentication flow, running a login or token-refresh command, or modifying stored credentials. Authentication setup is a separate operation and requires an explicit user request.

## Remote Mutation Workflow

For an authorized platform write:

1. Discover the exact command contract and read the current remote state.
2. Fetch field, transition, or repository metadata required by the product
   workflow.
3. Prepare long structured content in a local body file when supported.
4. Run only the mutation through a narrowly scoped out-of-sandbox path, using
   the existing secure-store authentication and an explicit site/host/project.
5. Read the object back authoritatively and verify the intended fields.

Do not fall back to browser authentication merely because the sandboxed write
cannot reach a mutation backend.

## GitLab

For `glab`, `glab auth status` and actual command/API behavior can diverge when the sandbox cannot access a healthy keyring-backed credential. Prefer a small read against the relevant host or project, and retry that same read outside the sandbox when allowed before declaring `glab` unauthenticated.

Errors about a missing OAuth refresh token or required access token can indicate unavailable Keychain/keyring access in managed execution. Report the exact error; do not fall back to `glab auth login`, `glab auth refresh`, token environment variables, plaintext token files, or browser authentication.

Prefer typed, repo-scoped `glab mr`, `glab ci`, and related commands over raw
`glab api` when they cover the operation. A healthy `glab auth status` does not
prove that a sandboxed API call can access the keyring or network.

## Atlassian TWG

Use the root `twg` skill plus the narrow Jira or Confluence skill for product
semantics. Keep `twg help`, command discovery, and local format preparation in
the sandbox.

Treat `twg doctor` as evidence that host OAuth is configured, not as proof that
the sandbox can reach every GraphQL and REST backend. For an authorized Jira or
Confluence mutation, complete the product-required read and metadata checks,
then run the write outside the sandbox with an explicit `--site` and read it
back. If a sandboxed write reports `TWG_COMMAND_FAILED: Unable to connect`,
retry the same write outside the sandbox before opening a browser or declaring
TWG unavailable.

## Host-Bound Companion Operations

Git LFS, Homebrew validation, local proxy/daemon tests, and tools that write
under user cache or state directories can fail even when the repository and
tool are healthy. On `EPERM`, `operation not permitted`, blocked LFS temp/cache
writes, or listener errors, rerun the same relevant operation outside the
sandbox before diagnosing a code or authentication regression.
