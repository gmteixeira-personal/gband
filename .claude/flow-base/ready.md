---
description: Merge the integration branch into a change, gate it, publish its branch and tag it ready, then return to the integration branch
model: opus
effort: high
allowed-tools: Bash(git:*), Bash(openspec:*), Bash(id:*), Bash(test:*), Bash(./.claude/flow-gate), Read, Write, Edit, Glob, Grep, ExitWorktree, ListAgents, SendMessage
argument-hint: "[--force]"
---
<!-- Installed by /flow:install from ~/.claude/commands/flow/ready.md. Project edits are kept: /flow:update merges source changes into this file against .claude/flow-base/ready.md. -->

Read `.claude/commands/flow-conventions.md` and follow it. The terms below — `<base>`, a change worktree, **Synchronise**, the offer round, the archive record, the coordination block, the authorship check, `<integrator>` — mean what it defines.

Offer the change in this worktree for integration: commit it, merge `<base>` into it, prove the merge passes the gate, correct its file declaration, publish the branch, tag it ready, return this session to `<base>`, and announce the change to the integrator.

**Input**: `$ARGUMENTS` is empty or `--force`, the override the conventions define. The change is whatever this worktree holds.

## Steps

### 1. Guard the position

This command runs **inside a change worktree on its own branch**.

```sh
git rev-parse --git-dir
git rev-parse --git-common-dir
git branch --show-current
```

Refuse unless the two paths **differ**, so this is a linked worktree, and the branch matches `change/<id>`. Take `<id>` from the branch. Name the wrong position, and create and change nothing. A session in the main worktree has no change to ready.

### 2. Synchronise

Run the **fetch half** of **Synchronise**, never the pull: the pull targets the checked-out branch, which here is `change/<id>`.

Every check below reads the remote — the round, the ready tag, `origin/<base>` for the merge and the declaration diff — so the fetch runs before any of them.

**Never rebase this branch onto the refreshed `origin/<base>`.** The branch is published from its first offer, and a rebase rewrites commits other agents may have fetched. Step 5 merges `origin/<base>` in instead.

### 3. Guard the state

Refuse unless all of these hold, in this order:

1. `flow/claimed/<id>` exists — **unsuffixed, whatever the round**. A change is claimed once; a re-offer after a conflict needs no second claim.
2. `<id>` has no archive record. An archived change is finished.
3. The **current round's** ready tag does not exist. If it does, the change is already offered in this round. A change carrying `flow/ready/<id>` and no conflict tag is in round 1 and is refused. One carrying `flow/ready/<id>` and `flow/conflict/<id>` is in round 2, `flow/ready/<id>.2` does not exist, and it is admitted.
4. `flow/blocked/<id>` does not exist. A blocked change is already on `<base>`, and its repair is not a re-offer: the report in `/integrate` step 7 states it.
5. **Last**, the authorship check, reading `### Author` from this worktree's `openspec/changes/<id>/proposal.md`, unless the operator typed `--force`. A change started under the override needs it again here: nothing records an override.

Every refusal here comes before step 4 commits anything, so a refused offer leaves the branch exactly as it was.

### 4. Commit the work

A merge left in progress is refused before anything is committed:

```sh
git rev-parse -q --verify MERGE_HEAD    # succeeds: refuse
```

Name it, and say to conclude it with `git commit` or abandon it with `git merge --abort`, then re-run. Committing it here would commit whatever it holds, conflict markers included.

Stage the dirty set by name, by the conventions' staging rule, and commit it with a message in the repository's style. Report every dirty path an ignore rule kept out: a file the change created that the repository ignores will not reach the integrator. If there is nothing to commit and the branch already carries the work, continue.

**The work is committed before the merge, never after.** A merge into a tree holding uncommitted work either refuses or mixes the change's edits with the merge's.

### 5. Merge the integration branch

```sh
git merge --no-edit -m "merge(<base>): <id>" origin/<base>
```

**`origin/<base>`, never `<base>`.** Step 2 fetched without pulling, so the local `<base>` is wherever the main worktree last left it.

**This is what makes the gate prove the merge result.** Without it the gate tests `<base>` as it stood when the change started, and two changes each green alone can merge into a red `<base>`.

When `origin/<base>` is already an ancestor of `HEAD`, git reports the branch up to date and adds nothing: continue to step 6.

**Resolve every conflict here.** This session holds the change's context, and `<base>`'s history records the other side's. The integrator holds neither, which is why `/integrate` resolves nothing. For each path `git diff --name-only --diff-filter=U` lists:

- **Read both sides.** This change's intent comes from its own artifacts. The other side's comes from `git log origin/<base> -- <path>` and the archived change that introduced it.
- **Keep both intents.** Never take a side wholesale because it is this change's own, and never merge with `-X ours` or `-X theirs`: either discards the other side's work unread.
- **This change's own `openspec/changes/<id>/`** takes this branch's version, `git checkout --ours -- <path>`. Nothing on `<base>` writes there while the change is in flight.

Then conclude the merge:

```sh
git add -- <each resolved path>
git commit --no-edit
```

Record each resolved path with one line on how it was resolved, for the report.

**A conflict nobody here can resolve aborts the merge.** When the two sides assert incompatible intents, choosing between them is a person's call:

```sh
git merge --abort
```

Then **stop**: create no tag, push nothing, and stay in the worktree. Report each conflicted path and why it was not resolved. Never leave a merge in progress.

**This step is the repair for a conflict.** A change `/integrate` marked conflicting is offered again by running this command again: this step merges in the `<base>` the integrator could not merge the change into, and step 9 publishes the next round's ready tag. No rebase and no separate merge comes first.

### 6. Gate

Run the gate in this worktree, against the merge step 5 produced:

```sh
openspec validate <id> --strict
test -x .claude/flow-gate && ./.claude/flow-gate
```

**The repository declares its own checks** — build, tests, linters — in an executable `.claude/flow-gate` at its root. This command names none of them. Run it from the worktree root with the leading `./`, so it is this tree's copy and not the main worktree's. Exit status zero passes; anything else fails. When the repository has no such file, the gate is the validation alone, and the report says the repository declares no gate. Never substitute a guessed build or test command for an absent one.

On any failure: **stop**. Create no tag, push nothing, and stay in the worktree, where the failure can be fixed and the command re-run. Report the decisive lines, not the full log.

**The merge commit stays on the branch when the gate fails.** The failure is most likely the combination itself, which is what has to be fixed here. Name which failing files this change touched, as `git diff --name-only origin/<base>...HEAD` lists them, and which it did not. A failure confined to files this change never touched can be `<base>` already broken, and whose repair it is turns on that.

This gate is what keeps `<base>` green. The integrator merges serially and runs no gate, so a broken branch would land unchecked.

### 7. Reconcile the declaration

Compare what the change touched against what it declared:

```sh
git diff --name-only origin/<base>...HEAD
```

**Three dots, never two.** The three-dot form diffs from the merge base, which after step 5 is the merged `origin/<base>` commit. A file the merge brought in unchanged drops out, and a file this change or its conflict resolution altered stays in.

Read `### Expected Files` from `openspec/changes/<id>/proposal.md`. A touched file is covered when it equals an entry or sits under an entry's directory prefix.

On any mismatch — a touched file no entry covers, an entry nothing touched, or an entry wider than the work beneath it:

1. **Report it.** It is a warning, not a refusal: the change is written, and blocking it helps nobody.
2. **Rewrite `### Expected Files` to match the diff**, each entry the narrowest path or prefix containing the files modified, keeping `openspec/changes/<id>/`. **Correct in both directions**: widen to cover an undeclared file, and narrow an entry wider than the work. This is the only point where a declaration is checked against reality, and a widened entry is the one every other agent's overlap check then reads. Rewrite that section alone. When the proposal has no coordination block, append one holding `### Expected Files` alone.
3. **Commit it on its own**, never as an amend:

   ```sh
   git add -- openspec/changes/<id>/proposal.md
   git commit -m "<message>" -- openspec/changes/<id>/proposal.md
   ```

   After step 5 the last commit can be the merge, and in a later round one the branch already published. Amending it rewrites history the remote holds, and step 9's push is rejected.

Commit only when something was corrected. The correction ships on the branch the integrator merges, so the archived record and the merged footprint agree.

### 8. Resynchronise

The gate may have taken minutes, and `<base>` may have moved meanwhile. Run the fetch half again and test whether this branch still carries `<base>`:

```sh
git fetch origin --tags --prune --prune-tags
git merge-base --is-ancestor origin/<base> HEAD
```

- **It succeeds.** Continue to step 9 at once, so nothing slow runs between this check and the push.
- **It fails.** Another change landed while the gate ran. Return to step 5 and run it and every step after it again.

**The loop has no bound.** A bound would publish a change proven against a `<base>` that has already moved, which is the failure step 5 exists to prevent. Count the merges for the report.

### 9. Publish and tag

```sh
git push -u origin change/<id>
git tag -a <round's ready tag> -m "ready <id>"
git push origin <round's ready tag>
```

**The round's ready tag** is `flow/ready/<id>` in round 1 and `flow/ready/<id>.<n>` in round `n` above 1. It is added and nothing is removed: the earlier round's tags stay, so every attempt the change made stays readable.

Push the branch before the tag. A ready tag pointing at a commit the remote lacks offers the integrator something it cannot merge. If the branch push fails, stop without tagging.

The tag is annotated because the integrator orders ready changes by `creatordate`. On a lightweight tag that resolves to the tagged commit's date, so a change committed early and readied late would sort ahead of one committed late and readied early.

### 10. Return to `<base>`

Only after every step above succeeded, leave the worktree:

```
ExitWorktree(action: "keep")
```

`keep`, because the branch and worktree must survive for the integrator to merge and reclaim them.

Then run **Synchronise** in full, pull included: the session is back in the main worktree on `<base>`, where the pull applies. **A fetch alone is not enough here.** It moves `origin/<base>` and leaves the local `<base>` behind, so the session would end on a branch that reads as current and is not.

Confirm the round trip: `pwd` is the repository root, `git branch --show-current` is `<base>`, `git rev-parse <base> origin/<base>` prints one commit twice, and `git worktree list` still shows this change's worktree. If the pull refuses, report it: the change is already offered, and the main worktree needs a person.

### 11. Announce the change to the integrator

```
ListAgents
SendMessage(to: "<integrator>", message: "<id> is offered and approved for integration.")
```

One message, naming the change and stating that it is offered and approved for integration. Running this command is the approval; the message lets a live integrator recognise it as the cue `/integrate` names. It grants nothing by itself.

**It runs here and nowhere earlier.** The branch is pushed, the ready tag is published and this session has left the worktree, so an integrator reacting at once finds the change merge-ready and the offering session out of its way.

**The announcement is never part of the protocol's state.** The ready tag is the whole of the offer, and `/integrate` selects, merges and archives a ready change identically whether it was announced or not. This step creates, moves and deletes no tag and writes no file. The session listing reaches only this machine, so an integrator that waited for messages would stall silently on every change offered from another clone. Git stays the shared channel; this is a doorbell on top of it.

**Every outcome is a report line and none is a refusal**: no session named `<integrator>` listed, one listed but not live, more than one live, or a failed send. More than one live `<integrator>` is reported rather than messaged, since `/integrate` refuses that state. **No integrator reached is the ordinary outcome**: the change is offered, and `/integrate` picks it up from the tag.

**A successful send is delivery to a session, never a read.** A peer can hold the message for its own operator's approval, let it expire or refuse it, and a peer off this machine reports nothing back. Claim delivery and nothing beyond it.

### 12. Report

State the change offered, the branch pushed, the ready tag published, and — past round 1 — the round and the conflict tags it still carries. State how many merges of `origin/<base>` the offer took, each conflict resolved with one line on how, the gate result and whether the repository declares a gate, any declaration drift corrected, the task progress, and what became of the announcement. Do not archive: `/integrate` is the only command that archives.

**Ask the operator to clear the session** before running another workflow command in it. This change's context is spent, and carrying its files, id and tasks into the next command is how they come to be attributed to the next change. The request gates nothing, and the session never clears itself.

## Guardrails

- Refuse unless in a linked worktree on `change/<id>` with `flow/claimed/<id>` present, unsuffixed, whatever the round. Refuse an archived or blocked change.
- Derive the round from the synchronised conflict tags alone. Refuse on that round's ready tag and publish that round's ready tag. Never delete an earlier round's tag.
- Never tag ready when the merge could not be completed or the gate failed.
- Never exit the worktree on failure. Stay where the code is.
- Commit the work before merging, and merge `origin/<base>`, never the local `<base>`.
- Never rebase this branch and never amend a commit on it.
- Resolve every conflict by reading both sides. Never merge with `-X ours` or `-X theirs`. This change's own `openspec/changes/<id>/` takes this branch's version.
- Abort a merge that cannot be resolved, and stop. Never leave a merge in progress.
- Run the repository's `.claude/flow-gate` when it exists, and never invent a gate when it does not.
- Correct `### Expected Files` in both directions, in a commit of its own.
- Publish only when `origin/<base>` is still an ancestor of `HEAD` after the resynchronisation.
- Push the branch before the tag, and push the tag by name.
- Always `ExitWorktree(action: "keep")`, never a removing action.
- The announcement is never a gate and never a tag. Report delivery, never that the integrator read or will act on it.
- Never archive, never sync specs, and never merge into `<base>`. Never pass `--store`.
- Never name the override or supply it.
