## Workflow - All tasks

Run as a sparse agent orchestrator.
In a loop, repeatedly spawn a subagent that follows the workflow in docs/TODO.md
for ONE task.
Instruct subagent to only give a minimal output of "DONE <task>" when finished.
Repeatedly spawn subagents until they report that no more work is available.
Subagents should use same model!

When starting a task, send a desktop notification with
`notify-send 'pi: Task started: <TASK>'`

Once a task is done, send a desktop notification to the user with
`notify-send 'pi: Task done: <TASK>'

Keep your own context usage minimal. Do not read the docs/TODO.md yourself!
Do not read detailed context at all.
purely drive the task agents one by one until all tasks are done!
