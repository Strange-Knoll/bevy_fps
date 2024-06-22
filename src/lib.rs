// ******************************* //
// this is a blatant copy of the character controller example
// on the bevy_rapier32 github examples repo.
// you can find it here.
// https://github.com/dimforge/bevy_rapier/blob/master/bevy_rapier3d/examples/character_controller3.rs
//
// all this code does is turn the example into a plugin


use bevy::{input::mouse::MouseMotion, prelude::*};
use bevy_rapier3d::{control::{KinematicCharacterController, KinematicCharacterControllerOutput}, dynamics::RigidBody, geometry::Collider };

#[derive(Default, Resource, Deref, DerefMut)]
struct MouseSensitivity(f32);
#[derive(Default, Resource, Deref, DerefMut)]
struct GroundTimer(f32);
#[derive(Default, Resource, Deref, DerefMut)]
struct MovementSpeed(f32);
#[derive(Default, Resource, Deref, DerefMut)]
struct JumpSpeed (f32);
#[derive(Default, Resource, Deref, DerefMut)]
struct Gravity(f32);

pub struct FpsPlugin{
    mouse_sensitivity: f32,
    ground_timer: f32,
    movement_speed: f32,
    jump_speed: f32,
    gravity: f32
}

impl Default for FpsPlugin{
    fn default() -> Self {
        FpsPlugin{
            gravity: -9.81,
            ground_timer: 0.5,
            jump_speed: 8.0,
            mouse_sensitivity: 0.3,
            movement_speed: 8.0
        }
    }
}

impl Plugin for FpsPlugin{
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
        .insert_resource(Gravity(self.gravity.clone()))
        .insert_resource(GroundTimer(self.ground_timer.clone()))
        .insert_resource(JumpSpeed(self.jump_speed.clone()))
        .insert_resource(MouseSensitivity(self.mouse_sensitivity.clone()))
        .insert_resource(MovementSpeed(self.movement_speed.clone()))
        .init_resource::<MovementInput>()
        .init_resource::<LookInput>()
        .add_systems(Update, (
            handle_input,
            player_movement,
            player_look
        ));
    }
}


#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct Fps();

#[derive(Bundle)]
pub struct FpsBundle{
    fps:Fps,
    transform:TransformBundle,
    kinematic:KinematicCharacterController,
    rigidbody:RigidBody,
    collider:Collider
}

impl Default for FpsBundle{
    fn default() -> Self {
        FpsBundle{ 
            fps:Fps(),
            transform:TransformBundle::from(Transform::from_xyz(0.0, 2.0, 0.0)),
            kinematic: KinematicCharacterController::default(),
            rigidbody:RigidBody::KinematicPositionBased,
            collider:Collider::capsule_y(1.0, 0.5)
        }
    }
}


//  --------FUNCTIONS--------  //
fn setup(mut commands: Commands){
    commands.spawn(FpsBundle::default()).with_children(|b|{
        b.spawn(Camera3dBundle {
            transform: Transform::from_xyz(0.0, 2.0, 0.0),
            ..Default::default()
        });
    });
}

#[derive(Default, Resource, Deref, DerefMut)]
struct MovementInput(Vec3);

#[derive(Default, Resource, Deref, DerefMut)]
struct LookInput(Vec2);



fn handle_input(
    keyboard:Res<ButtonInput<KeyCode>>,
    mut movement: ResMut<MovementInput>,
    mut look: ResMut<LookInput>,
    mut mouse_event: EventReader<MouseMotion>,
    mouse_sensitivity: Res<MouseSensitivity>
){
    if keyboard.pressed(KeyCode::KeyW){
        movement.z -= 1.0;
    }
     if keyboard.pressed(KeyCode::KeyS){
        movement.z += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA){
        movement.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD){
        movement.x += 1.0;
    }

    **movement = movement.normalize_or_zero();

    if keyboard.pressed(KeyCode::ShiftLeft){
        **movement *= 2.0
    }

    if keyboard.pressed(KeyCode::Space){
        movement.y = 1.0;
    }

    for event in mouse_event.read() {
        look.x -= event.delta.x * **mouse_sensitivity;
        look.y -= event.delta.y * **mouse_sensitivity;
        look.y = look.y.clamp(-89.9, 89.9);
    }
}

fn player_movement(
    time: Res<Time>,
    mut input: ResMut<MovementInput>,
    mut player: Query<(
        &Fps,
        &mut Transform,
        &mut KinematicCharacterController,
        Option<&KinematicCharacterControllerOutput>,
    )>,
    mut vertical_movement: Local<f32>,
    mut grounded_timer: Local<f32>,
    movement_speed:Res<MovementSpeed>,
    jump_speed: Res<JumpSpeed>,
    res_ground_timer: Res<GroundTimer>,
    gravity: Res<Gravity>

){
    let Ok((_, transform, mut controller, output)) = player.get_single_mut() else {
        return;
    };
    let delta_time = time.delta_seconds();
    // Retrieve input
    let mut movement = Vec3::new(input.x, 0.0, input.z) * **movement_speed;
    let jump_speed = input.y * **jump_speed;
    // Clear input
    **input = Vec3::ZERO;
    // Check physics ground check
    if output.map(|o| o.grounded).unwrap_or(false) {
        *grounded_timer = **res_ground_timer;
        *vertical_movement = 0.0;
    }
    // If we are grounded we can jump
    if *grounded_timer > 0.0 {
        *grounded_timer -= delta_time;
        // If we jump we clear the grounded tolerance
        if jump_speed > 0.0 {
            *vertical_movement = jump_speed;
            *grounded_timer = 0.0;
        }
    }
    movement.y = *vertical_movement;
    *vertical_movement += **gravity * delta_time * controller.custom_mass.unwrap_or(1.0);
    controller.translation = Some(transform.rotation * (movement * delta_time));

}

fn player_look(
    mut player: Query<&mut Transform, (With<KinematicCharacterController>, Without<Camera>)>,
    mut camera: Query<&mut Transform, With<Camera>>,
    input: Res<LookInput>,
) {
    let Ok(mut transform) = player.get_single_mut() else {
        return;
    };
    transform.rotation = Quat::from_axis_angle(Vec3::Y, input.x.to_radians());
    let Ok(mut transform) = camera.get_single_mut() else {
        return;
    };
    transform.rotation = Quat::from_axis_angle(Vec3::X, input.y.to_radians());
}