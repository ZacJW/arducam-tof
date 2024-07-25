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
typedef enum { CSI = 0, USB, CONNECT_COUNT } ArducamConnection;

/**
 * @brief DeviceType
 */
typedef enum {
    ARDUCAM_DEVICE_VGA,
    ARDUCAM_DEVICE_HQVGA,
} ArducamDeviceType;

/**
 * @brief Some types of frame data
 *
 */
typedef enum {
    RAW_FRAME = 0,
    CONFIDENCE_FRAME,
    DEPTH_FRAME,
    CACHE_FRAME,
    FRAME_TYPE_COUNT,
} ArducamFrameType;

typedef enum {
    ARDUCAM_CONTROL_RANGE = 0, /**< only support 4m and 2m range mode */
    ARDUCAM_CONTROL_FMT_WIDTH,
    ARDUCAM_CONTROL_FMT_HEIGHT,
    ARDUCAM_CONTROL_MODE = 0x10,
    ARDUCAM_CONTROL_FRAME_MODE,
    ARDUCAM_CONTROL_EXPOSURE = 0x20,
    ARDUCAM_CONTROL_FRAME_RATE,
    ARDUCAM_CONTROL_SKIP_FRAME = 0x70,
    ARDUCAM_CONTROL_SKIP_FRAME_LOOP,
} ArducamControl;

typedef enum {
    ARDUCAM_MODE_SINGLE_FREQ = 0u, //!< work with single frequency
    ARDUCAM_MODE_DOUBLE_FREQ,
    ARDUCAM_MODE_TRIPLE_FREQ,
    ARDUCAM_MODE_QUAD_FREQ,
    ARDUCAM_MODE_DISTANCE, //!< according to distance measurement parameters to choose parameters
    ARDUCAM_MODE_HDR,      //!< configure chip to measure very near or far object (useful for AE)
    ARDUCAM_MODE_AE,
    ARDUCAM_MODE_BG_OUTDOOR, //!< configure chip to measure background IR radiation (no light from vcsel during phase frame)
    ARDUCAM_MODE_GRAY_ONLY,
    ARDUCAM_MODE_CUSTOM1, //!< for hw-cstar project
    ARDUCAM_MODE_CUSTOM2,
    ARDUCAM_MODE_CUSTOM3,
} ArducamTofWorkMode;

typedef enum {
    ARDUCAM_SUBMODE_SINGLE_FREQ_2PHASE = 0u,
    ARDUCAM_SUBMODE_SINGLE_FREQ_4PHASE,
    ARDUCAM_SUBMODE_SINGLE_FREQ_4PHASE_GRAY,            //!< whole frame: 4phase+gray
    ARDUCAM_SUBMODE_SINGLE_FREQ_4PHASE_BG,              //!< whole frame: 4phase+bg
    ARDUCAM_SUBMODE_SINGLE_FREQ_4PHASE_4BG,             //!< whole frame: 4phase+4bg
    ARDUCAM_SUBMODE_SINGLE_FREQ_4PHASE_GRAY_5BG,        //!< whole frame: 4phase+gray+5bg
    ARDUCAM_SUBMODE_SINGLE_FREQ_GRAY_BG_4PHASE_GRAY_BG, // add for hw-cstar project
    ARDUCAM_SUBMODE_SINGLE_FREQ_GRAY_BG_4PHASE_BG,      // add for hw-cstar-v2 project
    ARDUCAM_SUBMODE_SINGLE_FREQ_BG_GRAY_BG_4PHASE,      // add for hw-cstar-v2 project
    ARDUCAM_SUBMODE_SINGLE_FREQ_BG_4PHASE_BG_GRAY,      // add for hw-cstar-v2 project

    ARDUCAM_SUBMODE_DOUBLE_FREQ_4PHASE,                //!< each frequency has 4phase, whole frame: 4phase+4phase
    ARDUCAM_SUBMODE_DOUBLE_FREQ_4PHASE_GRAY_4PHASE_BG, //!< whole frame: (4phase+gray)+(4phase+bg)
    ARDUCAM_SUBMODE_DOUBLE_FREQ_4PHASE_4BG,            //!< whole frame: (4phase+4bg)+(4phase+4bg)
    ARDUCAM_SUBMODE_DOUBLE_FREQ_4PHASE_GRAY_5BG,       //!< whole frame: (4phase+gray+5bg)+(4phase+gray+5bg)

    ARDUCAM_SUBMODE_TRIPLE_FREQ_4PHASE,                            //!< whole frame: (4phase)+(4phase)+(4phase)
    ARDUCAM_SUBMODE_TRIPLE_FREQ_4PHASE_GRAY_4PHASE_GRAY_4PHASE_BG, //!< whole frame: (4phase+gray)+(4phase+gray)+(4phase+bg)

    ARDUCAM_SUBMODE_QUAD_FREQ_4PHASE, //!< whole frame: (4phase)+(4phase)+(4phase)+(4phase)
    ARDUCAM_SUBMODE_QUAD_FREQ_4PHASE_GRAY_4PHASE_BG_4PHASE_GRAY_4PHASE_BG,
    ARDUCAM_SUBMODE_BG_OUTDOOR,
    ARDUCAM_SUBMODE_GRAY_ONLY,
    ARDUCAM_SUBMODE_CUSTOM,
} ArducamTofFrameWorkMode;

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
 * @brief Basic information of the camera module
 */
typedef struct {
    unsigned int index;
    ArducamConnection connect;
    ArducamDeviceType device_type;
    ArducamFrameType type;
    unsigned int width;
    unsigned int height;
    unsigned int bit_width;
    unsigned int bpp; // bytes per pixel
} ArducamCameraInfo;

/**
 * @brief Create a camera instance.
 *
 * @return Return a ArducamDepthCamera instance.
 */
extern ArducamDepthCamera createArducamDepthCamera();

/**
 * @brief Initialize the camera configuration and turn on the camera,
 *   set the initialization frame according to the `conn`.
 *
 * @param camera Camera instance, obtained through @ref createArducamDepthCamera().
 * @param conn Specify the connection method.
 *      This parameter can be one of the following values:
 *          @arg CSI
 *          @arg USB
 * @param index Device node, the default value is video0.
 *
 * @return Return Status code.
 */
extern Status arducamCameraOpen(ArducamDepthCamera camera, ArducamConnection conn, int index);

/**
 * @brief Initialize the camera configuration and turn on the camera,
 *   set the initialization frame according to the `path`.
 *
 * @param camera Camera instance, obtained through @ref createArducamDepthCamera().
 * @param path Specify the configuration file path.
 * @param index Device node, the default value is video0.
 *
 * @return Return Status code.
 */
extern Status arducamCameraOpenWithFile(ArducamDepthCamera camera, const char* path, int index);

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
 * @brief Get the Camera frames format.
 *
 * @param camera Camera instance.
 *
 * @return All frame data formats contained in frame, The returned value include: width, height and Frametype
 */
extern ArducamCameraInfo arducamCameraGetInfo(ArducamDepthCamera camera);

/**
 * @brief Get the format of the specified frame.
 *
 * @param fb Frame instance.
 * @param type Frame type.
 * This parameter can be one of the following values:
 *          @arg RAW_FRAME
 *          @arg CONFIDENCE_FRAME
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
extern Status arducamCameraSetCtrl(ArducamDepthCamera camera, ArducamControl id, int val);

/**
 * @brief Get the current camera output format.
 *
 * @param camera Camera instance.
 * @param id The id of the control.
 * @param val Return the value that needs to be get.
 *
 * @return Return Status code.
 */
extern Status arducamCameraGetCtrl(ArducamDepthCamera camera, ArducamControl id, int* val);

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
extern void* arducamCameraGetConfidenceData(ArducamFrameBuffer fb);

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
