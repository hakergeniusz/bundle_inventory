// The open bundle's panel, drawn over the plain survival inventory once a
// frame (after the screen and its tooltips): the panel box, one real slot
// frame per entry with the game's own item rendering — counts, durability,
// tints all come free — the hover highlight and the hovered entry's name,
// and the weight bar under the grid, blue the way the bundle's own bar is
// until the bundle is full, then red. Read-only: the clicks answer in
// their own helper.
// Copyright (c) 2026 hakergeniusz. MIT.
fn bundle_inventory_draw(
    game: &crate::game::Game<'_>,
    batch: &mut HudBatch,
    screen: &ScreenView<'_>,
) {
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

    use horizon_client::bundle::Contents;
    if screen.id != "inventory" || screen.menu.is_some() || !gates(game) {
        return;
    }
    if game.book_covers_menu(batch.width) {
        return;
    }
    let Some(open) = bundle_inventory::open_slot() else {
        return;
    };
    let items = game.res.items();
    let Some(stack) = slot_stack(game, open) else {
        return;
    };
    if !items.get(&stack.item).is_some_and(|info| info.bundle) {
        return;
    }
    let Some(contents) = Contents::of(items, stack) else {
        return;
    };
    let selected = stack.data().map_or(-1, |data| data.bundle_selected.0);
    let entries = contents.items();
    let view = bundle_inventory::View {
        bundle_slot: open,
        selected,
        entries: entries
            .iter()
            .map(|stack| (stack.item.to_string(), stack.count))
            .collect(),
        slots: Vec::new(),
    };
    let (ox, oy) = game.inventory_origin(batch);
    let panel = bundle_inventory::panel_rect([ox, oy], batch.width);
    let (shown, hidden) = bundle_inventory::shown_count(view.entries.len());
    // The panel box: a one-pixel border around the vanilla inventory gray,
    // so it reads as part of the screen it stands beside.
    batch.fill(&game.gui.atlas, panel[0], panel[1], panel[2], panel[3], 0xFF37_3737);
    batch.fill(
        &game.gui.atlas,
        panel[0] + bundle_inventory::BORDER,
        panel[1] + bundle_inventory::BORDER,
        panel[2] - bundle_inventory::BORDER * 2.0,
        panel[3] - bundle_inventory::BORDER * 2.0,
        0xFFC6_C6C6,
    );
    for index in 0..shown {
        let [x, y, ..] = bundle_inventory::entry_rect(panel, index);
        batch.sprite(&game.gui.atlas, "gui/sprites/container/slot", x, y);
        game.draw_item(batch, &entries[index], x, y);
    }
    let at = [game.cursor.0, game.cursor.1];
    if let Some(index) = bundle_inventory::entry_at(at, panel, view.entries.len()) {
        let [x, y, ..] = bundle_inventory::entry_rect(panel, index);
        horizon_client::ui::blit_sprite(
            batch,
            &game.gui.atlas,
            "gui/sprites/container/slot_highlight_front",
            x - 4.0,
            y - 4.0,
            24.0,
            24.0,
            u32::MAX,
        );
        let mut lines = vec![(
            horizon_client::stations::hover_name(&game.pack.names, &entries[index]),
            u32::MAX,
        )];
        if hidden > 0 {
            lines.push((format!("and {hidden} more…"), 0xFFA0_A0A0));
        }
        game.draw_tooltip_lines(batch, &lines, game.cursor.0, game.cursor.1);
    }
    // The weight bar, under the grid: the bundle's own colors, the width the
    // weight's fraction of the grid.
    let bar = bundle_inventory::bar_rect(panel, shown);
    batch.fill(&game.gui.atlas, bar[0], bar[1], bar[2], bar[3], 0x6800_0000);
    let fraction = contents.weight_fraction().clamp(0.0, 1.0);
    if fraction > 0.0 {
        let color = if contents.is_full() {
            0xFFFF_5454
        } else {
            0xFF70_87FF
        };
        batch.fill(
            &game.gui.atlas,
            bar[0],
            bar[1],
            (bar[2] * fraction).max(1.0),
            bar[3],
            color,
        );
    }
}
