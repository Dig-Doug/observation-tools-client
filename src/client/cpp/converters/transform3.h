#pragma once

#include "src/api/artifacts/math.pb.h"
#include "src/client/cpp/converters/matrix4x4.h"

namespace observation_tools {

proto::Transform3 Transform3Identity();

proto::Transform3 Transform3FromTranslation(const proto::Point3 &v);

proto::Transform3 Transform3FromTranslation(const proto::Vector3 &v);

proto::Transform3 ConvertToTransform3(const proto::Transform3 &t);

proto::Transform3 ConvertToTransform3(const proto::Matrix4x4 &m);

template <typename MatrixT>
proto::Transform3 ConvertToTransform3(const MatrixT &t) {
  return ConvertToTransform3(ConvertToMatrix4(t));
}

proto::Transform3 CoordinateSystemRHZUp();

} // namespace observation_tools