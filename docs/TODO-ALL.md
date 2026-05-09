## Workflow - All tasks

Run as a sparse agent orchestrator.
In a loop, repeatedly spawn a subagent that follows the workflow in docs/TODO.md
for ONE task.
Instruct subagent to only give a minimal output of "DONE" when finished.
Repeatedly spawn subagents until they report that no more work is available.
Subagents should use same model!

Keep your own context usage minimal. Do not read the docs/TODO.md yourself!
Do not read detailed context at all.
purely drive the task agents one by one until all tasks are done!
