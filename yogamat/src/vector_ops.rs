/*
use bevy::prelude::Vec3;

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
*/
