use super::{Mesh, Vertex};
use crate::math::{Matrix4, Vector2, Vector3};
use crate::mesh::VertexInserter;
use core::{f32, iter};

pub fn quad(vertex_insert: &mut VertexInserter, x_divisions: u32, y_divisions: u32) {
    let normal = Vector3 {
        x: 0.,
        y: 1.,
        z: 0.,
    };

    let x_quad_count = x_divisions + 1;
    let y_quad_count = y_divisions + 1;
    let quad_size = Vector2 {
        x: 2. / x_quad_count as f32,
        y: 2. / y_quad_count as f32,
    };

    let mut insert = vertex_insert.quads();
    insert.reserve((x_quad_count * y_quad_count) as usize);
    for y_index in 0..y_quad_count {
        for x_index in 0..x_quad_count {
            let quad_min_x = -1. + quad_size.x * x_index as f32;
            let quad_min_y = -1. + quad_size.y * y_index as f32;
            let quad_max_x = quad_min_x + quad_size.x;
            let quad_max_y = quad_min_y + quad_size.y;

            let min_uv_x = quad_min_x / 2. + 0.5;
            let min_uv_y = quad_min_y / 2. + 0.5;
            let max_uv_x = quad_max_x / 2. + 0.5;
            let max_uv_y = quad_max_y / 2. + 0.5;

            insert.quad([
                Vertex {
                    position: Vector3 {
                        x: quad_min_x,
                        y: 0.,
                        z: quad_min_y,
                    },
                    normal,
                    uv: Vector2 {
                        x: min_uv_x,
                        y: min_uv_y,
                    },
                },
                Vertex {
                    position: Vector3 {
                        x: quad_max_x,
                        y: 0.,
                        z: quad_min_y,
                    },
                    normal,
                    uv: Vector2 {
                        x: max_uv_x,
                        y: min_uv_y,
                    },
                },
                Vertex {
                    position: Vector3 {
                        x: quad_max_x,
                        y: 0.,
                        z: quad_max_y,
                    },
                    normal,
                    uv: Vector2 {
                        x: max_uv_x,
                        y: max_uv_y,
                    },
                },
                Vertex {
                    position: Vector3 {
                        x: quad_min_x,
                        y: 0.,
                        z: quad_max_y,
                    },
                    normal,
                    uv: Vector2 {
                        x: min_uv_x,
                        y: max_uv_y,
                    },
                },
            ]);
        }
    }
}

pub fn cube(insert: &mut VertexInserter, x_divisions: u32, y_divisions: u32) {
    let base_transform = Matrix4::translate(Vector3::unit_y());
    let uv_size = Vector2 {
        x: 1. / 3.,
        y: 1. / 4.,
    };

    let insert_params = [
        (base_transform, uv_size * Vector2 { x: 1., y: 0. }, uv_size),
        (
            Matrix4::rotate_x(-f32::consts::FRAC_PI_2) * base_transform,
            uv_size * Vector2 { x: 1., y: 1. },
            uv_size,
        ),
        (
            Matrix4::rotate_x(f32::consts::PI) * base_transform,
            uv_size * Vector2 { x: 1., y: 2. },
            uv_size,
        ),
        (
            Matrix4::rotate_x(f32::consts::FRAC_PI_2) * base_transform,
            uv_size * Vector2 { x: 1., y: 3. },
            uv_size,
        ),
        (
            Matrix4::rotate_z(f32::consts::FRAC_PI_2) * base_transform,
            Vector2 { x: 0., y: 0. },
            uv_size,
        ),
        (
            Matrix4::rotate_z(-f32::consts::FRAC_PI_2) * base_transform,
            uv_size * Vector2 { x: 2., y: 0. },
            uv_size,
        ),
    ];

    for &(transform, uv_pos, uv_size) in &insert_params {
        insert.with_params(transform, uv_pos, uv_size, |insert| {
            quad(insert, x_divisions, y_divisions)
        });
    }
}

pub fn cylinder(vertex_insert: &mut VertexInserter, meridians: u32) {
    let mut insert = vertex_insert.quads();
    insert.reserve(meridians as usize * 3);

    for meridian_index in 0..meridians {
        let start_norm_meridian = meridian_index as f32 / meridians as f32;
        let end_norm_meridian = (meridian_index + 1) as f32 / meridians as f32;

        let start_meridian_angle = 2. * f32::consts::PI * start_norm_meridian;
        let end_meridian_angle = 2. * f32::consts::PI * end_norm_meridian;

        let start_rotate_transform = Matrix4::rotate_y(start_meridian_angle);
        let start_rotate_point = start_rotate_transform
            * Vector3 {
                x: 1.,
                y: 0.,
                z: 0.,
            };
        let end_rotate_transform = Matrix4::rotate_y(end_meridian_angle);
        let end_rotate_point = end_rotate_transform
            * Vector3 {
                x: 1.,
                y: 0.,
                z: 0.,
            };

        let top_center_point = Vector3 {
            x: 0.,
            y: 1.,
            z: 0.,
        };
        let bottom_center_point = Vector3 {
            x: 0.,
            y: -1.,
            z: 0.,
        };

        // We have three quads: the two on the caps and the one on the side
        insert.quad([
            Vertex {
                position: top_center_point,
                normal: top_center_point,
                uv: Vector2 { x: 0., y: 0. },
            },
            Vertex {
                position: top_center_point + start_rotate_point,
                normal: top_center_point,
                uv: Vector2 { x: 0., y: 0. },
            },
            Vertex {
                position: top_center_point + end_rotate_point,
                normal: top_center_point,
                uv: Vector2 { x: 0., y: 0. },
            },
            Vertex {
                position: top_center_point,
                normal: top_center_point,
                uv: Vector2 { x: 0., y: 0. },
            },
        ]);
        insert.quad([
            Vertex {
                position: top_center_point + end_rotate_point,
                normal: end_rotate_point,
                uv: Vector2 { x: 0., y: 0. },
            },
            Vertex {
                position: top_center_point + start_rotate_point,
                normal: start_rotate_point,
                uv: Vector2 { x: 0., y: 0. },
            },
            Vertex {
                position: bottom_center_point + start_rotate_point,
                normal: start_rotate_point,
                uv: Vector2 { x: 0., y: 0. },
            },
            Vertex {
                position: bottom_center_point + end_rotate_point,
                normal: end_rotate_point,
                uv: Vector2 { x: 0., y: 0. },
            },
        ]);
        insert.quad([
            Vertex {
                position: bottom_center_point + start_rotate_point,
                normal: bottom_center_point,
                uv: Vector2 { x: 0., y: 0. },
            },
            Vertex {
                position: bottom_center_point,
                normal: bottom_center_point,
                uv: Vector2 { x: 0., y: 0. },
            },
            Vertex {
                position: bottom_center_point,
                normal: bottom_center_point,
                uv: Vector2 { x: 0., y: 0. },
            },
            Vertex {
                position: bottom_center_point + end_rotate_point,
                normal: bottom_center_point,
                uv: Vector2 { x: 0., y: 0. },
            },
        ]);
    }
}

pub fn sphere(vertex_insert: &mut VertexInserter, meridians: u32, parallels: u32) {
    let mut insert = vertex_insert.quads();
    insert.reserve(meridians as usize * parallels as usize);

    for meridian_index in 0..meridians {
        let start_norm_meridian = meridian_index as f32 / meridians as f32;
        let end_norm_meridian = (meridian_index + 1) as f32 / meridians as f32;

        let start_meridian_angle = 2. * f32::consts::PI * start_norm_meridian;
        let end_meridian_angle = 2. * f32::consts::PI * end_norm_meridian;

        for parallel_index in 0..parallels {
            let start_norm_parallel = parallel_index as f32 / parallels as f32;
            let end_norm_parallel = (parallel_index + 1) as f32 / parallels as f32;

            let start_parallel_angle = f32::consts::PI * start_norm_parallel;
            let end_parallel_angle = f32::consts::PI * end_norm_parallel;

            let base_point = Vector3 {
                x: 0.,
                y: 1.,
                z: 0.,
            };
            let top_left_p = Matrix4::rotate_y(start_meridian_angle)
                * Matrix4::rotate_x(start_parallel_angle)
                * base_point;
            let top_right_p = Matrix4::rotate_y(end_meridian_angle)
                * Matrix4::rotate_x(start_parallel_angle)
                * base_point;
            let bottom_left_p = Matrix4::rotate_y(start_meridian_angle)
                * Matrix4::rotate_x(end_parallel_angle)
                * base_point;
            let bottom_right_p = Matrix4::rotate_y(end_meridian_angle)
                * Matrix4::rotate_x(end_parallel_angle)
                * base_point;

            let mut vertices = [
                Vertex {
                    position: top_left_p,
                    normal: top_left_p,
                    uv: Vector2 {
                        x: start_norm_meridian,
                        y: start_norm_parallel,
                    },
                },
                Vertex {
                    position: top_right_p,
                    normal: top_right_p,
                    uv: Vector2 {
                        x: end_norm_meridian,
                        y: start_norm_parallel,
                    },
                },
                Vertex {
                    position: bottom_right_p,
                    normal: bottom_right_p,
                    uv: Vector2 {
                        x: end_norm_meridian,
                        y: end_norm_parallel,
                    },
                },
                Vertex {
                    position: bottom_left_p,
                    normal: bottom_left_p,
                    uv: Vector2 {
                        x: start_norm_meridian,
                        y: end_norm_parallel,
                    },
                },
            ];
            vertices.reverse();
            insert.quad(vertices);
        }
    }
}

/*impl Mesh {
    pub fn make_quad(x_divisions: u32, y_divisions: u32, transform: Matrix4) -> Mesh {
        let mut mesh = Mesh::new(4);
        quad(
            &mut mesh.get_mut().transformed(transform),
            x_divisions,
            y_divisions,
        );
        mesh
    }

    pub fn make_cube(x_divisions: u32, y_divisions: u32, transform: Matrix4) -> Mesh {
        let mut mesh = Mesh::new(4);
        cube(
            &mut mesh.get_mut().transformed(transform),
            x_divisions,
            y_divisions,
        );
        mesh
    }

    pub fn make_cylinder(meridians: u32, transform: Matrix4) -> Mesh {
        let mut mesh = Mesh::new(4);
        cylinder(&mut mesh.get_mut().transformed(transform), meridians);
        mesh
    }

    pub fn make_sphere(meridians: u32, parallels: u32, transform: Matrix4) -> Mesh {
        let mut mesh = Mesh::new(4);
        sphere(
            &mut mesh.get_mut().transformed(transform),
            meridians,
            parallels,
        );
        mesh
    }
}*/
