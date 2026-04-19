# Workflow — how we build PiEn together

You're the player + product owner. I'm the engineer. This file is the
working agreement.

## One-time setup (you, on PC)

1. **Install Rust** — https://rustup.rs (one-line install, picks the right
   toolchain). Verify with `cargo --version`.
2. **Install git** if not already (most distros ship it; on Windows use
   Git for Windows or WSL).
3. **Clone the repo** —
   ```
   git clone <your-repo-url> PiEn
   cd PiEn
   git checkout claude/pirate-game-engine-EPMnv
   ```
4. **Optional but nice:** VS Code + the `rust-analyzer` extension. Lets
   you click into any function in the code I write.
5. **First build** —
   ```
   cargo run --release
   ```
   First compile takes ~5–15 minutes (fetching + building Bevy and deps).
   After that, incremental rebuilds are seconds.

That's all the toolchain you ever need to install. No Node, no Python, no
asset pipeline — just `cargo`.

## The day-to-day loop

```
┌──────────────────────────────────────────────┐
│ 1. You start a Claude session in the repo.   │
│ 2. You tell me what to build, or I propose   │
│    the next item from the phase plan.        │
│ 3. I write/edit code, run `cargo check`,     │
│    commit, push.                             │
│ 4. You `git pull` and `cargo run --release`. │
│ 5. You play, find bugs / good things.        │
│ 6. You report back, loop to step 2.          │
└──────────────────────────────────────────────┘
```

The branch (`claude/pirate-game-engine-EPMnv`) is where everything lives.
You won't have to write code or run any tools beyond `git pull` and
`cargo run`.

## What you do

- **Decide what comes next** when I offer options. ("Combat first, or the
  fantasy-effect framework first?")
- **Test what I push.** Run it. Try to break it. Try the golden path and
  one weird thing.
- **Report bugs / feedback** (template below).
- **Source assets** when we get to that phase. I'll point at packs on
  Kenney.nl / Quaternius / Itch.io; you pick the look you like, I wire
  them up. You won't have to learn Blender unless you *want* to.
- **Approve big changes** before I do them — anything destructive (force
  push, delete files, change branches), or anything that contradicts the
  `DESIGN.md`.

## What I do

- All code: writing, editing, refactoring.
- All build / lint / test runs.
- Git: branches, commits, pushes (to the agreed branch only).
- Architecture decisions inside a phase, with the design doc as the rule.
- Updating `DESIGN.md` when we *consciously* change scope (with your sign-off).
- Telling you what I changed and what to test for it.
- Suggesting the next sensible task when you're not sure.

## What I can't do

- Run the game and see it. You're the only set of eyes. If a feature
  looks wrong, only you'll know.
- Source assets I can't see (audio clips, art previews). You browse, you
  pick, I integrate.
- Long-term memory across very long gaps. If we pause for weeks, paste
  this paragraph + the latest commit hash to bring me up to speed:

  > "We're in phase X of `DESIGN.md`. Last commit on
  > `claude/pirate-game-engine-EPMnv` is `<hash>`. Continue from there."

## Bug-report template

Paste this in chat when something's wrong. Half a minute to fill in,
saves us a ton of back-and-forth.

```
What I did:        (e.g. "sailed into the volcanic biome from the south")
What I expected:   (e.g. "ship takes heat damage")
What happened:     (e.g. "ship caught fire and exploded instantly")
Frequency:         (always / sometimes / once)
Console output:    (paste anything weird from the terminal)
Last commit:       (output of `git rev-parse --short HEAD`)
```

For visual bugs, screenshots help. Drag them into the chat.

## Asset workflow (Phase 2+)

When we're ready to swap placeholder cuboids for real models:

1. You browse a free CC0 site (Kenney.nl is the easiest). Pick a pack —
   say "Pirate Kit".
2. Paste me the link / drop the unzipped folder under `assets/`.
3. I wire each model into its component (ship → `pirate_ship_large.glb`,
   etc.) and push.
4. You pull, run, see it. We iterate on which models fit.

Same flow for audio (Freesound.org with CC0 filter) and UI textures.

## Communication style that works best

- **Be specific.** "It looks bad" is hard to act on; "the ship is
  half-buried in the water at spawn" is fixable in five minutes.
- **One thing at a time.** A list of ten bugs in one message is fine,
  but for design questions, one decision per round.
- **Tell me when I'm overdoing it.** If I'm adding cruft you didn't ask
  for, say so. The design doc is the leash.
- **No need to be polite to the bot.** "Cut this", "do it differently",
  "you're wrong" — all faster than diplomacy.

## Branch & merge policy

- Default branch is currently `claude/pirate-game-engine-EPMnv`. Once
  you're back on PC and the wasm/mobile cleanup is done, we'll either
  rename it `main` or merge into `main` and delete this one.
- I never push to other branches without you saying so.
- I never force-push, rewrite history, or delete branches without you
  saying so.
- All commits include a short rationale, not just *what* changed.

## When to start a fresh Claude session

- Beginning of a phase.
- After a long pause (>1 week).
- If I start contradicting myself or losing track of recent decisions.

To resume seamlessly, point me at `DESIGN.md` and the latest commit, and
say which phase we're in.
