#pragma once

#include "src/api/artifacts/math.pb.h"
#include "src/client/cpp/converters/number.h"

namespace observation_tools {

template <typename Vector3> proto::Point3 ConvertToPoint3(const Vector3 &p) {
  proto::Point3 point;
  *point.mutable_x() = ConvertNumber(p.x());
  *point.mutable_y() = ConvertNumber(p.y());
  *point.mutable_z() = ConvertNumber(p.z());
  return point;
}

template <typename Point_3> proto::Point3 ConvertPoint_3(const Point_3 &p) {
  proto::Point3 point;
  *point.mutable_x() = ConvertNumber(p.x());
  *point.mutable_y() = ConvertNumber(p.y());
  *point.mutable_z() = ConvertNumber(p.z());
  return point;
}

}