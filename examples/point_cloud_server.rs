extern crate kiss3d;
extern crate nalgebra as na;

use std::io::{BufRead, BufReader, Write};
use std::ops::RangeInclusive;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};

use bincode::{DefaultOptions, Options};
use kiss3d::camera::Camera;
use kiss3d::context::Context;
use kiss3d::planar_camera::PlanarCamera;
use kiss3d::post_processing::PostProcessingEffect;
use kiss3d::renderer::Renderer;
use kiss3d::resource::{
    AllocationType, BufferType, Effect, GPUVec, ShaderAttribute, ShaderUniform,
};
use kiss3d::text::Font;
use kiss3d::window::{State, Window};
use na::{Matrix4, Point2, Point3};
use serde::Deserialize;

// Custom renderers are used to allow rendering objects that are not necessarily
// represented as meshes. In this example, we will render a large, growing, point cloud
// with a color associated to each point.

// Writing a custom renderer requires the main loop to be
// handled by the `State` trait instead of a `while window.render()`
// like other examples.

#[derive(Deserialize)]
struct MyPoint {
    x: f32,
    y: f32,
    z: f32,
    confidence: f32,
}

struct AppState {
    point_cloud_renderer: PointCloudRenderer,
    point_receiver: Receiver<Vec<MyPoint>>,
    command_receiver: Receiver<Command>,
    max_depth: Option<f32>,
    min_depth: Option<f32>,
    confidence_range: Option<RangeInclusive<f32>>,
}

impl State for AppState {
    // Return the custom renderer that will be called at each
    // render loop.
    fn cameras_and_effect_and_renderer(
        &mut self,
    ) -> (
        Option<&mut dyn Camera>,
        Option<&mut dyn PlanarCamera>,
        Option<&mut dyn Renderer>,
        Option<&mut dyn PostProcessingEffect>,
    ) {
        (None, None, Some(&mut self.point_cloud_renderer), None)
    }

    fn step(&mut self, window: &mut Window) {
        match self.command_receiver.try_recv() {
            Ok(Command::SetMaxDepth(max_depth)) => self.max_depth = max_depth,
            Ok(Command::SetMinDepth(min_depth)) => self.min_depth = min_depth,
            Ok(Command::SetConfidenceRange(confidence_range)) => {
                self.confidence_range = confidence_range
            }
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => std::process::exit(2),
        }

        match self.point_receiver.try_recv() {
            Ok(points) => {
                self.point_cloud_renderer.clear();
                for point in points {
                    if self.max_depth.is_some_and(|max_depth| point.z > max_depth)
                        || self.min_depth.is_some_and(|min_depth| point.z < min_depth)
                    {
                        continue;
                    }

                    let colour = match &self.confidence_range {
                        Some(range) => {
                            let low = *range.start();
                            let high = *range.end();

                            if low < high {
                                if point.confidence < low {
                                    Point3::new(1.0, 0.0, 0.0)
                                } else if point.confidence > high {
                                    Point3::new(0.0, 1.0, 0.0)
                                } else {
                                    let confidence = (point.confidence - low) / (high - low);
                                    Point3::new(1.0 - confidence, confidence, 0.0)
                                }
                            } else {
                                if point.confidence > low {
                                    Point3::new(1.0, 0.0, 0.0)
                                } else if point.confidence < high {
                                    Point3::new(0.0, 1.0, 0.0)
                                } else {
                                    let confidence = (point.confidence - low) / (high - low);
                                    Point3::new(1.0 - confidence, confidence, 0.0)
                                }
                            }
                        }
                        None => Point3::new(1.0, 1.0, 1.0),
                    };

                    self.point_cloud_renderer
                        .push(Point3::new(point.x, point.y, point.z), colour);
                }
            }
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => std::process::exit(1),
        }

        let num_points_text = format!(
            "Number of points: {}",
            self.point_cloud_renderer.num_points()
        );
        window.draw_text(
            &num_points_text,
            &Point2::new(0.0, 20.0),
            60.0,
            &Font::default(),
            &Point3::new(1.0, 1.0, 1.0),
        );
    }
}

fn main() {
    let (point_sender, point_receiver) = std::sync::mpsc::channel::<Vec<MyPoint>>();

    std::thread::spawn(move || tcp_thread(point_sender));

    let (command_sender, command_receiver) = std::sync::mpsc::channel::<Command>();

    std::thread::spawn(move || control_thread(command_sender));

    let window = Window::new("Kiss3d: persistent_point_cloud");
    let app = AppState {
        point_cloud_renderer: PointCloudRenderer::new(4.0),
        point_receiver,
        command_receiver,
        max_depth: None,
        min_depth: None,
        confidence_range: None,
    };

    window.render_loop(app)
}

/// Structure which manages the display of long-living points.
struct PointCloudRenderer {
    shader: Effect,
    pos: ShaderAttribute<Point3<f32>>,
    color: ShaderAttribute<Point3<f32>>,
    proj: ShaderUniform<Matrix4<f32>>,
    view: ShaderUniform<Matrix4<f32>>,
    colored_points: GPUVec<Point3<f32>>,
    point_size: f32,
}

impl PointCloudRenderer {
    /// Creates a new points renderer.
    fn new(point_size: f32) -> PointCloudRenderer {
        let mut shader = Effect::new_from_str(VERTEX_SHADER_SRC, FRAGMENT_SHADER_SRC);

        shader.use_program();

        PointCloudRenderer {
            colored_points: GPUVec::new(Vec::new(), BufferType::Array, AllocationType::StreamDraw),
            pos: shader.get_attrib::<Point3<f32>>("position").unwrap(),
            color: shader.get_attrib::<Point3<f32>>("color").unwrap(),
            proj: shader.get_uniform::<Matrix4<f32>>("proj").unwrap(),
            view: shader.get_uniform::<Matrix4<f32>>("view").unwrap(),
            shader,
            point_size,
        }
    }

    fn push(&mut self, point: Point3<f32>, color: Point3<f32>) {
        if let Some(colored_points) = self.colored_points.data_mut() {
            colored_points.push(point);
            colored_points.push(color);
        }
    }

    fn num_points(&self) -> usize {
        self.colored_points.len() / 2
    }

    fn clear(&mut self) {
        if let Some(points) = self.colored_points.data_mut() {
            points.clear();
        }
    }
}

impl Renderer for PointCloudRenderer {
    /// Actually draws the points.
    fn render(&mut self, pass: usize, camera: &mut dyn Camera) {
        if self.colored_points.len() == 0 {
            return;
        }

        self.shader.use_program();
        self.pos.enable();
        self.color.enable();

        camera.upload(pass, &mut self.proj, &mut self.view);

        self.color.bind_sub_buffer(&mut self.colored_points, 1, 1);
        self.pos.bind_sub_buffer(&mut self.colored_points, 1, 0);

        let ctxt = Context::get();
        ctxt.point_size(self.point_size);
        ctxt.draw_arrays(Context::POINTS, 0, (self.colored_points.len() / 2) as i32);

        self.pos.disable();
        self.color.disable();
    }
}

const VERTEX_SHADER_SRC: &str = "#version 100
    attribute vec3 position;
    attribute vec3 color;
    varying   vec3 Color;
    uniform   mat4 proj;
    uniform   mat4 view;
    void main() {
        gl_Position = proj * view * vec4(position, 1.0);
        Color = color;
    }";

const FRAGMENT_SHADER_SRC: &str = "#version 100
#ifdef GL_FRAGMENT_PRECISION_HIGH
   precision highp float;
#else
   precision mediump float;
#endif

    varying vec3 Color;
    void main() {
        gl_FragColor = vec4(Color, 1.0);
    }";

fn tcp_thread(sender: Sender<Vec<MyPoint>>) {
    let listener = std::net::TcpListener::bind("0.0.0.0:8080").unwrap();
    let stream = listener.accept().unwrap().0;

    let mut stream = bincode::de::Deserializer::with_reader(
        stream,
        DefaultOptions::new().allow_trailing_bytes(),
    );

    loop {
        sender
            .send(Vec::<MyPoint>::deserialize(&mut stream).unwrap())
            .unwrap();
    }
}

enum Command {
    SetMaxDepth(Option<f32>),
    SetMinDepth(Option<f32>),
    SetConfidenceRange(Option<RangeInclusive<f32>>),
}

fn control_thread(sender: Sender<Command>) {
    let mut stdin = BufReader::new(std::io::stdin());
    let mut stdout = std::io::stdout();
    let mut input = String::new();
    loop {
        input.clear();
        print!("\n> ");
        let _ = stdout.flush();
        stdin.read_line(&mut input).unwrap();
        let input = input.trim();

        match input {
            "" => (),
            "clear max depth" => sender.send(Command::SetMaxDepth(None)).unwrap(),
            "clear min depth" => sender.send(Command::SetMinDepth(None)).unwrap(),
            "clear confidence range" => sender.send(Command::SetConfidenceRange(None)).unwrap(),
            input if input.starts_with("set max depth ") => {
                let max_depth = input.strip_prefix("set max depth ").unwrap();
                match max_depth.parse::<f32>() {
                    Ok(max_depth) => sender.send(Command::SetMaxDepth(Some(max_depth))).unwrap(),
                    Err(e) => println!("{e}"),
                }
            }
            input if input.starts_with("set min depth ") => {
                let min_depth = input.strip_prefix("set min depth ").unwrap();
                match min_depth.parse::<f32>() {
                    Ok(min_depth) => sender.send(Command::SetMinDepth(Some(min_depth))).unwrap(),
                    Err(e) => println!("{e}"),
                }
            }
            input if input.starts_with("set confidence range ") => {
                let range = input.strip_prefix("set confidence range ").unwrap();
                let Some((low, high)) = range.split_once(' ') else {
                    println!("set confidence range <LOW> <HIGH>");
                    continue;
                };

                let low = match low.parse::<f32>() {
                    Ok(low) => low,
                    Err(e) => {
                        println!("Bad lower bound: {e}");
                        continue;
                    }
                };

                let high = match high.parse::<f32>() {
                    Ok(high) => high,
                    Err(e) => {
                        println!("Bad higher bound: {e}");
                        continue;
                    }
                };

                sender
                    .send(Command::SetConfidenceRange(Some(low..=high)))
                    .unwrap()
            }

            _ => println!("Unrecognised input"),
        }
    }
}
