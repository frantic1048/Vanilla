# Filesystem Search

## Preferred Tools

- Use `rg` for text search.
- Use `rg --files` for fast file listing when the task is repo/file discovery.
- Use `fd` for filesystem finding when name, extension, depth, or type filters make it clearer than `rg --files`.

## Fallbacks

- If `rg` or `fd` are not installed on the current machine, fall back to standard tools already available in the task environment.
- Avoid adding dependencies just to search files.

## Command Style

- Keep discovery commands narrow and readable.
- When searching dotfiles, start in `~/Vanilla` and the relevant `orders/` subdirectory if the topic is known.
