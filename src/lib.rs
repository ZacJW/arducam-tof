use std::{marker::PhantomData, num::NonZero, ptr::NonNull, time::Duration};

use thiserror::Error;

mod raw {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(dead_code)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

/// The handle to use to perform camera operations
pub struct ArducamDepthCamera {
    inner: NonNull<std::ffi::c_void>,
    opened: bool,
    started: bool,
}

#[derive(Debug, Error)]
#[error("Failed to init camera")]
/// Returned when [ArducamDepthCamera::new] fails to create a camera handle
pub struct InitError;

#[derive(Debug, Error)]
#[error("Failed to request frame from camera")]
/// Returned when [ArducamDepthCamera::request_frame] fails
pub struct RequestFrameError;

#[derive(Debug, Error)]
#[error("Failed to open camera, got error code: {0}")]
pub struct OpenError(NonZero<std::ffi::c_int>);

#[derive(Debug, Error)]
#[error("Failed to close camera, got error code: {0}")]
pub struct CloseError(NonZero<std::ffi::c_int>);

#[derive(Debug, Error)]
#[error("Failed to start camera, got error code: {0}")]
pub struct StartError(NonZero<std::ffi::c_int>);

#[derive(Debug, Error)]
#[error("Failed to stop camera, got error code: {0}")]
pub struct StopError(NonZero<std::ffi::c_int>);

impl ArducamDepthCamera {
    pub fn new() -> Result<Self, InitError> {
        let inner = unsafe { raw::createArducamDepthCamera() };
        let inner = NonNull::<std::ffi::c_void>::new(inner).ok_or(InitError)?;
        Ok(Self {
            inner,
            opened: false,
            started: false,
        })
    }

    pub fn open(&mut self, conn: Connection, index: i32) -> Result<(), OpenError> {
        let status =
            unsafe { raw::arducamCameraOpen(self.inner.as_ptr(), conn.into(), index as _) };
        match NonZero::new(status) {
            Some(error) => Err(OpenError(error)),
            None => {
                self.opened = true;
                Ok(())
            }
        }
    }

    pub fn start(&mut self, frame_type: FrameType) -> Result<(), StartError> {
        let status = unsafe { raw::arducamCameraStart(self.inner.as_ptr(), frame_type.into()) };
        match NonZero::new(status) {
            Some(error) => Err(StartError(error)),
            None => {
                self.started = true;
                Ok(())
            }
        }
    }

    // pub fn get_info(&self) -> CameraInfo {
    //     let info = unsafe { raw::arducamCameraGetInfo(self.inner.as_ptr()) };
    //     CameraInfo {
    //         connection: info.connect.try_into().unwrap(),
    //         device_type: info.device_type.try_into().unwrap(),
    //         frame_type: info.type_.try_into().unwrap(),
    //         width: info.width,
    //         height: info.height,
    //         bit_width: info.bit_width,
    //         bpp: info.bpp,
    //     }
    // }

    pub fn close(&mut self) -> Result<(), CloseError> {
        let status = unsafe {
            raw::arducamCameraClose(todo!(
                "blocked by https://github.com/ArduCAM/Arducam_tof_camera/issues/78"
            ))
        };
        match NonZero::new(status) {
            Some(error) => Err(CloseError(error)),
            None => {
                // Not sure whether to set self.started to false too
                self.opened = false;
                Ok(())
            }
        }
    }

    pub fn stop(&mut self) -> Result<(), StopError> {
        let status = unsafe { raw::arducamCameraStop(self.inner.as_ptr()) };
        match NonZero::new(status) {
            Some(error) => Err(StopError(error)),
            None => {
                self.started = false;
                Ok(())
            }
        }
    }

    pub fn request_frame(
        &mut self,
        timeout: Option<Duration>,
    ) -> Result<ArducamFrameBuffer, RequestFrameError> {
        let timeout = match timeout {
            Some(timeout) => timeout.as_millis() as std::ffi::c_int,
            None => -1,
        };
        let inner = unsafe { raw::arducamCameraRequestFrame(self.inner.as_ptr(), timeout) };
        let inner = NonNull::<std::ffi::c_void>::new(inner).ok_or(RequestFrameError)?;
        Ok(ArducamFrameBuffer {
            marker: PhantomData,
            inner,
            camera: self.inner,
        })
    }
}

impl Drop for ArducamDepthCamera {
    fn drop(&mut self) {
        if self.started {
            self.stop().unwrap();
        }

        if self.opened {
            self.close().unwrap();
        }
    }
}

pub struct ArducamFrameBuffer<'a> {
    marker: PhantomData<&'a ()>,
    inner: NonNull<std::ffi::c_void>,
    camera: NonNull<std::ffi::c_void>,
}

impl<'a> ArducamFrameBuffer<'a> {
    pub fn get_format(&self, frame_type: FrameType) -> ArducamFrameFormat {
        let format = unsafe { raw::arducamCameraGetFormat(self.inner.as_ptr(), frame_type.into()) };
        ArducamFrameFormat {
            width: format.width,
            height: format.height,
            frame_type: format.type_.try_into().unwrap(),
            timestamp: format.timestamp,
        }
    }

    pub fn get_depth_data<'b>(&'b self) -> FrameData<'b, f32> {
        let data = unsafe { raw::arducamCameraGetDepthData(self.inner.as_ptr()) };

        if data.is_null() {
            panic!("Got null pointer from arducamCameraGetDepthData");
        }

        let format = self.get_format(FrameType::DepthFrame);

        FrameData {
            width: format.width,
            height: format.height,
            data: unsafe {
                std::slice::from_raw_parts(
                    data as *mut f32,
                    format.width as usize * format.height as usize,
                )
            },
        }
    }

    pub fn get_confidence_data<'b>(&'b self) -> FrameData<'b, f32> {
        let data = unsafe { raw::arducamCameraGetAmplitudeData(self.inner.as_ptr()) }; // 0.1.3 called confidence amplitude

        if data.is_null() {
            panic!("Got null pointer from arducamCameraGetAmplitudeData");
        }

        let format = self.get_format(FrameType::ConfidenceFrame);

        FrameData {
            width: format.width,
            height: format.height,
            data: unsafe {
                std::slice::from_raw_parts(
                    data as *mut f32,
                    format.width as usize * format.height as usize,
                )
            },
        }
    }
}

impl<'a> Drop for ArducamFrameBuffer<'a> {
    fn drop(&mut self) {
        let status =
            unsafe { raw::arducamCameraReleaseFrame(self.camera.as_ptr(), self.inner.as_ptr()) };
        if status != 0 {
            panic!("Failed to release camera frame, got status: {status}");
        }
    }
}

/// A row-major frame buffer reference.
///
/// Created by calling [ArducamFrameBuffer::get_depth_data] or [ArducamFrameBuffer::get_confidence_data],
/// this type references frame data that is still owned by [ArducamFrameBuffer]
pub struct FrameData<'a, T> {
    width: u16,
    height: u16,
    data: &'a [T],
}

impl<'a, T: Copy> FrameData<'a, T> {
    /// Get the pixel value of the frame at the specified co-ordinates, or None if out of bounds.
    pub fn get(&self, x: u16, y: u16) -> Option<T> {
        self.data
            .get(x as usize + y as usize * self.width as usize)
            .copied()
    }

    /// Get a reference to the row-major slice of frame data.
    pub fn as_slice(&self) -> &[T] {
        &self.data
    }

    /// Get the width of the frame in pixels
    pub fn width(&self) -> u16 {
        self.width
    }

    /// Get the height of the frame in pixels
    pub fn height(&self) -> u16 {
        self.height
    }
}

impl<'a, 'b, T> IntoIterator for &'b FrameData<'a, T> {
    type Item = &'b T;

    type IntoIter = <&'b [T] as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

macro_rules! make_enum_from_c {
    {
        $(#[$attr:meta])*
        $vis:vis enum $type_name:ident: $c_type:ty {
            $(
                $variant_name:ident => $c_value:expr
            ),+
            $(,)?
        }
        $(#[$error_attr:meta])*
        invalid_type = $error_vis:vis struct $error_name:ident;
    } => {
        $(#[$attr])*
        $vis enum $type_name {
            $(
                $variant_name,
            )+
        }

        impl From<$type_name> for $c_type {
            fn from(value: $type_name) -> Self {
                match value {
                    $(
                        $type_name::$variant_name => $c_value,
                    )+
                }
            }
        }

        $(#[$error_attr])*
        $error_vis struct $error_name($c_type);

        impl TryFrom<$c_type> for $type_name {
            type Error = $error_name;

            fn try_from(value: $c_type) -> Result<Self, Self::Error> {
                match value {
                    $(
                        x if x == $c_value => Ok($type_name::$variant_name),
                    )+
                    other => Err($error_name(other)),
                }
            }
        }
    };
}

make_enum_from_c! {
    #[derive(Debug)]
    pub enum FrameType: raw::ArducamFrameType {
        RawFrame => raw::ArducamFrameType_RAW_FRAME,
        ConfidenceFrame => raw::ArducamFrameType_AMPLITUDE_FRAME, // 0.1.3 called confidence amplitude
        DepthFrame => raw::ArducamFrameType_DEPTH_FRAME,
        // CacheFrame => raw::ArducamFrameType_CACHE_FRAME, // 0.1.3 has no cache frame variant
    }
    #[derive(Debug, Error)]
    #[error("Invalid frame type: {0}")]
    invalid_type = pub struct InvalidFrameType;
}

make_enum_from_c! {
    #[derive(Debug)]
    pub enum Connection: raw::ArducamCameraConn {
        CSI => raw::ArducamCameraConn_CSI,
        USB => raw::ArducamCameraConn_USB,
    }
    #[derive(Debug, Error)]
    #[error("Invalid connection type: {0}")]
    invalid_type = pub struct InvalidConnectionType;
}

// 0.1.3 has no device type
// make_enum_from_c! {
//     #[derive(Debug)]
//     pub enum DeviceType: raw::ArducamDeviceType {
//         VGA => raw::ArducamDeviceType_ARDUCAM_DEVICE_VGA,
//         HQVGA => raw::ArducamDeviceType_ARDUCAM_DEVICE_HQVGA,
//     }
//     #[derive(Debug, Error)]
//     #[error("Invalid device type: {0}")]
//     invalid_type = pub struct InvalidDeviceType;
// }

// 0.1.3 doesn't have get_info so this type isn't used
// #[derive(Debug)]
// pub struct CameraInfo {
//     pub connection: Connection,
//     pub device_type: DeviceType,
//     pub frame_type: FrameType,
//     pub width: u32,
//     pub height: u32,
//     pub bit_width: u32,
//     pub bpp: u32,
// }

pub struct ArducamFrameFormat {
    pub width: u16,
    pub height: u16,
    pub frame_type: FrameType,
    pub timestamp: u64,
}
