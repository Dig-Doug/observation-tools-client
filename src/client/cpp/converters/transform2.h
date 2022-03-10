#pragma once

#include "src/api/artifacts/math.pb.h"
#include "src/client/cpp/converters/matrix4x4.h"

namespace observation_tools {

proto::Transform2 Transform2Identity();

proto::Transform2 Transform2FromTranslation(const proto::Point2 &v);

proto::Transform2 Transform2FromTranslation(const proto::Vector2 &v);

proto::Transform2 ConvertToTransform2(const proto::Transform2 &t);

} // namespace observation_tools