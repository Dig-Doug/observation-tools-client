#pragma once

#include "src/client/cpp/artifact_uploader_2d.h"
#include "src/client/cpp/artifact_uploader_3d.h"
#include "src/client/cpp/converters/transform3.h"
#include "src/client/rust/generated/rust_cxx.h"

namespace observation_tools {

using GenericArtifactUploaderBox = ::rust::Box<::GenericArtifactUploader>;

class GenericArtifactUploader {
public:
  [[nodiscard]] static GenericArtifactUploader
  StartGroup(const UserMetadataBuilder &metadata);

  [[nodiscard]] static GenericArtifactUploader
  StartGroup(const std::string &name);

  static GenericArtifactUploader GetCurrent();

  static void FinishGroup(GenericArtifactUploader &&uploader);

  // GenericArtifactUploader GetChildUploader(const std::string &name) const;

  ArtifactUploader2d CreateChildUploader2d(const std::string &name) const;

  ArtifactUploader2d
  CreateChildUploader2d(const UserMetadataBuilder &metadata) const;

  template <typename TransformT>
  ArtifactUploader3d
  CreateChildUploader3d(const std::string &name,
                        const TransformT &base_transform) const {
    return CreateChildUploader3d(*new_user_metadata(name),
                                 base_transform);
  }

  template <typename TransformT>
  ArtifactUploader3d
  CreateChildUploader3d(const UserMetadataBuilder &metadata,
                        const TransformT &base_transform) const {
    return CreateChildUploader3d(metadata, ConvertToTransform3(base_transform));
  }

  /*
  template <>
  ArtifactUploader3d
  CreateChildUploader3d(const UserMetadataBuilder &metadata,
                        const proto::Transform3 &base_transform) const;
                        */

  std::string Upload(const std::string &name,
                           const proto::StructuredData &data) const;

  friend class Client;
  friend class RunUploader;

protected:
  GenericArtifactUploader(std::shared_ptr<GenericArtifactUploaderBox> impl);

  std::shared_ptr<GenericArtifactUploaderBox> impl;
};

} // namespace observation_tools