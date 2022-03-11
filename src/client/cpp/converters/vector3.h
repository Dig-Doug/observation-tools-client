#pragma once

#include "src/api/artifacts/math.pb.h"
#include "src/client/cpp/converters/number.h"

namespace observation_tools {

template <typename Vector3x>
proto::Vector3 ConvertToVector3(const Vector3x &p) {
  proto::Vector3 vector;
  *vector.mutable_x() = ConvertNumber(p.x());
  *vector.mutable_y() = ConvertNumber(p.y());
  *vector.mutable_z() = ConvertNumber(p.z());
  return vector;
}

template <typename Vector_3> proto::Vector3 ConvertVector_3(const Vector_3 &p) {
  proto::Vector3 vector;
  *vector.mutable_x() = ConvertNumber(p.x());
  *vector.mutable_y() = ConvertNumber(p.y());
  *vector.mutable_z() = ConvertNumber(p.z());
  return vector;
}

} // namespace observation_tools