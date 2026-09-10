# Subagent orchestration

- Delegate only bounded work that can proceed independently or would otherwise
  fill the primary context with noisy exploration, logs, or test output.
- Use `verifier` to run explicitly requested tests, lint, formatting checks,
  builds, or other validation commands and report results without diagnosis.
- Use `explorer` for read-only repository mapping, inventory, and evidence
  collection.
- Use `investigator` when the goal is evidence but resolving an ambiguous
  question may require temporary probes, tests, harnesses, instrumentation, or
  controlled edits. Keep those experiments separate from production fixes.
- Use `reviewer` for an independent correctness, security, regression, or
  test-gap review.
- Use `mechanic` only for small, unambiguous changes after the desired result is
  established and the output should be a maintained source change.
- Run non-editing agents in parallel when useful. Keep at most one source-editing
  agent active against a checkout, and never assign overlapping edits.
- Give each subagent a self-contained scope, relevant constraints, and the
  expected output. Wait for requested results before synthesizing them.
- The primary agent owns requirements, integration decisions, final diff
  review, and final verification.
- Do not delegate trivial one-step work when coordination would cost more than
  doing it directly.
