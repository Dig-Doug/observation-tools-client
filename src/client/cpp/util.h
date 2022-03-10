#pragma once

#include "src/client/rust/generated/rust_cxx.h"
#include <string>

namespace observation_tools {

rust::Slice<const uint8_t> StringToU8Slice(const std::string& s);

} // namespace observation_tools
