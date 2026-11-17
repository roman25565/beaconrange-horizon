# BeaconRange for Horizon

Server-side mod: beacons apply their effects in a radius **4 times larger** than vanilla (the multiplier
is configurable). Works for both the vanilla Java client and the Horizon client: players install nothing.

| Pyramid | Vanilla | With the mod (×4) |
|---|---|---|
| 1 level | 20 blocks | 80 |
| 2 levels | 30 | 120 |
| 3 levels | 40 | 160 |
| 4 levels | 50 | 200 |

The radius is measured from the beacon along X and Z (a square, as in vanilla). Vertically, as in vanilla, the effect
reaches any height above the beacon, and downwards the same distance as the radius (so also ×4). Effect duration and
strength are vanilla.

## Configuration

`config/beaconrange.toml` on the server (the mod creates it on the first beacon pulse):

```toml
multiplier = 4   # 1 = vanilla, 1 to 16, fractions allowed (2.5)
```

The file is read once after startup, so restart the server after changing it. An invalid value
(0, text, 100) is logged and 4 is used instead.

## How it works

One `MODIFY_VARIABLE` injection into the game function `beacon_effects_on_player` (`BeaconBlockEntity.applyEffects`):
the `range` variable (`10 + 10 * levels`) is multiplied by `multiplier` before the check whether the player is in range.
Everything else (a pulse every 80 ticks, effects, ambient, the secondary effect at level 4) stays as in the game.
The function is not in the catalogue of named hooks, so the mod is pinned to Horizon commit
`fdc87ae`. After updating Horizon to a new commit, check that it still builds.

## Files

- `beaconrange/`: the mod itself (`horizon-mixin.toml`, `range.rs`, `core/` with the logic and the default config).
- `beaconrange/core`: `cargo test` here checks the multiplier, the config and the range bounds (no game needed).
- `test-server/setup.ps1`: builds an engine with only this mod and sets up a test server on port 25578.
  It does not touch your main server. Paths at the top of the script point to the author's setup; pass your own.

## Installing on your server

Copy the `beaconrange` folder next to your other Horizon mods and rebuild the engine with them.

## License

MIT, see [LICENSE](LICENSE). A mod for [Horizon](https://github.com/horizon-rust/horizon) (MIT, Ludovic Marie and the Horizon contributors).

Unofficial project, not affiliated with Mojang Studios or Microsoft. Minecraft is a trademark of Mojang Synergies AB.
