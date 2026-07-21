# PokePark Wii demo project

This is a source-only Parkforge project for the Japanese PokePark Wii release
(`R8AJ01`). It does not include any game data or generated output.

The sample FSC script is located inside the same nested `.dac`/`.dan` archive
path used by the local test fixture. Build and rebuild it with an ISO you own:

```bash
# From the repository root
cargo run -p parkforge-cli -- extract examples/pokepark-wii-demo /path/to/PokeParkWii.iso --game-id R8AJ01
cargo run -p parkforge-cli -- overlay create examples/pokepark-wii-demo R8AJ01
cargo run -p parkforge-cli -- build examples/pokepark-wii-demo R8AJ01
cargo run -p parkforge-cli -- rebuild examples/pokepark-wii-demo R8AJ01
```

The rebuilt ISO is written to `dist/R8AJ01.iso`. The original ISO is extracted
to `original/R8AJ01/`; both directories, along with `build/`, are ignored by
Git.
