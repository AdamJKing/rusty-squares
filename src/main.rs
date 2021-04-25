mod centred_cursor;
use crate::centred_cursor::CentredCursor;
use crate::centred_cursor::CursorState;
use bevy::prelude::*;

struct Grid(usize, usize);

struct Highlight;

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: "Squares".to_string(),
            width: 710.0,
            height: 710.0,
            ..Default::default()
        })
        .insert_resource(ClearColor(Color::rgb(1.0, 1.0, 1.0)))
        .add_plugins(DefaultPlugins)
        .add_plugin(CentredCursor)
        .add_startup_system(load_assets.system())
        .add_system(manage_highlights.system())
        .run();
}

fn load_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    println!("Started");
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let dot_material = materials.add(asset_server.load("black-dot.png").into());

    let mut points = Vec::new();

    for x in 0..7 {
        for y in 0..7 {
            points.push(SpriteBundle {
                transform: Transform {
                    scale: [0.02, 0.02, 1.0].into(),
                    translation: translate(x, y),
                    ..Default::default()
                },
                material: dot_material.clone(),
                ..Default::default()
            });

            if y != 6 {
                commands
                    .spawn_bundle(SpriteBundle {
                        material: materials.add(Color::rgb(0.0, 0.0, 0.0).into()),
                        sprite: Sprite::new([10.0, 90.0].into()),
                        transform: Transform {
                            translation: translate(x, y) + [0.0, 50.0, 0.0].into(),
                            ..Default::default()
                        },
                        visible: Visible {
                            is_visible: false,
                            is_transparent: false,
                        },
                        ..Default::default()
                    })
                    .insert(Highlight);
            }

            if x != 6 {
                commands
                    .spawn_bundle(SpriteBundle {
                        material: materials.add(Color::rgb(0.0, 0.0, 0.0).into()),
                        sprite: Sprite::new([100.0, 10.0].into()),
                        transform: Transform {
                            translation: translate(x, y) + [50.0, 0.0, 0.0].into(),
                            ..Default::default()
                        },
                        visible: Visible {
                            is_visible: false,
                            is_transparent: false,
                        },
                        ..Default::default()
                    })
                    .insert(Highlight);
            }
        }
    }

    commands.spawn_batch(points);
    commands.spawn().insert(Grid(7, 7));
}

fn manage_highlights(
    mut highlights: Query<(&Transform, &Sprite, &mut Visible), With<Highlight>>,
    mouse_movement: Res<CursorState>,
) {
    for (transform, sprite, mut visible) in highlights.iter_mut() {
        visible.is_visible = (transform, sprite).is_hit_by(mouse_movement.position);
    }
}

fn translate(x: usize, y: usize) -> Vec3 {
    let _x = (x as f32 * 100.0) - 300.0;
    let _y = (y as f32 * 100.0) - 300.0;

    [_x, _y, 0.0].into()
}

trait HitBox<I> {
    fn is_hit_by(&self, coord: I) -> bool;
}

impl HitBox<Vec2> for (&Transform, &Sprite) {
    fn is_hit_by(&self, coord: Vec2) -> bool {
        let centre = self.0.translation.truncate();
        let dist = self.1.size / 2.0;
        let upper_right = centre + dist;
        let lower_left = centre - dist;

        (lower_left, upper_right).is_hit_by(coord)
    }
}

impl HitBox<Vec2> for (Vec2, Vec2) {
    fn is_hit_by(&self, coord: Vec2) -> bool {
        (coord.cmpgt(self.0) & coord.cmplt(self.1)).all()
    }
}

#[cfg(test)]
mod tests {

    use super::HitBox;
    use bevy::math::vec2;

    #[test]
    fn hitbox_positive() {
        assert!((vec2(-10.0, -10.0), vec2(10.0, 10.0)).is_hit_by(vec2(0.0, 0.0)));
    }

    #[test]
    fn hitbox_negative() {
        assert!(!(vec2(-10.0, -10.0), vec2(10.0, 10.0)).is_hit_by(vec2(20.0, 0.0)));
        assert!(!(vec2(-10.0, -10.0), vec2(10.0, 10.0)).is_hit_by(vec2(-20.0, 0.0)));
        assert!(!(vec2(-10.0, -10.0), vec2(10.0, 10.0)).is_hit_by(vec2(0.0, 20.0)));
        assert!(!(vec2(-10.0, -10.0), vec2(10.0, 10.0)).is_hit_by(vec2(0.0, -20.0)));
    }
}
