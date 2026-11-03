// Copyright (c) The Create Team / The Creators of Create
// Copyright (c) 2026 create-horizon contributors
// SPDX-License-Identifier: MIT
// KineticScrollValueBehaviour adapted to Horizon's own-screen and mod-net APIs.
// The two source direction rows and 1..256 range are preserved; UI drawing is
// native Rust. Inventory and the authoritative motor are never edited by clients.
use super::Game;
use crate::{containers::BlockEntity, mod_api::NativeScreenView};
use horizon_client::{first_person::Hand, hud::HudBatch};
use horizon_world::nbt::Tag;
use std::cell::{Cell, RefCell};
#[derive(Clone, Copy, PartialEq, Eq)]
struct Board {
    pos: [i32; 3],
    value: i32,
}
struct Request {
    player: u128,
    dimension: String,
    pos: [i32; 3],
    value: i32,
    at: std::time::Instant,
}
thread_local! {
    static BOARD:Cell<Option<Board>>=const{Cell::new(None)};
    static PENDING:Cell<Option<Board>>=const{Cell::new(None)};
    static CLOSE:Cell<bool>=const{Cell::new(false)};
    static REQUESTS:RefCell<Vec<Request>>=const{RefCell::new(Vec::new())};
}
fn value(game: &Game<'_>, pos: [i32; 3]) -> i32 {
    match game.world.block_entities.get(&(pos[0], pos[1], pos[2])) {
        Some(BlockEntity::Mod(e)) => match e.nbt.get(&"ScrollValue".into()) {
            Some(Tag::Int(v)) => *v,
            _ => 16,
        },
        _ => 16,
    }
}
fn target(game: &Game<'_>) -> Option<Board> {
    if !game.native_on("create") {
        return None;
    }
    let hit = game.pick()?;
    if game.native_block(hit.pos) != "create:creative_motor" {
        return None;
    }
    let facing = game.native_block_property(hit.pos, "facing")?;
    let axis = match facing.as_str() {
        "east" | "west" => 0,
        "up" | "down" => 1,
        _ => 2,
    };
    if hit.face[axis] != 0 || (axis != 1 && hit.face == [0, -1, 0]) {
        return None;
    }
    Some(Board {
        pos: hit.pos,
        value: value(game, hit.pos),
    })
}
pub(super) fn interaction(
    game: &mut Game<'_>,
    button: i32,
    hand: Hand,
    swing: &mut bool,
    canceled: &mut bool,
) {
    if button != 1
        || hand != Hand::Main
        || game
            .held()
            .is_some_and(|s| s.item.as_ref() != "create:wrench")
    {
        return;
    }
    let Some(board) = target(game) else {
        return;
    };
    if game.held().is_some() {
        return;
    }
    if game.native_mod_screen_open("create") {
        BOARD.set(Some(board));
        *swing = false;
        *canceled = true;
    }
}
pub(super) fn wheel(game: &mut Game<'_>, steps: f32, claimed: &mut bool) {
    let Some(mut board) = target(game) else {
        return;
    };
    if steps == 0.0 || !steps.is_finite() {
        return;
    }
    board.value = create_engine::controls::scroll(board.value, steps.round() as i32);
    PENDING.set(Some(board));
    *claimed = true;
}
pub(super) fn frame(game: &mut Game<'_>) {
    if let Some(board) = PENDING.take() {
        let mut bytes = Vec::with_capacity(16);
        for v in board.pos.into_iter().chain([board.value]) {
            bytes.extend_from_slice(&v.to_be_bytes());
        }
        if let Err(error) = game.native_net_send_to_server("create:motor_speed", &bytes) {
            log::debug!("Create motor control: {error}");
        }
    }
    if CLOSE.replace(false) {
        game.native_mod_screen_close("create");
        BOARD.set(None);
    }
}
pub(super) fn receive(game: &mut Game<'_>, player: u128, channel: &str, data: &[u8]) {
    if channel != "create:motor_speed" || data.len() != 16 || !game.native_on("create") {
        return;
    }
    let words: Vec<i32> = data
        .chunks_exact(4)
        .map(|b| i32::from_be_bytes(b.try_into().unwrap()))
        .collect();
    let pos = [words[0], words[1], words[2]];
    let value = words[3];
    if value == 0 || !(-256..=256).contains(&value) {
        return;
    }
    let Some(p) = game
        .native_server_players()
        .into_iter()
        .find(|p| p.uuid == player)
    else {
        return;
    };
    if p.game_mode == 3
        || pos
            .into_iter()
            .zip(p.position)
            .map(|(v, p)| (f64::from(v) + 0.5 - p).powi(2))
            .sum::<f64>()
            > 64.0
    {
        return;
    }
    REQUESTS.with(|requests| {
        let mut requests = requests.borrow_mut();
        requests.retain(|r| r.player != player || r.pos != pos);
        if requests.len() < 256 {
            requests.push(Request {
                player,
                dimension: p.dimension,
                pos,
                value,
                at: std::time::Instant::now(),
            });
        }
    });
}
pub(super) fn level_tick(game: &mut Game<'_>) {
    let dim = game.dim.id().to_owned();
    let requests = REQUESTS.with(|cell| {
        let mut pending = cell.borrow_mut();
        let mut selected = Vec::new();
        pending.retain(|r| {
            if r.at.elapsed().as_secs() > 2 {
                false
            } else if r.dimension == dim {
                selected.push((r.player, r.pos, r.value));
                false
            } else {
                true
            }
        });
        selected
    });
    for (player, pos, value) in requests {
        let Some(p) = game
            .native_server_players()
            .into_iter()
            .find(|p| p.uuid == player && p.dimension == dim && p.game_mode != 3)
        else {
            continue;
        };
        if pos
            .into_iter()
            .zip(p.position)
            .map(|(v, p)| (f64::from(v) + 0.5 - p).powi(2))
            .sum::<f64>()
            > 64.0
            || game.native_block(pos) != "create:creative_motor"
        {
            continue;
        }
        game.native_with_mod_block_entity(pos, |e| {
            e.nbt.insert("ScrollValue".into(), Tag::Int(value));
        });
        crate::game::create_port_kinetic_runtime::invalidate();
        game.native_block_entity_sync(pos);
    }
}
fn origin(screen: &NativeScreenView<'_>) -> [f32; 2] {
    [
        (screen.rect[2] - 280.0) / 2.0,
        (screen.rect[3] - 120.0) / 2.0,
    ]
}
pub(super) fn draw(game: &Game<'_>, batch: &mut HudBatch, screen: &NativeScreenView<'_>) {
    if game.native_mod_screen() != Some("create") {
        return;
    }
    let Some(board) = BOARD.get() else {
        return;
    };
    let [x, y] = origin(screen);
    if std::env::var_os("CREATE_HORIZON_TRACE").is_some() {
        thread_local! {static LAST:Cell<Option<Board>>=const{Cell::new(None)};}
        LAST.with(|last| {if last.get()!=Some(board) {
            last.set(Some(board));
            log::info!("Create motor board: {}",serde_json::json!({"pos":board.pos,"value":board.value,"origin":[x,y],"width":batch.width,"height":batch.height}));
        }});
    }

    game.native_fill(batch, [x - 2.0, y - 2.0, 284.0, 124.0], 0xFF3B3029);
    game.native_fill(batch, [x, y, 280.0, 120.0], 0xFFE4D6B9);
    let label = horizon_client::lang::get_or(
        "create.kinetics.creative_motor.rotation_speed",
        "Rotation Speed",
    );
    game.native_text(
        batch,
        &format!("{label}: {}", board.value),
        [x + 10.0, y + 8.0],
        0x3B3029,
        false,
    );
    for row in 0..2u8 {
        let top = y + 30.0 + f32::from(row) * 24.0;
        game.native_text(
            batch,
            if row == 0 { "-" } else { "+" },
            [x + 6.0, top + 3.0],
            0x3B3029,
            false,
        );
        game.native_fill(batch, [x + 18.0, top, 256.0, 16.0], 0xFF766956);
        for n in (32..=256).step_by(32) {
            game.native_fill(batch, [x + 17.0 + n as f32, top, 1.0, 16.0], 0xFFB3A17D);
        }
        if (board.value < 0) == (row == 0) {
            game.native_fill(
                batch,
                [x + 17.0 + board.value.abs() as f32, top - 2.0, 2.0, 20.0],
                0xFFFFD15C,
            );
        }
    }
    game.native_fill(batch, [x + 208.0, y + 96.0, 66.0, 18.0], 0xFF5F6C48);
    game.native_text(
        batch,
        &horizon_client::lang::get_or("gui.done", "Done"),
        [x + 214.0, y + 101.0],
        0xFFFFFF,
        true,
    );
    for n in [32, 64, 128, 192, 256] {
        game.native_text(
            batch,
            &n.to_string(),
            [x + 18.0 + n as f32 - 9.0, y + 74.0],
            0x635640,
            false,
        );
    }
}
pub(super) fn click(
    game: &Game<'_>,
    screen: &NativeScreenView<'_>,
    button: i32,
    canceled: &mut bool,
) {
    if game.native_mod_screen() != Some("create") {
        return;
    }
    let Some(mut board) = BOARD.get() else {
        return;
    };
    let [x, y] = origin(screen);
    let [mx, my] = screen.mouse;
    *canceled = true;
    if button != 0 {
        return;
    }
    if (x + 208.0..=x + 274.0).contains(&mx) && (y + 96.0..=y + 114.0).contains(&my) {
        CLOSE.set(true);
        return;
    }
    for row in 0..2u8 {
        if (y + 30.0 + f32::from(row) * 24.0..=y + 46.0 + f32::from(row) * 24.0).contains(&my)
            && (x + 18.0..=x + 274.0).contains(&mx)
        {
            board.value = create_engine::controls::setting(row, (mx - x - 18.0).round() as i32);
            BOARD.set(Some(board));
            PENDING.set(Some(board));
        }
    }
}
pub(super) fn hud(game: &Game<'_>, batch: &mut HudBatch) {
    if game.native_mod_screen().is_some() {
        return;
    }
    let Some(board) = target(game) else {
        return;
    };
    let label = horizon_client::lang::get_or(
        "create.kinetics.creative_motor.rotation_speed",
        "Rotation Speed",
    );
    game.native_text(
        batch,
        &format!("{label}: {}", board.value),
        [batch.width / 2.0 + 12.0, batch.height / 2.0 + 12.0],
        0xFFD982,
        true,
    );
}
