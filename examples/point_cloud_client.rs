use std::time::Duration;

use bincode::Options;
use serde::Serialize;

#[derive(Serialize)]
struct MyPoint {
    x: f32,
    y: f32,
    z: f32,
    confidence: f32,
}

fn main() {
    let mut cam = arducam_tof::ArducamDepthCamera::new().unwrap();
    cam.open(arducam_tof::Connection::CSI, 0).unwrap();
    cam.start(arducam_tof::FrameType::DepthFrame).unwrap();

    let addr = std::env::args().nth(1).unwrap();

    let stream = std::net::TcpStream::connect((addr, 8080)).unwrap();

    opencv::highgui::named_window("depth", opencv::highgui::WINDOW_NORMAL).unwrap();

    let mut points = Vec::new();

    let options = bincode::DefaultOptions::new().allow_trailing_bytes();

    let mut stream = bincode::Serializer::new(stream, options);
    loop {
        let frame = cam.request_frame(Some(Duration::from_millis(200))).unwrap();

        let depth = frame.get_depth_data();

        let confidence = frame.get_confidence_data();

        assert!(depth.width() == confidence.width());
        assert!(depth.height() == confidence.height());

        let pixels = depth
            .as_slice()
            .iter()
            .enumerate()
            .zip(confidence.as_slice())
            .map(|((i, d), c)| (i % depth.width() as usize, i / depth.width() as usize, d, c));

        let fx = depth.width() as f32 / (2.0 * f32::tan(0.5 * std::f32::consts::PI * 64.3 / 180.0)); // 640 / 2 / tan(0.5*64.3)
        let fy =
            depth.height() as f32 / (2.0 * f32::tan(0.5 * std::f32::consts::PI * 50.4 / 180.0)); // 480 / 2 / tan(0.5*50.4)

        points.clear();

        for (row, column, d, c) in pixels {
            let z = *d;
            let x = (((depth.width() / 2) as f32 - column as f32) / fx) * z;
            let y = (((depth.height() / 2) as f32 - row as f32) / fy) * z;

            points.push(MyPoint {
                x,
                y,
                z,
                confidence: *c,
            });
        }

        points.serialize(&mut stream).unwrap();

        let depth_mat = opencv::core::Mat::new_rows_cols_with_data(
            depth.height() as i32,
            depth.width() as i32,
            depth.as_slice(),
        )
        .unwrap();

        opencv::highgui::imshow("depth", &depth_mat).unwrap();

        let key = opencv::highgui::wait_key(10).unwrap();

        if key == b'q' as i32 {
            break;
        }
    }
}
