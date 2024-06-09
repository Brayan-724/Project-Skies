use cgmath::{perspective, InnerSpace, Matrix4, Point3, Rad, SquareMatrix, Vector3};
use sdl2::rect::Point;
use wgpu::{util::DeviceExt, BindGroup, BindGroupLayout, BindGroupLayoutDescriptor, Buffer, Device};
use std::f32::consts::FRAC_PI_2;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

pub struct CameraRenderizable {
    pub camera: Camera,
    pub projection: Projection,
    pub uniform: CameraUniform,
    pub buffer: Buffer,
    pub bind_group_layout: BindGroupLayout,
    pub bind_group: BindGroup
}

impl CameraRenderizable {
    pub fn new(device: &Device, config: &wgpu::SurfaceConfiguration) -> Self {
        let camera = Camera::new((-590.0, 10.0, 0.0), cgmath::Deg(-90.0), cgmath::Deg(-20.0));
        let projection = Projection::new(config.width, config.height, 45.0, 0.1, 100.0);

        // we create the 4x4 matrix of the camera
        let uniform = CameraUniform::new();

        // we create a buffer and a bind group
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("camera_bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform, // its a uniform buffer, duh
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("camera_bind_group"),
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: buffer.as_entire_binding(),
                    },
                ],
            }
        );

        return CameraRenderizable { camera, projection, uniform, buffer, bind_group, bind_group_layout };
    }
}

// we create the values that make our camera position and view angle
#[derive(Copy, Clone, Debug)]
pub struct Camera {
    pub position: Point3<f32>,
    pub yaw: Rad<f32>,
    pub pitch: Rad<f32>,
    pub look_at: Option<Point3<f32>>,
    pub up: Vector3<f32>
}

impl Camera {
    pub fn new< V: Into<Point3<f32>>, Y: Into<Rad<f32>>, P: Into<Rad<f32>>,>(position: V, yaw: Y, pitch: P) -> Self {
        Self {
            position: position.into(),
            yaw: yaw.into(),
            pitch: pitch.into(),
            look_at: None,
            up: Vector3::unit_y()
        }    
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        let (sin_pitch, cos_pitch) = self.pitch.0.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.0.sin_cos();

        match self.look_at {
            Some(look_at_pos) => {
                println!("executao");
                return Matrix4::look_at_rh(
                    self.position,
                    (look_at_pos.x, look_at_pos.y, look_at_pos.z).into(),
                    self.up,
                )
            },
            None => {
                return Matrix4::look_to_rh(
                    self.position,
                    Vector3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize(),
                    self.up,
                )
            },
        }
        
    }

    pub fn look_at(&mut self, target: Point3<f32>) {
        // first we get access to the position of the object we want to look relative to the camera position
        let direction = target - self.position;
        let normalized_dir = direction.normalize();

        // calculate pitch and yaw transforming the vector to radians for the pitch and yaw
        self.pitch = Rad(normalized_dir.y.asin());
        self.yaw = Rad(normalized_dir.z.atan2(normalized_dir.x));
    }

    pub fn set_up(&mut self, direction: Vector3<f32>) {
        self.up = direction;
    }

    pub fn set_position(&mut self, target_position: Point3<f32>, delta_time: f32) {
        // let smooth_factor = 10.0; // Adjust this factor to make the movement smoother
        self.position += target_position - self.position;
    }
}

// the cameraUniform will get us the positional matrix of the camera
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
    view_position: [f32; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
            view_position: [0.0; 4],
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera, projection: &Projection) {
        self.view_position = camera.position.to_homogeneous().into();
        self.view_proj = (projection.calc_matrix() * camera.calc_matrix()).into();
    }
}

// Projection will give us the image that the camera will see based on the position, fov and near or far values
// this will only change when we resize the window
pub struct Projection {
    aspect: f32,
    pub fovy: f32,
    znear: f32,
    zfar: f32
}

impl Projection {
    pub fn new(width: u32, height: u32, fovy: f32, znear: f32, zfar: f32,) -> Self {
        Self {
            aspect: width as f32 / height as f32,
            fovy: fovy,
            znear,
            zfar,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        OPENGL_TO_WGPU_MATRIX * perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar)
    }
}