use std::time::Duration;

fn main() {
    let mut cam = arducam_tof::ArducamDepthCamera::new().unwrap();
    cam.open(arducam_tof::Connection::CSI, 0).unwrap();
    cam.start(arducam_tof::FrameType::DepthFrame).unwrap();

    opencv::highgui::named_window("depth", opencv::highgui::WINDOW_NORMAL).unwrap();

    loop {
        let frame = cam.request_frame(Some(Duration::from_millis(200))).unwrap();

        let depth = frame.get_depth_data();

        let depth_mat = opencv::core::Mat::new_rows_cols_with_data(depth.height() as i32, depth.width() as i32, depth.as_slice()).unwrap();

        opencv::highgui::imshow("depth", &depth_mat).unwrap();
    }

}
