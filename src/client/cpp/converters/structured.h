#pragma once

#include "src/api/artifacts/artifact.pb.h"
#include "src/api/artifacts/math.pb.h"

namespace observation_tools {

proto::StructuredData ConvertToStructuredData(const proto::StructuredData &s);

proto::StructuredData ConvertToStructuredData(const proto::Point2 &p);

proto::StructuredData ConvertToStructuredData(const proto::Segment2 &p);

proto::StructuredData ConvertToStructuredData(const proto::Polygon2 &p);

proto::StructuredData ConvertToStructuredData(const proto::Graph &g);

proto::StructuredData ConvertToStructuredData(const proto::Image2 &g);

proto::StructuredData ConvertToStructuredData(const proto::Object3 &g);

proto::StructuredData ConvertToStructuredData(const proto::Object2 &g);

} // namespace observation_tools
