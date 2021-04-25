use bevy::prelude::*;
use bevy::render::camera::Camera;

pub struct CentredCursor;

pub struct CursorState {
    pub position: Vec2,
}

impl Plugin for CentredCursor {
    fn build(&self, app: &mut bevy::prelude::AppBuilder) {
        app.insert_resource(CursorState {
            position: [0.0, 0.0].into(),
        })
        .add_system(locate_cursor.system());
    }
}

// https://stackoverflow.com/questions/65396065/what-is-an-acceptable-approach-to-dragging-sprites-with-bevy-0-4
fn locate_cursor(
    mut cursor_state: ResMut<CursorState>,
    mut mouse_movement: EventReader<CursorMoved>,
    window: Res<WindowDescriptor>,
    q_camera: Query<&Transform, With<Camera>>,
) {
    let camera = q_camera.single().unwrap();

    if let Some(mouse) = mouse_movement.iter().last() {
        // get the size of the window
        let size = Vec2::new(window.width as f32, window.height as f32);

        // the default orthographic projection is in pixels from the center;
        // just undo the translation
        let screen_pos = mouse.position - size / 2.0;

        // apply the camera transform
        let out = camera.compute_matrix() * screen_pos.extend(0.0).extend(1.0);
        cursor_state.position = Vec2::new(out.x, out.y);
    }
}
