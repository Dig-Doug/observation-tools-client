#include "src/client/cpp/converters/transform2.h"
#include "src/client/cpp/converters/number.h"

namespace observation_tools {

proto::Transform2 Transform2Identity() {
  proto::Transform2 transform;
  transform.set_identity(true);
  return transform;
}

proto::Vector2 PointToVector2(const proto::Point2 &p) {
  proto::Vector2 v;
  *v.mutable_x() = p.x();
  *v.mutable_y() = p.y();
  return v;
}

proto::Transform2 Transform2FromTranslation(const proto::Point2 &p) {
  return Transform2FromTranslation(PointToVector2(p));
}

proto::Transform2 Transform2FromTranslation(const proto::Vector2 &v) {
  proto::Transform2 transform;
  auto *trs = transform.mutable_trs();
  *trs->mutable_translation() = v;
  return transform;
}

proto::Transform2 ConvertToTransform2(const proto::Transform2 &t) {
  return t;
}

}