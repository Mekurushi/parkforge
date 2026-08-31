# PokePark Wii demo project

This is a source-only Parkforge project for the Japanese PokePark Wii release
(`R8AJ01`). It does not include any game data or generated output.

The source tree demonstrates three built-in FSC workflows:

- `src/shared/.../ParkforgeCustomGate.fsc` is a loose source file for a new,
  custom script. Parkforge compiles it into `ParkforgeCustomGate.fsb`, and its
  placement under `shared/` makes it available to every supported game
  revision.
- `src/R8AJ01/.../Gk0101Gate.fsb.patch/` modifies the existing
  `Gk0101Gate.fsb` for the Japanese release. The partial `Gk0101Gate.fsc`
  defines the functions to replace, while `symbols.toml` identifies their
  addresses in the original compiled script.
- `src/shared/.../Gk0101MuffinBox.fsb.patch/` contains patch logic that can be
  reused across revisions. The matching patch directory under `src/R8AJ01/`
  supplies the revision-specific `symbols.toml`, keeping reusable function
  implementations separate from revision-dependent addresses.

Complete sources are only used for new files. Changes to original files use
`.patch/` directories. Reusable patch sources may live under `src/shared/`, but
metadata tied to a particular game revision remains under `src/<game-id>/`.

## Project structure

```text
pokepark-wii-demo/
├── project.toml
├── original/
│   └── R8AJ01/                       # Immutable extracted baseline
├── src/
│   ├── shared/                        # Sources shared across revisions
│   └── R8AJ01/                        # Japanese revision patches
├── build/
│   └── R8AJ01/                       # Generated patched game tree
└── dist/                              # Rebuilt ISOs and packages
```

## Example CLI workflow

From the repository root, extract an ISO you own to create the immutable
baseline for the selected game revision:

```bash
parkforge extract examples/pokepark-wii-demo /path/to/PokeParkWii.iso --game-id R8AJ01
```

Create the source overlay so its directories mirror the extracted game:

```bash
parkforge overlay create examples/pokepark-wii-demo R8AJ01
```

Build the modified game files from the authoritative sources and immutable
baseline:

```bash
parkforge build examples/pokepark-wii-demo R8AJ01
```

Finally, rebuild those files into a patched ISO:

```bash
parkforge rebuild examples/pokepark-wii-demo R8AJ01
```

The default output is `dist/R8AJ01.iso`. The original ISO is extracted to
`original/R8AJ01/`.
