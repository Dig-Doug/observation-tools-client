#include "src/client/cpp/converters/transform3.h"
#include "src/client/cpp/converters/number.h"

namespace observation_tools {

proto::Transform3 Transform3Identity() {
  proto::Transform3 transform;
  transform.set_identity(true);
  return transform;
}

proto::Vector3 PointToVector3(const proto::Point3 &p) {
  proto::Vector3 v;
  *v.mutable_x() = p.x();
  *v.mutable_y() = p.y();
  *v.mutable_z() = p.z();
  return v;
}

proto::Transform3 Transform3FromTranslation(const proto::Point3 &p) {
  return Transform3FromTranslation(PointToVector3(p));
}

proto::Transform3 Transform3FromTranslation(const proto::Vector3 &v) {
  proto::Transform3 transform;
  auto *trs = transform.mutable_trs();
  *trs->mutable_translation() = v;
  return transform;
}

proto::Transform3 ConvertToTransform3(const proto::Transform3 &t) {
  return t;
}

proto::Transform3 ConvertToTransform3(const proto::Matrix4x4 &m) {
  proto::Transform3 transform3;
  *transform3.mutable_matrix() = m;
  return transform3;
}

proto::Transform3 CoordinateSystemRHZUp() {
  // https://www.techarthub.com/wp-content/uploads/coordinate-comparison-chart-full.jpg
  proto::Matrix4x4 proto;
  *proto.mutable_m0_0() = ConvertNumber(1);
  *proto.mutable_m0_1() = ConvertNumber(0);
  *proto.mutable_m0_2() = ConvertNumber(0);
  *proto.mutable_m0_3() = ConvertNumber(0);
  *proto.mutable_m1_0() = ConvertNumber(0);
  *proto.mutable_m1_1() = ConvertNumber(0);
  *proto.mutable_m1_2() = ConvertNumber(1);
  *proto.mutable_m1_3() = ConvertNumber(0);
  *proto.mutable_m2_0() = ConvertNumber(0);
  *proto.mutable_m2_1() = ConvertNumber(-1);
  *proto.mutable_m2_2() = ConvertNumber(0);
  *proto.mutable_m2_3() = ConvertNumber(0);
  *proto.mutable_m3_0() = ConvertNumber(0);
  *proto.mutable_m3_1() = ConvertNumber(0);
  *proto.mutable_m3_2() = ConvertNumber(0);
  *proto.mutable_m3_3() = ConvertNumber(1);
  return ConvertToTransform3(proto);
}

}