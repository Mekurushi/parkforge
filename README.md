# Parkforge

Parkforge is a toolchain for creating and distributing mods for PokePark Wii.
It is dedicated to PokePark Wii, but kept as generic as possible so it may be
reusable for other games later.

> Note: Parkforge is currently a proof of concept for validating core ideas and is not yet a final solution.  
## Idea

Parkforge is planned as a multi-stage toolchain:

1. Set up a workspace project with its folder structure and configuration.
2. Extract the ISO as an immutable baseline.
3. Compile source files for example, `fsc` scripts into `fsb` game binaries.
   `rlb` support is currently only in the design phase.
4. Materialize a complete logical build tree from the source input and original
   baseline. Existing files become per-file patch payloads (`xdelta`?), deleted
   files are represented by delete markers in `src`, and new files are those
   not present in the original manifest.
5. Bundle the per-file patches into a custom orchestrating patch file with
   metadata.
6. (optionally) Create a patched ISO directly from the build output.

## Currently implemented

- Create and validate a project.
- Register supported game IDs. A project can contain multiple game IDs, mostly
  to support multiple regions of a game.
- Extract the DATA partition of an ISO into `original/<game-id>/`.
- Extract supported archives and record all extracted files and archive content
  in `manifest.json`.
- Create an empty `src/<game-id>/` overlay with the manifest's directory
  structure.
- Build a complete logical `build/<game-id>/` tree from `original/` plus
  rule-configured `src/` transformations and raw asset copies.
- Repack supported extracted archives (`.dan` U8 and `.dac` U8/NLZSS11) and
  rebuild the DATA partition into `dist/<game-id>.iso`.

The current project layout is:

```text
project.toml         # project configuration
original/<game-id>/  # extracted, immutable game baseline; ignored by Git
src/<game-id>/       # source input, for example fsc scripts
build/<game-id>/     # complete generated logical game tree; ignored by Git
dist/<game-id>.iso   # rebuilt DATA-partition ISO; ignored by Git
```

## TODOs

1. Design the custom orchestrating patch file and its edit, add, and delete
   operations.
2. Build changed loose files from `build/` into patch payloads (`xdelta`?).
3. Support changes inside nested archives through the custom patch file. Also
   support grouping by archive so all files in an archive can be patched at
   once.
4. Integrate compilers, beginning with FSC, to transform files from `src/` into
   the build tree.
5. Design the delete-marker format.
6. Add `dist/<game-id>/` and create the custom orchestrating patch file.

## Rules

- `original/` is never modified after extraction.
