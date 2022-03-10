#include "src/client/cpp/run_stage_uploader.h"

namespace observation_tools {

RunStageUploader::RunStageUploader(std::shared_ptr<RunStageUploaderBox> impl)
    : impl(std::move(impl)) {}

} // namespace observation_tools