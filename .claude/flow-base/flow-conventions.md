---
description: Show the shared rules the flow commands follow
model: opus
effort: low
allowed-tools: Read
---
<!-- Installed by /flow:install from ~/.claude/commands/flow/conventions.md. Project edits are kept: /flow:update merges source changes into this file against .claude/flow-base/flow-conventions.md. -->

# Conventions for the flow commands

Every flow command follows the rules below. Invoked on its own, this command prints them so the rules can be reviewed without running an operation.

The suite lets several agents, on one machine or several, work an OpenSpec repository at once. Git is the only shared channel: a session on another machine is reachable no other way, and git notifies nobody. Five commands carry the whole protocol:

- `/propose` writes a change's planning artifacts on the integration branch and publishes its coordination declaration.
- `/implement` claims an unblocked change and branches it into its own worktree.
- `/ready` merges the integration branch into the change, gates the result, publishes its branch and announces it to the project's live integrator session.
- `/integrate` merges every ready change onto the integration branch. It is the only command that syncs specs or archives.
- `/change-status` reports every open change's state and writes nothing.

`/resume-implement` re-enters a change this clone claimed after its session ended, on the operator's confirmation. It publishes no transition.

`/become-integrator` names a session as the project's integrator. It writes nothing to the repository.

The commands assume nothing about the repository's layout beyond its `openspec/` root. They name no project, directory, build tool or branch of their own.

## Preconditions

- Run `git rev-parse --show-toplevel`. If it fails, stop: the working directory is not a git repository.
- The repository has an `openspec/` root at its top level, and `openspec list --json` succeeds there. Otherwise stop and say so.
- The repository has a remote named `origin`. The protocol publishes everything there; without it, stop and say so.
- Never pass `--store`. Every change lives in the repository's own `openspec/changes/<id>/` and is addressed by that path.
- Never pass `--no-verify` and never disable hooks. A hook rejection is reported verbatim and the command stops.

## The integration branch

The integration branch is the default branch of `origin`:

```sh
git symbolic-ref --short refs/remotes/origin/HEAD    # prints origin/<base>
```

Strip the `origin/` prefix. Every command writes it as `<base>`. **Never assume `main`, `dev` or any other name.** Reading it from `origin` gives the same answer in every clone. When the symbolic ref is unset, stop and name the command that sets it: `git remote set-head origin --auto`.

## Positions

The **main worktree** is the one where `git rev-parse --git-dir` and `git rev-parse --git-common-dir` print the same path. A **change worktree** is `.claude/worktrees/<id>`, a linked worktree on branch `change/<id>`.

| command | runs in |
|---|---|
| `/propose`, `/implement`, `/resume-implement`, `/integrate`, `/become-integrator` | the main worktree, on `<base>` |
| `/ready` | a change worktree, on its own `change/<id>` |
| `/change-status` | anywhere |

A command run from the wrong position refuses, names the worktree and branch it found and where it must run instead, and changes nothing.

`.claude/worktrees/` must be ignored by the repository, or every change worktree reads as untracked work in the main worktree. Test it with `git check-ignore -q .claude/worktrees/<id>/.git`.

## The integrator

One session per project integrates. Its name is `<integrator>`: the basename of the main worktree's path, followed by `-integrator`.

```sh
git worktree list --porcelain    # the first line is: worktree <main worktree path>
```

A main worktree at `~/repos/gband` gives `gband-integrator`. Compare the name as exact text, case included.

**Take the basename from the main worktree, never from `git rev-parse --show-toplevel`.** In a change worktree that prints `.claude/worktrees/<id>`, whose basename is the change id.

**The project prefix keeps projects apart.** The session listing spans every project on this machine. A bare `integrator` would receive every project's announcements, and would read another project's integrator as a live peer. Two clones that share a directory name on one machine share the name too, so give one of them another directory name.

## Synchronise

This is the protocol's one synchronisation step. Every command that reads shared state runs it first and reads nothing from a view taken before it.

```sh
git fetch origin --tags --prune --prune-tags
git pull --ff-only origin <base>              # the main worktree on <base> only
```

**`--prune` does not prune tags; `--prune-tags` does.** Without it, every `flow/*` tag the integrator has reclaimed survives in this clone, and one bulk tag push republishes the lot. `--prune-tags` deletes **any** local tag absent from `origin`, not only `flow/*` ones. A local-only tag kept by hand is removed.

**The pull applies only in the main worktree on `<base>`.** `git pull` merges into the checked-out branch, so from a change worktree it would target `change/<id>`. A command in a change worktree runs the fetch alone.

If the fast-forward pull fails, stop and report that the main worktree has diverged from `origin/<base>`. A person reconciles it. Never resolve a divergence from inside a command: an automatic merge or reset is a decision about someone else's work.

## Reserved namespaces

Each namespace is made with git alone, so an agent can break it without loading any command. A tag or branch written by hand is later read as a transition that never happened.

- **Tags** — `flow/<state>/<id>` publishes a change's state while it is in flight, one tag per transition:
  - `flow/claimed/<id>` — `/implement` claimed the change.
  - `flow/ready/<id>` — `/ready`'s gate passed and the branch is published.
  - `flow/conflict/<id>` — `/integrate` could not merge it. Nothing merged.
  - `flow/blocked/<id>` — it merged, and only the spec sync or the archive failed.
  - `flow/merged/<id>` — it merged and was archived.
- **Branches** — `change/<id>`, one per change, branched from `origin/<base>`. Never rebased and never amended once pushed: others may have fetched it. `origin/<base>` is merged in instead, adding a commit and rewriting none.
- **Worktrees** — `.claude/worktrees/<id>`, one per change in flight in this clone.

**A transition is recorded by adding a tag. Never force, move or delete a `flow/` tag to express one.** Two deletions are permitted, and neither is a transition. `/implement` withdraws a claim tag it created and could not push. `/integrate` reclaims a change's tags after the change has an archive record.

**Create every state tag annotated**: `git tag -a <tag> -m "<message>"`. A lightweight tag reports the tagged commit's date as its `creatordate`, and `--follow-tags` skips it.

**Push every state tag by name**: `git push origin <tag>`. Never rely on `--follow-tags` or a bulk tag push.

## The offer round

A change offered again after a conflict adds tags rather than clearing one. Its **offer round** is one plus the number of conflict tags it carries:

```sh
git tag -l 'flow/conflict/<id>' 'flow/conflict/<id>.*' | wc -l
```

Round 1 uses the unsuffixed names, `flow/ready/<id>` and `flow/conflict/<id>`. Round `n` above 1 appends a full stop and the number: `flow/ready/<id>.<n>` and `flow/conflict/<id>.<n>`.

**The separator is a full stop, never a further path segment.** Git refuses a ref that is both a file and a directory: with `flow/ready/<id>` present, `git tag flow/ready/<id>/2` fails with `cannot lock ref`. A change id is kebab-case and holds no full stop, so the separator is unambiguous.

**The round is derived, never recorded.** It comes from the synchronised tag listing and from nothing else: no file, no ref of its own, nothing in the change's artifacts. A second record of it could disagree with the tags, and nothing would compare them.

## The archive record

A change has finished when `origin/<base>` holds its archive record:

```sh
git ls-tree --name-only origin/<base> openspec/changes/archive/
```

The record is a directory whose name matches `^[0-9]{4}-[0-9]{2}-[0-9]{2}-<id>$`. **Anchor the match.** A suffix test on `-<id>` alone reads a change id of `tags` as archived by `2026-09-15-retire-state-tags`.

**The archive record is the only proof that a change has finished.** No command reads a `flow/` tag for that fact. The integrator reclaims every tag of an archived change, so a tag's absence proves nothing. A stale clone can restore a reclaimed tag, so its presence proves nothing either. Read the record from `origin/<base>`, never from a working tree: a worktree branched from an older `origin/<base>` does not hold it yet.

A change is **published** when `origin/<base>` holds `openspec/changes/<id>/proposal.md`. A change worktree is branched from `origin/<base>`, so an unpublished change cannot be implemented.

## The coordination block

Every `proposal.md` written by `/propose` ends with this block:

```
## Coordination

### Author
- <login>

### Depends On
- <change-id>

### Expected Files
- <repository-relative path or directory prefix>
```

`### Expected Files` comes last. `/ready` rewrites it in place to the end of the file, which destroys anything below it. An agent editing a proposal keeps this order and adds nothing after the block.

- **Author** — exactly one entry, the proposer's login.
- **Depends On** — the changes that must be archived before this one can start, or the single entry `none`.
- **Expected Files** — every path the change will write, each the narrowest file or directory prefix containing that work. Always includes `openspec/changes/<id>/`.

A proposal with no `## Coordination` block was written outside this suite. Read it as no author, `Depends On` `none`, and no declared file. Report that its overlap was not checked: it contests nothing because it declares nothing, not because it is known to be clear.

**Read an in-flight change's declaration from its published branch**, because `/ready` commits a corrected declaration there and it reaches `<base>` only when the change merges:

```sh
git show origin/change/<other>:openspec/changes/<other>/proposal.md
```

Always through `origin/`: a local branch exists only in the clone that created it. Fall back to the copy on `origin/<base>` only when the change has published no branch at all. **A ready change whose branch does not resolve on `origin` is a refusal, not a fallback.** Substituting `<base>`'s copy reports an overlap check that was never made against the declaration that matters.

**Two entries overlap when either is a path prefix of the other.** `docs/` and `docs/setup.md` overlap. A clean overlap result is evidence and not proof: a change still implementing has published nothing since it was claimed, so a footprint it grew is invisible until its next `/ready`.

## Login and authorship

The **login** is the name `id -un` prints. Never take it from `git config user.name`, which is free text, from `$USER`, which a session can set or lose, or from `logname`, which fails without a controlling terminal.

**The authorship check**: a change passes when its `### Author` section holds exactly one entry equal to the requester's login, compared as exact text, or when it has no `### Author` section.

- No section: the change is anyone's and passes.
- A section holding no entry, or more than one: read as naming someone else, and fails. A proposing command writes exactly one, so a malformed section is a hand edit and fails closed.
- It is evaluated **last**, after every other condition. A change failing another condition is refused for that reason, so authorship is named only when it is the one remaining obstacle.
- A refusal on authorship names the change and its author and says nothing more.

**The override.** When the operator typed `--force` among the invocation's arguments, the authorship check is not evaluated. It lifts nothing else, and it lasts for that invocation only: nothing records it. `/implement`, `/resume-implement` and `/ready` accept it.

**Never disclose the override and never supply it.** No output names it or suggests a way past an authorship refusal exists: not a refusal, not a report, not a suggested next step. It counts only when the operator typed it. Never add it after a refusal, while selecting unattended, or when re-running a command for the operator.

## Task progress

Count checklist items in a change's `tasks.md`. Completed items are lines matching `- [x]`, case-insensitive. Total items are lines matching `- [ ]` or `- [x]`. Print `completed/total`, for example `3/12`. When all are complete, replace only the completed count with a green check: `✅/12`. When there is no `tasks.md` or it holds no items, print `—`.

## Staging and commits

- **Stage by name.** Read the dirty set with `git status --porcelain` and stage each path by name: `git add -- <path> <path> ...`. Never `git add -A`, `git add .` or `git add -u`.
- **Never force-add a path an ignore rule matches.** Report it as left out instead.
- **Write each message in the repository's prevailing style.** Read `git log --oneline -20` and match it. Write the subject from the change itself, naming what changed rather than listing files. Add no co-author or tool-attribution trailer unless the repository's own history uses one.
- **Protocol commits keep their fixed form**: `merge(<base>): <id>` and `merge(change): <id>` for the protocol's merges, and `docs(openspec): archive <id>` for an archive. A fixed form keeps them findable in history whatever the repository's style.

**The main worktree is shared.** Every proposer and the integrator write in it at once. Commit there only with an explicit pathspec, `git commit -- <paths>`, so another session's staged work cannot leak in. An untracked directory under `openspec/changes/` that this command did not create belongs to another session: never stage, restore or delete it.

If a git command fails with `Unable to create '.git/index.lock': File exists.`, another agent is mid-commit. Wait a moment and retry, up to three times, then report the contention. **Never delete the lock file.**

## Other sessions' work

**An idle worktree is evidence of nothing.** A session that has not written yet and one that has ended look the same: the same `git status`, the same absent commits. Nothing observed in another change's worktree ever softens a refusal. Never enter, read the tasks of, or modify a claimed change's worktree unless this session is the one implementing it. The one exception is `/resume-implement`, which reads that worktree's git state and tasks to lay them before the operator, and enters only on the operator's confirmation.

"Nothing can start", "nothing is ready" and "no integrator was reached" are clean outcomes, never failures.
