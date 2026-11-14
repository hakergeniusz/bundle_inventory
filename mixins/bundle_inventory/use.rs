// A right-click over the world (`Game::use_item`, before the game uses it):
// shift + right-click on a bundle in the hand opens the plain inventory with
// the mod's panel already on it — the panel draws on a screen, so the
// inventory opens under it — and the use is skipped. Every other use runs
// vanilla. The gate here is deliberately narrower than the panel's own (no
// craft-size or container checks): it answers while playing, with the window
// on, the mod enabled and a bundle in the hand.
// Copyright (c) 2026 hakergeniusz. MIT.
fn bundle_inventory_use(
    game: &mut crate::game::Game<'_>,
) -> std::ops::ControlFlow<Result<Option<Hand>>> {
    use crate::game::bundle_inventory as bi;
    if game.screen != crate::game::Screen::Playing
        || !game.native_on(bi::ID)
        || !bi::setting_on(|key| game.native_setting(bi::ID, key))
        || !game.native_window()
        || !game.has_shift_down()
    {
        return std::ops::ControlFlow::Continue(());
    }
    let bundle_in_hand = game
        .held()
        .is_some_and(|stack| {
            game.res.items().get(&stack.item).is_some_and(|info| info.bundle)
        });
    if !bundle_in_hand {
        return std::ops::ControlFlow::Continue(());
    }
    // The plain inventory opens the way the E key opens it, and the panel
    // comes up on the held bundle's slot.
    game.screen = crate::game::Screen::Inventory;
    game.capture(false);
    game.right = false;
    bi::set_open(Some(game.selected));
    std::ops::ControlFlow::Break(Ok(None))
}
