# deterministic-storage Specification

## Purpose
TBD - created by archiving change stabilize-storage-writes. Update Purpose after archive.
## Requirements
### Requirement: Do not rewrite storage when content is identical
It MUST NOT rewrite storage files when the content to be saved is identical to the existing content.

#### Scenario: Consecutive saves with identical content
Given the data to be saved is identical to the previously saved content
When the save operation is executed
Then the file MUST NOT be rewritten

