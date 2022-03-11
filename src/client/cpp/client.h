#pragma once

#include "src/client/rust/generated/rust_cxx.h"
#include "src/client/cpp/converters/all.h"
#include "src/client/cpp/run_uploader.h"
#include <memory>

namespace observation_tools {

using ClientBox = ::rust::Box<::Client>;

class Client {
public:
  Client(std::shared_ptr<ClientBox> impl);

  virtual ~Client();

  void Shutdown();

  RunUploader CreateRunBlocking();

  RunStageUploader DeserializeRunStage(const std::string& serialized);

  friend class RunUploader;

private:
  std::shared_ptr<ClientBox> impl;
};

std::shared_ptr<Client>
CreateClient(const std::string &public_project_id);

} // namespace observation_tools
