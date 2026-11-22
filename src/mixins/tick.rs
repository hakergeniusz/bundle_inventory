// Every tick: keep the panel honest, and run the plan the last click or key
// left. The panel closes itself the moment the world disagrees with it — the
// screen is no longer the plain survival inventory, or the bundle left its
// slot (a death, a vanilla click, the bundle emptied some other way). A plan
// runs step by step as ordinary menu clicks, each step's cursor expectation
// checked against the live game first, so a greedy transfer chain stops the
// moment the cursor empties and nothing is ever clicked onto a stack that
// would pick it up.
// Copyright (c) 2026 hakergeniusz. MIT.
fn bundle_inventory_tick(game: &mut crate::game::Game<'_>) {
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
    // the off hand.
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
    // Vanilla's bundle selection, the way the engine's own wheel does it:
    // the entry chosen in the client's copy of the stack, and on a server
    // the selection packet sent ahead of the click that acts on it. The
    // brain says whether a select is needed at all (`toggleSelectedItem`
    // toggles).
    fn select(game: &mut crate::game::Game<'_>, bundle_slot: usize, index: i32) {
        let selected = game
            .slot_mut(bundle_slot)
            .as_ref()
            .and_then(|stack| stack.data())
            .map_or(-1, |data| data.bundle_selected.0);
        if !bundle_inventory::select_needed(
            selected,
            usize::try_from(index).unwrap_or(usize::MAX),
        ) {
            return;
        }
        if let Some(stack) = game.slot_mut(bundle_slot) {
            horizon_client::bundle::toggle_selected(stack, index);
        }
        if game.remote.is_some() {
            let (_, menu) = game.remote_menu();
            if let Some(menu_slot) = crate::game::Game::menu_slot_index(&menu, bundle_slot) {
                game.send(horizon_protocol::packets::play::Serverbound::SelectBundleItem(
                    horizon_protocol::packets::play::SelectBundleItem {
                        slot: menu_slot as i32,
                        index,
                    },
                ));
            }
        }
    }
    // One plan step's menu click, as the engine's own verified menu engine
    // predicts it and a server hears it. False when the engine refused it:
    // vanilla keeps nothing of a refused click, so the plan stops there.
    fn menu_click(
        game: &mut crate::game::Game<'_>,
        slot: Option<usize>,
        right: bool,
    ) -> bool {
        let ops = [(
            slot,
            i32::from(right),
            horizon_client::menu::Input::Pickup,
        )];
        game.menu_ops(&ops)
    }

    // The panel's honesty first: it closes the moment its bundle is gone
    // from its slot (the open gate keeps the plain inventory's rule).
    if let Some(open) = bundle_inventory::open_slot() {
        let plain = game.screen == crate::game::Screen::Inventory && gates(game);
        let is_bundle = plain
            && slot_stack(game, open)
                .is_some_and(|stack| {
                    game.res
                        .items()
                        .get(&stack.item)
                        .is_some_and(|i| i.bundle)
                });
        if !is_bundle {
            bundle_inventory::set_open(None);
        }
    }
    let Some(steps) = bundle_inventory::take_deferred() else {
        return;
    };
    let Some(open) = bundle_inventory::open_slot() else {
        return;
    };
    for step in steps {
        match step {
            bundle_inventory::Step::Select(index) => {
                if index >= 0 {
                    select(game, open, index);
                }
            }
            bundle_inventory::Step::Take { bundle } => {
                // The menu's bundle click override takes the selected entry
                // out on a right-click — with an empty cursor only.
                if game.carried.is_some() || !menu_click(game, Some(bundle), true) {
                    break;
                }
            }
            bundle_inventory::Step::Give { slot, allow_empty } => {
                if game.carried.is_none() {
                    if allow_empty {
                        // The put-back onto a slot the plan emptied itself:
                        // with nothing on the cursor the click would do
                        // nothing, so it does not go out at all.
                        continue;
                    }
                    break;
                }
                if !menu_click(game, slot, false) {
                    break;
                }
            }
            bundle_inventory::Step::Carry { slot } => {
                if game.carried.is_some() || !menu_click(game, Some(slot), false) {
                    break;
                }
            }
        }
    }
}
