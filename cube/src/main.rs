use std::collections::HashMap;

use bevy::{
    pbr::NotShadowCaster,
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use bevy_mod_picking::*;
use camera::{CameraPlugin, PanOrbitCamera};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_inspector_egui::quick::FilterQueryInspectorPlugin;
use rusqlite::Connection;

mod camera;

#[derive(Component)]
struct LoadButton;

#[derive(Resource)]
struct YogaResources;

#[derive(Component)]
struct Clickable;

#[derive(Component)]
struct Bone {
    id: i32,
}

#[derive(Component)]
struct VisibleAxis;

#[derive(Debug)]
struct AsanaDB {
    asana_id: i32,
    sanskrit: String,
    english: String,
    notes: Option<String>,
}

struct JointMatrix {
    mat: Mat4,
    joint_id: i32,
}

struct BoneCube {
    x_top: f32,
    x_bottom: f32,
    z_top: f32,
    z_bottom: f32,
    y: f32,
    inset: f32,
    transform: Transform,
}

#[derive(Default)]
struct Joint {
    joint_id: i32,
    pose_id: i32,
    up_x: f32,
    up_y: f32,
    up_z: f32,
    forward_x: f32,
    forward_y: f32,
    forward_z: f32,
    origin_x: f32,
    origin_y: f32,
    origin_z: f32,
    angle_x: f32,
    angle_y: f32,
    angle_z: f32,
}

/*
CREATE TABLE asana (asanaID INTEGER PRIMARY KEY, sanskritName TEXT, englishName TEXT, userNotes TEXT);
CREATE TABLE pose ( poseID INTEGER PRIMARY KEY, asanaID INTEGER);
152|Tadasana|Mountain|

CREATE TABLE joint (
jointID INTEGER,
poseID INTEGER,
upX REAL,
upY REAL,
upZ REAL,
forwardX REAL,
forwardY REAL,
forwardZ REAL,
originX REAL,
originY REAL,
originZ REAL,
xAngle REAL,
yAngle REAL,
zAngle REAL,
PRIMARY KEY (jointID, poseID)
);
*/




fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.2, 0.2, 0.2)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "YogaMat".to_string(),
                width: 800.0,
                height: 800.0,
                ..default()
            },
            ..default()
        }))
        .add_plugin(WorldInspectorPlugin)
        .add_plugin(FilterQueryInspectorPlugin::<With<Bone>>::default())
        .insert_resource(YogaResources)
        .add_plugin(CameraPlugin)
        .add_plugins(DefaultPickingPlugins)
        //.add_plugin(DebugCursorPickingPlugin) // <- Adds the debug cursor (optional)
        //.add_plugin(DebugEventsPickingPlugin) // <- Adds debug event logging (optional)
        .add_startup_system(spawn_skeleton)
        .add_startup_system(spawn_camera)
        .add_startup_system(spawn_axis)
        .add_startup_system(setup_ui)
        .add_system(button_clicked)
        //.add_system(cube_click)
        .run();
}

fn button_clicked(
    mut commands: Commands,
    interactions: Query<&Interaction, (With<LoadButton>, Changed<Interaction>)>,
    menu_root: Query<Entity, With<LoadButton>>,
) {
    for interaction in &interactions {
        if matches!(interaction, Interaction::Clicked) {
            println!("button");
            let joints = load_pose("Tadasana".to_string());
            println!("got {} joints", joints.len());
        }
    }
}

fn load_pose(sanskrit: String) -> Vec<JointMatrix> {
    let path = "./yogamatdb.sql";
    let db = Connection::open(path).expect("couldn't open database");

    let sql = format!("select * from asana where sanskritName = '{}'", sanskrit);
    let mut stmt = db.prepare(&sql).expect("trouble preparing statement");
    let response = stmt.query_map([], |row| {
        Ok(AsanaDB {
            asana_id: row.get(0).expect("so may results"),
            sanskrit: row.get(1).expect("so may results"),
            english: row.get(2).expect("so may results"),
            notes: row.get(3).expect("so may results"),
        })
    }).expect("bad");
    let asanas = response.filter_map(|result| result.ok()).collect::<Vec<AsanaDB>>();

    for asana in &asanas {
        println!("Found asana {:?}", asana);
    }

    let sql = format!("select poseId from pose where asanaID = {};", asanas[0].asana_id);
    let mut stmt = db.prepare(&sql).expect("trouble preparing statement");
    let response = stmt.query_map([], |row| {
        let pose_id: i32 = row.get(0).expect("so may results");
        Ok(pose_id)
    }).expect("bad");
    let pose_ids = response.filter_map(|result| result.ok()).collect::<Vec<i32>>();
    
    println!("pose id {}", pose_ids[0]);

    let sql = format!("select * from joint where poseID = {};", pose_ids[0]);
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
            angle_x: row.get(11).expect("so may results"),
            angle_y: row.get(12).expect("so may results"),
            angle_z: row.get(13).expect("so may results"),
        })
    }).expect("bad");
    let joints = response.filter_map(|result| result.ok()).collect::<Vec<Joint>>();
    let mats = joints.iter().map(|joint| JointMatrix {
        mat: matrix_from_frame(joint),
        joint_id: joint.joint_id,
    }).collect::<Vec<JointMatrix>>();

    mats
}

fn skelly() -> HashMap<String, BoneCube> {
    let head_length = 11.0;
    let clavical_lengh = 9.5;
    let c_spine_length = 8.0;
    let t_spine_length = 19.0;
    let l_spine_length = 9.0;
    let hip_length = 12.0; //6.0; // half transofrm y was -4.0
    let femur_length = 29.0;
    let calf_length = 26.0;
    let foot_length = 15.0;
    let humerus_length = 19.0;
    let forearm_length = 17.0;
    let hand_length = 11.0;
    let the_inset = 0.75;
    let mut map = HashMap::new();

    map.insert(
        "Hips".to_string(),
        BoneCube {
            x_top: 15.0,
            x_bottom: 13.0,
            z_top: 6.0,
            z_bottom: 4.0,
            y: hip_length,
            inset: 1.0,
            transform: Transform::from_xyz(0.0, -hip_length / 2.0, 0.0),
        },
    );

    map.insert(
        "Left Femur".to_string(),
        BoneCube {
            x_top: 6.0,
            x_bottom: 4.0,
            z_top: 6.0,
            z_bottom: 4.0,
            y: femur_length,
            inset: 1.25,
            transform: Transform::from_xyz(0.0, -femur_length / 2.0, 0.0),
        },
    );

    map.insert(
        "Right Femur".to_string(),
        BoneCube {
            x_top: 6.0,
            x_bottom: 4.0,
            z_top: 6.0,
            z_bottom: 4.0,
            y: femur_length,
            inset: 1.25,
            transform: Transform::from_xyz(0.0, -femur_length / 2.0, 0.0),
        },
    );

    map.insert(
        "Left Calf".to_string(),
        BoneCube {
            x_top: 4.5,
            x_bottom: 2.5,
            z_top: 4.5,
            z_bottom: 2.5,
            y: calf_length,
            inset: the_inset,
            transform: Transform::from_xyz(0.0, -calf_length / 2.0, 0.0),
        },
    );

    map.insert(
        "Right Calf".to_string(),
        BoneCube {
            x_top: 4.5,
            x_bottom: 2.5,
            z_top: 4.5,
            z_bottom: 2.5,
            y: calf_length,
            inset: the_inset,
            transform: Transform::from_xyz(0.0, -calf_length / 2.0, 0.0),
        },
    );

    map.insert(
        "Left Foot".to_string(),
        BoneCube {
            x_top: 3.5,
            x_bottom: 6.0,
            z_top: 3.5,
            z_bottom: 2.0,
            y: foot_length,
            inset: the_inset,
            transform: Transform::from_xyz(0.0, -foot_length / 2.0, 0.0),
        },
    );

    map.insert(
        "Right Foot".to_string(),
        BoneCube {
            x_top: 3.5,
            x_bottom: 6.0,
            z_top: 3.5,
            z_bottom: 2.0,
            y: foot_length,
            inset: the_inset,
            transform: Transform::from_xyz(0.0, -foot_length / 2.0, 0.0),
        },
    );

    map.insert(
        "Lumbar".to_string(),
        BoneCube {
            x_top: 3.0,
            x_bottom: 3.0,
            z_top: 3.0,
            z_bottom: 3.0,
            y: (l_spine_length / 5.0),
            inset: 0.5,
            transform: Transform::from_xyz(0.0, -(l_spine_length / 5.0) / 2.0, 0.0),
        },
    );

    map.insert(
        "Thoracic".to_string(),
        BoneCube {
            x_top: 3.0,
            x_bottom: 3.0,
            z_top: 3.0,
            z_bottom: 3.0,
            y: t_spine_length / 12.0,
            inset: 0.5,
            transform: Transform::from_xyz(0.0, -(t_spine_length / 12.0) / 2.0, 0.0),
        },
    );

    map.insert(
        "Cervical".to_string(),
        BoneCube {
            x_top: 3.0,
            x_bottom: 3.0,
            z_top: 3.0,
            z_bottom: 3.0,
            y: (c_spine_length / 7.0),
            inset: 0.5,
            transform: Transform::from_xyz(0.0, -(c_spine_length / 7.0) / 2.0, 0.0),
        },
    );

    map.insert(
        "Head".to_string(),
        BoneCube {
            x_top: 12.0,
            x_bottom: 12.0,
            z_top: 12.0,
            z_bottom: 12.0,
            y: head_length,
            inset: 2.5,
            transform: Transform::from_xyz(0.0, head_length / 2.0, 0.0),
        },
    );

    map.insert(
        "Left Clavical".to_string(),
        BoneCube {
            x_top: 2.0,
            x_bottom: 2.0,
            z_top: 2.0,
            z_bottom: 2.0,
            y: clavical_lengh,
            inset: the_inset,
            transform: Transform::from_xyz(0.0, -clavical_lengh / 2.0, 0.0),
        },
    );

    map.insert(
        "Right Clavical".to_string(),
        BoneCube {
            x_top: 2.0,
            x_bottom: 2.0,
            z_top: 2.0,
            z_bottom: 2.0,
            y: clavical_lengh,
            inset: the_inset,
            transform: Transform::from_xyz(0.0, -clavical_lengh / 2.0, 0.0),
        },
    );

    map.insert(
        "Left Arm".to_string(),
        BoneCube {
            x_top: 4.20,
            x_bottom: 3.25,
            z_top: 5.075,
            z_bottom: 3.5,
            y: humerus_length,
            inset: 1.0,
            transform: Transform::from_xyz(0.0, -humerus_length / 2.0, 0.0),
        },
    );

    map.insert(
        "Right Arm".to_string(),
        BoneCube {
            x_top: 4.20,
            x_bottom: 3.25,
            z_top: 5.075,
            z_bottom: 3.5,
            y: humerus_length,
            inset: 1.0,
            transform: Transform::from_xyz(0.0, -humerus_length / 2.0, 0.0),
        },
    );

    map.insert(
        "Left Forearm".to_string(),
        BoneCube {
            x_top: 3.75,
            x_bottom: 2.75,
            z_top: 3.75,
            z_bottom: 2.75,
            y: forearm_length,
            inset: 1.0,
            transform: Transform::from_xyz(0.0, -forearm_length / 2.0, 0.0),
        },
    );

    map.insert(
        "Right Forearm".to_string(),
        BoneCube {
            x_top: 3.75,
            x_bottom: 2.75,
            z_top: 3.75,
            z_bottom: 2.75,
            y: forearm_length,
            inset: 1.0,
            transform: Transform::from_xyz(0.0, -forearm_length / 2.0, 0.0),
        },
    );

    map.insert(
        "Left Hand".to_string(),
        BoneCube {
            x_top: 3.0,
            x_bottom: 2.0,
            z_top: 4.0,
            z_bottom: 5.0,
            y: hand_length,
            inset: the_inset,
            transform: Transform::from_xyz(0.0, -hand_length / 2.0, 0.0),
        },
    );

    map.insert(
        "Right Hand".to_string(),
        BoneCube {
            x_top: 3.0,
            x_bottom: 2.0,
            z_top: 4.0,
            z_bottom: 5.0,
            y: hand_length,
            inset: the_inset,
            transform: Transform::from_xyz(0.0, -hand_length / 2.0, 0.0),
        },
    );

    map
}

#[rustfmt::skip]
fn make_bone_mesh(cube: &BoneCube) -> Mesh {
    let mut corners = Vec::new();
    let x_top = cube.x_top;
    let x_bottom = cube.x_bottom;
    let z_top = cube.z_top;
    let z_bottom = cube.z_bottom;
    let y = cube.y;
    let inset = cube.inset;
	let half_xtop = x_top / 2.0;
	let half_ztop = z_top / 2.0;
	let half_xbottom = x_bottom / 2.0;
	let half_zbottom = z_bottom / 2.0;

	corners.push([-x_top/2.0+inset, y/2.0, half_ztop-inset]);
	corners.push([-half_xtop+inset, y/2.0, -half_ztop+inset]);
	corners.push([half_xtop-inset, y/2.0, -half_ztop+inset]);
	corners.push([half_xtop-inset, y/2.0 , half_ztop-inset]);
	
	corners.push([-half_xbottom+inset, -y/2.0, -half_zbottom+inset]);
	corners.push([half_xbottom-inset, -y/2.0, -half_zbottom+inset]);
	corners.push([half_xbottom-inset, -y/2.0, half_zbottom-inset]);
	corners.push([-half_xbottom+inset, -y/2.0, half_zbottom-inset]);
	
	corners.push([-half_xtop+inset, y/2.0-inset, -half_ztop]);
	corners.push([half_xtop-inset, y/2.0-inset, -half_ztop]);
	corners.push([-half_xbottom+inset, -y/2.0+inset, -half_zbottom]);
	corners.push([half_xbottom-inset, -y/2.0+inset, -half_zbottom]);

	corners.push([half_xtop-inset, y/2.0-inset, half_ztop]);
	corners.push([-half_xtop+inset, y/2.0-inset, half_ztop]);
	corners.push([-half_xbottom+inset, -y/2.0+inset, half_zbottom]);
	corners.push([half_xbottom-inset, -y/2.0+inset, half_zbottom]);

	corners.push([-half_xtop, y/2.0-inset, half_ztop-inset]);
	corners.push([-half_xbottom, -y/2.0+inset, half_zbottom-inset]);
	corners.push([-half_xbottom, -y/2.0+inset, -half_zbottom+inset]);
	corners.push([-half_xtop, y/2.0-inset, -half_ztop+inset]);

	corners.push([half_xtop, y/2.0-inset, half_ztop-inset]);
	corners.push([half_xbottom, -y/2.0+inset, half_zbottom-inset]);
	corners.push([half_xbottom, -y/2.0+inset, -half_zbottom+inset]);
	corners.push([half_xtop, y/2.0-inset, -half_ztop+inset]);

    for corner in corners.iter_mut() {
        corner[0] += cube.transform.translation.x;
        corner[1] += cube.transform.translation.y;
        corner[2] += cube.transform.translation.z;
    }

	let indices/*[108+24]*/ = [
		// 12 faces
		0,3,2, 0,2,1,			// top
		4,5,7, 5,6,7,			// bottom
		16,19,18, 18,17,16,		// left
		23,20,21, 21,22,23,		// right
		12,13,14, 14,15,12,		// front
		8,9,10, 9,11,10,		// back
		// 8 corners
		0,16,13,				// top left front
		14,17,7,				// bottom left front
		19,1,8,					// top left back
		18,10,4,				// bottom left back
		2,23,9,					// top right back
		5,11,22,				// bottom right back
		3,12,20,				// top right front
		15,6,21,				// bottom right front
		// 24 bevels
		13,12,3, 3,0,13,		// top front
		15,14,7, 7,6,15,		// bottom front
		0,1,19, 19,16,0,		// top left
		17,18,4, 4,7,17,		// bottom left
		1,2,9, 9,8,1,			// top back
		10,11,4, 4,11,5,		// bottom back
		2,3,20, 20,23,2,		// top right
		22,21,6, 6,5,22,			// bottom right
		
		13,16,17, 17,14,13,		// front left
		19,8,10, 10,18,19,		// back left	
		9,23,22, 22,11,9,		// back right
		20,12,21, 21, 12,15		// front right
    ];
    let indices: Vec<u16> = Vec::from(indices);
    let indices = Indices::U16(indices);

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, corners);
    mesh.set_indices(Some(indices));
    mesh
}
/*
fn cube_click(
    selection: Query<(&Transform, &Selection)>,
    mut camera: Query<(&mut PanOrbitCamera, &Transform)>,
) {
    if !selection.iter().any(|(_, selection)| selection.selected()) {
        return;
    }

    let mut total = Vec3::ZERO;
    let mut point_count = 0;

    for (transform, selection) in &selection {
        if selection.selected() {
            total += transform.translation;
            point_count += 1;
        }
    }

    let center = total / point_count as f32;

    let (mut camera, camera_transform) = camera.single_mut();
    camera.radius = (camera_transform.translation - center).length();
    camera.focus = center;
}
*/

fn spawn_camera(mut commands: Commands) {
    let focus: Vec3 = Vec3::ZERO;

    let mut transform = Transform::default();
    transform.translation = Vec3 {
        x: -2.0,
        y: 2.5,
        z: 5.0,
    };
    transform.look_at(focus, Vec3::Y);

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
    ));
}

fn matrix_from_frame(frame: &Joint) -> Mat4 {
    Mat4 {
        x_axis: Vec4::new(
            (frame.up_y * frame.forward_z) - (frame.up_z * frame.forward_y),
            (frame.up_z * frame.forward_x) - (frame.up_x * frame.forward_z),
            (frame.up_x * frame.forward_y) - (frame.up_y * frame.forward_x),
            0.0),
        y_axis: Vec4::new(
            frame.up_x,
            frame.up_y,
            frame.up_z,
            0.0),
        z_axis: Vec4::new(
            frame.forward_x,
            frame.forward_y,
            frame.forward_z,
            0.0),
        w_axis: Vec4::new(
            frame.origin_x,
            frame.origin_y,
            frame.origin_z,
            1.0),
    }
}

fn spawn_cubes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(-1.0, 0.0, 0.0),
            ..default()
        })
        .insert(PickableBundle::default())
        .insert(Clickable);
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.5, 0.1, 0.1).into()),
            transform: Transform::from_xyz(1.0, 0.0, 0.0),
            ..default()
        })
        .insert(PickableBundle::default())
        .insert(Clickable);
}

fn spawn_skeleton(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let skeleton_parts = skelly();
    let mut bone_id = 1;

    let name = "Hips".to_string();
    let bone = skeleton_parts.get(&name).unwrap();
    let mesh = make_bone_mesh(bone);
    let hips = commands
        .spawn(PbrBundle {
            mesh: meshes.add(mesh),
            material: materials.add(Color::rgb(0.5, 0.1, 0.1).into()),
            ..default()
        })
        .insert(PickableBundle::default())
        .insert(Clickable)
        .insert(Name::from(name))
        .insert(Bone { id: bone_id })
        .id();
    commands.entity(hips).add_children(|parent| {
        spawn_bone_axis(parent, &mut meshes, &mut materials);
    });
    bone_id += 1;

    let prev = bone;
    let name = "Left Femur".to_string();
    let bone = skeleton_parts.get(&name).unwrap();
    let mut transform = Transform::IDENTITY;
    transform.translation += Vec3::new(7.5, -prev.y, 1.55);
    let mesh = make_bone_mesh(bone);
    let left_femur = commands.entity(hips).add_children(|parent| {
        parent
            .spawn(PbrBundle {
                mesh: meshes.add(mesh),
                material: materials.add(Color::rgb(0.5, 0.1, 0.1).into()),
                transform,
                ..default()
            })
            .insert(PickableBundle::default())
            .insert(Clickable)
            .insert(Name::from(name))
            .insert(Bone { id: bone_id })
            .id()
    });
    commands.entity(left_femur).add_children(|parent| {
        spawn_bone_axis(parent, &mut meshes, &mut materials);
    });
    bone_id += 1;

    let prev = bone;
    let name = "Left Calf".to_string();
    let bone = skeleton_parts.get(&name).unwrap();
    let mut transform = Transform::IDENTITY;
    transform.translation += Vec3::new(0.0, -prev.y, 0.0);
    let mesh = make_bone_mesh(bone);
    let left_calf = commands.entity(left_femur).add_children(|parent| {
        parent
            .spawn(PbrBundle {
                mesh: meshes.add(mesh),
                material: materials.add(Color::rgb(0.5, 0.1, 0.1).into()),
                transform,
                ..default()
            })
            .insert(PickableBundle::default())
            .insert(Clickable)
            .insert(Name::from(name))
            .insert(Bone { id: bone_id })
            .id()
    });
    commands.entity(left_calf).add_children(|parent| {
        spawn_bone_axis(parent, &mut meshes, &mut materials);
    });
    bone_id += 1;

    let prev = bone;
    let name = "Left Foot".to_string();
    let bone = skeleton_parts.get(&name).unwrap();
    let mut transform = Transform::IDENTITY;
    transform.translation += Vec3::new(0.0, -prev.y, 0.0);
    let mesh = make_bone_mesh(bone);
    let left_foot = commands.entity(left_calf).add_children(|parent| {
        parent
            .spawn(PbrBundle {
                mesh: meshes.add(mesh),
                material: materials.add(Color::rgb(0.5, 0.1, 0.1).into()),
                transform,
                ..default()
            })
            .insert(PickableBundle::default())
            .insert(Clickable)
            .insert(Name::from(name))
            .insert(Bone { id: bone_id })
            .id()
    });
    commands.entity(left_foot).add_children(|parent| {
        spawn_bone_axis(parent, &mut meshes, &mut materials);
    });
    bone_id += 1;

    let name = "Hips".to_string();
    let bone = skeleton_parts.get(&name).unwrap();
    let prev = bone;
    let name = "Right Femur".to_string();
    let bone = skeleton_parts.get(&name).unwrap();
    let mut transform = Transform::IDENTITY;
    transform.translation += Vec3::new(-7.5, -prev.y, 1.55);
    let mesh = make_bone_mesh(bone);
    let right_femur = commands.entity(hips).add_children(|parent| {
        parent
            .spawn(PbrBundle {
                mesh: meshes.add(mesh),
                material: materials.add(Color::rgb(0.5, 0.1, 0.1).into()),
                transform,
                ..default()
            })
            .insert(PickableBundle::default())
            .insert(Clickable)
            .insert(Name::from(name))
            .insert(Bone { id: bone_id })
            .id()
    });
    commands.entity(right_femur).add_children(|parent| {
        spawn_bone_axis(parent, &mut meshes, &mut materials);
    });
    bone_id += 1;

    let prev = bone;
    let name = "Right Calf".to_string();
    let bone = skeleton_parts.get(&name).unwrap();
    let mut transform = Transform::IDENTITY;
    transform.translation += Vec3::new(0.0, -prev.y, 0.0);
    let mesh = make_bone_mesh(bone);
    let right_calf = commands.entity(right_femur).add_children(|parent| {
        parent
            .spawn(PbrBundle {
                mesh: meshes.add(mesh),
                material: materials.add(Color::rgb(0.5, 0.1, 0.1).into()),
                transform,
                ..default()
            })
            .insert(PickableBundle::default())
            .insert(Clickable)
            .insert(Name::from(name))
            .insert(Bone { id: bone_id })
            .id()
    });
    commands.entity(right_calf).add_children(|parent| {
        spawn_bone_axis(parent, &mut meshes, &mut materials);
    });
    bone_id += 1;

    let prev = bone;
    let name = "Right Foot".to_string();
    let bone = skeleton_parts.get(&name).unwrap();
    let mut transform = Transform::IDENTITY;
    transform.translation += Vec3::new(0.0, -prev.y, 0.0);
    let mesh = make_bone_mesh(bone);
    let right_foot = commands.entity(right_calf).add_children(|parent| {
        parent
            .spawn(PbrBundle {
                mesh: meshes.add(mesh),
                material: materials.add(Color::rgb(0.5, 0.1, 0.1).into()),
                transform,
                ..default()
            })
            .insert(PickableBundle::default())
            .insert(Clickable)
            .insert(Name::from(name))
            .insert(Bone { id: bone_id })
            .id()
    });
    commands.entity(right_foot).add_children(|parent| {
        spawn_bone_axis(parent, &mut meshes, &mut materials);
    });
    bone_id += 1;

    let name = "Hips".to_string();
    let prev = skeleton_parts.get(&name).unwrap();
    let name = "Lumbar".to_string();
    let bone = skeleton_parts.get(&name).unwrap();
    let mut prev_entity = hips;
    let l_spine_length = 9.0;

    for i in (1..=5).rev() {
        let mut transform = Transform::IDENTITY;
        if i == 5 {
            // this is 0, 0, 0 in original
            // /Users/matt/Documents/former_desktop/My\ PROJECT/shared\ source/Skeleton.m
            transform.translation += Vec3::new(0.0, l_spine_length / 5.0, 0.0);
        } else {
            transform.translation += Vec3::new(0.0, l_spine_length / 5.0, 0.0);
        }
        let mesh = make_bone_mesh(bone);
        let lumbar = commands.entity(prev_entity).add_children(|parent| {
            parent
                .spawn(PbrBundle {
                    mesh: meshes.add(mesh),
                    material: materials.add(Color::rgb(0.5, 0.1, 0.1).into()),
                    transform,
                    ..default()
                })
                .insert(PickableBundle::default())
                .insert(Clickable)
                .insert(Bone { id: bone_id })
                .insert(Name::from(format!("{} {}", name, i)))
                .id()
        });
        commands.entity(lumbar).add_children(|parent| {
            spawn_bone_axis(parent, &mut meshes, &mut materials);
        });
        bone_id += 1;
        prev_entity = lumbar;
    }

    let name = "Thoracic".to_string();
    let bone = skeleton_parts.get(&name).unwrap();
    let t_spine_length = 19.0;
    for i in (1..=12).rev() {
        let mut transform = Transform::IDENTITY;
        transform.translation += Vec3::new(0.0, t_spine_length / 12.0, 0.0);
        let mesh = make_bone_mesh(bone);
        let thoracic = commands.entity(prev_entity).add_children(|parent| {
            parent
                .spawn(PbrBundle {
                    mesh: meshes.add(mesh),
                    material: materials.add(Color::rgb(0.5, 0.1, 0.1).into()),
                    transform,
                    ..default()
                })
                .insert(PickableBundle::default())
                .insert(Clickable)
                .insert(Name::from(format!("{} {}", name, i)))
                .insert(Bone { id: bone_id })
                .id()
        });
        commands.entity(thoracic).add_children(|parent| {
            spawn_bone_axis(parent, &mut meshes, &mut materials);
        });
    bone_id += 1;
        prev_entity = thoracic;
    }

    let name = "Cervical".to_string();
    let bone = skeleton_parts.get(&name).unwrap();
    let c_spine_length = 8.0;
    for i in (1..=7).rev() {
        let mut transform = Transform::IDENTITY;
        transform.translation += Vec3::new(0.0, c_spine_length / 7.0, 0.0);
        let mesh = make_bone_mesh(bone);
        let cervical = commands.entity(prev_entity).add_children(|parent| {
            parent
                .spawn(PbrBundle {
                    mesh: meshes.add(mesh),
                    material: materials.add(Color::rgb(0.5, 0.1, 0.1).into()),
                    transform,
                    ..default()
                })
                .insert(PickableBundle::default())
                .insert(Clickable)
                .insert(Name::from(format!("{} {}", name, i)))
                .insert(Bone { id: bone_id })
                .id()
        });
        commands.entity(cervical).add_children(|parent| {
            spawn_bone_axis(parent, &mut meshes, &mut materials);
        });
    bone_id += 1;
        prev_entity = cervical;
    }
    let c7 = prev_entity;

    let prev = bone;
    let name = "Head".to_string();
    let bone = skeleton_parts.get(&name).unwrap();
    let mut transform = Transform::IDENTITY;
    transform.translation += Vec3::new(0.0, 0.0, 0.0);
    let mesh = make_bone_mesh(bone);
    let head = commands.entity(c7).add_children(|parent| {
        parent
            .spawn(PbrBundle {
                mesh: meshes.add(mesh),
                material: materials.add(Color::rgb(0.5, 0.1, 0.1).into()),
                transform,
                ..default()
            })
            .insert(PickableBundle::default())
            .insert(Clickable)
            .insert(Name::from(name))
            .insert(Bone { id: bone_id })
            .id()
    });
    commands.entity(head).add_children(|parent| {
        spawn_bone_axis(parent, &mut meshes, &mut materials);
    });
    bone_id += 1;

    let prev = bone;
    let name = "Left Clavical".to_string();
    let bone = skeleton_parts.get(&name).unwrap();
    let mut transform = Transform::IDENTITY;
    transform.translation += Vec3::new(0.0, 0.0, 5.0);
    let mesh = make_bone_mesh(bone);
    let left_clavical = commands.entity(c7).add_children(|parent| {
        parent
            .spawn(PbrBundle {
                mesh: meshes.add(mesh),
                material: materials.add(Color::rgb(0.5, 0.1, 0.1).into()),
                transform,
                ..default()
            })
            .insert(PickableBundle::default())
            .insert(Clickable)
            .insert(Name::from(name))
            .insert(Bone { id: bone_id })
            .id()
    });
    commands.entity(left_clavical).add_children(|parent| {
        spawn_bone_axis(parent, &mut meshes, &mut materials);
    });
    bone_id += 1;
    prev_entity = left_clavical;

    let prev = bone;
    let name = "Left Arm".to_string();
    let bone = skeleton_parts.get(&name).unwrap();
    let mut transform = Transform::IDENTITY;
    transform.translation += Vec3::new(0.0, -prev.y, 0.0);
    let mesh = make_bone_mesh(bone);
    let left_arm = commands.entity(prev_entity).add_children(|parent| {
        parent
            .spawn(PbrBundle {
                mesh: meshes.add(mesh),
                material: materials.add(Color::rgb(0.5, 0.1, 0.1).into()),
                transform,
                ..default()
            })
            .insert(PickableBundle::default())
            .insert(Clickable)
            .insert(Name::from(name))
            .insert(Bone { id: bone_id })
            .id()
    });
    commands.entity(left_arm).add_children(|parent| {
        spawn_bone_axis(parent, &mut meshes, &mut materials);
    });
    bone_id += 1;
    prev_entity = left_arm;

    let prev = bone;
    let name = "Left Forearm".to_string();
    let bone = skeleton_parts.get(&name).unwrap();
    let mut transform = Transform::IDENTITY;
    transform.translation += Vec3::new(0.0, -prev.y, 0.0);
    let mesh = make_bone_mesh(bone);
    let left_forearm = commands.entity(prev_entity).add_children(|parent| {
        parent
            .spawn(PbrBundle {
                mesh: meshes.add(mesh),
                material: materials.add(Color::rgb(0.5, 0.1, 0.1).into()),
                transform,
                ..default()
            })
            .insert(PickableBundle::default())
            .insert(Clickable)
            .insert(Name::from(name))
            .insert(Bone { id: bone_id })
            .id()
    });
    commands.entity(left_forearm).add_children(|parent| {
        spawn_bone_axis(parent, &mut meshes, &mut materials);
    });
    bone_id += 1;
    prev_entity = left_forearm;

    let prev = bone;
    let name = "Left Hand".to_string();
    let bone = skeleton_parts.get(&name).unwrap();
    let mut transform = Transform::IDENTITY;
    transform.translation += Vec3::new(0.0, -prev.y, 0.0);
    let mesh = make_bone_mesh(bone);
    let left_hand = commands.entity(prev_entity).add_children(|parent| {
        parent
            .spawn(PbrBundle {
                mesh: meshes.add(mesh),
                material: materials.add(Color::rgb(0.5, 0.1, 0.1).into()),
                transform,
                ..default()
            })
            .insert(PickableBundle::default())
            .insert(Clickable)
            .insert(Name::from(name))
            .insert(Bone { id: bone_id })
            .id()
    });
    commands.entity(left_hand).add_children(|parent| {
        spawn_bone_axis(parent, &mut meshes, &mut materials);
    });
    bone_id += 1;

    prev_entity = c7;

    let name = "Right Clavical".to_string();
    let bone = skeleton_parts.get(&name).unwrap();
    let mut transform = Transform::IDENTITY;
    transform.translation += Vec3::new(0.0, 0.0, 5.0);
    let mesh = make_bone_mesh(bone);
    let right_clavical = commands.entity(prev_entity).add_children(|parent| {
        parent
            .spawn(PbrBundle {
                mesh: meshes.add(mesh),
                material: materials.add(Color::rgb(0.5, 0.1, 0.1).into()),
                transform,
                ..default()
            })
            .insert(PickableBundle::default())
            .insert(Clickable)
            .insert(Name::from(name))
            .insert(Bone { id: bone_id })
            .id()
    });
    commands.entity(right_clavical).add_children(|parent| {
        spawn_bone_axis(parent, &mut meshes, &mut materials);
    });
    bone_id += 1;
    prev_entity = right_clavical;

    let prev = bone;
    let name = "Right Arm".to_string();
    let bone = skeleton_parts.get(&name).unwrap();
    let mut transform = Transform::IDENTITY;
    transform.translation += Vec3::new(0.0, -prev.y, 0.0);
    let mesh = make_bone_mesh(bone);
    let right_arm = commands.entity(prev_entity).add_children(|parent| {
        parent
            .spawn(PbrBundle {
                mesh: meshes.add(mesh),
                material: materials.add(Color::rgb(0.5, 0.1, 0.1).into()),
                transform,
                ..default()
            })
            .insert(PickableBundle::default())
            .insert(Clickable)
            .insert(Name::from(name))
            .insert(Bone { id: bone_id })
            .id()
    });
    commands.entity(right_arm).add_children(|parent| {
        spawn_bone_axis(parent, &mut meshes, &mut materials);
    });
    bone_id += 1;
    prev_entity = right_arm;

    let prev = bone;
    let name = "Right Forearm".to_string();
    let bone = skeleton_parts.get(&name).unwrap();
    let mut transform = Transform::IDENTITY;
    transform.translation += Vec3::new(0.0, -prev.y, 0.0);
    let mesh = make_bone_mesh(bone);
    let right_forearm = commands.entity(prev_entity).add_children(|parent| {
        parent
            .spawn(PbrBundle {
                mesh: meshes.add(mesh),
                material: materials.add(Color::rgb(0.5, 0.1, 0.1).into()),
                transform,
                ..default()
            })
            .insert(PickableBundle::default())
            .insert(Clickable)
            .insert(Name::from(name))
            .insert(Bone { id: bone_id })
            .id()
    });
    commands.entity(right_forearm).add_children(|parent| {
        spawn_bone_axis(parent, &mut meshes, &mut materials);
    });
    bone_id += 1;
    prev_entity = right_forearm;

    let prev = bone;
    let name = "Right Hand".to_string();
    let bone = skeleton_parts.get(&name).unwrap();
    let mut transform = Transform::IDENTITY;
    transform.translation += Vec3::new(0.0, -prev.y, 0.0);
    let mesh = make_bone_mesh(bone);
    let right_hand = commands.entity(prev_entity).add_children(|parent| {
        parent
            .spawn(PbrBundle {
                mesh: meshes.add(mesh),
                material: materials.add(Color::rgb(0.5, 0.1, 0.1).into()),
                transform,
                ..default()
            })
            .insert(PickableBundle::default())
            .insert(Clickable)
            .insert(Name::from(name))
            .insert(Bone { id: bone_id })
            .id()
    });
    commands.entity(right_hand).add_children(|parent| {
        spawn_bone_axis(parent, &mut meshes, &mut materials);
    });
    bone_id += 1;

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.5,
    });
    /*
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    */
}

fn setup_ui(
    mut commands: Commands,
) {
    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ..default()
        })
        .with_children(|commands| {
            commands.spawn(TextBundle {
                style: Style {
                    align_self: AlignSelf::Center,
                    margin: UiRect::all(Val::Percent(3.0)),
                    ..default()
                },
                text: Text::from_section(
                    "Asanas yogaMat !!!",
                    TextStyle {
                        //font: my_assets.font.clone(),
                        font_size: 70.0,
                        //color: my_assets.color,
                        ..Default::default()
                    },
                ),
                ..default()
            })
            .insert(Name::new("GameTitle"));

    let button_margin = UiRect::all(Val::Percent(2.0));
        commands
            .spawn(ButtonBundle {
                style: Style {
                    size: Size::new(Val::Px(91.0), Val::Px(91.0)),
                    align_self: AlignSelf::Center,
                    justify_content: JustifyContent::Center,
                    margin: button_margin,
                    ..default()
                },
                //image: img.into(),
                ..default()
            }).insert(LoadButton)
            .with_children(|commands| {
                commands.spawn(TextBundle {
                    style: Style {
                        align_self: AlignSelf::FlexStart,
                        margin: UiRect::all(Val::Percent(3.0)),
                        position: UiRect::new(Val::Px(0.0), Val::Px(-110.0), Val::Px(20.0), Val::Px(0.0)),
                        ..default()
                    },
                    text: Text::from_section(
                        "load me",
                        TextStyle {
                            //font: my_assets.font.clone(),
                            font_size: 44.0,
                            //color: my_assets.color,
                            ..Default::default()
                        },
                    ),
                    ..default()
                });
            });
        });
}

fn spawn_bone_axis(
    commands: &mut ChildBuilder,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let length = 15.0;
    let width = 0.1;
    //let x = Box::new(x_length, y_length, z_length);
    let x = shape::Box::new(length, width, width);
    let y = shape::Box::new(width, length, width);
    let z = shape::Box::new(width, width, length);

    let mut empty = commands.spawn_empty();
    empty
        .insert(TransformBundle::from_transform(Transform::IDENTITY))
        .insert(Visibility::default())
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
                    //visibility: Visibility { is_visible: false },
                    ..default()
                },
                NotShadowCaster,
                VisibleAxis,
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
                    //visibility: Visibility { is_visible: false },
                    ..default()
                },
                NotShadowCaster,
                VisibleAxis,
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
                    //visibility: Visibility { is_visible: false },
                    ..default()
                },
                NotShadowCaster,
                VisibleAxis,
            ))
            .insert(Name::from("z axis"));
    });
}

fn spawn_axis(
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
        .insert(Visibility::default())
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
                    visibility: Visibility { is_visible: false },
                    ..default()
                },
                NotShadowCaster,
                VisibleAxis,
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
                    visibility: Visibility { is_visible: false },
                    ..default()
                },
                NotShadowCaster,
                VisibleAxis,
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
                    visibility: Visibility { is_visible: false },
                    ..default()
                },
                NotShadowCaster,
                VisibleAxis,
            ))
            .insert(Name::from("z axis"));
    });
}
