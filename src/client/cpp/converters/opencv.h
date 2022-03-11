#pragma once

#include "opencv2/opencv.hpp"
#include "src/api/artifacts/artifact.pb.h"

namespace observation_tools {

proto::Image2 ConvertImage2(const cv::Mat &val);

} // namespace observation_tools
