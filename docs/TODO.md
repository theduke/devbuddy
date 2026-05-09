# TODO

## Workflow

* Pick the first unfinished (not checked off) task in docs/TODO.md
* Implement the task
* Validate with `cargo check --quiet --message-format=short`
* Run `cargo fmt`
* Check status with `jj st` and commit changes with `jj commit -m '...'`
* Check off the task item in docs/TODO.md

If instructed to work on multiple tasks:
Run as a sparse agent orchestrator.
Spawn a subagent that follows the workflow in docs/TODO.md for ONE task.
Instruct subagent to only give a minimal output of "DONE" when finished.
Repeatedly spawn subagents until they report that no more work is available.

## Tasks

