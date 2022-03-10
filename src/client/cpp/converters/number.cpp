#include "src/client/cpp/converters/number.h"

namespace observation_tools {

template<>
proto::Number ConvertNumber(const double &d) {
  proto::Number number;
  number.set_d(d);
  return number;
}

}