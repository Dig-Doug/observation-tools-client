#pragma once

#include "src/client/cpp/artifact_uploader_2d.h"
#include "src/client/cpp/converters/transform3.h"
#include "src/client/rust/generated/rust_cxx.h"
#include "src/api/artifacts/math.pb.h"
#include "src/client/cpp/util.h"

namespace observation_tools {

using ArtifactUploader3dBox = ::rust::Box<::ArtifactUploader3d>;

class ArtifactUploader3d {
public:
  template <typename Type3d>
  std::string Upload(const std::string &name, const Type3d &data) const {
    static_assert(std::is_base_of<proto::Object3, Type3d>::value,
                  "Type3d is not allowed");
    auto raw_data = ConvertToStructuredData(data);
    auto id = (*impl)->ffi_upload(*new_user_metadata(name),
                                  StringToU8Slice(raw_data.SerializeAsString()));
    return id.operator std::string();
  }

  ArtifactUploader2d
  CreateChildUploader2d(const std::string &name,
                        const proto::Transform3 &to_3d_transform) const;

  friend class GenericArtifactUploader;

private:
  ArtifactUploader3d(std::shared_ptr<ArtifactUploader3dBox> impl);

  std::shared_ptr<ArtifactUploader3dBox> impl;
};

} // namespace observation_tools