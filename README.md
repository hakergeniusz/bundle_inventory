# Bundle Inventory

A [Horizon](https://github.com/horizon-rust/horizon) native mod (`.hnmod`):
shift + right-click a bundle in your inventory to open an interactive panel
of its contents beside the survival inventory.

- **Open** — shift + right-click a bundle in any of your inventory slots
  (hotbar, main inventory, off hand). The panel lists every entry as its own
  slot, drawn with the game's own item rendering, and the bundle's weight bar
  underneath, in the bundle's own colors (blue, red when full).
- **Take** — click an entry to take it onto the cursor. The mod selects the
  entry first (the vanilla wheel-selection protocol), so it is exactly that
  entry that comes out, not the first one.
- **Move into the inventory** — shift-click an entry: it merges onto partial
  stacks of its kind first, then fills empty slots (main rows before the
  hotbar), and whatever no slot took goes back into the bundle instead of
  riding the cursor.
- **Put in** — click any item in your inventory with the panel open: what
  fits goes into the bundle, the remainder stays where it was. Shulker boxes
  don't fit (vanilla's rule).
- **Drop** — Q over an entry drops it into the world. Escape closes the
  panel (not the inventory); a click outside the panel closes it too, and
  never drops what your cursor carries.

Every move is an ordinary inventory click — the engine's own verified menu
engine predicts it and a server hears it — plus, for taking a chosen entry
out, the same bundle-selection packet the vanilla wheel sends. The panel is
honest in single player, on LAN and on servers, and adapts to resource packs
for free, because the slots and items are the game's own rendering.

## Install

Drop `target/bundle_inventory-<version>.hnmod` into your launcher's mod
list (or your loader's mods folder). The mod has one setting, **Enabled**
(default on), under Options > Mods > Bundle Inventory.

## Scope

Inventory-only for now: the panel draws beside the plain survival inventory.
Container and workstation screens are out of scope for this version.

## Clean-room notice

This is an **independent reimplementation** of the feature set of the
Modrinth mod [Bundle Inventory](https://modrinth.com/mod/bundle-inventory)
(all rights reserved by its author, no public source repository). Nothing
was copied from it — not code, not assets, not text; this mod was written
from scratch against Horizon's own item and menu systems, using only the
original's public feature description as the behavioral spec. This mod is
MIT licensed; the original keeps its own license.

## Development

- `tools/test.py` — the standalone suite (format, clippy, the runtime's
  unit tests; the runtime is pure standard library).
- `tools/horizon.py stage|check|test|build` — stage the mod onto the pinned
  Horizon revision and check/test/build the patched client.
- `tools/package.py` — the source-only `.hnmod` archive.
- `tools/ingame-test.txt` — the in-game verification script (horizon-ctl).

## License

MIT — see [LICENSE](LICENSE).
