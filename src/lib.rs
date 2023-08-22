use bevy::{
    math::Vec3Swizzles,
    prelude::*,
    render::camera::ScalingMode,
    sprite::{Anchor, MaterialMesh2dBundle},
};
use bevy_asset_loader::prelude::*;
use bevy_ggrs::{ggrs::PlayerType, *};
use bevy_matchbox::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::*;
use components::*;
use input::*;
use leafwing_input_manager::{
    prelude::{ActionState, InputMap},
    InputManagerBundle,
};

pub mod components;
pub mod input;

pub struct GgrsConfig;

impl ggrs::Config for GgrsConfig {
    // 4-directions + fire fits easily in a single
    // byte
    type Input = u8;
    type State = u8;
    // Matchbox' WebRtcSocket addresses are called
    // `PeerId`s
    type Address = PeerId;
}

#[derive(
    States, Clone, Eq, PartialEq, Debug, Hash, Default,
)]
pub enum GameState {
    #[default]
    AssetLoading,
    Matchmaking,
    InGame,
}

#[derive(Resource)]
struct LocalPlayerHandle(usize);

const MAP_SIZE: i32 = 41;

pub fn setup(mut commands: Commands) {
    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.projection.scaling_mode =
        ScalingMode::FixedVertical(100.);
    commands.spawn(camera_bundle);
    commands.spawn(InputManagerBundle::<PlayerAction> {
        // Stores "which actions are currently pressed"
        action_state: ActionState::default(),
        // Describes how to convert from player inputs into those actions
        input_map: InputMap::new([
            (KeyCode::Up, PlayerAction::Up),
            (KeyCode::W, PlayerAction::Up),
            (KeyCode::Down, PlayerAction::Down),
            (KeyCode::S, PlayerAction::Down),
        ]),
    });
}

pub fn spawn_players(mut commands: Commands) {
    info!("Spawning players");

    // Player 1
    commands.spawn((
        Player { handle: 0 },
        SpriteBundle {
            transform: Transform::from_translation(
                Vec3::new(-50., 0., 100.),
            ),
            sprite: Sprite {
                color: Color::rgb(0., 0., 0.),
                custom_size: Some(Vec2::new(2., 10.)),
                ..default()
            },
            ..default()
        },
        RigidBody::KinematicPositionBased,
        KinematicCharacterController::default(),
        Collider::cuboid(1., 5.),
        // Paddle,
        ActiveEvents::COLLISION_EVENTS,
    ));
    // .add_rollback();

    // Player 2
    commands
        .spawn((
            Player { handle: 1 },
            SpriteBundle {
                transform: Transform::from_translation(
                    Vec3::new(50., 0., 100.),
                ),
                sprite: Sprite {
                    color: Color::rgb(0., 0., 0.),
                    custom_size: Some(Vec2::new(2., 10.)),
                    ..default()
                },
                ..default()
            },
            RigidBody::KinematicPositionBased,
            KinematicCharacterController::default(),
            Collider::cuboid(1., 5.),
            // Paddle,
            ActiveEvents::COLLISION_EVENTS,
        ))
        .add_rollback();
}

pub fn start_matchbox_socket(mut commands: Commands) {
    let room_url = "ws://127.0.0.1:3536/ping?room=abcd";
    info!("connecting to matchbox server: {room_url}");
    commands.insert_resource(MatchboxSocket::new_ggrs(
        room_url,
    ));
}

pub fn wait_for_players(
    mut commands: Commands,
    mut socket: ResMut<MatchboxSocket<SingleChannel>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if socket.get_channel(0).is_err() {
        return; // we've already started
    }

    // Check for new connections
    socket.update_peers();
    let players = socket.players();

    let num_players = 2;
    if players.len() < num_players {
        return; // wait for more players
    }

    info!("All peers have joined, going in-game");

    // create a GGRS P2P session
    let mut session_builder =
        ggrs::SessionBuilder::<GgrsConfig>::new()
            .with_num_players(num_players)
            .with_input_delay(2);

    for (i, player) in players.into_iter().enumerate() {
        if player == PlayerType::Local {
            commands.insert_resource(LocalPlayerHandle(i));
        }

        session_builder = session_builder
            .add_player(player, i)
            .expect("failed to add player");
    }

    // move the channel out of the socket (required
    // because GGRS takes ownership of it)
    let socket = socket.take_channel(0).unwrap();

    // start the GGRS session
    let ggrs_session = session_builder
        .start_p2p_session(socket)
        .expect("failed to start session");

    commands.insert_resource(bevy_ggrs::Session::P2P(
        ggrs_session,
    ));
    next_state.set(GameState::InGame);
}

pub fn move_players(
    inputs: Res<PlayerInputs<GgrsConfig>>,
    mut player_query: Query<(
        &mut KinematicCharacterController,
        &Player,
    )>,
) {
    for (mut controller, player) in player_query.iter_mut()
    {
        let (input, _) = inputs[player.handle];
        let direction = direction(input);

        if direction == Vec2::ZERO {
            continue;
        }

        // move_direction.0 = direction;

        let move_speed = 1.;
        let move_delta = direction * move_speed;

        // let old_pos = controller.translation.xy();
        // let limit = Vec2::splat(MAP_SIZE as f32 / 2. -
        // 0.5); let new_pos =
        //     (old_pos + move_delta).clamp(-limit,
        // limit);
        dbg!(controller.translation);
        controller.translation =
            match controller.translation {
                Some(mut vector) => {
                    vector = vector + move_delta;
                    Some(vector)
                }
                None => Some(move_delta),
            };

        // controller.translation.x = new_pos.x;
        // controller.translation.y = new_pos.y;
    }
}

const BORDER_SIZE: f32 = 0.4;

pub fn spawn_playing_area(mut commands: Commands) {
    // board width and height
    let board = (120.0, 80.0);
    let board_extents = (board.0 / 2.0, board.1 / 2.0);

    // board background
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::Rgba {
                red: 1.0,
                green: 1.0,
                blue: 1.0,
                alpha: 0.3,
            },
            custom_size: Some(Vec2::new(board.0, board.1)),
            // anchor: Anchor::BottomLeft,
            ..Default::default()
        },
        transform: Transform::from_xyz(0.0, 0.0, 1.0),
        ..Default::default()
    });

    // border
    let shape = shapes::Rectangle {
        // extents: Vec2::new(board.0, board.1),
        extents: Vec2::new(
            board.0 + BORDER_SIZE,
            board.1 + BORDER_SIZE,
        ),
        ..Default::default()
    };

    // border??
    commands.spawn((
        ShapeBundle {
            path: GeometryBuilder::build_as(&shape),
            transform: Transform::from_xyz(
                0.0, // board.0 / 2.0,
                0.0, // board.1 / 2.0,
                2.0,
            ),
            ..default()
        },
        Fill::color(Color::rgba(0.0, 0.0, 0.0, 0.0)),
        Stroke::new(
            Color::rgba(82.0, 90.0, 94.0, 1.0),
            BORDER_SIZE,
        ),
    ));

    commands.spawn((
        SpatialBundle::default(),
        RigidBody::Fixed,
        Restitution {
            coefficient: 1.0,
            combine_rule: CoefficientCombineRule::Min,
        },
        Friction {
            coefficient: 0.0,
            combine_rule: CoefficientCombineRule::Min,
        },
        Collider::polyline(
            vec![
                Vect::new(
                    -board_extents.0,
                    -board_extents.1,
                ),
                Vect::new(
                    board_extents.0,
                    -board_extents.1,
                ),
                Vect::new(board_extents.0, board_extents.1),
                Vect::new(
                    -board_extents.0,
                    board_extents.1,
                ),
            ],
            Some(vec![[0, 1], [1, 2], [2, 3], [3, 0]]),
        ),
        // PlayingAreaBorder,
    ));
}

pub fn spawn_ball(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let material =
        materials.add(ColorMaterial::from(Color::WHITE));
    // let mesh = meshes
    //     .add(bevy::prelude::shape::Circle::new(0.1).into());
    let mesh = meshes.add(
        bevy::prelude::shape::Quad::new(Vec2::new(2., 2.))
            .into(),
    );
    commands
        .spawn((
            Velocity::linear(Vec2::new(50.0, 50.0)),
            MaterialMesh2dBundle {
                mesh: mesh.into(),
                material,
                transform: Transform::from_xyz(
                    0.0, 0.0, 10.0,
                ),
                ..default()
            },
            RigidBody::Dynamic,
            Restitution {
                coefficient: 1.0,
                combine_rule: CoefficientCombineRule::Min,
            },
            Friction {
                coefficient: 0.0,
                combine_rule: CoefficientCombineRule::Min,
            },
            // Collider::ball(0.1),
            Collider::cuboid(1., 1.),
            // Ball,
            LockedAxes::ROTATION_LOCKED,
            ActiveEvents::COLLISION_EVENTS,
            GravityScale(0.0),
            Ccd::enabled(),
            Sleeping::disabled(),
            Ball,
        ))
        .add_rollback();
}

#[derive(Component)]
pub struct Ball;

pub fn gizmos(
    mut config: ResMut<GizmoConfig>,
    balls: Query<(&Velocity, &Transform), With<Ball>>,
    mut gizmos: Gizmos,
) {
    config.line_width = 5.;
    for (velocity, transform) in balls.iter() {
        // total velocity
        gizmos.ray_2d(
            transform.translation.xy(),
            Vec2::new(velocity.linvel.x, velocity.linvel.y),
            Color::BLUE,
        );
        // x velocity
        gizmos.ray_2d(
            transform.translation.xy(),
            Vec2::new(
                velocity.linvel.x,
                0., //               transform.translation.y,
            ),
            Color::RED,
        );
        // y velocity
        gizmos.ray_2d(
            transform.translation.xy(),
            Vec2::new(
                0., // transform.translation.x,
                velocity.linvel.y,
            ),
            Color::GREEN,
        );
    }
}
