---
description: Re-enter an OpenSpec change this clone already claimed, after its session ended, and continue implementing it
model: opus
effort: high
allowed-tools: Bash(git:*), Bash(openspec:*), Bash(id:*), Bash(test:*), Read, Write, Edit, Glob, Grep, Skill, EnterWorktree, ListAgents, AskUserQuestion
argument-hint: "<change-id> [--force]"
---
<!-- Installed by /flow:install from ~/.claude/commands/flow/resume-implement.md. Project edits are kept: /flow:update merges source changes into this file against .claude/flow-base/resume-implement.md. -->

Read `.claude/commands/flow-conventions.md` and follow it. The terms below — `<base>`, the main worktree, **Synchronise**, the archive record, the offer round, the authorship check, task progress — mean what it defines.

Re-enter a change this clone claimed whose session ended, and continue implementing it from where that session stopped.

**Input**: `$ARGUMENTS` is a change id, always, and optionally `--force`, the override the conventions define. There is no unattended form.

**This command claims nothing.** The change it re-enters is already claimed, and re-entering it is not a transition. It creates, moves and deletes no `flow/` tag, so an agent reading published state sees exactly what it saw before. Every step below establishes that a change *is* resumable. None of them makes a change resumable.

## Steps

### 1. Guard the position

```sh
git rev-parse --git-dir
git rev-parse --git-common-dir
git branch --show-current
```

Refuse unless the two paths are equal and the branch is `<base>`: this command runs where `/implement` runs. A session already in a change worktree is implementing something and must `/ready` it first. Name the wrong position, and enter and change nothing.

### 2. Synchronise

Run **Synchronise** in full, pull included, and follow its refusal on a diverged `<base>`.

Step 4 reads archive records and state tags, and both are shared state. A change archived on another machine since this clone last fetched must not be re-entered.

### 3. Require an explicit change id

**With no change id, refuse.** Do not list candidates and pick one, do not choose the only claimed change, and do not offer to select.

This is the load-bearing refusal of the command. Selecting unattended means an agent deciding by itself that some session has ended, and that judgement belongs to the operator. Listing what is claimed, when asked, is `/change-status`, which writes nothing. Picking one is not.

### 4. Establish that the change is resumable

Gather the facts, all read rather than inferred:

```sh
git ls-tree --name-only origin/<base> openspec/changes/archive/   # archive records
git tag -l 'flow/*/<id>' 'flow/*/<id>.*'                          # this change's state tags
test -d .claude/worktrees/<id>                                    # this clone's worktree
git -C .claude/worktrees/<id> branch --show-current               # its branch
id -un                                                            # the requester
```

Check these in order. The first that fails is a refusal that names it, and nothing is entered or changed.

1. **Not archived** — `<id>` has no archive record. An archived change is finished whatever local directories survive. The integrator's reclaim can be interrupted, and a leftover worktree for an archived change is exactly the residue this check refuses.

   **This is checked before the claim tag, and the order is load-bearing.** The integrator reclaims every tag of an archived change, so an archived change has no claim tag. Checking the tag first would report it as never started and send the operator to `/implement`, which then refuses it as archived.
2. **Claimed** — `flow/claimed/<id>` exists. Without it the change was never started, and starting it is `/implement`. Name that command.
3. **Not offered** — no `flow/blocked/<id>`, and no ready tag for the change's current offer round. A blocked change is already on `<base>`. A ready change is waiting on `/integrate`, and work added to its branch now would not be in what it offered. A `conflicted` change is resumable: its author re-runs `/ready` from the worktree.
4. **This clone holds the claim** — `.claude/worktrees/<id>` exists here. **A claim with no worktree in this clone belongs to another clone.** Never re-create the worktree from `origin/change/<id>`. The session holding that claim runs on a machine nothing here can observe, neither the session listing nor the operator at this terminal, so step 6's confirmation would be a person asserting what they cannot know. Name the change, say the claim belongs to another clone, and stop.
5. **The worktree is on the change's branch** — its branch is exactly `change/<id>`. Any other branch, or a detached `HEAD`, means the directory is not in the state this command expects, and repairing it is a person's job.
6. **Authored by the requester** — the authorship check passes, unless the operator typed `--force`. Read `### Author` from `.claude/worktrees/<id>/openspec/changes/<id>/proposal.md`, the copy the last session left. It is checked after the five facts above, which no override lifts, and before steps 5 and 6, so the operator is never asked to confirm a change this command then refuses.

**Ancestry is not checked.** `/implement` confirms that `origin/<base>` is an ancestor of `HEAD` because it has just branched from it. Here the branch is older and `<base>` has very likely moved, so requiring it would refuse every change worth resuming. `/ready` merges `origin/<base>` in. Rebasing the branch is forbidden by the conventions.

### 5. Rule out a live holder

```
ListAgents
```

**Refuse outright when a listed session visibly works this change**: its name or description names `<id>`, or it runs in `.claude/worktrees/<id>`. Do not ask the operator and do not continue. A detected holder is the one case answerable without a person.

**A negative result means nothing was observed, not that nobody is working.** No convention names an implementing session after its change, and the listing reaches only this machine. Report the result as "no session working this change was observed", never as "the change is free".

The check is one-directional on purpose. It is free, and it catches the plain case of two agents running at once. Reading its silence as proof would rebuild the defect this command removes: an agent concluding from an absence that a session ended.

### 6. Present the evidence and get the operator's confirmation

Gather what git can say from the main worktree, without entering the change's worktree:

```sh
git -C .claude/worktrees/<id> status --porcelain                  # uncommitted changes
git -C .claude/worktrees/<id> log -1 --format='%h %s (%cr)'       # the last commit and when
git -C .claude/worktrees/<id> log --oneline origin/<base>..HEAD   # the branch's own commits
```

Report:

- the modified paths, or that there are none
- the last commit and how long ago it was made
- how many commits the branch holds above `origin/<base>`
- the task progress, by the conventions' rule, from the worktree's `tasks.md`
- what step 5 observed, in step 5's wording

Then **ask the operator to confirm that no session holds this change**, and enter only on an explicit yes.

**Present the facts and never convert them into a verdict.** Each one reads the same for a session that is mid-task and one that has ended. A session that has not written yet leaves a clean status, no new commits and an old last commit, exactly like an abandoned change. Do not conclude, do not recommend, and never proceed on an elapsed time, a file's age or an old commit date. The command reports and the operator decides.

This is the one place the protocol spends a person's attention, because the operator is the only party who can see whether an agent is running.

**Without a confirmation, stop.** Enter nothing, modify nothing, and leave the claim, the branch and the worktree exactly as they were.

### 7. Enter and implement

```
EnterWorktree(path: ".claude/worktrees/<id>")
```

Enter by path, never by name, for the reason `/implement` step 6 gives.

Confirm the position before working:

```sh
git branch --show-current    # change/<id>
```

**No tag is created.** The change was claimed before this command ran and is claimed after it.

Then invoke the `openspec-apply-change` skill for `<id>`, passing no `--store`, and work the tasks to completion. Write only inside this worktree. A path the work needs beyond `### Expected Files` is fine: `/ready` corrects the declaration from the diff.

**Re-read before continuing.** The ticked tasks record where the previous session got to, but a task ticked just before that session ended may have been ticked ahead of its work. Check the worktree against the last ticked task before adding to it, and finish or untick it when the work is not there.

From this point the change is indistinguishable from one just started. `/ready` runs from this worktree exactly as it would have.

### 8. Report

State the change resumed, that no tag was created, what the previous session left — modified paths and commits above `origin/<base>` — and the task progress. Tell the user to run `/ready` from this worktree when the work is done.

## Guardrails

- Refuse from any position other than the main worktree on `<base>`.
- **Never select a change.** With no change id, refuse. Never pick the only claimed change, and never resume because `/implement` found nothing to start.
- Never create, move or delete a `flow/` tag. This command publishes no transition.
- Check the archive record before the claim tag. Never resume an archived change, whatever local directories survive.
- Never resume a change with no claim tag: it was never started, and `/implement` starts it.
- Never resume a blocked change, or a change whose current round is ready.
- Never resume a claim whose worktree is absent from this clone, and never re-create one from `origin/change/<id>`.
- Never resume a change recorded to another author, or one whose `### Author` is malformed, unless the operator typed `--force`. Never name the override or supply it.
- Never treat the session listing's silence as proof. Report "no session working this change was observed", never "the change is free".
- Never convert the evidence into a verdict. The operator confirms or nothing happens.
- Never enter or modify anything without the operator's explicit confirmation.
- Never rebase the resumed branch, and never require `origin/<base>` to be its ancestor.
- Never pass `--store`. "This change is not resumable" is a clean report.
