// A right-click over the world (`Game::use_item`, before the game uses it):
// shift + right-click on a bundle in the hand opens the plain inventory with
// the mod's panel already on it — the panel draws on a screen, so the
// inventory opens under it — and the use is skipped. Every other use runs
// vanilla. The gate here is deliberately narrower than the panel's own (no
// craft-size or container checks): it answers while playing, on the window's
// game, with the mod enabled and a bundle in the hand.
// Copyright (c) 2026 hakergeniusz. MIT.
fn bundle_inventory_use(
    game: &mut crate::game::Game<'_>,
) -> core::ops::ControlFlow<Result<Option<Hand>>> {
    if game.screen != crate::game::Screen::Playing
        || !bundle_inventory::setting_on(|key| {
            game.stored_setting(bundle_inventory::ID, key)
        })
        || !game.options.execution_role.is_client()
        || !game.has_shift_down()
    {
        return core::ops::ControlFlow::Continue(());
    }
    let bundle_in_hand = game
        .inventory
        .get(game.selected)
        .and_then(|stack| stack.as_ref())
        .is_some_and(|stack| {
            game.res
                .items()
                .get(&stack.item)
                .is_some_and(|info| info.bundle)
        });
    if !bundle_in_hand {
        return core::ops::ControlFlow::Continue(());
    }
    // The plain inventory opens the way the E key opens it, and the panel
    // comes up on the held bundle's slot.
    game.screen = crate::game::Screen::Inventory;
    game.capture(false);
    game.right = false;
    bundle_inventory::set_open(Some(game.selected));
    core::ops::ControlFlow::Break(Ok(None))
}
