# Merge command

`repoconf merge` synchronizes the selected local branch with its upstream, merges template branches, runs the post-merge hook, and pushes the result.

## Destination branch

- `--local-branch <name>` selects an existing local branch.
- The default value, `-`, selects `main` if it exists, otherwise `master`.
- A missing destination branch is an error.
- The command switches to the selected branch before pulling or merging templates.
- Repositories without any `repoconf-` remotes are left unchanged, including any pending merge.

## Repository state and continuation

- A pending merge requires `--continue`, including when `--allow-dirty` or `--skip-dirty` is supplied.
- When there is no pending merge, dirty-state handling precedes destination-branch resolution. `--skip-dirty` also skips dirty repositories whose selected or default destination branch does not exist.
- Before committing a pending merge, the current branch must match the selected destination. A mismatch is an error and leaves the pending merge unchanged.
- Unresolved conflicts must be resolved and staged before continuation.
- Continuation runs the pre-commit hook and commits the pending merge before pulling.
- After handling a pending merge, ordinary uncommitted changes cause an error unless `--allow-dirty` or `--skip-dirty` is supplied.
- `--skip-dirty` skips ordinary uncommitted changes before switching branches, pulling, fetching templates, running the post-merge hook, or pushing.
- `--continue` does not disable pulling. Invocations with no pending merge follow the ordinary workflow.
- The caller must keep repository configuration and merge options consistent when continuing a previous invocation.

## Synchronization

1. Detect a pending merge. Without one, check the working tree before selecting and validating the destination branch. With one, validate the destination branch, continue the merge, and then check the working tree.
2. Switch to the destination branch.
3. Pull its configured upstream with fast-forward-only updates, unless `--no-pull` is supplied. This also applies when the destination was already checked out.
4. Fetch template remotes unless `--no-remote-update` is supplied, then read the updated refs and merge template branches.
5. Run the post-merge hook unless `--skip-post-merge` is supplied.
6. Push unless `--no-push` is supplied.

The upstream must be configured when pulling is enabled. A missing upstream is an error that advises configuring one or supplying `--no-pull`. The command must not assume that the upstream is `origin` or that its branch name matches the local branch.

A branch behind its upstream is fast-forwarded. A branch already up to date or ahead is preserved. Divergence causes an error before template merging, requiring the caller to reconcile the histories or explicitly skip pulling. Git configuration must not cause the pull to rebase local commits, squash upstream changes, or create an upstream merge commit.

`--no-pull`, `--no-remote-update`, and `--no-push` independently control upstream synchronization, template fetching, and publishing. `REPOCONF_NO_PULL` and `REPOCONF_NO_PUSH` provide the corresponding defaults for callers that cannot forward arguments.

`repoconf propagate` also accepts `--no-pull` and `REPOCONF_NO_PULL`, forwarding the selected value to every merge invocation.

## Shell shortcuts

`rcm` forwards arguments to `repoconf merge --dir "$PWD"` and does not run Git commands itself. `rcma` and `rcmt` invoke the same synchronization workflow through their mise tasks, including when those tasks always supply `--continue`. `rcmc` runs `rcmt` and then forwards its arguments to `rcm`.
