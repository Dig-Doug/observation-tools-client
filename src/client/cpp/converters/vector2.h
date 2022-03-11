#pragma once

#include "src/api/artifacts/math.pb.h"
#include "src/client/cpp/converters/number.h"

namespace observation_tools {

template <typename Vector2x>
proto::Vector2 ConvertToVector2(const Vector2x &p) {
  proto::Vector2 vector;
  *vector.mutable_x() = ConvertNumber(p.x());
  *vector.mutable_y() = ConvertNumber(p.y());
  return vector;
}

template <typename Vector_2> proto::Vector2 ConvertVector_2(const Vector_2 &p) {
  proto::Vector2 vector;
  *vector.mutable_x() = ConvertNumber(p.x());
  *vector.mutable_y() = ConvertNumber(p.y());
  return vector;
}

}