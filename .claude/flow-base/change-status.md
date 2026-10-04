---
description: Report every open OpenSpec change, or one author's and what they depend on, with its workflow state, and name what can start next
model: opus
effort: medium
allowed-tools: Bash(git:*), Bash(openspec:*), Bash(id:*), Bash(ls:*), Read, Glob, Grep
argument-hint: "[login]"
---
<!-- Installed by /flow:install from ~/.claude/commands/flow/change-status.md. Project edits are kept: /flow:update merges source changes into this file against .claude/flow-base/change-status.md. -->

Read `.claude/commands/flow-conventions.md` and follow it. The terms below — `<base>`, **Synchronise**, the offer round, the archive record, published, the coordination block, the login, task progress — mean what it defines.

Report every open change — or, given a login, that author's open changes and the open changes they depend on — with its author, dependencies, task progress and workflow state, and name the change that can start next.

**Input**: `$ARGUMENTS` is empty or one login. With none, every open change is shown. With one, the report is narrowed to that author, as step 3 defines.

This command is **read-only**. It creates no tag, branch, worktree or commit, and leaves the working tree byte for byte as it found it. It **never claims** a change: naming one as next is advice, and `/implement`'s claim is still the arbiter.

It **guards no position**. It writes nothing, so it is correct from every worktree and every branch, and gives the same answer from each. It is most useful from a change worktree, by an agent asking what else is in flight.

## Steps

### 1. Read the argument

Zero arguments or one. With more than one, refuse: name every argument after the first as unexpected, and report nothing.

One argument is a login and is never read as anything else — in particular never as the override, which this command never applies. A login no open change records is not refused: it is a question with an empty answer.

### 2. Refresh

Run the fetch half of **Synchronise**:

```sh
git fetch origin --tags --prune --prune-tags
```

State is published on the remote, so read the remote before reading the tags. Without the fetch, a change claimed on another machine reads as nobody having started it — the most damaging wrong answer this report can give.

**A failed fetch degrades; it does not fail.** Produce the report from the local refs anyway, and say in the output that the state may be stale and why, quoting the fetch error.

### 3. Read the open set

```sh
openspec list --json
```

The input is the open changes — the directories under `openspec/changes/`, excluding `archive/` — and nothing else.

**Never derive the set of changes from the tag namespace.** A change is reported because its planning directory is open. A tag a stale clone restores names an archived change, which is not open, so it can neither appear nor alter a reported state.

Read each open change's coordination block and `tasks.md` from published state, as step 5 sets out, not from the worktree this command runs in.

**With a login, narrow what is shown, never what is read.** Read every open change's declaration and state as without one. The shown set is:

- **The named author's changes** — those whose `### Author` holds exactly one entry equal to the login. A change with no `### Author` section never matches: it is anyone's, not this author's.
- **Their dependencies** — every open change a shown change depends on, directly or through another open dependency, whatever its author. It is what blocks the author's work. An archived dependency ends the walk and is shown as satisfied.

Nothing else is shown. Every fact eligibility reads stays unnarrowed, so another author's in-flight change still contests a shown change's path.

### 4. Derive each change's state

For each open change, look its own tags up **by name**:

```sh
git tag -l 'flow/claimed/<id>' \
  'flow/ready/<id>' 'flow/ready/<id>.*' \
  'flow/conflict/<id>' 'flow/conflict/<id>.*' \
  'flow/blocked/<id>'
```

Derive its offer round by the conventions' rule. Then give it exactly one state:

| tags present for `<id>` | state | meaning |
|---|---|---|
| none | `proposed` | nobody has started it |
| `claimed` | `implementing` | an agent holds it; its branch may not be pushed yet |
| `claimed`, this round's `ready` | `ready` | the gate passed; waiting on the integrator |
| `claimed`, a `conflict`, this round's `ready` absent | `conflicted` | nothing merged; waiting on its author's `/ready` |
| `blocked`, with anything else | `blocked` | merged onto `<base>`; only the sync or the archive failed |

`blocked` outranks everything, then `ready` or `conflicted`, which cannot both hold, then `implementing`, then `proposed`. `blocked` outranks `conflicted` because only `blocked` means the work is already on `<base>`: reporting the other sends someone to rebase work that has landed.

Mark a change that is open locally but has no `openspec/changes/<id>/proposal.md` on `origin/<base>` as **unpublished**, beside its state. Nobody else can see it, and `/implement` refuses it.

**Report the round for any change past round 1.**

**Give `conflicted` and `blocked` their own repairs, which are mutually wrong:**

- `conflicted` — nothing merged. Its author re-runs `/ready`, which merges `origin/<base>` into the branch, resolves the conflict in its worktree and publishes the next round's ready tag. Delete no tag.
- `blocked` — the work is already on `<base>`. A person fixes the change's delta specs on `<base>` and archives it by hand. Never rebase it and never re-offer it.

### 5. Report progress and dependencies

**Read `proposal.md` and `tasks.md` from the change's published source**: `origin/change/<id>` when that branch exists, `origin/<base>` otherwise, and the local working tree only for an unpublished change. Never the local copy of a published change: ticks in a change worktree exist nowhere else, so a report reading them would answer differently from each worktree. Unpublished ticks are work no other agent can see, and leaving them uncounted is correct.

Print task progress by the conventions' rule. Print each change's author as its `### Author` entry, `—` when there is no section, and the entries as written when there are none or several.

Draw the shown changes as a **tree** when at least one declares another shown change as a dependency, and as a flat table otherwise. When narrowed, mark every row shown only as a dependency with `(dependency)` after its id. Mark an archived dependency as satisfied.

### 6. Name what can start next

Apply **the eligibility rules in `/implement` step 3** and **the ordering in its step 4**, unchanged, reading `.claude/commands/implement.md`. Do **not** perform its step 5: nothing is claimed here. The requester is the login `id -un` prints, and the override is never applied.

Those rules are stated once, there. Never restate or paraphrase them here or in the output: a second copy goes stale and names a change `/implement` then refuses.

**With a login, the candidates are the named author's changes alone.** A row shown only as a dependency is never named as next. The requester is still `id -un`, whoever the login names: naming another login is a way to look, not a way to act as that person.

Report a clean overlap result as **evidence, not proof**, in the conventions' terms.

**"Nothing can start" is a normal outcome.** Complete the report, name no next change, and give one reason per candidate: already claimed, already archived, unpublished, waiting on a named unarchived dependency, contesting a named path with the change holding it, or recorded to a named other author.

### 7. Report

Print the table or tree, one row per shown change, with:

- the change id, plus `(dependency)` and `unpublished` where they apply
- its author, or `—`
- its state, using only the five names above
- its offer round, when past 1
- its task progress
- its declared dependencies

Then name the change that can start next, or state that nothing can and give each candidate's reason. Add the staleness note from step 2 when the fetch failed.

When narrowed, name the login. When it matches no open change, say that no open change is recorded to that login, print no rows and name no next change.

## Guardrails

- Never create, move, force or delete a `flow/` tag, including one that looks spent.
- Never claim a change, create a branch or worktree, record a commit, or enter a worktree. Never modify any file.
- Never refuse on position. Refuse more than one argument, and never refuse a login matching no open change.
- Never narrow what is read: claims, archive records and in-flight declarations are read across every open change.
- Never name a row shown only as a dependency as next.
- Never derive the set of reported changes from the tag namespace.
- Never restate `/implement`'s eligibility or ordering; cite it.
- Never offer the `conflicted` repair for a `blocked` change or the reverse, and never tell anyone to delete a tag.
- Never apply, name or suggest the override.
- Never pass `--store`. "Nothing can start" is a clean report.
