# Parkforge

Parkforge is an opinionated source-to-distribution toolchain for PokéPark Wii
mods. It provides built-in support for relevant game formats and compilers.

> Note: Parkforge is currently a proof of concept for validating core ideas and is not yet in any form a stable implementation

## Currently implemented

- Creating and validating Parkforge projects.
- Registering multiple game IDs, primarily for different game revisions and
  regions.
- Detecting the game ID of an ISO and extracting its DATA partition into the
  immutable `original/<game-id>/` baseline.
- Extracting nested `.dan` archives as uncompressed U8 archives and `.dac`
  archives as NLZSS11-compressed U8 archives.
- Recording archive relationships, virtual paths, compression, and original
  file hashes in `manifest.json`.
- Creating an empty `src/<game-id>/` tree from the extracted manifest's
  directory structure.
- Atomically building a complete logical `build/<game-id>/` tree from the
  original baseline and rule-configured project sources.
- Copying raw source files through an explicit `copy` build rule.
- Compiling `.fsc` sources into `.fsb` scripts with the integrated FSC compiler.
- Detecting unmatched sources, ambiguous rules, and multiple sources that
  produce the same build target.
- Repacking nested archives and rebuilding the DATA partition as
  `dist/<game-id>.iso`.

## Current project layout

The proof of concept currently uses this layout:

```text
project.toml                     # Project and game configuration
original/<game-id>/              # Extracted immutable game baseline
src/<game-id>/                   # FSC and raw source inputs
build/<game-id>/                 # Complete generated logical game tree
dist/<game-id>.iso               # Rebuilt development ISO
```

## TODOs

- Resolving `src/shared/` together with the selected `src/<game-id>/` overlay.
- Integrating DOL patching functionality from the Archipelago patcher.
- Integrating RLB editing functionality.
- Defining the `.pfmod` patchfile and its metadata.
- Defining delete markers in src.
- Defining added-file, edited-file, and deleted-file package operations.
- Creating one self-contained `.pfmod` package per supported game revision.
- Applying a `.pfmod` without access to project sources or developer toolchains.
- Verifying that applying a generated package reproduces the source build
  exactly.
- Optional `.fsa` patching for fsb scripts.
- Additional game-format integrations.

## Design and example

- [Project design](docs/DESIGN.md)
- [PokéPark Wii demo project](examples/pokepark-wii-demo/README.md)

## Rules

- `original/` is never modified after extraction.
