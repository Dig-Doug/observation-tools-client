#include "src/client/cpp/converters/structured.h"
#include "src/client/cpp/converters/cgal.h"

namespace observation_tools {

proto::StructuredData ConvertToStructuredData(const proto::Point2 &p) {
  proto::StructuredData s;
  *s.mutable_point2() = p;
  return s;
}

proto::StructuredData ConvertToStructuredData(const proto::Segment2 &p) {
  proto::StructuredData s;
  *s.mutable_segment2() = p;
  return s;
}

proto::StructuredData ConvertToStructuredData(const proto::Polygon2 &p) {
  proto::StructuredData s;
  *s.mutable_polygon2() = p;
  return s;
}

proto::StructuredData ConvertToStructuredData(const proto::Graph &g) {
  proto::StructuredData s;
  *s.mutable_graph() = g;
  return s;
}

proto::StructuredData ConvertToStructuredData(const proto::Image2 &g) {
  proto::StructuredData s;
  *s.mutable_image2() = g;
  return s;
}

proto::StructuredData ConvertToStructuredData(const proto::Object3 &g) {
  proto::StructuredData s;
  *s.mutable_object3() = g;
  return s;
}

proto::StructuredData ConvertToStructuredData(const proto::Object2 &g) {
  proto::StructuredData s;
  *s.mutable_object2() = g;
  return s;
}

} // namespace observation_tools
