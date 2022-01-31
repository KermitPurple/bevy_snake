use rand::prelude::*;
use bevy::{
    prelude::*,
    core::FixedTimestep,
};

const MOVE_STEP: f64 = 1.0 / 5.0;
const TIME_STEP: f32 = 1.0 / 60.0;

#[derive(Component, Copy, Clone, PartialEq)]
enum Facing {
    Up,
    Down,
    Left,
    Right,
}

impl Default for Facing {
    fn default() -> Self {
        Facing::Up
    }
}

impl Facing {
    fn opposite(self) -> Self {
        use Facing::*;
        match self {
            Up => Down,
            Left => Right,
            Down => Up,
            Right => Left,
        }
    }

    fn is_opposite(self, other: Facing) -> bool {
        self.opposite() == other
    }

    fn from_key_code(key_code: KeyCode) -> Option<Self> {
        use KeyCode::*;
        match key_code {
            Up | W | K => Some(Facing::Up),
            Left | A | H => Some(Facing::Left),
            Down | S | J => Some(Facing::Down),
            Right | D | L => Some(Facing::Right),
            _ => None,
        }
    }
}

#[derive(Copy, Clone)]
struct ScoreBoard(u32);

#[derive(Component, Copy, Clone)]
struct Fruit;

#[derive(Component, Clone, Default)]
struct Head(Vec<Entity>);

#[derive(Component, Clone, Default)]
struct Tail;

#[derive(Component, Copy, Clone, PartialEq)]
struct Position {
    x: i32,
    y: i32,
}

impl Position {
    fn new(x: i32, y: i32) -> Self {
        Self {
            x,
            y,
        }
    }

    fn in_bounds(&self, size: Size<i32>) -> bool {
        self.x >= 0 && self.y >= 0 && self.x < size.width && self.y < size.height
    }

    fn random(size: Size<i32>) -> Self {
        let mut rng = rand::thread_rng();
        Self {
            x: rng.gen_range(0..size.width),
            y: rng.gen_range(0..size.height),
        }
    }

    fn center(size: Size<i32>) -> Self {
        Self {
            x: size.width / 2,
            y: size.height / 2,
        }
    }
}

#[derive(Component, Copy, Clone)]
struct Size<T: Copy>{
    width: T,
    height: T,
}

impl<T: Copy> Size<T> {
    fn square(val: T) -> Self {
        Self {
            width: val,
            height: val,
        }
    }
}

#[derive(Copy, Clone)]
struct FruitColor(Color);

#[derive(Copy, Clone)]
struct HeadColor(Color);

#[derive(Copy, Clone)]
struct TailColor(Color);

#[derive(Copy, Clone)]
struct Grid{
    cell_size: f32,
    size: Size<i32>,
}

type WindowSize = Size<f32>;

fn grid_to_real(coord: Position, cell_size: Size<f32>, window_size: WindowSize) -> Vec2 {
    Vec2::new(
        (-window_size.width + cell_size.width) / 2.0 + coord.x as f32 * cell_size.width,
        (window_size.height - cell_size.height) / 2.0 - coord.y as f32 * cell_size.height,
    )
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::BLACK))
        .add_startup_system(startup)
        .add_startup_stage("adding_fruit", SystemStage::single(add_fruit_system))
        .add_startup_stage("adding_head", SystemStage::single(add_snake_system))
        .add_system_set(
            SystemSet::new()
            .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
            .with_system(change_direction_system)
        )
        .add_system_set(
            SystemSet::new()
            .with_run_criteria(FixedTimestep::step(MOVE_STEP))
            .with_system(move_snake_system)
            .with_system(collide_snake_system)
            .with_system(eat_fruit_system)
            .with_system(pos_trans_size_scale_system)
        )
        .run();
}

fn startup(
    mut commands: Commands,
    windows: Res<Windows>
) {
    // camera
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    //
    let window = windows.get_primary().unwrap();
    let width = window.width();
    let height = window.height();
    // resources
    commands.insert_resource(WindowSize {
        width,
        height,
    });
    commands.insert_resource(ScoreBoard(0));
    let cell_size = width.min(height) / 20.0;
    commands.insert_resource(Grid {
        cell_size,
        size: Size {
            width: (width / cell_size) as i32,
            height: (height / cell_size) as i32,
        },
    });
    commands.insert_resource(FruitColor(Color::rgb(1.0, 0.0, 0.0)));
    commands.insert_resource(HeadColor(Color::rgb(0.0, 1.0, 0.5)));
    commands.insert_resource(TailColor(Color::rgb(0.0, 1.0, 0.0)));
    commands.insert_resource(Tail::default());
}

fn spawn_tail_segment(mut commands: Commands, position: Position, tail_color: Color, size: Size<f32>) -> Entity {
    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            color: tail_color,
            ..Default::default()
        },
        ..Default::default()
    })
    .insert(Tail)
    .insert(position)
    .insert(size)
    .id()
}

fn add_fruit_system(
    mut commands: Commands,
    grid: Res<Grid>,
    fruit_color: Res<FruitColor>,
 ){
    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            color: fruit_color.0,
            ..Default::default()
        },
        ..Default::default()
    })
    .insert(Fruit)
    .insert(Position::random(grid.size))
    .insert(Size::square(grid.cell_size));
}

fn add_snake_system(
    mut commands: Commands,
    grid: Res<Grid>,
    head_color: Res<HeadColor>,
) {
    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            color: head_color.0,
            ..Default::default()
        },
        ..Default::default()
    })
    .insert(Head::default())
    .insert(Facing::Up)
    .insert(Position::center(grid.size))
    .insert(Size::square(grid.cell_size));
}

fn pos_trans_size_scale_system(
    window_size: Res<WindowSize>,
    mut query: Query<(&Position, &Size<f32>, &mut Transform)>,
) {
    for (pos, size, mut transform) in query.iter_mut() {
        transform.scale = Vec3::new(
            size.width,
            size.height,
            1.0,
            );
        let pos = grid_to_real(*pos, *size, window_size.clone());
        transform.translation = Vec3::new(
            pos.x,
            pos.y,
            0.0,
        );
    }
}

fn change_direction_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Facing, With<Head>>,
) {
    let mut facing = query.single_mut();
    for code in keyboard_input.get_pressed() {
        if let Some(val) = Facing::from_key_code(*code) {
            *facing = val
        };
    }
}

fn move_snake_system(
    facing: Query<&Facing, With<Head>>,
    mut head: Query<(Entity, &Head)>,
    mut positions: Query<&mut Position>,
) {
    let (entity, head) = head.single_mut();
    let tail_positions = head
        .0
        .iter()
        .map(|e| *positions.get_mut(*e).unwrap())
        .collect::<Vec<Position>>();
    let mut head_pos = positions.get_mut(entity).unwrap();
    match *facing.single() {
        Facing::Up => head_pos.y -= 1,
        Facing::Left => head_pos.x -= 1,
        Facing::Down => head_pos.y += 1,
        Facing::Right => head_pos.x += 1,
    }
    tail_positions.iter()
        .zip(head.0.iter().skip(1))
        .for_each(|(pos, tail_seg)| {
            println!("{} {}   {:?}", pos.x, pos.y, tail_seg);
            *positions.get_mut(*tail_seg).unwrap() = *pos;
        });
}

fn collide_snake_system(
    grid: Res<Grid>,
    mut query: Query<&mut Position, With<Head>>,
) {
    let pos = query.single_mut();
    if !pos.in_bounds(grid.size) {
        panic!("DEAD!");
    }
}

fn eat_fruit_system(
    grid: Res<Grid>,
    tail_color: Res<TailColor>,
    commands: Commands,
    mut scoreboard: ResMut<ScoreBoard>,
    mut fruit: Query<&mut Position, (With<Fruit>, Without<Head>)>,
    mut head: Query<(&Position, &mut Head), (With<Head>, Without<Fruit>)>,
) {
    let mut fruit = fruit.single_mut();
    let (head_pos, mut head) = head.single_mut();
    if *fruit == *head_pos {
        *fruit = Position::random(grid.size);
        scoreboard.0 += 1;
        head.0.push(spawn_tail_segment(commands, *head_pos, tail_color.0, Size::square(grid.cell_size)));
    }
}
