#include "src/client/cpp/converters/opencv.h"
#include "src/client/cpp/converters/structured.h"
#include "opencv2/opencv.hpp"
#include "src/api/artifacts/artifact.pb.h"
#include <string>

namespace observation_tools {

proto::Image2 ConvertImage2(const cv::Mat &val) {
  std::vector<unsigned char> buff;
  cv::imencode(".png", val, buff);
  proto::Image2 raw_data;
  *raw_data.mutable_data() =
      std::string{reinterpret_cast<char *>(buff.data()), buff.size()};
  raw_data.set_mime_type("image/png");
  return raw_data;
}

} // namespace observation_tools
