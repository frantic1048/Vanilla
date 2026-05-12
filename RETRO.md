# Retrospective Notes

## Software Package Awareness

Some macOS orders deploy configuration for tools that are not currently
installed by the default Brewfile path. This is a known limitation rather than a
bootstrap blocker for now: `blend` manages dotfiles, while Homebrew/proto handle
software installation separately.

Future work may let `blend` understand system package availability, report
missing packages for active orders, or grow into a broader package-management
layer. That is intentionally out of scope for the current bootstrap path.

## Nickel Source Editing Without a Native CST

Nickel gives us an embeddable evaluator and a parser with useful AST byte spans,
but it does not currently give `blend` one official, typed, trivia-preserving
CST suitable for source edits. The implementation therefore carries two
representations:

- `nickel-lang-parser` AST and `TermPos` spans for finding active literal values
  through `match` and `if` branches.
- `tree-sitter-nickel` CST traversal for record shape, field ranges, commas,
  indentation, comments, and insert/delete support.

That split works, but it is a real maintenance cost. The CST path knows about
tree-sitter node names and wrapper shapes, while the AST path knows about Nickel
evaluation patterns. Future source-rewrite features should budget for keeping
those views aligned, or wait for a better upstream Nickel CST/editing API.

## Reverse Sync Is Selectively Reversible

`from_file` entries are naturally bidirectional, but `from_config` entries are
only reversible where the deployed data maps back to a concrete source leaf.
Literal values and simple metadata-driven branches can be rewritten. Computed
expressions, interpolation, or structural edits inside conditional/non-literal
merge operands need a manual merge path.

This is not just an implementation gap. It is the data-versus-logic boundary of
using Nickel as a config DSL. `blend` should keep degrading per field instead of
pretending that every generated file can be fully reversed.

## Source Ownership Beats Target Tree Walking

Directory entries must treat the merged Source view as the ownership boundary.
`blend` should snapshot, diff, and redeploy files it can build from the order
source and local overlays, after excludes are applied. Files that merely happen
to exist under the deployed target directory are target-only state and should not
drive snapshot refresh, sync latency, or conflict prompts.

This mattered in practice for large deployed trees: walking the whole target can
turn a no-op sync into a slow bookkeeping job and can accidentally make `blend`
responsible for files it never built.

## Metadata Injection Shape Is Narrow on Purpose

Runtime metadata is currently injected by replacing the canonical
`import "../metadata.ncl"` form with an `&` merge against runtime values. The
tracked `metadata.ncl` exists so editors and Nickel contracts still see a real
module with defaults.

This keeps orders simple and LSP-friendly, but it depends on a deterministic
import shape. If orders start using alternate import paths, extra whitespace, or
helper wrappers around metadata, the injection strategy should be revisited
rather than patched with more ad hoc string matching.

## Legacy Symlinks Are Structural Drift

The Stow-to-`blend` migration leaves a class of changes where file contents are
already equal but the deployed path is still a symlink, or a parent/inner file is
symlinked. That is not a content diff, but it is still drift from the new
explicit-copy model.

`view`, `status`, and `sync` need to surface and repair this as structural state:
replace unexpected symlinks with real files/directories when the order does not
declare `symlink = true`, while preserving intentional symlink entries.

## Comment-Aware Source Deletes Are Still Incomplete

The structure map already collects comment ranges, but deletion currently
removes the field range and can leave a leading doc comment attached to the next
field. The inverted test for deleting a field with a leading comment is a useful
marker: source editing should eventually treat attached comments as part of the
field deletion range.
