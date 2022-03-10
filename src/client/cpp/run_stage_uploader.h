#pragma once

#include "src/client/cpp/generic_artifact_uploader.h"
#include "src/client/rust/generated/rust_cxx.h"

namespace observation_tools {

using RunStageUploaderBox = ::rust::Box<::RunStageUploader>;

class RunStageUploader {
public:
  friend class Client;
  friend class RunUploader;

protected:
  RunStageUploader(std::shared_ptr<RunStageUploaderBox> impl);

  std::shared_ptr<RunStageUploaderBox> impl;
};

} // namespace observation_tools