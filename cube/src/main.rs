use std::{collections::HashMap, fs::File};

use bevy::{
    pbr::NotShadowCaster,
    prelude::*,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_mod_picking::*;
use camera::{CameraPlugin, PanOrbitCamera};

#[cfg(not(target_arch="wasm32"))]
use rusqlite::Connection;

use skeleton::{make_bone_mesh, JointMatrix, Joint, BoneCube};

use serde::{Serialize, Deserialize};
use std::io::Write;

mod camera;
mod skeleton;
mod vector_ops;

#[derive(Component)]
struct MainMenu;

#[derive(Component)]
struct AsanaName;

#[derive(Component)]
struct ResetViewButton;
#[derive(Component)]
struct UpButton;
#[derive(Component)]
struct DownButton;

#[derive(Resource)]
pub struct YogaAssets {
    font: Handle<Font>,
    font_color: Color,
    asanas: Vec<AsanaDB>,
    current_idx: usize,
    poses: HashMap<i32, Vec<Joint>>,
}

#[derive(Component)]
struct Clickable;

#[derive(Component)]
struct Bone {
    id: i32,
}

#[derive(Component)]
struct BoneAxis;

#[derive(Debug, Serialize, Deserialize)]
struct AsanaDB {
    asana_id: i32,
    pose_id: i32,
    sanskrit: String,
    english: String,
    notes: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct AsanaData {
    asanas: Vec<AsanaDB>,
    poses: HashMap<i32, Vec<Joint>>,
}

//static DB: &[u8] = include_bytes!("../yogamatdb.sql");

fn main() {

    #[cfg(not(target_arch="wasm32"))]
    let width = 1290.0;
    #[cfg(not(target_arch="wasm32"))]
    let height = 1400.0;
    #[cfg(target_arch="wasm32")]
    let width = 645.0;
    #[cfg(target_arch="wasm32")]
    let height = 700.0;

    App::new()
        .insert_resource(ClearColor(Color::hex("292929").unwrap()))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "YogaMat".to_string(),
                width,
                height,
                position: WindowPosition::At(Vec2::new(0.0, 0.0)),
                ..default()
            },
            ..default()
        }))
        //.add_plugin(WorldInspectorPlugin)
        //.add_plugin(FilterQueryInspectorPlugin::<With<Bone>>::default())
        .register_type::<PanOrbitCamera>()
        .add_plugin(CameraPlugin)
        .add_plugins(DefaultPickingPlugins)
        //.add_plugin(bevy_transform_gizmo::TransformGizmoPlugin::new(Quat::default()))
        .add_startup_system(spawn_skeleton)
        .add_startup_system(spawn_camera)
        .add_startup_system(spawn_main_axis)
        .add_startup_system(setup_ui)
        .add_startup_system(spawn_mat)
        .add_system(keyboard_input_system)
        .add_system(button_clicked)
        .add_startup_system_to_stage(StartupStage::PreStartup, load_resources)
        .add_startup_system_to_stage(StartupStage::PostStartup, initial_pose)
        .run();
}

fn initial_pose(
    mut yoga_assets: ResMut<YogaAssets>,
    mut bones: Query<(&mut Transform, &Bone)>,
    mut asana_text: Query<&mut Text, With<AsanaName>>,
) {
        yoga_assets.current_idx = 127;
        let name = yoga_assets.asanas[yoga_assets.current_idx].sanskrit.clone();

        let name_text = TextSection::new(
            name.clone(),
            TextStyle {
                font: yoga_assets.font.clone(),
                font_size: 24.0,
                color: yoga_assets.font_color,
            }
        );

        let mut change_me = asana_text.single_mut();
        *change_me = Text::from_sections([name_text]);

        let pose_joints = load_pose(name, &yoga_assets);
        for (mut transform, bone) in bones.iter_mut() {
            let pose_mat = pose_joints.iter().find(|j| j.joint_id == bone.id).unwrap();
            *transform = Transform::from_matrix(pose_mat.mat);
        }
}

fn load_resources(mut commands: Commands, asset_server: Res<AssetServer>) {
    //serialize_db();
    let asana_data: AsanaData = deserialize_db();
    commands.insert_resource(YogaAssets {
        font: asset_server.load("fonts/Roboto-Regular.ttf"),
        font_color: Color::rgb_u8(207, 207, 207),
        asanas: asana_data.asanas,
        current_idx: 0,
        poses: asana_data.poses,
    });
}

fn keyboard_input_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut yoga_assets: ResMut<YogaAssets>,
    mut bones: Query<(&mut Transform, &Bone)>,
    mut asana_text: Query<&mut Text, With<AsanaName>>,
) {
    if keyboard_input.just_pressed(KeyCode::Up) {
        yoga_assets.current_idx += 1;
        if yoga_assets.current_idx > yoga_assets.asanas.len() - 1 {
            yoga_assets.current_idx = 0;
        }
        let name = yoga_assets.asanas[yoga_assets.current_idx].sanskrit.clone();

        let name_text = TextSection::new(
            name.clone(),
            TextStyle {
                font: yoga_assets.font.clone(),
                font_size: 24.0,
                color: yoga_assets.font_color,
            }
        );

        let mut change_me = asana_text.single_mut();
        *change_me = Text::from_sections([name_text]);

        let joints = load_pose(name, &yoga_assets);
        for (mut transform, bone) in bones.iter_mut() {
            let mat = joints.iter().find(|j| j.joint_id == bone.id).unwrap();
            *transform = Transform::from_matrix(mat.mat);
        }
    }

    if keyboard_input.just_pressed(KeyCode::Down) {
        if yoga_assets.current_idx == 0 {
            yoga_assets.current_idx = yoga_assets.asanas.len() - 1;
        } else {
            yoga_assets.current_idx -= 1;
        }
        let name = yoga_assets.asanas[yoga_assets.current_idx].sanskrit.clone();

        let name_text = TextSection::new(
            name.clone(),
            TextStyle {
                font: yoga_assets.font.clone(),
                font_size: 24.0,
                color: yoga_assets.font_color,
            }
        );
        let mut change_me = asana_text.single_mut();
        *change_me = Text::from_sections([name_text]);

        let joints = load_pose(name, &yoga_assets);
        for (mut transform, bone) in bones.iter_mut() {
            let mat = joints.iter().find(|j| j.joint_id == bone.id).unwrap();
            *transform = Transform::from_matrix(mat.mat);
        }
    }
}

#[cfg(not(target_arch="wasm32"))]
fn get_asanas_from_db() -> Vec<AsanaDB> {
    let path = "./yogamatdb.sql";
    let db = Connection::open(path).expect("couldn't open database");
    //let sql = "select asanaID, sanskritName, englishName, userNotes from asana";
    let sql = r#"
SELECT a.poseId, a.asanaID, b.sanskritName, b.englishName, b.userNotes
FROM pose a, asana b
WHERE a.asanaID = b.asanaID;
"#;
    let mut stmt = db.prepare(&sql).expect("trouble preparing statement");
    let response = stmt.query_map([], |row| {
        Ok(AsanaDB {
            pose_id: row.get(0).expect("poseId"), 
            asana_id: row.get(1).expect("asanaID"),
            sanskrit: row.get(2).expect("sanskritName"),
            english: row.get(3).expect("englishName"),
            notes: row.get(4).expect("userNotes"),
        })
    }).expect("bad");
    let asanas = response.filter_map(|result| result.ok()).collect::<Vec<AsanaDB>>();

    asanas
}

#[cfg(not(target_arch="wasm32"))]
fn serialize_db() {
    let asanas = get_asanas_from_db();
    let mut data = AsanaData {
        asanas,
        poses: HashMap::new(),
    };

    let path = "./yogamatdb.sql";
    let db = Connection::open(path).expect("couldn't open database");

    for asana in data.asanas.iter() {

        let sql = format!("select * from joint where poseID = {};", asana.pose_id);
        let mut stmt = db.prepare(&sql).expect("trouble preparing statement");
        let response = stmt.query_map([], |row| {
            Ok(Joint {
                joint_id: row.get(0).expect("so may results"),
                pose_id: row.get(1).expect("so may results"),
                up_x: row.get(2).expect("so may results"),
                up_y: row.get(3).expect("so may results"),
                up_z: row.get(4).expect("so may results"),
                forward_x: row.get(5).expect("so may results"),
                forward_y: row.get(6).expect("so may results"),
                forward_z: row.get(7).expect("so may results"),
                origin_x: row.get(8).expect("so may results"),
                origin_y: row.get(9).expect("so may results"),
                origin_z: row.get(10).expect("so may results"),
            })
        }).expect("bad");
        let joints = response.filter_map(|result| result.ok()).collect::<Vec<Joint>>();
        // do we store the matrices instead of these joints at some point in the future?
        /*let matrices = joints.iter().map(|joint| JointMatrix {
            mat: joint.matrix(),
            joint_id: joint.joint_id,
        }).collect::<Vec<JointMatrix>>();*/
        let already = data.poses.insert(asana.pose_id, joints);
        assert!(already.is_none());
    }

    let encoded: Vec<u8> = bincode::serialize(&data).unwrap();
    let mut out_file = File::options()
        .write(true)
        .create(true)
        .truncate(true)
        .open("out_db")
        .expect("file couldn't be opened");
    let success = out_file.write_all(&encoded);
    match success {
        Ok(_) => println!("encoded db written to out_db"),
        Err(_) => panic!("encoded db file write failed"),
    }
}

fn deserialize_db() -> AsanaData {
    // asanas: Vec<AsanaDB>,
    // poses: HashMap<i32, Vec<Joint>>,
    let db = include_bytes!("../out_db");
    let decoded = bincode::deserialize(db).unwrap();
    decoded
}

fn load_pose(sanskrit: String, yoga: &YogaAssets) -> Vec<JointMatrix> {
    let asana = yoga.asanas.iter().find(|asana| asana.sanskrit == sanskrit).unwrap();
    let joints = yoga.poses.get(&asana.pose_id).unwrap();
    let mats = joints.iter().map(|joint| JointMatrix {
        mat: joint.matrix(),
        joint_id: joint.joint_id,
    }).collect::<Vec<JointMatrix>>();
    mats
}

fn default_viewpoint() -> (Transform, Vec3) {
    let mut transform = Transform::default();
    transform.translation.x = 108.36059;
    transform.translation.y = -6.946327;
    transform.translation.z = -190.86304;
    transform.rotation.x = -0.019470416;
    transform.rotation.y = 0.96626204;
    transform.rotation.z = 0.076762356;
    transform.rotation.w = 0.2450841;
    let focus = Vec3 {
        x: 0.87379193,
        y: -43.005276,
        z: 7.39,
    };
    (transform, focus)
}

fn spawn_camera(mut commands: Commands) {
    let (transform, focus) = default_viewpoint();
    let camera = Camera3dBundle {
        transform,
        ..Default::default()
    };
    commands.spawn((
        camera,
        PanOrbitCamera {
            radius: (transform.translation - focus).length(),
            focus,
            ..Default::default()
        },
        PickingCameraBundle::default(),
        //bevy_transform_gizmo::GizmoPickSource::default(),
    ));
}

fn spawn_mat(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let material = StandardMaterial {
        base_color: Color::rgb(0.1, 0.1, 0.5).into(),
        reflectance: 0.2,
        perceptual_roughness: 0.95,
        ..Default::default()
    };
    let material_handle = materials.add(material);

    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(114.7, 1.0, 43.4))),
            material: material_handle,
            transform: Transform::from_xyz(0.0, -109.5, 0.0),
            ..default()
        });

    let height = 75.0;
    let lights = vec![Vec3::new(-50.0, height, 25.0), Vec3::new(50.0, height, 25.0),
    Vec3::new(-50.0, height, -25.0), Vec3::new(50.0, height, -25.0)];
    for (light_number, light_translation) in lights.into_iter().enumerate() {
        let rotation = Quat::from_rotation_x((-90.0_f32).to_radians());
        let light = commands.spawn(SpotLightBundle {
            spot_light: SpotLight {
                intensity: 400000.0,
                range: 250.0,
                radius: 45.0,
                shadows_enabled: true,
                ..Default::default()
            },
            transform: Transform::from_translation(light_translation).with_rotation(rotation),
            ..Default::default()
        }).insert(Name::from(format!("my spot {}", light_number))).id();
        commands.entity(light).add_children(|parent| {
            spawn_entity_axis(parent, &mut meshes, &mut materials, true);
        });
    }
}

fn spawn_bone(
    commands: &mut Commands,
    mut meshes: &mut ResMut<Assets<Mesh>>,
    material: Handle<StandardMaterial>,
    mut materials: &mut ResMut<Assets<StandardMaterial>>,
    bone_cube: &BoneCube,
    bone_id: i32,
    bone_parent: Entity,
    transform: Transform,
) -> Entity {
    let pickable = false;
    let new_bone = commands.entity(bone_parent).add_children(|parent| {
        if pickable {
            parent
                .spawn(PbrBundle {
                    mesh: meshes.add(make_bone_mesh(bone_cube)),
                    material,
                    transform,
                    ..default()
                })
                .insert(PickableBundle::default())
                //.insert(bevy_transform_gizmo::GizmoTransformable)
                .insert(Clickable)
                .insert(Name::from(bone_cube.name.clone()))
                .insert(Bone { id: bone_id })
                .id()
        } else {
            parent
                .spawn(PbrBundle {
                    mesh: meshes.add(make_bone_mesh(bone_cube)),
                    material,
                    transform,
                    ..default()
                })
                .insert(Clickable)
                .insert(Name::from(bone_cube.name.clone()))
                .insert(Bone { id: bone_id })
                .id()
            }
    });
    commands.entity(new_bone).add_children(|parent| {
        spawn_entity_axis(parent, &mut meshes, &mut materials, false);
    });
    new_bone
}

fn spawn_skeleton(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let material = StandardMaterial {
        base_color: Color::rgba_u8(166, 116, 51, 255).into(),
        //base_color: Color::rgba_u8(133, 0, 0, 255).into(),
        reflectance: 0.2,
        perceptual_roughness: 0.95,
        ..Default::default()
    };
    let material_handle = materials.add(material);

    let skeleton_parts = crate::skeleton::skelly();
    let mut bone_id = 1;
    let axis_visible = false;

    let name = "Hips".to_string();
    let mut bone = skeleton_parts.get(&name).unwrap();
    let hip_bone = bone;
    let mesh = make_bone_mesh(bone);
    let hips = commands
        .spawn(PbrBundle {
            mesh: meshes.add(mesh),
            material: material_handle.clone(),
            ..default()
        })
        .insert(PickableBundle::default())
        //.insert(bevy_transform_gizmo::GizmoTransformable)
        .insert(Clickable)
        .insert(Name::from(name))
        .insert(Bone { id: bone_id })
        .id();
    commands.entity(hips).add_children(|parent| {
        spawn_entity_axis(parent, &mut meshes, &mut materials, axis_visible);
    });
    bone_id += 1;

    let mut transform = Transform::IDENTITY;
    transform.translation += Vec3::new(7.5, -bone.y, 1.55);
    let name = "Left Femur".to_string();
    bone = skeleton_parts.get(&name).unwrap();
    let mut prev_entity = spawn_bone(&mut commands, &mut meshes, material_handle.clone(), &mut materials, bone, bone_id, hips, transform);
    bone_id += 1;

    let name = "Left Calf".to_string();
    let mut transform = Transform::IDENTITY;
    transform.translation += Vec3::new(0.0, -bone.y, 0.0);
    let bone = skeleton_parts.get(&name).unwrap();
    prev_entity = spawn_bone(&mut commands, &mut meshes, material_handle.clone(), &mut materials, bone, bone_id, prev_entity, transform);
    bone_id += 1;

    let name = "Left Foot".to_string();
    let mut transform = Transform::IDENTITY;
    transform.translation += Vec3::new(0.0, -bone.y, 0.0);
    let bone = skeleton_parts.get(&name).unwrap();
    _ = spawn_bone(&mut commands, &mut meshes, material_handle.clone(), &mut materials, bone, bone_id, prev_entity, transform);
    bone_id += 1;

    let name = "Right Femur".to_string();
    let mut transform = Transform::IDENTITY;
    transform.translation += Vec3::new(-7.5, -hip_bone.y, 1.55);
    let bone = skeleton_parts.get(&name).unwrap();
    prev_entity = spawn_bone(&mut commands, &mut meshes, material_handle.clone(), &mut materials, bone, bone_id, hips, transform);
    bone_id += 1;

    let name = "Right Calf".to_string();
    let mut transform = Transform::IDENTITY;
    transform.translation += Vec3::new(0.0, -bone.y, 0.0);
    let bone = skeleton_parts.get(&name).unwrap();
    prev_entity = spawn_bone(&mut commands, &mut meshes, material_handle.clone(), &mut materials, bone, bone_id, prev_entity, transform);
    bone_id += 1;

    let name = "Right Foot".to_string();
    let mut transform = Transform::IDENTITY;
    transform.translation += Vec3::new(0.0, -bone.y, 0.0);
    let bone = skeleton_parts.get(&name).unwrap();
    _ = spawn_bone(&mut commands, &mut meshes, material_handle.clone(), &mut materials, bone, bone_id, prev_entity, transform);
    bone_id += 1;

    let name = "Lumbar".to_string();
    let bone = skeleton_parts.get(&name).unwrap();
    prev_entity = hips;
    let l_spine_length = 9.0;

    for i in (1..=5).rev() {
        let mut transform = Transform::IDENTITY;
        if i == 5 {
            // this is 0, 0, 0 in original
            // /Users/matt/Documents/former_desktop/My\ PROJECT/shared\ source/Skeleton.m
            transform.translation += Vec3::new(0.0, -hip_bone.y, 0.0);
        } else {
            transform.translation += Vec3::new(0.0, -l_spine_length / 5.0, 0.0);
        }
        prev_entity = spawn_bone(&mut commands, &mut meshes, material_handle.clone(), &mut materials, bone, bone_id, prev_entity, transform);
        bone_id += 1;
    }

    let name = "Thoracic".to_string();
    let bone = skeleton_parts.get(&name).unwrap();
    let t_spine_length = 19.0;
    for i in (1..=12).rev() {
        let mut transform = Transform::IDENTITY;
        if i == 12 {
            transform.translation += Vec3::new(0.0, -l_spine_length / 5.0, 0.0);
        } else {
            transform.translation += Vec3::new(0.0, -t_spine_length / 12.0, 0.0);
        }
        prev_entity = spawn_bone(&mut commands, &mut meshes, material_handle.clone(), &mut materials, bone, bone_id, prev_entity, transform);
        bone_id += 1;
    }

    let name = "Cervical".to_string();
    let mut bone = skeleton_parts.get(&name).unwrap().clone();
    let c_spine_length = 8.0;
    let mut c7 = prev_entity;
    for i in (1..=7).rev() {
        let mut transform = Transform::IDENTITY;
        if i == 12 {
            transform.translation += Vec3::new(0.0, -t_spine_length / 12.0, 0.0);
        } else {
            transform.translation += Vec3::new(0.0, -c_spine_length / 7.0, 0.0);
        }
        bone.name = format!("{} {}", name, i);
        prev_entity = spawn_bone(&mut commands, &mut meshes, material_handle.clone(), &mut materials, &bone, bone_id, prev_entity, transform);
        if i == 7 {
            c7 = prev_entity;
        }
        bone_id += 1;
    }

    let name = "Head".to_string();
    let bone = skeleton_parts.get(&name).unwrap();
    let mut transform = Transform::IDENTITY;
    transform.translation += Vec3::new(0.0, -c_spine_length / 7.0, 0.0);
    _ = spawn_bone(&mut commands, &mut meshes, material_handle.clone(), &mut materials, bone, bone_id, c7, transform);
    bone_id += 1;

    info!("left clavical id {}", bone_id);
    let name = "Left Clavical".to_string();
    let bone = skeleton_parts.get(&name).unwrap();
    let mut transform = Transform::IDENTITY;
    transform.translation += Vec3::new(0.0, 0.0, -5.0);
    prev_entity = spawn_bone(&mut commands, &mut meshes, material_handle.clone(), &mut materials, bone, bone_id, c7, transform);
    bone_id += 1;

    let name = "Left Arm".to_string();
    let mut transform = Transform::IDENTITY;
    transform.translation += Vec3::new(0.0, -bone.y, 0.0);
    let bone = skeleton_parts.get(&name).unwrap();
    prev_entity = spawn_bone(&mut commands, &mut meshes, material_handle.clone(), &mut materials, bone, bone_id, prev_entity, transform);
    bone_id += 1;

    let name = "Left Forearm".to_string();
    let mut transform = Transform::IDENTITY;
    transform.translation += Vec3::new(0.0, -bone.y, 0.0);
    let bone = skeleton_parts.get(&name).unwrap();
    prev_entity = spawn_bone(&mut commands, &mut meshes, material_handle.clone(), &mut materials, bone, bone_id, prev_entity, transform);
    bone_id += 1;

    let name = "Left Hand".to_string();
    let mut transform = Transform::IDENTITY;
    transform.translation += Vec3::new(0.0, -bone.y, 0.0);
    let bone = skeleton_parts.get(&name).unwrap();
    _ = spawn_bone(&mut commands, &mut meshes, material_handle.clone(), &mut materials, bone, bone_id, prev_entity, transform);
    bone_id += 1;

    prev_entity = c7;

    info!("right clavical id {}", bone_id);
    let name = "Right Clavical".to_string();
    let mut transform = Transform::IDENTITY;
    transform.translation += Vec3::new(0.0, 0.0, -5.0);
    let bone = skeleton_parts.get(&name).unwrap();
    prev_entity = spawn_bone(&mut commands, &mut meshes, material_handle.clone(), &mut materials, bone, bone_id, prev_entity, transform);
    bone_id += 1;

    let name = "Right Arm".to_string();
    let mut transform = Transform::IDENTITY;
    transform.translation += Vec3::new(0.0, -bone.y, 0.0);
    let bone = skeleton_parts.get(&name).unwrap();
    prev_entity = spawn_bone(&mut commands, &mut meshes, material_handle.clone(), &mut materials, bone, bone_id, prev_entity, transform);
    bone_id += 1;

    let name = "Right Forearm".to_string();
    let mut transform = Transform::IDENTITY;
    transform.translation += Vec3::new(0.0, -bone.y, 0.0);
    let bone = skeleton_parts.get(&name).unwrap();
    prev_entity = spawn_bone(&mut commands, &mut meshes, material_handle.clone(), &mut materials, bone, bone_id, prev_entity, transform);
    bone_id += 1;

    let name = "Right Hand".to_string();
    let mut transform = Transform::IDENTITY;
    transform.translation += Vec3::new(0.0, -bone.y, 0.0);
    let bone = skeleton_parts.get(&name).unwrap();
    _ = spawn_bone(&mut commands, &mut meshes, material_handle.clone(), &mut materials, bone, bone_id, prev_entity, transform);
}

fn button_clicked(
    mut query: Query<(&mut PanOrbitCamera, &mut Transform)>,
    interactions: Query<&Interaction, (With<ResetViewButton>, Changed<Interaction>)>,
) {
    for interaction in &interactions {
        if matches!(interaction, Interaction::Clicked) {
            let (default_transform, default_focus) = default_viewpoint();
            if let Ok((mut pan_orbit, mut transform)) = query.get_single_mut() {
                pan_orbit.focus = default_focus;
                *transform = default_transform;
                pan_orbit.upside_down = false;
                pan_orbit.radius = (transform.translation - default_focus).length();
            }
        }
    }
}

fn setup_ui(
    mut commands: Commands,
    my_assets: Res<YogaAssets>,
) {
    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::FlexStart,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ..default()
        })
        .insert(MainMenu)
        .insert(Name::new("Yoga Menu"))
        .with_children(|commands| {
            commands.spawn(TextBundle {
                style: Style {
                    align_self: AlignSelf::Center,
                    margin: UiRect::all(Val::Percent(1.0)),
                    ..default()
                },
                text: Text::from_section(
                    "YogaMat Lives!",
                    TextStyle {
                        font: my_assets.font.clone(),
                        font_size: 30.0,
                        color: my_assets.font_color,
                        ..Default::default()
                    },
                ),
                ..default()
            })
            .insert(AsanaName)
            .insert(Name::new("AsanaName"));

        let button_margin = UiRect::all(Val::Percent(2.0));
        commands
            .spawn(ButtonBundle {
                style: Style {
                    size: Size::new(Val::Px(80.0), Val::Px(40.0)),
                    align_self: AlignSelf::FlexEnd,
                    justify_content: JustifyContent::FlexStart,
                    margin: button_margin,
                    ..default()
                },
                background_color: Color::rgb_u8(28, 31, 33).into(),
                //image: img.into(),
                ..default()
            }).insert(ResetViewButton)
            .with_children(|commands| {
                commands.spawn(TextBundle {
                    style: Style {
                        align_self: AlignSelf::Center,
                        justify_content: JustifyContent::Center,
                        margin: UiRect::all(Val::Percent(3.0)),
                        ..default()
                    },
                    text: Text::from_section(
                        "Reset View",
                        TextStyle {
                            font: my_assets.font.clone(),
                            font_size: 18.0,
                            color: my_assets.font_color,
                            ..Default::default()
                        },
                    ),
                    ..default()
                });
            });
        });
}

fn spawn_entity_axis(
    commands: &mut ChildBuilder,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    initial_visibility: bool,
) {
    let length = 7.0;
    let width = 0.1;
    //let x = Box::new(x_length, y_length, z_length);
    let x = shape::Box::new(length, width, width);
    let y = shape::Box::new(width, length, width);
    let z = shape::Box::new(width, width, length);

    let mut empty = commands.spawn_empty();
    empty
        .insert(TransformBundle::from_transform(Transform::IDENTITY))
        .insert(Visibility { is_visible: initial_visibility })
        .insert(ComputedVisibility::default())
        .insert(Name::from("bone axis"));

    let mut transform = Transform::default();
    transform.translation.x = length / 2.0;

    empty.add_children(|parent| {
        parent
            .spawn((
                PbrBundle {
                    mesh: meshes.add(Mesh::from(x)),
                    material: materials.add(Color::rgb(1.0, 0.0, 0.0).into()),
                    transform,
                    ..default()
                },
                NotShadowCaster,
                BoneAxis,
            ))
            .insert(Name::from("x axis"));
        let mut transform = Transform::default();
        transform.translation.y = length / 2.0;
        parent
            .spawn((
                PbrBundle {
                    mesh: meshes.add(Mesh::from(y)),
                    material: materials.add(Color::rgb(0.0, 1.0, 0.0).into()),
                    transform,
                    ..default()
                },
                NotShadowCaster,
                BoneAxis,
            ))
            .insert(Name::from("y axis"));
        let mut transform = Transform::default();
        transform.translation.z = length / 2.0;
        parent
            .spawn((
                PbrBundle {
                    mesh: meshes.add(Mesh::from(z)),
                    material: materials.add(Color::rgb(0.0, 0.0, 1.0).into()),
                    transform,
                    ..default()
                },
                NotShadowCaster,
                BoneAxis,
            ))
            .insert(Name::from("z axis"));
    });
}

fn spawn_main_axis(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let length = 200.0;
    let width = 0.1;
    //let x = Box::new(x_length, y_length, z_length);
    let x = shape::Box::new(length, width, width);
    let y = shape::Box::new(width, length, width);
    let z = shape::Box::new(width, width, length);

    let empty_transform = Transform::from_translation(Vec3::ZERO);
    let empty: Entity = commands
        .spawn_empty()
        .insert(TransformBundle::from_transform(empty_transform))
        .insert(Visibility { is_visible: false })
        .insert(ComputedVisibility::default())
        .insert(Name::from("Main Axis"))
        .id();

    let mut transform = Transform::default();
    transform.translation.x = length / 2.0;

    commands.entity(empty).add_children(|parent| {
        parent
            .spawn((
                PbrBundle {
                    mesh: meshes.add(Mesh::from(x)),
                    material: materials.add(Color::rgb(1.0, 0.0, 0.0).into()),
                    transform,
                    visibility: Visibility { is_visible: true },
                    ..default()
                },
                NotShadowCaster,
                BoneAxis,
            ))
            .insert(Name::from("x axis"));
        let mut transform = Transform::default();
        transform.translation.y = length / 2.0;
        parent
            .spawn((
                PbrBundle {
                    mesh: meshes.add(Mesh::from(y)),
                    material: materials.add(Color::rgb(0.0, 1.0, 0.0).into()),
                    transform,
                    visibility: Visibility { is_visible: true },
                    ..default()
                },
                NotShadowCaster,
                BoneAxis,
            ))
            .insert(Name::from("y axis"));
        let mut transform = Transform::default();
        transform.translation.z = length / 2.0;
        parent
            .spawn((
                PbrBundle {
                    mesh: meshes.add(Mesh::from(z)),
                    material: materials.add(Color::rgb(0.0, 0.0, 1.0).into()),
                    transform,
                    visibility: Visibility { is_visible: true },
                    ..default()
                },
                NotShadowCaster,
                BoneAxis,
            ))
            .insert(Name::from("z axis"));
    });
}
