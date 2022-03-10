#pragma once

#include "src/client/cpp/artifact_uploader.h"
#include "src/client/rust/generated/rust_cxx.h"
#include <memory>

namespace observation_tools {

using RunUploaderBox = ::rust::Box<::RunUploader>;

class Client;

class RunUploader {
public:
  RunUploader(std::shared_ptr<RunUploaderBox> impl);

  RunStageUploader CreateInitialRunStage(const std::string& name) const;

  std::string GetViewerUrl() const;

private:
  std::shared_ptr<RunUploaderBox> impl;
};

} // namespace observation_tools