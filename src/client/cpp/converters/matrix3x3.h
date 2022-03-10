#pragma once

#include "src/api/artifacts/math.pb.h"
#include "eigen3/Eigen/Dense"
#include "eigen3/Eigen/Geometry"

namespace observation_tools {

using Transform2d = ::Eigen::Transform<double, 2, Eigen::Affine>;
proto::Matrix3x3 ConvertToMatrix3(const Transform2d &p);

}