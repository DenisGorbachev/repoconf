# Guidelines

## General

* Name: `repoconf`
* Description: a set of script for merging common config files from template git remotes
* Commit message: follow conventional commits (enforced by commitlint)

## Commands

Always use `mise run ...` commands to run the tests / lints.

* Run tests: `mise run test`
* Run specific test: `mise run test <test_file_path>`
* Format code: `mise run fmt`
* Lint code: `mise run lint`
* Check types: `mise run check`

Always execute `mise run fmt` after making changes to the code.

## Do this before committing changes

* Run `mise run test`
* Run `mise run lint`
