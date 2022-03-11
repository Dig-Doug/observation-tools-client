#include "src/client/cpp/converters/matrix3x3.h"
#include "src/client/cpp/converters/number.h"

namespace observation_tools {

proto::Matrix3x3 ConvertToMatrix3(const Transform2d &p) {
  proto::Matrix3x3 proto;
  *proto.mutable_m0_0() = ConvertNumber(p.matrix()(0, 0));
  *proto.mutable_m0_1() = ConvertNumber(p.matrix()(0, 1));
  *proto.mutable_m0_2() = ConvertNumber(p.matrix()(0, 2));
  *proto.mutable_m1_0() = ConvertNumber(p.matrix()(1, 0));
  *proto.mutable_m1_1() = ConvertNumber(p.matrix()(1, 1));
  *proto.mutable_m1_2() = ConvertNumber(p.matrix()(1, 2));
  *proto.mutable_m2_0() = ConvertNumber(p.matrix()(2, 0));
  *proto.mutable_m2_1() = ConvertNumber(p.matrix()(2, 1));
  *proto.mutable_m2_2() = ConvertNumber(p.matrix()(2, 2));
  return proto;
}

}