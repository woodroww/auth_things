use std::collections::HashMap;

use bevy::{
    pbr::NotShadowCaster,
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    tasks::AsyncComputeTaskPool,
};
use bevy_mod_picking::*;
use camera::{CameraPlugin, PanOrbitCamera};
//use sqlx::SqliteConnection;
use bevy_inspector_egui::WorldInspectorPlugin;

mod camera;

#[derive(Component)]
struct Clickable;

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
        .add_plugin(WorldInspectorPlugin::new())
        .add_startup_system(spawn_cubes)
        .add_startup_system(spawn_camera)
        .add_startup_system(spawn_axis)
        //.add_startup_system(setup_database)
        .add_plugin(CameraPlugin)
        .add_plugins(DefaultPickingPlugins)
        //.add_plugin(DebugCursorPickingPlugin) // <- Adds the debug cursor (optional)
        //.add_plugin(DebugEventsPickingPlugin) // <- Adds debug event logging (optional)
        .add_system(cube_click)
        .run();
}

/*
fn dooooo() {
    //let jam = AsyncComputeTaskPool;
}
async fn setup_database() {
    //let conn = SqliteConnection::connect("sqlite::memory:").await?;

    // CREATE TABLE asana (asanaID INTEGER PRIMARY KEY, sanskritName TEXT, englishName TEXT, userNotes TEXT);

    // CREATE TABLE pose ( poseID INTEGER PRIMARY KEY, asanaID INTEGER);

    let sql = "select * from asana where sanskritName = 'Tadasana'";
    // 152|Tadasana|Mountain|
    let asana_id = "select asanaID from asana where sanskritName = 'Tadasana';";
    let pose_id = "select poseId from pose where asanaID = 152;";
    let joint_data = "select * from joint where poseID = 152;"; // 152-poseID
}
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



    bone polygons from Bone.m sharedPolygonWithName
*/

struct BoneCube {
    x_top: f32,
    x_bottom: f32,
    z_top: f32,
    z_bottom: f32,
    y: f32,
    inset: f32,
    transform: Transform,
}

fn skelly() -> HashMap<String, BoneCube> {
    let head_length = 11.0;
    let clavical_lengh = 9.5;
    let c_spine_length = 8.0;
    let t_spine_length = 19.0;
    let l_spine_length = 9.0;
    let hip_length = 6.0; // half transofrm y was -4.0
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
            y: 12.0,
            inset: 1.0,
            transform: Transform::from_xyz(0.0, -4.0, 0.0),
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
            transform: Transform::from_xyz(0.0, -head_length / 2.0, 0.0),
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

fn cube_click(
    selection: Query<(&Transform, &Selection)>,
    mut camera: Query<(&mut PanOrbitCamera, &Transform)>,
) {
    /*
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
    */
}

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

fn spawn_cubes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    /*
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

    */
    let skeleton_parts = skelly();
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
        .id();
    commands.entity(hips).add_children(|parent| {
        spawn_bone_axis(parent, &mut meshes, &mut materials);
    });

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
            .id()
    });
    commands.entity(left_femur).add_children(|parent| {
        spawn_bone_axis(parent, &mut meshes, &mut materials);
    });

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
            .id()
    });
    commands.entity(left_calf).add_children(|parent| {
        spawn_bone_axis(parent, &mut meshes, &mut materials);
    });

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
            .id()
    });
    commands.entity(left_foot).add_children(|parent| {
        spawn_bone_axis(parent, &mut meshes, &mut materials);
    });

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
            .id()
    });
    commands.entity(right_femur).add_children(|parent| {
        spawn_bone_axis(parent, &mut meshes, &mut materials);
    });

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
            .id()
    });
    commands.entity(right_calf).add_children(|parent| {
        spawn_bone_axis(parent, &mut meshes, &mut materials);
    });

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
            .id()
    });
    commands.entity(right_foot).add_children(|parent| {
        spawn_bone_axis(parent, &mut meshes, &mut materials);
    });

    let name = "Hips".to_string();
    let prev = skeleton_parts.get(&name).unwrap();
    let name = "Lumbar".to_string();
    let bone = skeleton_parts.get(&name).unwrap();
    let mut prev_entity = hips;
    let l_spine_length = 9.0;

    for i in (1..=5).rev() {
        let mut transform = Transform::IDENTITY;
        if i == 5 {
            transform.translation += Vec3::new(0.0, 0.0, 0.0);
            println!("here");
        } else {
            transform.translation += Vec3::new(0.0, -l_spine_length / 5.0, 0.0);
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
                .insert(Name::from(format!("{} {}", name, i)))
                .id()
        });
        prev_entity = lumbar;
    }

    /*
    // lumbar parent is hips
        prev = hips;
        // 7 8 9 10 11			// LUMBAR
        for (int i = 5; i >= 1; i--) {
            section = [self createSectionNamed: [NSString stringWithFormat: @"Lumbar %i", i]
                                        offset: off];
            section.parent = prev;
        }

        // 12 13 14 15 16 17 18 19 20 21 22 23				// THORACIC
        for (int i = 12; i >= 1; i--) {
            section = [self createSectionNamed: [NSString stringWithFormat: @"Thoracic %i", i]
                                        offset: Vertex3DMake(0, -prev.length, 0)];
        }

        // 24 25 26 27 28 29 30		// CERVICAL
        for (int i = 7; i >= 1; i--) {
            section = [self createSectionNamed: [NSString stringWithFormat: @"Cervical %i", i + 2]
                                        offset: Vertex3DMake(0, -prev.length, 0)];
        }

        // 31
        section = [self createSectionNamed: @"Head"
                                withLength: headLength
                                 jointName: @"Head to Atlas"
                                      maxX: 20 minX: -10
                                      maxY: 60 minY: -60
                                      maxZ: 15 minZ: -15
                                    offset: Vertex3DMake(0, -prev.length, 0)];

        // clavical parent is c7
        prev = c7;

        // 32   db jointID == 33
        section = [self createSectionNamed: @"Left Clavical"
                                    offset: Vertex3DMake(0, 0, 5)]; // 5 forward
        // 33
        section = [self createSectionNamed: @"Left Arm"
                                    offset: Vertex3DMake(0, -prev.length, 0)];
        // 34
        section = [self createSectionNamed: @"Left Forearm"
                                    offset: Vertex3DMake(0, -prev.length, 0)];
        // 35
        section = [self createSectionNamed: @"Left Hand"
                                    offset: Vertex3DMake(0, -prev.length, 0)];
        // 36 db jointID == 37
        prev = c7;
        section = [self createSectionNamed: @"Right Clavical"
                                    offset: Vertex3DMake(0, 0, 5)]; // move four inches forward
        // 37
        section = [self createSectionNamed: @"Right Arm"
                                    offset: Vertex3DMake(0, -prev.length, 0)];
        // 38
        section = [self createSectionNamed: @"Right Forearm"
                                withLength: forearmLength
                                 jointName: @"Right Elbow"
                                      maxX: 0 minX: -150		// flex
                                      maxY: 90 minY: -90		// medial rot / lateral rot
                                      maxZ: 0 minZ: 0			// 0
                                    offset: Vertex3DMake(0, -prev.length, 0)];
        // 39
        section = [self createSectionNamed: @"Right Hand"
                                    offset: Vertex3DMake(0, -prev.length, 0)];
    */

    /*
    for (name, bone) in skelly() {
        let transform = bone.transform;
        let mesh = make_bone_mesh(bone);
        commands
            .spawn(PbrBundle {
                mesh: meshes.add(mesh),
                material: materials.add(Color::rgb(0.5, 0.1, 0.1).into()),
                transform,
                ..default()
            })
            .insert(Name::from(name));
    }
    */

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

#[derive(Component)]
struct VisibleAxis;

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
