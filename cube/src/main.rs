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
struct MainMenu;

#[derive(Component)]
struct AsanaName;

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
}

#[derive(Component)]
struct Clickable;

#[derive(Component)]
struct Bone {
    id: i32,
}

#[derive(Component)]
struct BoneAxis;

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
    name: String,
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

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::hex("292929").unwrap()))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "YogaMat".to_string(),
                width: 1290.0,
                height: 1400.0,
                position: WindowPosition::At(Vec2::new(0.0, 0.0)),
                ..default()
            },
            ..default()
        }))
        .add_plugin(WorldInspectorPlugin)
        .register_type::<PanOrbitCamera>()
        //.add_plugin(FilterQueryInspectorPlugin::<With<Bone>>::default())
        .add_plugin(CameraPlugin)
        .add_plugins(DefaultPickingPlugins)
        //.add_plugin(DebugCursorPickingPlugin) // <- Adds the debug cursor (optional)
        //.add_plugin(DebugEventsPickingPlugin) // <- Adds debug event logging (optional)
        .add_startup_system(spawn_skeleton)
        .add_startup_system(spawn_camera)
        .add_startup_system(spawn_main_axis)
        .add_startup_system(setup_ui)
        .add_startup_system(spawn_mat)
        //.add_system(button_clicked)
        .add_system(keyboard_input_system)
        .add_startup_system_to_stage(StartupStage::PreStartup, load_resources)
        .add_startup_system_to_stage(StartupStage::PostStartup, initial_pose)
        //.add_system(cube_click)
        .run();
}

fn initial_pose(
    mut yoga_assets: ResMut<YogaAssets>,
    mut bones: Query<(&mut Transform, &Bone)>,
    mut asana_text: Query<&mut Text, With<AsanaName>>,
) {
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

        let joints = load_pose(name);
        for (mut transform, bone) in bones.iter_mut() {
            let mat = joints.iter().find(|j| j.joint_id == bone.id).unwrap();
            *transform = Transform::from_matrix(mat.mat);
        }
}

fn load_resources(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(YogaAssets {
        font: asset_server.load("fonts/Roboto-Regular.ttf"),
        font_color: Color::rgb_u8(207, 207, 207),
        asanas: get_asanas_from_db(),
        current_idx: 0,
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

        let joints = load_pose(name);
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
        let joints = load_pose(name);
        for (mut transform, bone) in bones.iter_mut() {
            let mat = joints.iter().find(|j| j.joint_id == bone.id).unwrap();
            *transform = Transform::from_matrix(mat.mat);
        }
    }
}

fn button_clicked(
    //mut commands: Commands,
    interactions: Query<&Interaction, (With<UpButton>, Changed<Interaction>)>,
    mut bones: Query<(&mut Transform, &Bone)>,
    mut yoga_assets: ResMut<YogaAssets>,
) {
    for interaction in &interactions {
        if matches!(interaction, Interaction::Clicked) {
            // Utpluthi Tadasana Hanumanasana
            if yoga_assets.current_idx > yoga_assets.asanas.len() - 1 {
                yoga_assets.current_idx = 0;
            }
            let name = yoga_assets.asanas[yoga_assets.current_idx].sanskrit.clone();
            let joints = load_pose(name);
            for (mut transform, bone) in bones.iter_mut() {
                let mat = joints.iter().find(|j| j.joint_id == bone.id).unwrap();
                *transform = Transform::from_matrix(mat.mat);
            }
            yoga_assets.current_idx += 1;
        }
    }
}

fn get_asanas_from_db() -> Vec<AsanaDB> {
    let path = "./yogamatdb.sql";
    let db = Connection::open(path).expect("couldn't open database");
    let sql = "select * from asana";
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
    //println!("Found {} asanas", asanas.len());
    asanas
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

    let sql = format!("select poseId from pose where asanaID = {};", asanas[0].asana_id);
    let mut stmt = db.prepare(&sql).expect("trouble preparing statement");
    let response = stmt.query_map([], |row| {
        let pose_id: i32 = row.get(0).expect("so may results");
        Ok(pose_id)
    }).expect("bad");
    let pose_ids = response.filter_map(|result| result.ok()).collect::<Vec<i32>>();
    
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

    let mut name = "Hips".to_string();
    map.insert(
        name.clone(),
        BoneCube {
            x_top: 15.0,
            x_bottom: 13.0,
            z_top: 6.0,
            z_bottom: 4.0,
            y: hip_length,
            inset: 1.0,
            transform: Transform::from_xyz(0.0, -hip_length / 2.0, 0.0),
            name,
        },
    );

    name = "Left Femur".to_string();
    map.insert(
        name.clone(),
        BoneCube {
            x_top: 6.0,
            x_bottom: 4.0,
            z_top: 6.0,
            z_bottom: 4.0,
            y: femur_length,
            inset: 1.25,
            transform: Transform::from_xyz(0.0, -femur_length / 2.0, 0.0),
            name,
        },
    );

    name = "Right Femur".to_string();
    map.insert(
        name.clone(),
        BoneCube {
            x_top: 6.0,
            x_bottom: 4.0,
            z_top: 6.0,
            z_bottom: 4.0,
            y: femur_length,
            inset: 1.25,
            transform: Transform::from_xyz(0.0, -femur_length / 2.0, 0.0),
            name,
        },
    );

    name = "Left Calf".to_string();
    map.insert(
        name.clone(),
        BoneCube {
            x_top: 4.5,
            x_bottom: 2.5,
            z_top: 4.5,
            z_bottom: 2.5,
            y: calf_length,
            inset: the_inset,
            transform: Transform::from_xyz(0.0, -calf_length / 2.0, 0.0),
            name,
        },
    );

    name = "Right Calf".to_string();
    map.insert(
        name.clone(),
        BoneCube {
            x_top: 4.5,
            x_bottom: 2.5,
            z_top: 4.5,
            z_bottom: 2.5,
            y: calf_length,
            inset: the_inset,
            transform: Transform::from_xyz(0.0, -calf_length / 2.0, 0.0),
            name,
        },
    );

    name = "Left Foot".to_string();
    map.insert(
        name.clone(),
        BoneCube {
            x_top: 3.5,
            x_bottom: 6.0,
            z_top: 3.5,
            z_bottom: 2.0,
            y: foot_length,
            inset: the_inset,
            transform: Transform::from_xyz(0.0, -foot_length / 2.0, 0.0),
            name,
        },
    );

    name = "Right Foot".to_string();
    map.insert(
        name.clone(),
        BoneCube {
            x_top: 3.5,
            x_bottom: 6.0,
            z_top: 3.5,
            z_bottom: 2.0,
            y: foot_length,
            inset: the_inset,
            transform: Transform::from_xyz(0.0, -foot_length / 2.0, 0.0),
            name,
        },
    );

    name = "Lumbar".to_string();
    map.insert(
        name.clone(),
        BoneCube {
            x_top: 3.0,
            x_bottom: 3.0,
            z_top: 3.0,
            z_bottom: 3.0,
            y: (l_spine_length / 5.0),
            inset: 0.5,
            transform: Transform::from_xyz(0.0, -(l_spine_length / 5.0) / 2.0, 0.0),
            name,
        },
    );

    name = "Thoracic".to_string();
    map.insert(
        name.clone(),
        BoneCube {
            x_top: 3.0,
            x_bottom: 3.0,
            z_top: 3.0,
            z_bottom: 3.0,
            y: t_spine_length / 12.0,
            inset: 0.5,
            transform: Transform::from_xyz(0.0, -(t_spine_length / 12.0) / 2.0, 0.0),
            name,
        },
    );

    name = "Cervical".to_string();
    map.insert(
        name.clone(),
        BoneCube {
            x_top: 3.0,
            x_bottom: 3.0,
            z_top: 3.0,
            z_bottom: 3.0,
            y: (c_spine_length / 7.0),
            inset: 0.5,
            transform: Transform::from_xyz(0.0, -(c_spine_length / 7.0) / 2.0, 0.0),
            name,
        },
    );

    name = "Head".to_string();
    map.insert(
        name.clone(),
        BoneCube {
            x_top: 12.0,
            x_bottom: 12.0,
            z_top: 12.0,
            z_bottom: 12.0,
            y: head_length,
            inset: 2.5,
            transform: Transform::from_xyz(0.0, -head_length / 2.0, 0.0),
            name,
        },
    );

    name = "Left Clavical".to_string();
    map.insert(
        name.clone(),
        BoneCube {
            x_top: 2.0,
            x_bottom: 2.0,
            z_top: 2.0,
            z_bottom: 2.0,
            y: clavical_lengh,
            inset: the_inset,
            transform: Transform::from_xyz(0.0, -clavical_lengh / 2.0, 0.0),
            name,
        },
    );

    name = "Right Clavical".to_string();
    map.insert(
        name.clone(),
        BoneCube {
            x_top: 2.0,
            x_bottom: 2.0,
            z_top: 2.0,
            z_bottom: 2.0,
            y: clavical_lengh,
            inset: the_inset,
            transform: Transform::from_xyz(0.0, -clavical_lengh / 2.0, 0.0),
            name,
        },
    );

    name = "Left Arm".to_string();
    map.insert(
        name.clone(),
        BoneCube {
            x_top: 4.20,
            x_bottom: 3.25,
            z_top: 5.075,
            z_bottom: 3.5,
            y: humerus_length,
            inset: 1.0,
            transform: Transform::from_xyz(0.0, -humerus_length / 2.0, 0.0),
            name,
        },
    );

    name = "Right Arm".to_string();
    map.insert(
        name.clone(),
        BoneCube {
            x_top: 4.20,
            x_bottom: 3.25,
            z_top: 5.075,
            z_bottom: 3.5,
            y: humerus_length,
            inset: 1.0,
            transform: Transform::from_xyz(0.0, -humerus_length / 2.0, 0.0),
            name,
        },
    );

    name = "Left Forearm".to_string();
    map.insert(
        name.clone(),
        BoneCube {
            x_top: 3.75,
            x_bottom: 2.75,
            z_top: 3.75,
            z_bottom: 2.75,
            y: forearm_length,
            inset: 1.0,
            transform: Transform::from_xyz(0.0, -forearm_length / 2.0, 0.0),
            name,
        },
    );

    name = "Right Forearm".to_string();
    map.insert(
        name.clone(),
        BoneCube {
            x_top: 3.75,
            x_bottom: 2.75,
            z_top: 3.75,
            z_bottom: 2.75,
            y: forearm_length,
            inset: 1.0,
            transform: Transform::from_xyz(0.0, -forearm_length / 2.0, 0.0),
            name,
        },
    );

    name = "Left Hand".to_string();
    map.insert(
        name.clone(),
        BoneCube {
            x_top: 3.0,
            x_bottom: 2.0,
            z_top: 4.0,
            z_bottom: 5.0,
            y: hand_length,
            inset: the_inset,
            transform: Transform::from_xyz(0.0, -hand_length / 2.0, 0.0),
            name,
        },
    );

    name = "Right Hand".to_string();
    map.insert(
        name.clone(),
        BoneCube {
            x_top: 3.0,
            x_bottom: 2.0,
            z_top: 4.0,
            z_bottom: 5.0,
            y: hand_length,
            inset: the_inset,
            transform: Transform::from_xyz(0.0, -hand_length / 2.0, 0.0),
            name,
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

    // 0,3,2, 0,2,1,			// top
	corners.push([-x_top/2.0+inset, y/2.0, half_ztop-inset]);
	corners.push([-half_xtop+inset, y/2.0, -half_ztop+inset]);
	corners.push([half_xtop-inset, y/2.0, -half_ztop+inset]);
	corners.push([half_xtop-inset, y/2.0 , half_ztop-inset]);
	
	// 4,5,7, 5,6,7,			// bottom
	corners.push([-half_xbottom+inset, -y/2.0, -half_zbottom+inset]);
	corners.push([half_xbottom-inset, -y/2.0, -half_zbottom+inset]);
	corners.push([half_xbottom-inset, -y/2.0, half_zbottom-inset]);
	corners.push([-half_xbottom+inset, -y/2.0, half_zbottom-inset]);
	
	// 8,9,10, 9,11,10,         // back
	corners.push([-half_xtop+inset, y/2.0-inset, -half_ztop]);
	corners.push([half_xtop-inset, y/2.0-inset, -half_ztop]);
	corners.push([-half_xbottom+inset, -y/2.0+inset, -half_zbottom]);
	corners.push([half_xbottom-inset, -y/2.0+inset, -half_zbottom]);

	// 12,13,14, 14,15,12,		// front
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

	let indices: [usize; 132] /*[108+24]*/ = [
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
		22,21,6, 6,5,22,		// bottom right
		
		13,16,17, 17,14,13,		// front left
		19,8,10, 10,18,19,		// back left	
		9,23,22, 22,11,9,		// back right
		20,12,21, 21, 12,15		// front right
    ];

    let mut triangles: Vec<[f32; 3]> = Vec::new();
    for i in indices.chunks(3) {
        triangles.push(corners[i[0]]);
        triangles.push(corners[i[1]]);
        triangles.push(corners[i[2]]);
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, triangles);
    mesh.compute_flat_normals();
    mesh
}

fn vector_magnitude(vector: &[f32; 3]) -> f32 {
	((vector[0] * vector[0]) + (vector[1] * vector[1]) + (vector[2] * vector[2])).sqrt()
}

fn vector_normalize(vector: &mut [f32; 3]) {
	let vec_mag: f32 = vector_magnitude(&vector);
    // float comparision WARNING
	if vec_mag == 0.0 {
		vector[0] = 1.0;
		vector[1] = 0.0;
		vector[2] = 0.0;
	} else {
        vector[0] /= vec_mag;
        vector[1] /= vec_mag;
        vector[2] /= vec_mag;
    }
}

fn  vector_make_with_start_and_end_points(start: [f32; 3], end: [f32; 3]) -> [f32; 3] {
    let mut ret = 
	[end[0] - start[0],
	end[1] - start[1],
	end[2] - start[2]];
	vector_normalize(&mut ret);
	ret
}

fn triangle_calculate_surface_normal(triangles: [[f32; 3]; 3]) -> [f32; 3] {
	let u = vector_make_with_start_and_end_points(triangles[1], triangles[0]);
	let v = vector_make_with_start_and_end_points(triangles[2], triangles[0]);
	[(u[1] * v[2]) - (u[2] * v[1]),
	(u[2] * v[0]) - (u[0] * v[2]),
	(u[0] * v[1]) - (u[1] * v[0])]
}

fn calculate_vertex_normals(triangles: &Vec<[[f32; 3]; 3]>) -> Vec<Vec3> {
    let triangle_count = triangles.len();
	let mut surface_normals: Vec<[f32; 3]> = Vec::with_capacity(triangle_count);

	for i in 0..triangle_count {
        let mut surface_normal = triangle_calculate_surface_normal(triangles[i]);
        vector_normalize(&mut surface_normal);
        surface_normals.push(surface_normal);
	}
	
    let mut vertex_normals: Vec<[f32; 3]> = Vec::with_capacity(triangle_count * 3);
    vertex_normals.resize(triangle_count * 3, [0.0, 0.0, 0.0]);
    todo!()
}

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

fn spawn_camera(mut commands: Commands) {

    let mut transform = Transform::default();
    transform.translation.x = -92.828;
    transform.translation.y = -18.648;
    transform.translation.z = 198.376;
    transform.rotation.x = -0.136;
    transform.rotation.y = -0.425;
    transform.rotation.x = -0.028;

    let focus = Vec3 {
        x: -0.045,
        y: -46.059,
        z: 3.105,
    };

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

fn spawn_mat(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(114.7, 1.0, 43.4))),
            material: materials.add(Color::rgb(0.1, 0.1, 0.5).into()),
            transform: Transform::from_xyz(0.0, -109.5, 0.0),
            ..default()
        });

    /*
    commands.insert_resource(AmbientLight {
        color: Color::rgb_u8(242, 226, 201),
        brightness: 0.2,
    });
    let light = commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..default()
    }).id();
    */

    let lights = vec![Vec3::new(-50.0, 50.0, 25.0), Vec3::new(50.0, 50.0, 25.0),
    Vec3::new(-50.0, 50.0, -25.0), Vec3::new(50.0, 50.0, -25.0)];
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
    mut commands: &mut Commands,
    mut meshes: &mut ResMut<Assets<Mesh>>,
    material: Handle<StandardMaterial>,
    mut materials: &mut ResMut<Assets<StandardMaterial>>,
    bone_cube: &BoneCube,
    bone_id: i32,
    bone_parent: Entity,
    transform: Transform,
) -> Entity {
    let new_bone = commands.entity(bone_parent).add_children(|parent| {
        parent
            .spawn(PbrBundle {
                mesh: meshes.add(make_bone_mesh(bone_cube)),
                material,
                transform,
                ..default()
            })
            //.insert(PickableBundle::default())
            .insert(Clickable)
            .insert(Name::from(bone_cube.name.clone()))
            .insert(Bone { id: bone_id })
            .id()
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
        base_color: Color::rgba_u8(133, 80, 0, 255).into(),
        //base_color: Color::rgba_u8(133, 0, 0, 255).into(),
        reflectance: 0.2,
        perceptual_roughness: 0.75,
        ..Default::default()
    };
    let material_handle = materials.add(material);

    let skeleton_parts = skelly();
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
        //.insert(PickableBundle::default())
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
    prev_entity = spawn_bone(&mut commands, &mut meshes, material_handle.clone(), &mut materials, bone, bone_id, prev_entity, transform);
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
    prev_entity = spawn_bone(&mut commands, &mut meshes, material_handle.clone(), &mut materials, bone, bone_id, prev_entity, transform);
    bone_id += 1;
/*
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
                material: material_handle.clone(),
                transform,
                ..default()
            })
            //.insert(PickableBundle::default())
            .insert(Clickable)
            .insert(Name::from(name))
            .insert(Bone { id: bone_id })
            .id()
    });
    commands.entity(left_femur).add_children(|parent| {
        spawn_entity_axis(parent, &mut meshes, &mut materials, axis_visible);
    });
    bone_id += 1;

    let prev = bone;
    let name = "Left Calf".to_string();
    let bone = skeleton_parts.get(&name).unwrap();
    let mut transform = Transform::IDENTITY;
    transform.translation += Vec3::new(0.0, -prev.y, 0.0);
    let mesh = make_bone_mesh(bone);
    let left_calf = commands.entity(prev_entity).add_children(|parent| {
        parent
            .spawn(PbrBundle {
                mesh: meshes.add(mesh),
                material: materials.add(Color::rgb(0.5, 0.1, 0.1).into()),
                transform,
                ..default()
            })
            //.insert(PickableBundle::default())
            .insert(Clickable)
            .insert(Name::from(name))
            .insert(Bone { id: bone_id })
            .id()
    });
    commands.entity(left_calf).add_children(|parent| {
        spawn_entity_axis(parent, &mut meshes, &mut materials, axis_visible);
    });
    bone_id += 1;

    let prev = bone;
    let name = "Left Foot".to_string();
    let bone = skeleton_parts.get(&name).unwrap();
    let mut transform = Transform::IDENTITY;
    transform.translation += Vec3::new(0.0, -prev.y, 0.0);
    let mesh = make_bone_mesh(bone);
    let left_foot = commands.entity(prev_entity).add_children(|parent| {
        parent
            .spawn(PbrBundle {
                mesh: meshes.add(mesh),
                material: materials.add(Color::rgb(0.5, 0.1, 0.1).into()),
                transform,
                ..default()
            })
            //.insert(PickableBundle::default())
            .insert(Clickable)
            .insert(Name::from(name))
            .insert(Bone { id: bone_id })
            .id()
    });
    commands.entity(left_foot).add_children(|parent| {
        spawn_entity_axis(parent, &mut meshes, &mut materials, axis_visible);
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
            //.insert(PickableBundle::default())
            .insert(Clickable)
            .insert(Name::from(name))
            .insert(Bone { id: bone_id })
            .id()
    });
    commands.entity(right_femur).add_children(|parent| {
        spawn_entity_axis(parent, &mut meshes, &mut materials, axis_visible);
    });
    bone_id += 1;

    let prev = bone;
    let name = "Right Calf".to_string();
    let bone = skeleton_parts.get(&name).unwrap();
    let mut transform = Transform::IDENTITY;
    transform.translation += Vec3::new(0.0, -prev.y, 0.0);
    let mesh = make_bone_mesh(bone);
    let right_calf = commands.entity(prev_entity).add_children(|parent| {
        parent
            .spawn(PbrBundle {
                mesh: meshes.add(mesh),
                material: materials.add(Color::rgb(0.5, 0.1, 0.1).into()),
                transform,
                ..default()
            })
            //.insert(PickableBundle::default())
            .insert(Clickable)
            .insert(Name::from(name))
            .insert(Bone { id: bone_id })
            .id()
    });
    commands.entity(right_calf).add_children(|parent| {
        spawn_entity_axis(parent, &mut meshes, &mut materials, axis_visible);
    });
    bone_id += 1;

    let prev = bone;
    let name = "Right Foot".to_string();
    let bone = skeleton_parts.get(&name).unwrap();
    let mut transform = Transform::IDENTITY;
    transform.translation += Vec3::new(0.0, -prev.y, 0.0);
    let mesh = make_bone_mesh(bone);
    let right_foot = commands.entity(prev_entity).add_children(|parent| {
        parent
            .spawn(PbrBundle {
                mesh: meshes.add(mesh),
                material: materials.add(Color::rgb(0.5, 0.1, 0.1).into()),
                transform,
                ..default()
            })
            //.insert(PickableBundle::default())
            .insert(Clickable)
            .insert(Name::from(name))
            .insert(Bone { id: bone_id })
            .id()
    });
    commands.entity(right_foot).add_children(|parent| {
        spawn_entity_axis(parent, &mut meshes, &mut materials, axis_visible);
    });
    bone_id += 1;

    */
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
                //.insert(PickableBundle::default())
                .insert(Clickable)
                .insert(Bone { id: bone_id })
                .insert(Name::from(format!("{} {}", name, i)))
                .id()
        });
        commands.entity(lumbar).add_children(|parent| {
            spawn_entity_axis(parent, &mut meshes, &mut materials, axis_visible);
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
                //.insert(PickableBundle::default())
                .insert(Clickable)
                .insert(Name::from(format!("{} {}", name, i)))
                .insert(Bone { id: bone_id })
                .id()
        });
        commands.entity(thoracic).add_children(|parent| {
            spawn_entity_axis(parent, &mut meshes, &mut materials, axis_visible);
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
                //.insert(PickableBundle::default())
                .insert(Clickable)
                .insert(Name::from(format!("{} {}", name, i)))
                .insert(Bone { id: bone_id })
                .id()
        });
        commands.entity(cervical).add_children(|parent| {
            spawn_entity_axis(parent, &mut meshes, &mut materials, axis_visible);
        });
    bone_id += 1;
        prev_entity = cervical;
    }
    let c7 = prev_entity;

    let prev = bone;
    let name = "Head".to_string();
    let bone = skeleton_parts.get(&name).unwrap();
    let mut transform = Transform::IDENTITY;
    transform.translation += Vec3::new(0.0, c_spine_length / 7.0, 0.0);
    let mesh = make_bone_mesh(bone);
    let head = commands.entity(c7).add_children(|parent| {
        parent
            .spawn(PbrBundle {
                mesh: meshes.add(mesh),
                material: materials.add(Color::rgb(0.5, 0.1, 0.1).into()),
                transform,
                ..default()
            })
            //.insert(PickableBundle::default())
            .insert(Clickable)
            .insert(Name::from(name))
            .insert(Bone { id: bone_id })
            .id()
    });
    commands.entity(head).add_children(|parent| {
        spawn_entity_axis(parent, &mut meshes, &mut materials, axis_visible);
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
            //.insert(PickableBundle::default())
            .insert(Clickable)
            .insert(Name::from(name))
            .insert(Bone { id: bone_id })
            .id()
    });
    commands.entity(left_clavical).add_children(|parent| {
        spawn_entity_axis(parent, &mut meshes, &mut materials, axis_visible);
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
            //.insert(PickableBundle::default())
            .insert(Clickable)
            .insert(Name::from(name))
            .insert(Bone { id: bone_id })
            .id()
    });
    commands.entity(left_arm).add_children(|parent| {
        spawn_entity_axis(parent, &mut meshes, &mut materials, axis_visible);
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
            //.insert(PickableBundle::default())
            .insert(Clickable)
            .insert(Name::from(name))
            .insert(Bone { id: bone_id })
            .id()
    });
    commands.entity(left_forearm).add_children(|parent| {
        spawn_entity_axis(parent, &mut meshes, &mut materials, axis_visible);
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
            //.insert(PickableBundle::default())
            .insert(Clickable)
            .insert(Name::from(name))
            .insert(Bone { id: bone_id })
            .id()
    });
    commands.entity(left_hand).add_children(|parent| {
        spawn_entity_axis(parent, &mut meshes, &mut materials, axis_visible);
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
            //.insert(PickableBundle::default())
            .insert(Clickable)
            .insert(Name::from(name))
            .insert(Bone { id: bone_id })
            .id()
    });
    commands.entity(right_clavical).add_children(|parent| {
        spawn_entity_axis(parent, &mut meshes, &mut materials, axis_visible);
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
            //.insert(PickableBundle::default())
            .insert(Clickable)
            .insert(Name::from(name))
            .insert(Bone { id: bone_id })
            .id()
    });
    commands.entity(right_arm).add_children(|parent| {
        spawn_entity_axis(parent, &mut meshes, &mut materials, axis_visible);
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
            //.insert(PickableBundle::default())
            .insert(Clickable)
            .insert(Name::from(name))
            .insert(Bone { id: bone_id })
            .id()
    });
    commands.entity(right_forearm).add_children(|parent| {
        spawn_entity_axis(parent, &mut meshes, &mut materials, axis_visible);
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
            //.insert(PickableBundle::default())
            .insert(Clickable)
            .insert(Name::from(name))
            .insert(Bone { id: bone_id })
            .id()
    });
    commands.entity(right_hand).add_children(|parent| {
        spawn_entity_axis(parent, &mut meshes, &mut materials, axis_visible);
    });
    bone_id += 1;
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
                    margin: UiRect::all(Val::Percent(3.0)),
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

            /*
        let button_margin = UiRect::all(Val::Percent(2.0));
        commands
            .spawn(ButtonBundle {
                style: Style {
                    size: Size::new(Val::Px(80.0), Val::Px(40.0)),
                    align_self: AlignSelf::FlexEnd,
                    justify_content: JustifyContent::FlexEnd,
                    margin: button_margin,
                    ..default()
                },
                //image: img.into(),
                ..default()
            }).insert(UpButton)
            .with_children(|commands| {
                commands.spawn(TextBundle {
                    style: Style {
                        align_self: AlignSelf::Center,
                        justify_content: JustifyContent::Center,
                        margin: UiRect::all(Val::Percent(3.0)),
                        ..default()
                    },
                    text: Text::from_section(
                        "load me",
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
            */
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
                    visibility: Visibility { is_visible: false },
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
                    visibility: Visibility { is_visible: false },
                    ..default()
                },
                NotShadowCaster,
                BoneAxis,
            ))
            .insert(Name::from("z axis"));
    });
}
