# align-default-output-text Proposal

## Why
When `--output` is not specified, `text` is the default format. However, early error paths (during startup) output in JSON format, creating an inconsistency in default behavior. This breaks user expectations and makes error handling unpredictable for scripts and automation tools.

## What Changes
Early errors (such as argument parsing failures or invalid `--log-format` values) will respect the `--output` default value and explicit specification, unifying output format behavior across all error paths.

### In Scope
- Unify early error output to `text` when `--output` is not specified
- Respect explicit `--output` format even for early errors
- Maintain existing command success/failure output behavior

### Out of Scope
- Adding or modifying output formats themselves
- Changing error content (codes or messages) specification
- Modifying log output (`--log-format`) format

### Deliverables
- Early error output defaults to `text`
- Early error output alignment when `--output` is explicitly specified
- Updated corresponding tests

### Risks and Considerations
- Existing tests that assume JSON as default may need modification
- Do not change output destination (stdout/stderr) for early errors
