#include "src/client/cpp/generic_artifact_uploader.h"

#include "src/client/cpp/client.h"
#include "src/client/cpp/util.h"
#include <utility>

namespace observation_tools {

GenericArtifactUploader
GenericArtifactUploader::StartGroup(const UserMetadataBuilder &metadata) {
  auto uploader = ffi_get_current_group()->ffi_child_uploader(metadata);
  return GenericArtifactUploader{
      std::make_shared<GenericArtifactUploaderBox>(std::move(uploader))};
}

GenericArtifactUploader
GenericArtifactUploader::StartGroup(const std::string &name) {
  return StartGroup(*new_user_metadata(name));
}

GenericArtifactUploader GenericArtifactUploader::GetCurrent() {
  auto uploader = ffi_get_current_group();
  return GenericArtifactUploader{
      std::make_shared<GenericArtifactUploaderBox>(std::move(uploader))};
}

void GenericArtifactUploader::FinishGroup(GenericArtifactUploader &&uploader) {
  //(*uploader.impl)->drop();
}

/*
GenericArtifactUploader
GenericArtifactUploader::GetChildUploader(const std::string &name) const {
  auto request = internal::BaseChildGroupRequest(group_id_, name);
  request.mutable_artifact_data()->set_artifact_type(proto::ARTIFACT_TYPE_GENERIC);
  client_->CreateGroup(request);
  return GenericArtifactUploader{client_, request.group_id(), false};
}
 */

ArtifactUploader2d
GenericArtifactUploader::CreateChildUploader2d(const std::string &name) const {
  return CreateChildUploader2d(*new_user_metadata(name));
}

ArtifactUploader2d GenericArtifactUploader::CreateChildUploader2d(
    const UserMetadataBuilder &metadata) const {
  auto uploader = (*impl)->ffi_child_uploader_2d(metadata);
  return ArtifactUploader2d{
      std::make_shared<ArtifactUploader2dBox>(std::move(uploader))};
}

template <>
ArtifactUploader3d GenericArtifactUploader::CreateChildUploader3d(
    const UserMetadataBuilder &metadata,
    const proto::Transform3 &base_transform) const {
  auto transform_bytes = base_transform.SerializeAsString();
  auto uploader = (*impl)->ffi_child_uploader_3d(
      metadata, StringToU8Slice(transform_bytes));
  return ArtifactUploader3d{
      std::make_shared<ArtifactUploader3dBox>(std::move(uploader))};
}

std::string GenericArtifactUploader::Upload(const std::string &name,
                                            const proto::StructuredData &data) const {
  auto id = (*impl)->ffi_upload(*new_user_metadata(name),
                                StringToU8Slice(data.SerializeAsString()));
  return id.operator std::string();
}

GenericArtifactUploader::GenericArtifactUploader(
    std::shared_ptr<GenericArtifactUploaderBox> impl)
    : impl(std::move(impl)) {}

} // namespace observation_tools