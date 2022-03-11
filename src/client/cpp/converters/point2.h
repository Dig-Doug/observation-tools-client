#pragma once

#include "src/api/artifacts/math.pb.h"
#include "src/client/cpp/converters/number.h"

namespace observation_tools {

template <typename Vector2> proto::Point2 ConvertToPoint2(const Vector2 &p) {
  proto::Point2 point;
  *point.mutable_x() = ConvertNumber(p.x());
  *point.mutable_y() = ConvertNumber(p.y());
  return point;
}

template <typename Point_2> proto::Point2 ConvertPoint_2(const Point_2 &p) {
  proto::Point2 point;
  *point.mutable_x() = ConvertNumber(p.x());
  *point.mutable_y() = ConvertNumber(p.y());
  return point;
}

}