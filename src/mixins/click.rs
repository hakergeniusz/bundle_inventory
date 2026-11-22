// A press on the survival inventory, before anything the screen does with it:
// this is the modal panel's whole routing. Shift + right-click on a bundle
// opens it; with the panel open a click on an entry takes it (shift: into the
// inventory), a click on any other player slot puts that stack in, and a
// click anywhere else closes the panel — which also keeps the click from
// falling through and dropping what the cursor carries. Every move is
// deferred to the tick as a plan of ordinary menu clicks, because this hook
// reads the game through a shared borrow and the menu clicks want the
// exclusive one.
// Copyright (c) 2026 hakergeniusz. MIT.
fn bundle_inventory_screen_click(
    game: &crate::game::Game<'_>,
    screen: &ScreenView<'_>,
    button: i32,
    modifiers: u32,
) -> core::ops::ControlFlow<()> {
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
    // The stack one of the player's own slots holds: the main inventory, or
    // the off hand. The panel's slots and the insert's targets are all
    // these.
    fn slot_stack<'a, 'g>(
        game: &'a crate::game::Game<'g>,
        slot: usize,
    ) -> Option<&'a horizon_client::items::ItemStack> {
        if slot == 504 {
            game.offhand.as_ref()
        } else {
            game.inventory.get(slot).and_then(|stack| stack.as_ref())
        }
    }

    let _ = modifiers;
    if screen.id != "inventory" || screen.menu.is_some() || !gates(game) {
        return core::ops::ControlFlow::Continue(());
    }
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
    let at = [game.cursor.0, game.cursor.1];
    let Some(open) = bundle_inventory::open_slot() else {
        // Closed: shift + right-click on a bundle in one of the player's own
        // slots opens the panel. Vanilla's right-click on a bundle takes one
        // out, so this press is ours to take instead.
        if button == 1 && game.has_shift_down() {
            let (target, _) = game.inventory_target();
            if let Some(hit) = target
                && bundle_inventory::player_slot(hit)
                && slot_stack(game, hit)
                    .is_some_and(|stack| {
                        game.res
                            .items()
                            .get(&stack.item)
                            .is_some_and(|i| i.bundle)
                    })
            {
                bundle_inventory::set_open(Some(hit));
                return core::ops::ControlFlow::Break(());
            }
        }
        return core::ops::ControlFlow::Continue(());
    };
    // The snapshot a plan is built from: the open bundle's entries and
    // selection, and the player's own slots. None when the slot holds no
    // bundle — the caller closes the panel rather than act on it.
    let view = {
        use horizon_client::bundle::Contents;
        let Some(stack) = slot_stack(game, open) else {
            // The bundle left the slot between the frame and the click:
            // close and keep the click from falling anywhere.
            bundle_inventory::set_open(None);
            return core::ops::ControlFlow::Break(());
        };
        let items = game.res.items();
        if !items.get(&stack.item).is_some_and(|info| info.bundle) {
            bundle_inventory::set_open(None);
            return core::ops::ControlFlow::Break(());
        }
        let Some(contents) = Contents::of(items, stack) else {
            bundle_inventory::set_open(None);
            return core::ops::ControlFlow::Break(());
        };
        bundle_inventory::View {
            bundle_slot: open,
            selected: stack.data().map_or(-1, |data| data.bundle_selected.0),
            entries: contents
                .items()
                .iter()
                .map(|stack| (stack.item.to_string(), stack.count))
                .collect(),
            slots: (0..36)
                .chain([504])
                .map(|slot| {
                    let item =
                        slot_stack(game, slot).map(|stack| bundle_inventory::SlotItem {
                            id: stack.item.to_string(),
                            count: stack.count,
                        });
                    (slot, item)
                })
                .collect(),
        }
    };
    let (ox, oy) = game.inventory_origin(&batch);
    let panel = bundle_inventory::panel_rect([ox, oy], batch.width);
    if bundle_inventory::in_rect(at, panel) {
        // The panel, its chrome included, never lets a click through to
        // whatever it covers. With nothing on the cursor an entry answers:
        // plain click takes it, shift-click moves it into the inventory.
        if game.carried.is_none()
            && let Some(index) = bundle_inventory::entry_at(at, panel, view.entries.len())
        {
            let action = if game.has_shift_down() {
                bundle_inventory::Action::TransferOut(index)
            } else {
                bundle_inventory::Action::Extract(index)
            };
            let steps = bundle_inventory::plan(action, &view);
            if !steps.is_empty() {
                bundle_inventory::defer(steps);
            }
        }
        return core::ops::ControlFlow::Break(());
    }
    // Outside the panel: the modal insert. A click on another player slot
    // with an empty cursor puts that stack into the open bundle (what fits;
    // the rest back where it came from). The bundle's own slot stays
    // vanilla's, a click with the cursor carrying stays vanilla's, and a
    // click on nothing at all closes the panel — and keeps the vanilla
    // click-outside from dropping what the cursor carries.
    let (target, _) = game.inventory_target();
    match target {
        Some(hit) if hit == open => core::ops::ControlFlow::Continue(()),
        Some(hit)
            if bundle_inventory::player_slot(hit)
                && game.carried.is_none()
                && slot_stack(game, hit).is_some_and(|stack| {
                    bundle_inventory::can_insert(&stack.item)
                }) =>
        {
            let steps = bundle_inventory::plan(bundle_inventory::Action::Insert(hit), &view);
            if !steps.is_empty() {
                bundle_inventory::defer(steps);
            }
            core::ops::ControlFlow::Break(())
        }
        Some(_) => core::ops::ControlFlow::Continue(()),
        None => {
            bundle_inventory::set_open(None);
            core::ops::ControlFlow::Break(())
        }
    }
}
