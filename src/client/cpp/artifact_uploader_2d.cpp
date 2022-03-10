#include "src/client/cpp/artifact_uploader_2d.h"

#include <utility>

namespace observation_tools {

ArtifactUploader2d::ArtifactUploader2d(std::shared_ptr<ArtifactUploader2dBox> impl)
    : impl(std::move(impl)) {}

} // namespace observation_tools