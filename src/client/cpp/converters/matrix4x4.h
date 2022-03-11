#pragma once

#include "src/api/artifacts/math.pb.h"
#include "eigen3/Eigen/Dense"
#include "eigen3/Eigen/Geometry"

namespace observation_tools {

using Transform3d = ::Eigen::Transform<double, 3, Eigen::Affine>;
proto::Matrix4x4 ConvertToMatrix4(const Transform3d &p);

}