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

- [x] Set up a gitub ci workflow to run on PRs , following best practices
      Should run format checks, clippy, and build using dx build --desktop
      Should run for linux, macos, windows

      Must sets up all dependencies correctly, if additional ones are needed!
      If required, add scripts/install-dependencies-<platform>.sh helper scripts
      for dependency setup!

      should upload artifacts (final dioxus desktop binary) to ci, whith low
      retention time (eg 2 days or so)

- [ ] ensure the recently setup CI works correctly
      use the github-ci-wait skill to wait for ci failure of the ci branch
      if failed, analyse logs from tail , fix issues, use jj st + jj commit -m '<msg>'
      to commit, 'jj bookmark set ci -r <revision>' and 'jj git push' to push

      then use available skill to compact context, and wait again for ci

      repeat until CI is green!

- [ ] set up release CI
      runs on tag push, with v<xxx> tags
      should run jobs to build win, mac os and linux binaries , add them all
      to the release
