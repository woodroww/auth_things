use std::collections::HashMap;
use bevy::{prelude::*, render::render_resource::PrimitiveTopology};
use serde::{Serialize, Deserialize};

pub struct JointMatrix {
    pub mat: Mat4,
    pub joint_id: i32,
}

#[derive(Clone)]
pub struct BoneCube {
    x_top: f32,
    x_bottom: f32,
    z_top: f32,
    z_bottom: f32,
    pub y: f32,
    inset: f32,
    transform: Transform,
    pub name: String,
}

#[derive(Default, Serialize, Deserialize)]
pub struct Joint {
    pub joint_id: i32,
    pub pose_id: i32,
    pub up_x: f32,
    pub up_y: f32,
    pub up_z: f32,
    pub forward_x: f32,
    pub forward_y: f32,
    pub forward_z: f32,
    pub origin_x: f32,
    pub origin_y: f32,
    pub origin_z: f32,
}

impl Joint {
    pub fn matrix(&self) -> Mat4 {
        Mat4 {
            x_axis: Vec4::new(
                (self.up_y * self.forward_z) - (self.up_z * self.forward_y),
                (self.up_z * self.forward_x) - (self.up_x * self.forward_z),
                (self.up_x * self.forward_y) - (self.up_y * self.forward_x),
                0.0),
            y_axis: Vec4::new(
                self.up_x,
                self.up_y,
                self.up_z,
                0.0),
            z_axis: Vec4::new(
                self.forward_x,
                self.forward_y,
                self.forward_z,
                0.0),
            w_axis: Vec4::new(
                self.origin_x,
                self.origin_y,
                self.origin_z,
                1.0),
        }
    }
}

#[rustfmt::skip]
pub fn make_bone_mesh(cube: &BoneCube) -> Mesh {
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

pub fn skelly() -> HashMap<String, BoneCube> {
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
            transform: Transform::from_xyz(0.0, -4.0, 0.0),
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
            transform: Transform::from_xyz(0.0, -head_length, 0.0),
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
