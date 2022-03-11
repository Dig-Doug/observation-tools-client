#include "src/client/cpp/util.h"

namespace observation_tools {

rust::Slice<const uint8_t> StringToU8Slice(const std::string &s) {
  return rust::Slice<const uint8_t>{
      reinterpret_cast<const unsigned char *>(s.data()), s.size()};
}

} // namespace observation_tools