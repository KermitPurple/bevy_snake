use bevy::prelude::*;

const TIME_STEP: f32 = 1.0 / 60.0;

#[derive(Component, Copy, Clone)]
struct Fruit(Vec2);

#[derive(Copy, Clone)]
struct FruitColor(Color);

#[derive(Copy, Clone)]
struct HeadColor(Color);

#[derive(Copy, Clone)]
struct TailColor(Color);

#[derive(Copy, Clone)]
struct CellSize(f32);

#[derive(Copy, Clone)]
struct  WindowSize {
    width: f32,
    height: f32,
}

fn grid_to_real(coord: Vec2, cell_size: f32, window_size: WindowSize) -> Vec2 {
    Vec2::new(
        (-window_size.width + cell_size) / 2.0 + coord.x * cell_size,
        (window_size.height - cell_size) / 2.0 - coord.y * cell_size,
    )
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::BLACK))
        .add_startup_system(startup)
        .add_startup_stage("adding_fruit", SystemStage::single(add_fruit_system))
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
    commands.insert_resource(CellSize(width.min(height) / 20.0));
    commands.insert_resource(FruitColor(Color::rgb(1.0, 0.0, 0.0)));
    commands.insert_resource(HeadColor(Color::rgb(0.2, 1.0, 0.0)));
    commands.insert_resource(TailColor(Color::rgb(0.0, 1.0, 0.0)));
}

fn add_fruit_system(
    mut commands: Commands,
    cell_size: Res<CellSize>,
    window_size: Res<WindowSize>,
    fruit_color: Res<FruitColor>,
 ){
    let pos = grid_to_real(Vec2::new(0.0, 0.0), cell_size.0, window_size.clone());
    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            color: fruit_color.0,
            ..Default::default()
        },
        transform: Transform {
            translation: Vec3::new(pos.x, pos.y, 10.0),
            scale:  Vec3::new(cell_size.0, cell_size.0, 1.0),
            ..Default::default()
        },
        ..Default::default()
    })
    .insert(Fruit(Vec2::new(0.0, 0.0)));
}
