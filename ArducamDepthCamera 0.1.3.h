/* Copyright 2021 Arducam Technology co., Ltd. All Rights Reserved.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

        http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
==============================================================================*/

#ifndef _ARDUCAM_DEPTH_CAMERA_H_
#define _ARDUCAM_DEPTH_CAMERA_H_

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#ifndef DOXYGEN_SHOULD_SKIP_THIS

#define Status int

typedef void* ArducamDepthCamera;

typedef void* ArducamFrameBuffer;

#endif

/**
 * @brief camera connection method.
 * 
*/
typedef enum { CSI = 0, USB, CONNECT_COUNT } ArducamCameraConn;

/**
 * @brief Some types of frame data
 *
 */
typedef enum {
    RAW_FRAME = 0,
    AMPLITUDE_FRAME,
    DEPTH_FRAME,
    FRAME_TYPE_COUNT,
} ArducamFrameType;

typedef enum {
    ArducamCameraRange = 0,
} ArducamCameraCtrl;

/**
 * @brief Description of frame data format
 *
 */
typedef struct {
    //! width of frame
    uint16_t width;  
    //! height of frame     
    uint16_t height;      
    //! type of frame
    ArducamFrameType type;
    //! timestamp of frame
    uint64_t timestamp;
} ArducamFrameFormat;

/**
 * @brief Create a camera instance.
 *
 * @return Return a ArducamDepthCamera instance.
 */
extern ArducamDepthCamera createArducamDepthCamera();

/**
 * @brief Initialize the camera configuration and turn on the camera, set the initialization frame according to the @ref
 * conn.
 *
 * @param camera Camera instance, obtained through @ref createArducamDepthCamera().
 * @param conn Specify the connection method.
 *      This parameter can be one of the following values:
 *          @arg CSI
 *          @arg USB
 * @param path Device node, the default value is video0.
 *
 * @return Return Status code.
 */
extern Status arducamCameraOpen(ArducamDepthCamera camera, ArducamCameraConn conn, int path);

/**
 * @brief Close camera.
 *
 * @param camera Camera instance.
 *
 * @return Return Status code.
 */
extern Status arducamCameraClose(ArducamDepthCamera* camera);

/**
 * @brief Start the camera stream and start processing.
 *
 * @param camera Camera instance.
 * @param type Type of camera output frame.
 *
 * @return Return Status code.
 */
extern Status arducamCameraStart(ArducamDepthCamera camera, ArducamFrameType type);

/**
 * @brief Stop camera stream and processing.
 *
 * @param camera Camera instance.
 *
 * @return Return Status code.
 */
extern Status arducamCameraStop(ArducamDepthCamera camera);

// /**
//  * @brief Specifies the frame format.
//  *
//  * @param camera Camera instance.
//  * @param format Specifies the frame format. If the set format is not supported,
//  * the function will modify the value of this parameter to return the actual value used.
//  *
//  * @return Return Status code.
//  */
// extern Status setFormat(ArducamDepthCamera *camera, FrameFormat *format);

/**
 * @brief Get the format of the specified frame.
 *
 * @param fb Frame instance.
 * @param type Frame type.
 * This parameter can be one of the following values:
 *          @arg RAW_FRAME
 *          @arg AMPLITUDE_FRAME
 *          @arg DEPTH_FRAME
 *
 * @return Return frame format.
 */
extern ArducamFrameFormat arducamCameraGetFormat(ArducamFrameBuffer fb, ArducamFrameType type);

/**
 * @brief Get the current camera output format.
 *
 * @param camera Camera instance.
 * @param id The id of the control.
 * @param val The value that needs to be set.
 *
 * @return Return Status code.
 */
extern Status arducamCameraSetCtrl(ArducamDepthCamera camera, ArducamCameraCtrl id, int val);

/**
 * @brief Get the current camera output format.
 *
 * @param camera Camera instance.
 * @param id The id of the control.
 * @param val Return the value that needs to be get.
 *
 * @return Return Status code.
 */
extern Status arducamCameraGetCtrl(ArducamDepthCamera camera, ArducamCameraCtrl id, int *val);

/**
 * @brief Read frame from the camera.
 *
 * @param camera Camera instance.
 * @param timeout Timeout time, -1 means to wait all the time, 0 means immediate range,
 * other values indicate the maximum waiting time, the unit is milliseconds.
 *
 * @return Return Status code.
 */
extern ArducamFrameBuffer arducamCameraRequestFrame(ArducamDepthCamera camera, int timeout);

/**
 * @brief Release the ArducamFrameBuffer.
 *
 * @param camera Camera instance.
 * @param fb  ArducamFrameBuffer.
 *
 * @return Return Status code.
 */
extern Status arducamCameraReleaseFrame(ArducamDepthCamera camera, ArducamFrameBuffer fb);

/**
 * @brief Read depth data from the frame.
 * @note The output mode is the depth type, and the function can be called to obtain data
 *
 * @param fb dataframe object.
 *
 * @return Return Status code.
 */
extern void* arducamCameraGetDepthData(ArducamFrameBuffer fb);
/**
 * @brief Read depth data from the frame.
 * @note The output mode is the depth type, and the function can be called to obtain data.
 *
 * @param fb dataframe object.
 *
 * @return Return Status code.
 */
extern void* arducamCameraGetAmplitudeData(ArducamFrameBuffer fb);

/**
 * @brief Read raw data from the frame.
 * @note The output mode is the raw type, and the function can be called to obtain data.
 *
 * @param fb dataframe object.
 *
 * @return Return Status code.
 */
extern void* arducamCameraGetRawData(ArducamFrameBuffer fb);

#ifdef __cplusplus
}
#endif
#endif /*__ARDUCAM_DEPTH_CAMERA_H_*/
