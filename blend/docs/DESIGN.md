# Design

This document records the durable reasoning behind Blend's current shape. It is
not an implementation backlog or a chronology of completed work. Observable
behavior belongs in [GUIDE.md](GUIDE.md), and implementation detail belongs in
[DEVELOPMENT.md](DEVELOPMENT.md).

## Goal

Blend manages dotfiles as structured state while remaining usable for ordinary
files and applications that edit their own configuration.

Its reconciliation principle is:

> Satisfy declared constraints at best effort: automatically, interactively, or
> by reporting an unsatisfied condition.

That is intentionally narrower than promising general bidirectional
transformation of arbitrary programs or text.

## From Stow to explicit state

Vanilla previously combined a Nushell orchestrator with GNU Stow. Symlinks make
reverse synchronization trivial because repository and deployed file are the
same node, but they also couple Source layout to Target layout and provide no
natural place for structured composition, platform-dependent values,
format-aware rendering, or semantic comparison.

Blend separates Source from Target and writes explicit deployed files. That
makes these capabilities possible:

- one Nickel declaration can vary by platform or machine;
- structured data can render into several configuration formats;
- comparison can operate on parsed values rather than incidental formatting;
- deployment can validate paths, node types, and policy before mutation.

The cost is real: Source and Target can diverge, so reconciliation and
provenance become first-class responsibilities. Blend accepts that cost instead
of hiding it behind one-way generation.

## Why Nickel

Blend needs a general expression language, not a new template language for
every target format. Nickel provides:

- a JSON-like data model suitable for configuration;
- normal functions, bindings, imports, records, arrays, `if`, and `match`;
- contracts and language-server support;
- native Rust embedding without a subprocess or FFI boundary.

Logic stays in `order.ncl` rather than being embedded into TOML, JSON, YAML, or
line-oriented target syntax. Target files remain valid files for their owning
applications.

Nickel is deliberately not used to model every byte of every file. Literal
`from_file` entries remain part of the design for formats or workloads where
structured composition is the wrong trade-off.

## Two source modes

`from_config` and `from_file` express different ownership choices.

`from_config` treats configuration as structured data. It gains platform logic,
semantic diffing, recursive constraints, and selective Source rewriting. It
does not promise presentation fidelity or reversal of arbitrary computation.

`from_file` treats content as literal state. It preserves syntax and supports
natural copying in both directions, but Blend cannot reason about individual
fields inside it.

Keeping both modes explicit is preferable to pretending one representation is
ideal for stable declarative settings, GUI-churned files, scripts, and entire
configuration directories.

## Constraints instead of named merge modes

Practical dotfiles need policies such as:

- own only declared fields;
- preserve application-generated fields;
- remove unexpected fields;
- supply a default when missing;
- migrate selected old values;
- enforce an exact value;
- verify a safety property without repairing it.

Adding a special mode named `ignore`, `whitelist`, `default`, or
`prefer-source` for every case would create a growing second language beside
Nickel.

Blend instead uses a small semantic core:

- declaration says which Source fields participate;
- resolvers observe a concrete Target and return enriched values;
- constraints describe what the resulting state must satisfy;
- unresolved differences fall back to interactive or manual work;
- unmanaged nodes form explicit ownership boundaries.

Reusable higher-level policies can be ordinary Nickel functions built from
these primitives.

### Ordinary values remain non-authoritative

An ordinary literal is a declaration, but divergence remains interactive. This
preserves Blend's useful two-way workflow: writing `theme = "dark"` does not
silently erase an application or machine change.

Authority is explicit. A resolver-returned scalar or `'Enforce` value may
repair the Target automatically; `'Assert` verifies without repairing;
`'Unmanaged` opts out; `'Unresolved` deliberately declines a resolver decision.

The syntax therefore communicates not only a desired value but the authority
attached to it.

### Structured results recurse

A record returned from a resolver is interpreted recursively. It is not
automatically an exact whole-record replacement. This lets policy attach at the
field or subtree where it is understood and leaves other differences
interactive.

Explicit `'Enforce { ... }` is the escape hatch when exact whole-node ownership
is actually intended.

### Absence is structural

Nickel `null` is a present value. `'Absent` uniquely represents non-existence.
Used directly it permits removal; wrapped by `'Assert` it requires that the
node already be absent without authorizing deletion.

Keeping one sentinel avoids confusing missing state with a format's ordinary
null value.

## Why one target-only helper exists

All declared Source fields can carry their own semantics. Target-only children
are the single structural case with no Source key on which to attach a policy.

`blend.with_target_only` fills only that gap. Its constant form applies one
result to every extra child; its resolver form receives `{ key, value }` for
selective handling.

This avoids a separate observation/projection language while still expressing
source-shaped ownership, exact structural membership, selective adoption, and
unmanaged application state.

## Selective reversibility is the boundary

A generated value does not contain all information from the expression that
produced it. Data changes can often be reversed; logic changes cannot be
inferred.

Blend rewrites concrete literals and active conditional leaves when provenance
is clear. It can also perform supported record insertions and deletions.
Computed expressions, interpolation, resolver logic, and ambiguous structural
changes require a manual declaration edit.

This is a semantic boundary, not merely missing automation. Blend degrades per
field and reports what could not be applied instead of claiming universal
two-way synchronization.

Resolver branches returning `'Unresolved` are especially important: Blend must
not replace the resolver with an observed Target value merely because the user
selected that side. There is no concrete declaration value to edit.

## Source defines directory ownership

For a directory `from_file` entry, Blend owns the merged tracked Source and
local overlay after excludes. It does not acquire responsibility for every file
that happens to exist below the deployed Target directory.

Walking target-only directory state would increase sync cost, refresh snapshots
for files Blend never built, and turn application caches or runtime files into
accidental conflicts. Source-shaped traversal keeps ownership predictable.

## Filesystem structure matters

Matching bytes are insufficient when the deployed node has the wrong type.
Blend compares missing nodes, files, directories, and symlinks without
following unexpected symlinks.

An ancestor symlink is path-resolution infrastructure rather than the exact
managed node. Reconciliation may replace the exact Target after explicit
selection, but it must not replace ancestor symlinks or import an unexpected
symlink's referent as if it were the declared node.

Intentional `symlink = true` entries remain explicit.

## Source editing uses two representations

Nickel exposes an embeddable evaluator and an AST parser with useful byte spans,
but no single official typed, trivia-preserving CST for editing.

Blend therefore combines:

- the Nickel AST and `TermPos` spans to understand active expressions and
  concrete leaves;
- tree-sitter-nickel to preserve record shape, field ranges, commas,
  indentation, and comments during insertion/deletion.

This split is a maintenance cost, but it keeps ordinary Target-to-Source edits
surgical. Future source-editing work should preserve the agreement between both
views or adopt a better upstream editing API when one exists.

## Metadata injection stays narrow

The generated `metadata.ncl` module gives editors and Nickel contracts a real
module with stable defaults. At runtime Blend recognizes the canonical import
and merges actual metadata into it.

The deliberately narrow import shape keeps Orders readable and language-tooling
friendly. Supporting arbitrary wrappers or import spellings through ad hoc
source matching would make diagnostics and provenance less reliable. If the
authoring model needs broader metadata access later, it should be redesigned
explicitly.

## Sandbox boundary

Blend's first sandbox layer denies later process execution and socket-based
communication after startup. This reduces what evaluated configuration can
cause through the process, while keeping Blend's existing filesystem workflow
usable.

It is not a filesystem capability model. Source, Target, overlay, and state
paths are still protected by path validation, command classification, normal OS
permissions, and user review. Documentation should describe this accurately
rather than presenting the sandbox as complete isolation.

## Scope boundaries

Blend currently does not try to own:

- system package installation or application availability;
- arbitrary lifecycle scripts and hooks;
- external dependency fetching or automatic third-party updates;
- arbitrary byte-level transformations;
- general multi-source package layering;
- secrets storage;
- fully automatic reversal of Nickel logic;
- presentation-preserving round trips for structured formats.

These may integrate with separate tools or motivate future work, but absorbing
them into the core would weaken Blend's small declarative model.

## Repository history

The current Rust/Nickel Blend replaced Vanilla's earlier Nushell and Stow
workflow. Versions through the current 0.2.x line are developed and released
from the Vanilla repository, which also supplies the primary real-world Order
corpus.

Historical implementation plans and resolved defects remain available through
Git history and private project records. They are intentionally not copied into
the canonical documentation.
