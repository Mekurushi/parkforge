# DRAFT: Parkforge

Parkforge is an opinionated source-to-distribution toolchain for PokéPark mods,
with built-in integrations for tools such as the FSC compiler.

## Stages

1. Set up the project layout.
2. Extract the original ISO as an immutable baseline.
3. Build a complete patched logical game tree from the project sources.
4. Create a self-contained distributable patch package that end users can
   apply without access to the developer toolchains.
5. Optionally verify that Parkforge can consume the patch package and reproduce
   an ISO from the extracted original.

## Project directories

```text
project.toml
original/                 # Immutable extracted game baselines
src/                      # Authoritative mod sources
├── shared/               # Sources shared between game revisions
└── <game-id>/            # Game-specific source overlay
build/                    # Complete patched logical game trees
dist/                     # Distributable patch packages and rebuilt ISOs
```

The responsibilities are:

- `original/` is extracted once and never modified by a build.
- `src/` contains the human-maintained sources that define the mod.
- `build/` contains the complete result of applying the project sources to an
  original game revision.
- `dist/` contains consumer-facing output, such as a `.pfmod` package or a
  rebuilt ISO.

## Project configuration

`project.toml` describes the project itself and how its sources are built. This
includes:

- Project name and version.
- The game IDs and game revisions supported by the project.
- Which compiler rules are enabled and which source files they match.
- How source extensions are mapped to their final target files, e.g. `.fsc`
  sources compiled into `.fsb` files.
- General game-specific configuration that is needed by the build.

Parkforge provides integrations for relevant tools, such as the FSC compiler
and an RLB editor, by default. The tools themselves can remain separate
projects; Parkforge only needs to contain the compiler rules and integrations
required to use them. This allows a tool to be updated or replaced without
moving its entire implementation into Parkforge, while normal projects do not
have to integrate common tools manually.

A tool is only required when a project enables its compiler rule or contains
sources that need it.

### DOL configuration

DOL-specific configuration should stay in `project.toml`, scoped to the game
revision it describes:

```toml
[[game]]
game_id = "R8AJ01"

[game.dol]
target = "DATA/sys/main.dol"
base_sha256 = "ed906e7dfbef9762b4281a9404ce8275f0353386866c75003603c3bd6c5f126a"
free_space_address = 0x80486BA0
sda_base = 0x80579440
patches = ["custom_funcs", "archipelago"]

[game.dol.arena_start]
high = 0x8020DDE0
low = 0x8020DDEC

[[game.dol.arena_end]]
high = 0x8020DDDC
low = 0x8020DDE4

[[game.dol.arena_end]]
high = 0x802049B8
low = 0x802049BC

[game.dol.stack_end]
high = 0x80004234
low = 0x80004238
```

- `target` identifies the executable in the logical game tree.
- `base_sha256` identifies the exact immutable DOL the addresses and layout
  were derived from.
- `free_space_address` is the runtime address where the linked custom payload
  starts.
- `sda_base` is the revision-specific small-data-area base used when resolving
  small-data references in assembly.
- `patches` explicitly selects `patches/<name>.asm` source units from the
  resolved DOL source tree and defines their processing order.
- `arena_start`, `arena_end`, and `stack_end` identify the instruction pairs
  Parkforge owns when moving the game's memory boundaries past the custom
  payload and reserved memory.

## Extraction and virtual paths

The extraction phase turns the original ISO into the logical game tree used by
the rest of Parkforge. Paths inside this tree are virtual paths relative to the
game root, e.g.:

```text
DATA/files/Field/Example.dac/Gimmick/Example.dan/script.fsb
```

Supported archives are extracted and represented as directories
that keep the archive file name and extension:

```text
Example.dac
    -> Example.dac/

Example.dac/Gimmick/Example.dan
    -> Example.dac/Gimmick/Example.dan/
```

The extraction manifest records which directories represent archives, their
format and compression, how nested archives relate to each other, and the
original files and hashes.

`original/`, `src/`, and `build/` use the same virtual path layout. This allows
sources to target files inside nested archives without each compiler having to
understand the archive formats. When rebuilding an ISO, Parkforge repacks the
nested archives and then reconstructs the game filesystem from the complete
logical build tree.

## Shared and game-specific source overlays

A project can technically contain game IDs for completely different games, but
that is not the primary use case. Multiple game IDs are mainly intended to
support different revisions of the same game.

The `shared` directory mirrors the same logical game tree as each game-specific
directory:

```text
src/
├── shared/
│   └── DATA/
│       ├── files/
│       └── sys/
├── R8AJ01/
│   └── DATA/
│       ├── files/
│       └── sys/
└── R8AE01/
    └── DATA/
        ├── files/
        └── sys/
```

For a selected game ID, Parkforge resolves one logical source tree by overlaying
the game-specific directory on top of `shared`.

The overlay rules are (WIP):

1. `shared` is the lower-priority layer.
2. `src/<game-id>/` is the higher-priority layer.
3. Directories merge recursively.
4. A game-specific file replaces a shared file at the exact same relative
   source path.
5. Different source paths that produce the same build target are an error.
6. Shared sources are still compiled and validated separately for every game
   revision.

The `shared` tree is intended for semantic sources and independent components,
such as a custom-functions Rust project, whose logic is not tied to a specific
game revision. Sources that patch existing files, contain assembly patches, or
operate on specific addresses should remain game-specific.

Even if an assembly patch only refers to original symbols and could
theoretically be shared, proving that it is revision-independent would be
fragile and add unnecessary complexity.

Good shared candidates include:

- Complete FSC scripts used at the same virtual path.
- Shared Rust functions.
- Assembly macros and linker scripts.

Game-specific inputs include:

- Original executable symbol addresses.
- Free-space, section, arena, and stack layouts.
- Assembly patches.
- Raw revision-specific addresses and hooks.
- Region-specific scripts, assets, and localization.

When uncertain, an input should remain game-specific. Sharing should remove
known duplication rather than assert compatibility that has not been verified.

## Source and target conventions

A source that produces a complete target is placed at the corresponding
logical location:

```text
src/shared/DATA/files/example.fsc
    -> build/<game-id>/DATA/files/example.fsb
```

A source that transforms an existing target uses a sidecar directory:

```text
src/<game-id>/DATA/files/example.fsb.patch/
    -> build/<game-id>/DATA/files/example.fsb

src/<game-id>/DATA/sys/main.dol.patch/
    -> build/<game-id>/DATA/sys/main.dol
```

Each build target has one owning transformation. Two unrelated rules may not
produce or patch the same target. A target-specific transformation may combine
multiple internal source units, such as several assembly files contributing to
one `main.dol`.

## FSC script sources and patching

FSC scripts are generally small enough that a complete `.fsc` source should be
preferred over patching an existing compiled script. The complete source is
easier to understand and maintain than a collection of edits at specific
addresses. Once rewritten in full, scripts can also be reused freely.

```text
src/shared/DATA/files/example.fsc
    -> build/<game-id>/DATA/files/example.fsb
```

In the future, Parkforge may also support `.fsa` sources for cases where a
complete rewrite is impractical. These sources would patch an existing `.fsb`,
analogous to applying `.asm` sources to a DOL:

```text
src/<game-id>/DATA/files/example.fsb.patch/
└── example.fsa
    -> build/<game-id>/DATA/files/example.fsb
```

Because `.fsa` patches depend on the layout or addresses of an existing
compiled script, they are revision-specific sources and should not be placed in
`src/shared/`. The sidecar transformation owns the resulting `.fsb` and is
responsible for validating and applying all of its `.fsa` edits together.

## DOL patching sources

DOL patching is part of the source build. Its assembly, Rust, linker inputs, and
symbol definitions belong in `src/`. The revision-specific build contract
belongs in the matching `[game.dol]` section of `project.toml`.

An example layout is:

```text
src/
├── shared/
│   └── DATA/sys/main.dol.patch/
│       ├── asm_macros.asm
│       ├── linker.ld
│       └── custom-functions/
│           ├── Cargo.toml
│           ├── Cargo.lock
│           ├── rust-toolchain.toml
│           └── src/
│
└── R8AJ01/
    └── DATA/sys/main.dol.patch/
        ├── original_symbols.yaml
        └── patches/
            ├── custom_funcs.asm
            └── archipelago.asm
```

The resolved R8AJ01 DOL source combines the shared implementation with the
R8AJ01 symbols and region-specific assembly. Its `[game.dol]` configuration
then supplies the baseline, layout, and selected patch sources needed to build
that resolved source for R8AJ01.

The DOL transformation:

1. Verifies the immutable baseline DOL.
2. Compiles the optional shared Rust crate.
3. Assembles the game-specific patch sources using the resolved shared support
   files.
4. Links them using the game-specific symbols and layout.
5. Plans and validates all writes for `main.dol` together.
6. Applies the result only to the staged build tree.

Unlike the Archipelago patcher, where the Rust project creates and commits diff
files, Parkforge only uses such diffs as temporary intermediate data when
constructing the patched `main.dol`. Consumer-facing differences should be
distributed through the self-contained patch package in `dist/`.

## Build pipeline

The logical build pipeline is:

```text
original/<game-id>
        +
resolve(src/shared, src/<game-id>)
        |
        v
compile complete-file sources
and run target-specific transformations
        |
        v
atomically commit build/<game-id>
```

Source changes should be reflected by the next `parkforge build`. Tools are
opt-in through the sources and project configuration that require them. A
project using DOL Rust sources may require the configured Rust and PowerPC
toolchains, while a project without those sources does not.

## Distribution packages

The generic patch boundary belongs after the build:

```text
original/<game-id> + build/<game-id>
        |
        v
dist/<mod>-<game-id>.pfmod
```

Packaging compares the immutable original tree with the completed build tree
and records only their logical differences. A package may represent changes as:

- Added files (TODO: determine an appropriate solution regarding copyright).
- Deleted files (TODO: define delete markers).
- Edited files (TODO: define a unified approach, possibly using only xdelta3
  patches for simplicity).

The `.pfmod` package contains the metadata needed by a dedicated patcher,
including its game ID, expected baseline hashes, logical target paths, payload
integrity information, and the operations required to reproduce the build. End
users apply this package without access to `src/` or its toolchains.

Parkforge should ideally also be able to apply a `.pfmod` to a project's clean
`original/` tree and rebuild an ISO. This provides a reference implementation
of the package format and allows mod authors to verify:

```text
source build result == packaged patch application result
```
