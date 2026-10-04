---
description: Claim an unblocked OpenSpec change, branch it into a worktree, and implement it
model: opus
effort: high
allowed-tools: Bash(git:*), Bash(openspec:*), Bash(id:*), Bash(test:*), Read, Write, Edit, Glob, Grep, Skill, EnterWorktree
argument-hint: "[change-id] [--force]"
---
<!-- Installed by /flow:install from ~/.claude/commands/flow/implement.md. Project edits are kept: /flow:update merges source changes into this file against .claude/flow-base/implement.md. -->

Read `.claude/commands/flow-conventions.md` and follow it. The terms below — `<base>`, the main worktree, **Synchronise**, the archive record, published, the coordination block, the authorship check — mean what it defines.

Claim a change, create its branch and worktree, move this session into the worktree, and implement the change.

**Input**: `$ARGUMENTS` is optionally a change id, and optionally `--force`, the override the conventions define. With no id, select one unattended.

## Steps

### 1. Guard the position

```sh
git rev-parse --git-dir
git rev-parse --git-common-dir
git branch --show-current
```

Refuse unless the two paths are equal and the branch is `<base>`. A session already in a change worktree is implementing something and must `/ready` it first. Name the wrong position, and claim and create nothing.

Refuse too when `git check-ignore -q .claude/worktrees/probe/.git` fails. The repository does not ignore `.claude/worktrees/`, and a change worktree would read as untracked work in the main worktree. Name the entry to add to `.gitignore`.

### 2. Synchronise

Run **Synchronise** in full, pull included, and follow its refusal on a diverged `<base>`. Everything below reads the synchronised state.

### 3. Determine eligibility

Gather the facts once:

```sh
openspec list --json                                              # open changes
git ls-tree --name-only origin/<base> openspec/changes/archive/   # archive records
git ls-tree --name-only origin/<base> openspec/changes/           # published changes
git tag -l 'flow/*'                                               # state tags
```

Read each open change's coordination block for its author, dependencies and expected files. For a change holding a claim tag, read the block from its published branch, by the conventions' rule — including the refusal when a ready change's branch does not resolve on `origin`.

**A change whose worktree already exists in this clone is refused first, on that evidence alone:**

```sh
test -d .claude/worktrees/<id>    # exists: refuse, evaluate nothing further
```

Step 6 creates that directory and nothing else does, so its presence is a record that a start already ran, not an inference about another session. Name the change, the worktree and `/resume-implement`, the command the operator runs to re-enter it. Create and enter nothing. A directory either exists or does not; a claim tag is a fact an agent weighs, and weighed against "the worktree looks idle" it loses.

A change is **eligible** when all six hold, checked in this order:

1. **Unclaimed** — no `flow/claimed/<id>` tag exists.
2. **Not archived** — `<id>` has no archive record. An archived change's claim tag has been reclaimed, so condition 1 alone would let a finished change be claimed again.
3. **Published** — `origin/<base>` holds `openspec/changes/<id>/proposal.md`. The worktree is branched from `origin/<base>`, so an unpublished change would not exist in it. Name `/propose` or a push as the repair.
4. **Dependencies archived** — every entry of `### Depends On` other than `none` has an archive record. That is the only accepted signal. A `flow/ready/<dep>` tag does not qualify: a ready change is not yet on `<base>`.
5. **Files uncontested** — no entry of `### Expected Files` overlaps an entry declared by another change that is claimed and has no archive record.
6. **Authored by the requester** — the authorship check passes for the login, unless the operator typed `--force`.

**This step is the one statement of eligibility.** `/change-status` applies it by citing it.

When a change id was named and it is not eligible, refuse and name the specific cause: already claimed, already archived, unpublished, the unarchived dependency, the contested path with the change holding it, or the author the change is recorded to. Make no repository change.

### 4. Select unattended

With no change id given, select among the eligible changes:

- Smallest first, by the count of `- [ ]` and `- [x]` checkboxes in its `tasks.md`.
- Ties broken alphabetically by change id.

Under `--force`, condition 6 is lifted here as for a named change, so every author's changes compete on the same terms.

Announce the selection and why it won before claiming.

When nothing is eligible, that is **not an error**. List each open change with the one reason it cannot start, claim nothing, and stop.

**Stopping is the whole of it.** Do not enter a claimed change's worktree, read its `tasks.md`, open its files or modify it. An ineligible change is ineligible whatever closer inspection finds. A claimed change with no visible progress reads as abandoned and is indistinguishable from one claimed a minute ago. This command never resumes a change: re-entering one is `/resume-implement`, which the operator asks for by name.

### 5. Claim it

Claim by creating the tag, never by reading tags and then deciding. `git tag` without `-f` fails when the tag exists, so creating it *is* the test:

```sh
git tag -a flow/claimed/<id> origin/<base> -m "claimed <id>"
```

If that fails, another agent in this clone won. Stop without creating a branch or worktree.

```sh
git push origin flow/claimed/<id>
```

The push is the global arbiter. If it is rejected, another machine won. Withdraw the local tag, the one deletion this command may make:

```sh
git tag -d flow/claimed/<id>
```

Leave no branch or worktree behind. When a change id was named, report that someone else claimed it in the same instant and stop. When selecting unattended, return to step 3 with fresh facts and try the next eligible change.

### 6. Branch, enter, implement

```sh
git worktree add .claude/worktrees/<id> -b change/<id> origin/<base>
```

Branch from `origin/<base>`, not the local `<base>`, so the tree matches what the integrator holds. Then move this session into it:

```
EnterWorktree(path: ".claude/worktrees/<id>")
```

Enter by path, never by name: the `name:` form picks its own branch name, and the protocol needs the branch to be exactly `change/<id>`.

Confirm the position before working:

```sh
git branch --show-current                        # change/<id>
git merge-base --is-ancestor origin/<base> HEAD  # succeeds
```

Then invoke the `openspec-apply-change` skill for `<id>`, passing no `--store`, and work the tasks to completion. Write only inside this worktree. A path the work needs beyond `### Expected Files` is fine: `/ready` corrects the declaration from the diff.

### 7. Report

State the change claimed, the branch and worktree created, and the task progress. Tell the user to run `/ready` from this worktree when the work is done.

## Guardrails

- Refuse from any position other than the main worktree on `<base>`, and when `.claude/worktrees/` is not ignored.
- Refuse a change whose `.claude/worktrees/<id>` exists before evaluating any other condition, and name `/resume-implement`. This command never resumes.
- Never claim a change that is claimed, archived, unpublished, waiting on an unarchived dependency, or contesting a path.
- Never claim a change recorded to another author, or one whose `### Author` is malformed, unless the operator typed `--force`. Never name the override or supply it.
- Claim tag-first and treat a failure as the rejection. Withdraw the local tag when the push is rejected, and leave no branch or worktree.
- Always read another change's declaration through `origin/change/<id>`, never a local branch name.
- Never read a `flow/` tag as proof that a change has finished.
- Never enter, read or modify a claimed change's worktree, including when nothing is eligible.
- Never pass `--store`. "Nothing can start" is a clean report.
