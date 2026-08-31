# Parkforge

> Parkforge is currently a design concept without a usable implementation. Its
> architecture and behavior may change during development

Parkforge is an opinionated source-to-distribution toolchain for PokéPark Wii
mods. It orchestrates extraction, source compilation, file patching, archive
repacking, and ISO rebuilding as one reproducible project workflow.

## How Parkforge works

A Parkforge project starts with an original game ISO and a source tree. 
Parkforge extracts the original game ISO, applies the project's
sources with its integrated tooling, and produces a complete build
tree from which distributable packages or development ISOs can be created.

```text
original ISO
    |
    v
immutable original/<game-id>/ baseline
    +
shared and revision-specific sources
    |
    v
generated build/<game-id>/ tree
    |
    +--> rebuilt development ISO
    `--> self-contained patch package
```

The source layout:

- `src/shared/` contains sources that are used by all revision in the project.
- `src/<game-id>/` contains revision-specific sources and patches.
- Source files placed directly in the source tree create new game files.
- `<target>.patch/` groups the sources and metadata needed to modify an existing game file.

Common formats and compilers for PokéPark are built-in and supported by Parkforge.

## Core Concepts

- `original/` is an extracted game and contains the original files in a suitable way.
- `src/` contains the mod source files.
- `build/` and `dist/` contain generated output.
- Distribution packages should be applicable without project sources or
  developer toolchains.

## More information

- Read the [project design](docs/DESIGN.md) for the intended architecture.
- Explore the [PokéPark Wii demo project](examples/pokepark-wii-demo/README.md)
  for a concrete project layout, shared sources, revision-specific file
  patching, and the planned CLI workflow.
