# Subagent orchestration

- The primary agent owns the user's intent, task scope, architectural and
  integration decisions, changes requiring the full task context, and the
  final assessment of correctness.
- Proactively delegate substantial, bounded work when independent execution
  can reduce completion time, add useful independent scrutiny, or keep noisy
  exploration and command output out of the primary context. Keep trivial
  work and tightly coupled decisions local.
- Use the smallest set of agents that covers useful independent workstreams.
  Choose the configured specialist role that fits the deliverable. Do not
  create agents merely to occupy available concurrency.

- Use `explorer` for read-only repository mapping, execution-path tracing,
  inventory, and evidence collection. Assign a concrete question and request
  relevant files, symbols, and references. Escalate unresolved behavior to
  the primary agent or an `investigator` instead of expanding into fixes.
- Use `investigator` for ambiguous questions that need hypotheses, temporary
  probes, tests, harnesses, instrumentation, or controlled edits. Request
  evidence that distinguishes the hypotheses, remaining uncertainty, and
  artifacts left behind. Keep experiments separate from production fixes.
- Use `mechanic` for small, unambiguous maintained source changes whose
  desired behavior is already established. Assign exact ownership and
  acceptance criteria. Return unexpected design decisions to the primary
  agent rather than expanding scope.
- Use `reviewer` for independent scrutiny when change complexity, risk, or
  uncertainty warrants it. Provide requirements, the relevant code state,
  and known constraints. Ask the reviewer to trace behavior and challenge
  assumptions, returning actionable findings with evidence and coverage
  limits. Arrange experimental reproduction separately if needed.
- Use `verifier` to execute exact tests, lint, formatting checks, builds, or
  other validation commands assigned by the primary agent. Assignment by
  the primary agent counts as an explicit request within the authorized task.
  Request commands, exit statuses, and concise failure excerpts. Keep
  diagnosis, changes to the validation plan, and fixes with the primary
  agent or an appropriately scoped specialist.

- Give each agent a self-contained brief with its question or deliverable,
  relevant context, scope, constraints, completion criteria, and expected
  evidence. Supply the context it needs without unrelated conversation or
  logs. Keep inter-agent messages concise and legible.
- Continue useful independent work while agents run. Do not duplicate their
  assigned work. Reuse an existing agent for related follow-ups when its
  context remains relevant; reassess scope before expanding the team.
- Run non-editing work in parallel when useful. Allow at most one source
  editor per checkout, including the primary agent, mechanics, and
  investigators making controlled edits. Never assign overlapping edits.
  Coordinate commands that share mutable caches or build artifacts when they
  could interfere.
- Review and validate stable inputs: pause writes to the relevant files or use
  an isolated snapshot. Identify the checked revision or diff, and reassess
  affected results after edits. Wait for required results before making
  dependent decisions or reporting completion.
- Require concise conclusions, supporting evidence, uncertainties, and any
  changes or artifacts from each agent. Resolve material disagreements
  against the underlying evidence rather than taking a vote.
- Final verification means assessing the combined evidence, reviewing the
  final diff, and closing remaining gaps. Scale checks to the change and
  complete required validation. Repeat or broaden checks only after relevant
  changes, failures, incomplete evidence, or unresolved concerns.
