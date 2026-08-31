# Parkforge design

> Status: The document describes only the target design for Parkforge; Implementation details may change

## Core concepts

### Project
A Parkforge project consists of configurations and sources for a mod. 
The project can support multiple game IDs, mainly to target different
regions or revisions of the same game.

`project.toml` identifies the project, its supported game revisions and defines metadata.

### Game tree

Parkforge handles the games in a form of a directory structure rooted
at the game partition. Paths within extracted archives remain usable as
paths.

For example:

```text
DATA/files/Field/Ar01Zn01Dat.dac/Gimmick/GkDatArc.dan/script.fsb
```

An archive is represented as a directory that retains the original name and extension of the archive:

```text
Ar01Zn01Dat.dac/
Ar01Zn01Dat.dac/Gimmick/GkDatArc.dan/
```

The same paths are used under `original/`, `src/`, and `build/`.

### Original baseline

`original/<game-id>/` contains the extracted original for one game
revision. It includes the game tree and the metadata Parkforge needs to
reconstruct archives and the game partition.

The original baseline is never modified during a build.

### Sources

`src/` is the source layer and contains specific source files of the mod. 
The content structure mirrors the game tree.

Sources can be placed in two layers:

```text
src/
├── shared/
└── <game-id>/
```

- `src/shared/` contains sources that can be used by all revisions.
- `src/<game-id>/` contains sources and metadata specific to one revision.

Revision-dependent addresses, symbols, and other metadata belong
under `src/<game-id>/`. Shared source logic can be placed under `src/shared/`.

### New files and patches
Loose source files are creating new files at their corresponding path.
It must not replace files from the original.

```text
src/shared/DATA/files/example.fsc
    -> build/<game-id>/DATA/files/example.fsb
```

A directory named `<target>.patch/` modifies an existing file from the original
baseline:

```text
src/<game-id>/DATA/files/example.fsb.patch/
    -> build/<game-id>/DATA/files/example.fsb
```

A patch directory can exist at the same path under both `src/shared/`
and `src/<game-id>/`. This allows shared patch logic to be combined with the
metadata required by a specific revision.

### Custom symbol metadata

> TODO: Define custom symbol metadata output for patched files, at least
> FSC/FSB and DOL patches, and how those symbols can be exposed by distribution
> packages in a usable way.

### Generated output

`build/<game-id>/` contains the complete modified game tree for one
revision. It is produced from the immutable original and the
project sources.

`dist/` contains output, including rebuilt development ISOs and
distributable patch packages.

## Project structure

```text
project.toml
original/
└── <game-id>/             # Immutable extracted baseline
src/
├── shared/                # Sources shared across revisions
└── <game-id>/             # Revision-specific sources and metadata
build/
└── <game-id>/             # Complete generated logical game tree
dist/                         # Rebuilt ISOs and distributable packages
```

## Workflows

### Project setup

A project starts with a `project.toml`, the standard project directories, and at
least one configured game ID.

### ISO extraction

Extraction phase identifies the game revision and extracts its DATA partition into
`original/<game-id>/`.

Nested `.dan` archives are uncompressed U8 archive directories.
Nested `.dac` archives are NLZSS11-compressed U8 archive
directories. Their file names and extensions remain part of the path.

The extraction manifest records:

- The original files and additional metadata, e.g. hashes.
- Archive Directories
- Extracted archive formats and compression.
- Hierarchy of nested archives.
- Virtual paths within the game tree.

### Source overlay creation

Source overlay creation produces the directory structure of the extracted
original under `src/<game-id>/`. It creates the path in which
revision-specific sources and patch directories can be placed.

### Source overlay merge

The source overlay merge is an internal step performed automatically whenever 
Parkforge builds a selected game ID. This step merges
`src/shared/` with `src/<game-id>/` into one source tree for that
build.

The merge follows these rules:

1. `src/shared/` is the shared base layer.
2. `src/<game-id>/` is the revision layer.
3. A revision file replaces a shared file at the same relative source
   path.
4. Matching patch directories merge so shared patch logic can use
   revision metadata.
5. Shared sources are processed and validated for every selected game
   revision.

### Build

A build starts from the immutable `original/<game-id>/` tree. As part of the
build, Parkforge performs the source overlay merge and uses
the resulting source tree for compilation and patching.

```text
original/<game-id>/
        +
source overlay merge
        |
        v
compile new files and apply patches
        |
        v
build/<game-id>/
```

Parkforge copies the original tree into a staged build, compiles new
source files, and applies patch directories to their targets. Archives remain
expanded as directories at their expanded paths in `build/<game-id>/`. The
completed result is built to `build/<game-id>/`.

The build fails if a loose source would replace an original file or if multiple
sources attempt to own the same output target.

### Development ISO rebuild

Parkforge repacks the expanded archive directories from the complete
`build/<game-id>/` tree, rebuilds the game partition, and produces a runnable development
ISO under `dist/`.

## Integrated tooling

Parkforge provides integrated support for
specific source and game formats. Implementations should be decoupled from
Parkforge usage, keeping them extensible and replaceable.

### FSC compilation

A loose `.fsc` source defines a complete new script. Parkforge compiles it to an
`.fsb` at the corresponding path.

Complete `.fsc` sources are only valid for new files. Existing `.fsb` files are
modified through patch directories.

### FSC function patching

An existing `.fsb` is modified through a matching `.fsb.patch/` directory. A
partial `.fsc` file defines the functions that replace functions in the
original script.

```text
src/<game-id>/DATA/files/example.fsb.patch/
├── example.fsc
└── symbols.toml
```

`symbols.toml` provides the addresses of
the functions being replaced.

Patch logic that is identical across revisions can be stored under
`src/shared/`:

```text
src/shared/DATA/files/example.fsb.patch/example.fsc
```

The matching revision-specific patch directory supplies `symbols.toml`:

```text
src/<game-id>/DATA/files/example.fsb.patch/symbols.toml
```

The source overlay merge combines these inputs before the build is applied.

### DOL patching

> TODO: Finalize DOL patching. It will most likely use a system similar
> to FSC/FSB patching and references the current Archipelago patcher.

### RLB editing

> TODO: Define RLB editing.

## File deletion

> TODO: Define file deletion, most likely through delete markers.

## Distribution packages

> TODO: Define distribution packages.
