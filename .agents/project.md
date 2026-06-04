# Repoconf project

## Concepts

### CreateCommand

- Must be callable as `create`
- Must have methods:
  - `run`
    - Must create a repo on GitHub
    - Must call [InitCommand](#initcommand)

### InitCommand

- Must be callable as `init`
- Must have methods:
  - `run`
