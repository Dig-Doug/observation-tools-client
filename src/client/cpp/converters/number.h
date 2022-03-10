#pragma once

#include "src/api/artifacts/math.pb.h"
#include "CGAL/to_rational.h"

namespace observation_tools {

template <typename FT> proto::Number ConvertNumber(const FT &ft) {
  proto::Number number;
  number.set_d(CGAL::to_double(ft));
  return number;
}

template <>
proto::Number ConvertNumber(const double &d);

}