#include "src/client/cpp/converters/matrix4x4.h"
#include "src/client/cpp/converters/number.h"

namespace observation_tools {

proto::Matrix4x4 ConvertToMatrix4(const Transform3d &p) {
  proto::Matrix4x4 proto;
  *proto.mutable_m0_0() = ConvertNumber(p.matrix()(0, 0));
  *proto.mutable_m0_1() = ConvertNumber(p.matrix()(0, 1));
  *proto.mutable_m0_2() = ConvertNumber(p.matrix()(0, 2));
  *proto.mutable_m0_3() = ConvertNumber(p.matrix()(0, 3));
  *proto.mutable_m1_0() = ConvertNumber(p.matrix()(1, 0));
  *proto.mutable_m1_1() = ConvertNumber(p.matrix()(1, 1));
  *proto.mutable_m1_2() = ConvertNumber(p.matrix()(1, 2));
  *proto.mutable_m1_3() = ConvertNumber(p.matrix()(1, 3));
  *proto.mutable_m2_0() = ConvertNumber(p.matrix()(2, 0));
  *proto.mutable_m2_1() = ConvertNumber(p.matrix()(2, 1));
  *proto.mutable_m2_2() = ConvertNumber(p.matrix()(2, 2));
  *proto.mutable_m2_3() = ConvertNumber(p.matrix()(2, 3));
  *proto.mutable_m3_0() = ConvertNumber(p.matrix()(3, 0));
  *proto.mutable_m3_1() = ConvertNumber(p.matrix()(3, 1));
  *proto.mutable_m3_2() = ConvertNumber(p.matrix()(3, 2));
  *proto.mutable_m3_3() = ConvertNumber(p.matrix()(3, 3));
  return proto;
}

}