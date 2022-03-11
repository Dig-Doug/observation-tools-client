#pragma once

#include "src/client/cpp/converters/number.h"
#include "src/client/cpp/converters/vector2.h"
#include "src/client/cpp/converters/transform2.h"
#include "src/api/artifacts/artifact.pb.h"
#include "src/api/artifacts/math.pb.h"

namespace observation_tools {

template <typename PositionT> proto::Object2 Point(const PositionT &position) {
  std::vector<PositionT> positions = {position};
  return Points(positions.begin(), positions.end());
}

template <typename PositionIt>
proto::Object2 Points(PositionIt points_begin, PositionIt points_end) {
  proto::Object2 obj;
  obj.mutable_geometry()->mutable_point2();
  while (points_begin != points_end) {
    *obj.add_transforms() =
        Transform2FromTranslation(ConvertToVector2(*points_begin));
    points_begin++;
  }
  return obj;
}

} // namespace observation_tools