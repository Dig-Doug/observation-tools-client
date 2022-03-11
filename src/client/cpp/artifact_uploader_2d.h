#pragma once

#include "src/client/cpp/util.h"
#include "src/client/rust/generated/rust_cxx.h"
#include "src/api/artifacts/artifact.pb.h"
#include "src/api/artifacts/math.pb.h"

namespace observation_tools {

using ArtifactUploader2dBox = ::rust::Box<::ArtifactUploader2d>;

class ArtifactUploader2d {
public:
  template <typename Type2d>
  std::string Upload(const std::string &name, const Type2d &data) const {
    static_assert(std::is_base_of<proto::Polygon2, Type2d>::value ||
                      std::is_base_of<proto::Point2, Type2d>::value ||
                      std::is_base_of<proto::Segment2, Type2d>::value ||
                      std::is_base_of<proto::Image2, Type2d>::value ||
                      std::is_base_of<proto::Object2, Type2d>::value,
                  "Type2d is not allowed");
    auto raw_data = ConvertToStructuredData(data);
    auto id = (*impl)->ffi_upload(*new_user_metadata(name),
                                  StringToU8Slice(raw_data.SerializeAsString()));
    return id.operator std::string();
  }

  friend class GenericArtifactUploader;
  friend class ArtifactUploader3d;

private:
  ArtifactUploader2d(std::shared_ptr<ArtifactUploader2dBox> impl);

  std::shared_ptr<ArtifactUploader2dBox> impl;
};

} // namespace observation_tools