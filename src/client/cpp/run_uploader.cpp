#include "src/client/cpp/run_uploader.h"

#include <utility>

namespace observation_tools {

RunUploader::RunUploader(std::shared_ptr<RunUploaderBox> impl)
    : impl(std::move(impl)) {}

RunStageUploader
RunUploader::CreateInitialRunStage(const std::string &name) const {
  auto run_stage_uploader =
      (*impl)->ffi_create_initial_run_stage(*new_user_metadata(name));
  return RunStageUploader{
      std::make_shared<RunStageUploaderBox>(std::move(run_stage_uploader))};
}

std::string RunUploader::GetViewerUrl() const {
  return (*impl)->viewer_url().operator std::string();
}

} // namespace observation_tools