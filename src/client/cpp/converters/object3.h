#pragma once

#include "src/api/artifacts/math.pb.h"
#include "src/api/artifacts/artifact.pb.h"
#include "src/client/cpp/converters/number.h"
#include "src/client/cpp/converters/vector3.h"

namespace observation_tools {

template <typename PositionT, typename RadiusT>
proto::Object3 Sphere(const PositionT &position, const RadiusT &radius) {
  std::vector<PositionT> positions = {position};
  return Spheres(positions.begin(), positions.end(), radius);
}

template <typename PositionIt, typename RadiusT>
proto::Object3 Spheres(PositionIt points_begin,
                       PositionIt points_end, const RadiusT &radius) {
  proto::Object3 obj;
  auto *sphere = obj.mutable_geometry()->mutable_sphere();
  *sphere->mutable_radius() = ConvertNumber(radius);
  while (points_begin != points_end) {
    *obj.add_transforms() =
        Transform3FromTranslation(ConvertToVector3(*points_begin));
    points_begin++;
  }
  return obj;
}

} // namespace observation_tools