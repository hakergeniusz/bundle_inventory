// A key went down or up, after rebinding (`mapped_key`): while the panel is
// open over the plain survival inventory, Escape closes the panel instead of
// the inventory, and Q over an entry drops that entry into the world — the
// vanilla drop key, acting on the entry under the cursor instead of the slot
// behind the panel. Presses only: a release must always reach the game's own
// bookkeeping. The drop defers to the tick, as a plan of ordinary menu
// clicks (select the entry, take it out, throw what the cursor carries).
// Copyright (c) 2026 hakergeniusz. MIT.
fn bundle_inventory_key(
    game: &crate::game::Game<'_>,
    key: winit::keyboard::KeyCode,
    pressed: bool,
    repeat: bool,
) -> core::ops::ControlFlow<Result<()>> {
    // Whether the mod's panel may act here: on, on the window's game, and
    // the plain survival inventory (no container, no workstation, the 2x2
    // grid, no mount screen, no creative tabs).
    fn gates(game: &crate::game::Game<'_>) -> bool {
        game.options.execution_role.is_client()
            && bundle_inventory::setting_on(|key| {
                game.stored_setting(bundle_inventory::ID, key)
            })
            && bundle_inventory::plain_inventory(
                game.container.is_none(),
                game.station.is_none(),
                game.craft_size == 2,
                !game.mount_active(),
                !game.creative_tab_open(),
            )
    }
    // The snapshot a plan is built from: the open bundle's entries and
    // selection, and the player's own slots. None when the slot holds no
    // bundle — the caller closes the panel rather than act on it.
    fn snapshot(
        game: &crate::game::Game<'_>,
        bundle_slot: usize,
    ) -> Option<bundle_inventory::View> {
        use horizon_client::bundle::Contents;
        let slot_stack = |slot: usize| {
            if slot == 504 {
                game.offhand.as_ref()
            } else {
                game.inventory.get(slot).and_then(|stack| stack.as_ref())
            }
        };
        let stack = slot_stack(bundle_slot)?;
        let items = game.res.items();
        if !items.get(&stack.item).is_some_and(|info| info.bundle) {
            return None;
        }
        let contents = Contents::of(items, stack)?;
        let entries = contents
            .items()
            .iter()
            .map(|stack| (stack.item.to_string(), stack.count))
            .collect();
        Some(bundle_inventory::View {
            bundle_slot,
            selected: stack.data().map_or(-1, |data| data.bundle_selected.0),
            entries,
            slots: (0..36)
                .chain([504])
                .map(|slot| {
                    let item = slot_stack(slot).map(|stack| bundle_inventory::SlotItem {
                        id: stack.item.to_string(),
                        count: stack.count,
                    });
                    (slot, item)
                })
                .collect(),
        })
    }

    if !pressed
        || repeat
        || game.screen != crate::game::Screen::Inventory
        || !gates(game)
    {
        return core::ops::ControlFlow::Continue(());
    }
    let Some(open) = bundle_inventory::open_slot() else {
        return core::ops::ControlFlow::Continue(());
    };
    let Some(name) = horizon_client::keybinds::key_name(key) else {
        return core::ops::ControlFlow::Continue(());
    };
    match name {
        "key.keyboard.escape" => {
            bundle_inventory::set_open(None);
            core::ops::ControlFlow::Break(Ok(()))
        }
        "key.keyboard.q" => {
            let Some(size) = game
                .renderer
                .as_ref()
                .map(|r| [r.config.width, r.config.height])
            else {
                return core::ops::ControlFlow::Continue(());
            };
            let batch = horizon_client::hud::HudBatch::new(size, game.settings.gui_scale_inputs());
            if game.book_covers_menu(batch.width) {
                return core::ops::ControlFlow::Continue(());
            }
            let Some(view) = snapshot(game, open) else {
                return core::ops::ControlFlow::Continue(());
            };
            let (ox, oy) = game.inventory_origin(&batch);
            let panel = bundle_inventory::panel_rect([ox, oy], batch.width);
            let at = [game.cursor.0, game.cursor.1];
            let Some(index) = bundle_inventory::entry_at(at, panel, view.entries.len()) else {
                // Q over nothing of the panel's: the game's own drop key.
                return core::ops::ControlFlow::Continue(());
            };
            if game.carried.is_some() {
                // The cursor carries a stack of its own; a drop under it
                // would throw the wrong thing. The click stays vanilla's.
                return core::ops::ControlFlow::Continue(());
            }
            let steps = bundle_inventory::plan(bundle_inventory::Action::Drop(index), &view);
            if steps.is_empty() {
                return core::ops::ControlFlow::Continue(());
            }
            bundle_inventory::defer(steps);
            core::ops::ControlFlow::Break(Ok(()))
        }
        _ => core::ops::ControlFlow::Continue(()),
    }
}
