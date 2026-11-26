# Agent instructions

Read [`CLAUDE.md`](CLAUDE.md) first. It holds the commands, the branch
model (what goes on `freeknet` and what goes upstream as a topic-branch
PR), the rules the build enforces, and the lint decisions.

Before you finish a piece of work:

1. Run the gates in `CLAUDE.md` (rustfmt, both clippy runs, tests,
   rustdoc with `-D warnings`, `cargo deny`).
2. Take every Material 3 value from the Jetpack Compose material3 token
   files, and cite the token next to it.
3. Commit only the paths you changed, with a conventional subject and no
   tool attribution.

Preserve unrelated changes in a dirty worktree, and never revert another
contributor's edits.
