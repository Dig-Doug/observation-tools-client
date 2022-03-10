#include "src/client/cpp/artifact_uploader_3d.h"
#include "src/client/cpp/util.h"

namespace observation_tools {

ArtifactUploader3d::ArtifactUploader3d(
    std::shared_ptr<ArtifactUploader3dBox> impl)
    : impl(std::move(impl)) {}

ArtifactUploader2d ArtifactUploader3d::CreateChildUploader2d(
    const std::string &name, const proto::Transform3 &to_3d_transform) const {
  auto raw_data = to_3d_transform.SerializeAsString();
  auto uploader = (*impl)->ffi_child_uploader_2d(*new_user_metadata(name),
                                                 StringToU8Slice(raw_data));
  return ArtifactUploader2d{
      std::make_shared<ArtifactUploader2dBox>(std::move(uploader))};
}

} // namespace observation_tools