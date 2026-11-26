# Contribution guide

## Verify CLI error paths

When a test targets a specific CLI error, supply valid values for the other
arguments, including required input paths. For example, a test for
`--format json --fix` can pass a valid YAML fixture so that argument parsing
reaches the check for incompatible options. Assert the exit status and the
diagnostic on stderr. If stdout is part of the command's interface, assert its
expected contents as well.

Run the relevant integration tests while developing and follow the
[repository checks](../AGENTS.md#development-checks) before committing Rust
changes.

## Check the working tree after a merge

1. Fetch the base branch and verify that it contains the merged PR. Use
   `git status --short --branch` and `git diff HEAD` to check the current
   branch and local changes. Inspect untracked files listed by status as well.
2. For any remaining change, establish what it does, how much work remains,
   and whether it belongs to the current task. Complete and verify in-scope
   work when authorized. If it needs a separate PR and publication is
   authorized, use a branch based on the updated base. Preserve other work
   safely and record where it can be recovered.
3. Verify that the local base branch matches the fetched remote base when
   synchronization is part of the task. In the handoff, report the branch and
   worktree state, the PR or recovery location for remaining changes, and any
   verification still pending.
