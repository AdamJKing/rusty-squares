use bevy::input::mouse::MouseButtonInput;
use bevy::input::ElementState;
use bevy::math::uvec2;
use bevy::math::vec2;
use bevy::math::vec3;
use bevy::prelude::*;
use bevy::render::camera::{OrthographicProjection, ScalingMode, WindowOrigin};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct Point {
  x: u32,
  y: u32,
}

impl From<Vec2> for Point {
  fn from(vec: Vec2) -> Point {
    Point {
      x: vec.x as u32,
      y: vec.y as u32,
    }
  }
}

impl From<Point> for Vec2 {
  fn from(point: Point) -> Self {
    vec2(point.x as f32, point.y as f32)
  }
}

impl From<&Point> for Vec2 {
  fn from(point: &Point) -> Self {
    vec2(point.x as f32, point.y as f32)
  }
}

impl From<(u32, u32)> for Point {
  fn from(point: (u32, u32)) -> Self {
    Point {
      x: point.0,
      y: point.1,
    }
  }
}

impl PartialEq<Vec2> for Point {
  fn eq(&self, vec: &Vec2) -> bool {
    self.x == (vec.x as u32) && self.y == (vec.y as u32)
  }
}

fn spawn_point(coord: (u32, u32), material: Handle<ColorMaterial>) -> SpriteBundle {
  SpriteBundle {
    transform: Transform {
      translation: Vec3::new(coord.0 as f32, coord.1 as f32, 0.0),
      scale: Vec3::ONE * 0.00025,
      ..Default::default()
    },
    material,
    ..Default::default()
  }
}

// a cell references four edges, which can be shared
struct Cell(Vec2, [Entity; 4]);

pub struct Game;

impl Plugin for Game {
  fn build(&self, app: &mut bevy::prelude::AppBuilder) {
    app
      .add_startup_system_to_stage(StartupStage::PreStartup, load_assets.system())
      .add_startup_system_to_stage(StartupStage::Startup, spawn_world.system())
      .add_startup_system_to_stage(StartupStage::PostStartup, scale_entities.system())
      .add_system(manage_highlights.system().label("highlight"))
      .add_system(on_click_handler.system().label("click").after("highlight"))
      .add_system(
        check_for_taken_squares
          .system()
          .label("check_win")
          .before("highlight"),
      );
  }
}

struct Materials {
  point_material: Handle<ColorMaterial>,
  black_material: Handle<ColorMaterial>,
  player_one_material: Handle<ColorMaterial>,
  player_two_material: Handle<ColorMaterial>,
}

fn load_assets(
  mut commands: Commands,
  asset_server: Res<AssetServer>,
  mut materials: ResMut<Assets<ColorMaterial>>,
) {
  let point_material = materials.add(asset_server.load("black-dot.png").into());
  let black_material = materials.add(Color::rgb(0.0, 0.0, 0.0).into());
  let player_one_material = materials.add(Color::rgb(100.0, 0.0, 0.0).into());
  let player_two_material = materials.add(Color::rgb(0.0, 100.0, 0.0).into());

  commands.insert_resource(Materials {
    point_material,
    black_material,
    player_one_material,
    player_two_material,
  });

  commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn spawn_world(mut commands: Commands, materials: Res<Materials>) {
  let mut horizontal = Vec::with_capacity(36);
  let mut vertical = Vec::with_capacity(36);

  for y in 0..7 {
    for x in 0..7 {
      commands.spawn_bundle(spawn_point((x, y), materials.point_material.clone()));

      if x < 6 {
        horizontal.push(
          commands
            .spawn()
            .insert(Edge((x, y).into(), Alignment::Horizontal))
            .insert_bundle(SpriteBundle {
              material: materials.black_material.clone(),
              transform: Transform {
                translation: uvec2(x, y).extend(0).as_f32() + vec3(0.5, 0.0, 0.0),
                ..Default::default()
              },
              sprite: Sprite::new(vec2(1.0, 0.1)),
              visible: Visible {
                is_visible: true,
                ..Default::default()
              },
              ..Default::default()
            })
            .id(),
        );
      }

      if y < 6 {
        vertical.push(
          commands
            .spawn()
            .insert(Edge((x, y).into(), Alignment::Vertical))
            .insert_bundle(SpriteBundle {
              material: materials.black_material.clone(),
              transform: Transform {
                translation: uvec2(x, y).extend(0).as_f32() + vec3(0.0, 0.5, 0.0),
                ..Default::default()
              },
              sprite: Sprite::new(vec2(0.1, 1.0)),
              visible: Visible {
                is_visible: false,
                ..Default::default()
              },
              ..Default::default()
            })
            .id(),
        );
      }
    }
  }

  let rows: Vec<&[Entity]> = horizontal.chunks(6).collect();
  let columns = vertical.chunks(7);
  let mut loc = vec2(0.0, 0.0);

  for (row, column) in rows.windows(2).zip(columns) {
    for ((&lower, &upper), left_right) in row[0].iter().zip(row[1]).zip(column.windows(2)) {
      commands
        .spawn()
        .insert(Cell(loc, [lower, upper, left_right[0], left_right[1]]));

      loc.x += 1.0;
    }

    loc.y += 1.0;
    loc.x = 0.0;
  }

  commands.insert_resource(Turn(Player::One, false));
}

fn scale_entities(mut q_camera: Query<(&mut OrthographicProjection, &mut Transform)>) {
  let (mut camera, mut transform) = q_camera.single_mut().unwrap();
  camera.window_origin = WindowOrigin::BottomLeft;
  camera.scaling_mode = ScalingMode::None;
  camera.left = 0.0;
  camera.bottom = 0.0;
  camera.top = 7.0;
  camera.right = 7.0;
  camera.scale = 0.9;
  transform.translation -= vec3(0.15, 0.15, 0.0);
}

#[derive(Clone, Copy)]
enum Player {
  One,
  Two,
}

struct Turn(Player, bool);
struct Taken(Player);

struct Activated;
struct Highlight;

enum Alignment {
  Horizontal,
  Vertical,
}

struct Edge(Point, Alignment);

impl Edge {
  fn is_hit_by(&self, point: Vec2) -> bool {
    let root: Vec2 = self.0.into();

    let higher: Vec2;
    let lower: Vec2;

    match self.1 {
      Alignment::Horizontal => {
        higher = root + vec2(0.9, 0.1);
        lower = root - vec2(-0.1, 0.1);
      }
      Alignment::Vertical => {
        higher = root + vec2(0.1, 0.9);
        lower = root - vec2(0.1, -0.1);
      }
    }

    let cmps = lower.cmplt(point) & (higher.cmpgt(point));
    cmps.all()
  }
}

fn manage_highlights(
  mut commands: Commands,
  mut q_edges: Query<(Entity, &Edge, &mut Visible), Without<Activated>>,
  mut mouse_movement: EventReader<CursorMoved>,
  windows: Res<Windows>,
) {
  if let Some(mouse) = mouse_movement.iter().last() {
    let cursor = locate_cursor(mouse.position, windows.get_primary().unwrap());
    let mut highlight_set = false;
    for (id, edge, mut visiblity) in q_edges.iter_mut() {
      if !highlight_set && edge.is_hit_by(cursor) {
        visiblity.is_visible = true;
        highlight_set = true;
        commands.entity(id).insert(Highlight);
      } else {
        visiblity.is_visible = false;
        commands.entity(id).remove::<Highlight>();
      }
    }
  }
}

// use known projection constants to "reverse" the projection
fn locate_cursor(cursor_position: Vec2, window: &Window) -> Vec2 {
  ((cursor_position / vec2(window.width(), window.height()) * 7.0) * 0.9) - vec2(0.1, 0.1)
}

fn on_click_handler(
  mut commands: Commands,
  mut mouse_clicks: EventReader<MouseButtonInput>,
  q_highlighted: Query<Entity, With<Highlight>>,
  mut turn: ResMut<Turn>,
) {
  if let Some(MouseButtonInput {
    button: MouseButton::Left,
    state: ElementState::Pressed,
  }) = mouse_clicks.iter().last()
  {
    if let Ok(id) = q_highlighted.single() {
      commands.entity(id).insert(Activated).remove::<Highlight>();
      turn.1 = true;
    }
  }
}

fn check_for_taken_squares(
  mut commands: Commands,
  q_cells: Query<(Entity, &Cell), Without<Taken>>,
  q_edges: Query<Entity, With<Activated>>,
  materials: Res<Materials>,
  mut turn: ResMut<Turn>,
) {
  if turn.1 {
    let edges = q_edges.iter().collect::<Vec<Entity>>();
    let mut another_turn = false;

    for (id, Cell(loc, target)) in q_cells.iter() {
      println!("edges: {}", edges.len());
      if target.iter().all(|edge| edges.contains(edge)) {
        println!("winner!");
        commands
          .entity(id)
          .insert(Taken(turn.0))
          .insert_bundle(SpriteBundle {
            material: match turn.0 {
              Player::One => materials.player_one_material.clone(),
              Player::Two => materials.player_two_material.clone(),
            },
            sprite: Sprite::new(vec2(1.0, 1.0)),
            transform: Transform {
              translation: (*loc + vec2(0.5, 0.5)).extend(-0.1),
              ..Default::default()
            },
            ..Default::default()
          });

        another_turn = true;
      }
    }

    if !another_turn {
      *turn = match turn.0 {
        Player::One => Turn(Player::Two, false),
        Player::Two => Turn(Player::One, false),
      }
    } else {
      turn.1 = false;
    }
  }
}
