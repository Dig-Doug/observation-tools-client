#pragma once

#include "CGAL/Point_2.h"
#include "CGAL/Polygon_2.h"
#include "src/api/artifacts/artifact.pb.h"
#include "src/api/artifacts/math.pb.h"
#include "src/client/cpp/converters/point2.h"

namespace observation_tools {

template <typename Segment_2> proto::Segment2 ConvertSegment_2(const Segment_2 &p) {
  proto::Segment2 segment;
  *segment.mutable_start() = ConvertPoint_2(p.source());
  *segment.mutable_end() = ConvertPoint_2(p.target());
  return segment;
}

template <typename Polygon_2>
proto::Polygon2 ConvertPolygon_2(const Polygon_2 &p) {
  proto::Polygon2 poly;
  for (auto vertex_index = 0; vertex_index < p.size(); vertex_index++) {
    auto *edge = poly.add_edges();
    *edge->mutable_vertex() = ConvertPoint_2(p.vertex(vertex_index));
  }
  return poly;
}

} // namespace observation_tools
