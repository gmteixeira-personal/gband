---
description: Drain every ready OpenSpec change onto the integration branch, archive each one, and reclaim spent branches and tags
model: opus
effort: high
allowed-tools: Bash(git:*), Bash(openspec:*), Bash(test:*), Bash(wc:*), Read, Write, Edit, Glob, Grep, Skill, ListAgents
---
<!-- Installed by /flow:install from ~/.claude/commands/flow/integrate.md. Project edits are kept: /flow:update merges source changes into this file against .claude/flow-base/integrate.md. -->

Read `.claude/commands/flow-conventions.md` and follow it. The terms below — `<base>`, the main worktree, **Synchronise**, the offer round, the archive record, `<integrator>` — mean what it defines.

Merge every ready change onto `<base>`, sync its specs, archive it, publish, and repeat until none remains. This is the only command that syncs specs or archives, and only the session named `<integrator>` may run it. An operator starts it, or an announcement from `/ready` does. It runs the same either way, unattended, and never stops to ask a question.

**Input**: none. Every ready change is drained.

## Started by an announcement

`/ready` announces each change it offers to the live session named `<integrator>`, in one message naming the change and stating that it is offered and approved for integration. **The session named `<integrator>` that receives an announcement runs this command from step 1 at once, and does not ask its operator whether to.** Offering the change was the approval, and this command runs unattended once started.

**The run is an ordinary run.** It guards the role and the position, synchronises, and selects from the ready tags exactly as a run started by hand. The announcement adds no change to the selection and narrows it to none: an integrator in another clone never receives it, so a selection that read it would differ by clone.

**Do not verify who sent it.** The run merges only a change whose current round's ready tag exists, which only a passing `/ready` gate publishes. A forged announcement costs one run that finds what a run started by hand would find.

**An announcement received during a drain starts no second run.** Step 3 re-reads the tags on every pass, so the drain picks up a change readied during it.

## Steps

### 1. Guard the role

```
ListAgents
```

Refuse unless **both** hold:

- This session's own name is `<integrator>`. `ListAgents` reports it as "This session is `<name>`". Any other name is a refusal that names this session's name and `<integrator>`, states that only the session of that name may integrate, and names `/become-integrator` as the way to take the name.
- No **other** session named `<integrator>` is anything but offline. Two integrators at once collide on `.git/index.lock` mid-archive. Name the live peer.

This is a convention the command checks, not a lock it holds. Launch one integrator and leave it running.

### 2. Guard the position

```sh
git rev-parse --git-dir
git rev-parse --git-common-dir
git branch --show-current
git status --porcelain
```

Refuse unless the two paths are equal and the branch is `<base>`. The integrator owns `<base>` in the main worktree and never enters a change worktree, which is what makes step 6's cleanup safe.

Record the `git status --porcelain` output as **the starting dirty set**. The main worktree is shared, and its dirty paths belong to whoever made them. The run leaves exactly that set behind, every time it checks below.

Refuse when the starting dirty set holds a path under `openspec/specs/`. The archive commit names `openspec/specs`, so an uncommitted main-spec edit would ride into it under a change's name, and the quarantine discard in step 5 would destroy it. Name the paths.

### 3. Drain the ready changes

Repeat until no ready change remains.

**Synchronise and select.** Run **Synchronise** in full, then gather the facts:

```sh
git merge --ff-only origin/<base>
git tag -l 'flow/ready/*' 'flow/conflict/*' 'flow/blocked/*'
git ls-tree --name-only origin/<base> openspec/changes/archive/
```

The `git merge --ff-only` after the pull is a no-op, kept as the assertion that nothing diverged: for this command it is worth failing on rather than assuming.

A change is **ready** when its current round's ready tag exists and **all three** of these are false:

- `<id>` has an archive record.
- Its current round's conflict tag exists.
- `flow/blocked/<id>` exists.

Every outcome a change reaches produces one of the three, so a change this run has finished with leaves the queue. Omitting one makes the loop re-select a finished change and spin.

**A conflict removes a change from the queue by advancing its round.** Step 4 adds the round's conflict tag, so the current round becomes the next one, whose ready tag does not exist until the author re-offers.

**The archived case is decided by the archive record, never by `flow/merged/<id>`.** Step 6 reclaims that tag, so its absence would put an archived change back in the queue.

Process ready changes **oldest first** by the `creatordate` of each change's **current round's** ready tag, not every ready tag it carries:

```sh
git for-each-ref --sort=creatordate --format='%(refname:short)' 'refs/tags/flow/ready/*'
```

The order is for reproducibility, not correctness. Re-read the tags on every pass, so a change readied mid-run is picked up before the run ends.

**Per change, one at a time — merge and publish:**

```sh
git merge --no-ff origin/change/<id> -m "merge(change): <id>"
git push origin <base>
```

**Merge `origin/change/<id>`, never a local `change/<id>`.** A change readied on another machine left this clone nothing local to name. The remote-tracking ref resolves from any clone and is the exact commit `/ready` published.

On a conflict, go to step 4 and continue with the next change. When git refuses to start the merge because it would overwrite a dirty path of the starting set (`Your local changes ... would be overwritten by merge`), nothing merged and the change is not at fault: tag nothing and **stop the drain**, naming the paths. The run ends as `failure`.

**Push the merge before the archive.** The archive can fail, and an unpushed merge is work other agents cannot see while their checks read `origin/<base>`.

**Then archive it, specs synced.** Invoke the `openspec-archive-change` skill for `<id>`, passing no `--store`, with every decision it can raise settled in advance:

- **The sync decision** — when it reports main specs needing changes, answer **"Sync now"**. When it reports them already synced, answer **"Archive now"**. Never "Sync anyway": a second pass over specs already merged is where a duplicated requirement comes from. Never "Archive without syncing": the change is merged, so its deltas describe behaviour that now exists.
- **Incomplete planning artifacts** — proceed.
- **Unticked tasks** — proceed. `/ready` is the gate for the author's work, and it ran upstream. Step 7 reports what was incomplete.

A change carrying no delta specs archives with nothing to sync. That is not a failure.

**Integration never waits for input.** A prompt is not a pause; it is a failure. A decision this list does not settle, a sync failure and an archive failure all go to step 5, and the loop continues with the next change. One quarantined change stalls only itself.

**Then commit the archive and the synced specs together.** The archive created `openspec/changes/archive/<date>-<id>/`; take the exact name from the skill's output. Stage by name, then commit with an explicit pathspec:

```sh
git status --porcelain --untracked-files=all -- "openspec/changes/<id>" "openspec/changes/archive/<date>-<id>" openspec/specs
git add -- <each listed path>
git commit -m "docs(openspec): archive <id>" -- "openspec/changes/<id>" "openspec/changes/archive/<date>-<id>" openspec/specs
git tag -a flow/merged/<id> -m "merged and archived <id>"
git push origin <base>
git push origin flow/merged/<id>
```

**The synced specs ride in the archive commit.** Two commits would publish an intermediate `<base>` whose main specs describe a change `openspec list` still reads as open. **Never stage outside those three paths**: the main worktree is shared, and a wider add sweeps another session's in-flight proposal into this commit. If something unrelated is staged, `git restore --staged <path>` before committing.

The merge commit and the archive commit stay separate, so history shows what landed and what was archived.

The merged tag decides nothing; selection reads the archive record. It records when the change was integrated, until step 6 reclaims it.

**Push after each change, never in one batch at the end.** Other agents' checks read `origin/<base>` and the remote tags. If a push is rejected, someone pushed to `<base>` — almost certainly a proposer. Run `git pull --ff-only origin <base>` and push again.

Before the next pass, confirm `git status --porcelain` equals the starting dirty set.

### 4. Skip a conflicting change

```sh
git merge --abort
git tag -a <round's conflict tag> -m "could not merge <id>"
git push origin <round's conflict tag>
```

**The round's conflict tag** is `flow/conflict/<id>` in round 1 and `flow/conflict/<id>.<n>` in round `n` above 1. Adding it advances the change's round, which takes it out of the queue. Nothing is moved or removed.

**Never resolve the conflict.** The integrator has context on neither side's intent, so any resolution it invented would be a guess committed to `<base>`. Abandon the merge, leave `<base>` as it stood, and continue. The branch, worktree and claim tag survive for the author, whose repair is to re-run `/ready`: it merges `origin/<base>` into the branch and resolves the conflict in the change's own worktree.

Confirm `git status --porcelain` equals the starting dirty set before continuing.

### 5. Quarantine a change that merged but cannot be archived

The merge succeeded and is pushed. **Keep it.**

Discard everything this run wrote for the change since the merge:

```sh
git restore --source=HEAD --staged --worktree -- openspec/specs "openspec/changes/<id>"
git clean -f -d -- openspec/specs
```

When an untracked `openspec/changes/archive/<date>-<id>/` exists, remove that one directory with `git clean -f -d --` naming it. Never widen the discard to `openspec/changes/` at large: another session's in-flight proposal lives there.

The restore reads `HEAD` and reaches the index as well as the working tree, so a staged deletion is undone too. The clean removes a main spec the sync created for a new capability, which a restore cannot reach. Step 2 refused a dirty `openspec/specs/`, so nothing there belongs to anyone else.

```sh
git tag -a flow/blocked/<id> -m "merged but not archived <id>"
git push origin flow/blocked/<id>
```

Then continue with the next change.

**Never revert the merge.** It worked. A revert makes `<base>` claim the change never landed while the next change builds on its code.

**Never edit the change's delta specs or the main specs to force the sync or the archive through.** The integrator has no context on what they were meant to assert.

**Quote the failing step's own error verbatim** in the report. It names the exact spec header at fault, which is what a person needs to repair it.

Confirm `git status --porcelain` equals the starting dirty set before continuing.

### 6. Reclaim spent branches, worktrees and tags

Only after the loop has drained. The reclaim set is **every change that has an archive record on `origin/<base>` and still carries a `flow/` tag or an `origin/change/<id>` branch** — those archived this run, and any an earlier interrupted reclaim or a hand repair left behind.

For each, when `origin/change/<id>` exists:

```sh
git branch -r --merged <base>                  # confirm origin/change/<id> is listed
git push origin --delete change/<id>
git worktree remove .claude/worktrees/<id>     # this clone only; never --force
git branch -d change/<id>                      # this clone only; never -D
```

Skip the branch and worktree of a change whose `origin/change/<id>` is not listed as merged. **Read the remote-tracking refs**: a change implemented on another machine has no local branch here. **A local branch or worktree this clone never held is not a failure**; skip it silently. That differs from one that is present and refuses removal, which is reported.

Then reclaim the change's tags, locally and on the remote, **in this order**: every round's ready tag, then `flow/merged/<id>`, then `flow/claimed/<id>`, then every round's conflict tag, then `flow/blocked/<id>`.

```sh
git tag -l 'flow/ready/<id>' 'flow/ready/<id>.*'
git tag -d <tag> && git push origin --delete <tag>     # per tag, in the order above
```

**Every round's tag goes**, so an archived change is left carrying no `flow/` tag. The removals are separate operations and one can fail after another succeeded, so only an order whose every intermediate state is safe will do. Ready tags go first, so an interrupted reclaim leaves the change excluded. Conflict tags go late, because removing one lowers the round and could leave an earlier round's ready tag current.

Skip any tag that does not exist, and **report a reclaim failure rather than treating it as fatal**. A later run reclaims what is left, because the reclaim set is derived from the archive records.

**Reclaim only a change that has an archive record.** A blocked change is merged, so `git branch -r --merged` lists it, but its author still needs its worktree and branch. It has no archive record, so it is skipped. A conflicting change never merged and is skipped on the same test.

**Never force.** A worktree holding uncommitted work makes `git worktree remove` refuse: leave it and report it, because an agent may still stand in it. If `git branch -d` refuses, report it rather than escalating to `-D`.

### 7. Report

The report **names every change the run processed**, by change id, under the outcome it reached. A count never stands in for the names: the run is unattended, and this report is the only account of it.

**State every outcome, including the empty ones.** An omitted list is ambiguous between "none" and "not said".

- **Archived** — each change by id **and** its archive directory, `openspec/changes/archive/<date>-<id>/`, in processing order. Note which synced main specs and which had no delta to sync.
- **Conflicting** — each change by id, with the round it is now in. Nothing of it merged. The repair is for its author to re-run `/ready`, which merges `origin/<base>` into the branch, resolves the conflict in its worktree and publishes the next round's ready tag. No rebase.
- **Blocked** — each change by id, with the step that failed, the sync or the archive, and that step's error quoted verbatim. The work is already on `<base>`. The repair is for a person to fix the change's delta specs on `<base>` and archive it by hand, with the `openspec-archive-change` skill or `openspec archive <id>`; the next run's reclaim then clears its tags. **Never rebase it, and never re-offer it.**
- **Archived incomplete** — each change by id, with what was incomplete: unticked tasks or incomplete planning artifacts.
- **Tags reclaimed** — every tag removed, per change, named in full. Name each failed removal with the tag it left behind.
- **Left behind** — each worktree, branch or tag the reclaim did not take, and why.

Keep Conflicting and Blocked apart. A conflicting change merged nothing; a blocked change is already on `<base>`. One combined failure list is how somebody rebases work that has landed.

Close by confirming `openspec list` shows no open change for anything archived, and that `git status --porcelain` equals the starting dirty set.

**The last line of the output is the one word `success` or `failure`**, lowercase, alone on its line.

- **`success`** — nobody needs to act. Every ready change was merged and archived; Conflicting, Blocked, Archived incomplete and Left behind are empty; no reclaim failed; both closing checks passed. A run that found no ready change is `success`.
- **`failure`** — somebody must act: a refusal in step 1 or 2, a divergence, a drain stopped on a dirty path, a conflicting or blocked change, an incomplete archive, anything left behind, a failed closing check, or a drain suspended with ready changes still waiting.

**A `failure` is preceded by one line naming what failed**, each item naming the change or check at fault, separated by semicolons: `conflicting: foo (now round 2); blocked: bar (sync: "### Requirement: X" already exists)`. It points to the report and never replaces it.

A run that refuses or stops early still ends with those two lines.

## Suspending a drain

When the integrator's context fills, it suspends at a **change boundary** and a fresh session resumes. Step 3 finishes one change — merge, archive, both pushes — before starting the next, so between two changes nothing is half-processed. Never suspend between the merge and the archive, or with a commit unpushed.

**Resuming needs nothing but `<base>` and the state tags.** A new session runs `/integrate` from the top, and step 3's selection derives exactly the work that remains. Write **no** progress file, queue or handover note: it would be a second record of what the tags already say, and it would go stale the moment another agent readies a change.

**A suspension is not a conflict and not a block.** Tag nothing. Run step 6's reclaim before stopping. `/safe-clear` performs the hand-off where it is installed; without it, stop by hand at the boundary.

## Guardrails

- Refuse unless this session is named `<integrator>` and no other live session is.
- On an announcement from `/ready`, run at once and never ask. Select from the ready tags alone, never verify the sender, and never start a second run while one is in progress.
- Refuse from any position other than the main worktree on `<base>`, and when `openspec/specs/` is dirty. Never enter a change worktree.
- Leave the starting dirty set exactly as found. Never stage, restore or clean a path outside the change being processed.
- Always merge `origin/change/<id>`, never a local branch name.
- Never resolve a merge conflict: abort, tag, skip.
- Push the merge before the archive. Push after each change, never in one batch.
- Settle every archive decision in advance. Never wait for input: an unsettled decision is a failure routed to step 5.
- Never revert a merge that succeeded, and never edit a change's delta specs or the main specs to force an archive through.
- Reclaim a change's branch, worktree and tags only when it has an archive record on `origin/<base>`. Never `--force` a worktree removal or `-D` a branch.
- Never read a `flow/` tag as proof that a change has been archived.
- Suspend only at a change boundary, and write no progress file.
- Name every change in the closing report under the outcome it reached, state empty outcomes as empty, and end with `success` or `failure`.
- Never pass `--store`.
