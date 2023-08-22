use bevy::prelude::*;
use bevy_ggrs::ggrs;
use leafwing_input_manager::prelude::*;

// const INPUT_UP: u8 = 1 << 0;
// const INPUT_DOWN: u8 = 1 << 1;
// const INPUT_LEFT: u8 = 1 << 2;
// const INPUT_RIGHT: u8 = 1 << 3;
// const INPUT_FIRE: u8 = 1 << 4;

#[derive(
    Actionlike,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Hash,
    Debug,
    Reflect,
)]
pub enum PlayerAction {
    Up = 1 << 0,
    Down = 1 << 1,
}

impl PlayerAction {
    fn active(&self, input: u8) -> bool {
        input & (*self as u8) != 0
    }
}

pub fn input(
    _: In<ggrs::PlayerHandle>,
    // keys: Res<Input<KeyCode>>,
    player_action: Query<&ActionState<PlayerAction>>,
) -> u8 {
    let mut input = 0u8;

    let action_state = player_action.single();

    for action in
        [PlayerAction::Up, PlayerAction::Down].into_iter()
    {
        if action_state.pressed(action) {
            input |= action as u8;
        }
    }

    input
}

pub fn direction(input: u8) -> Vec2 {
    let mut direction = Vec2::ZERO;
    if PlayerAction::Up.active(input) {
        direction.y += 1.;
    }
    if PlayerAction::Down.active(input) {
        direction.y -= 1.;
    }
    direction.normalize_or_zero()
}
