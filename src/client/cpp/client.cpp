#include "src/client/cpp/client.h"
#include "src/client/cpp/util.h"
#include "src/client/rust/generated/rust_cxx.h"
#include <sstream>
#include <utility>

namespace observation_tools {

std::shared_ptr<Client> CreateClient(const std::string &public_project_id) {
  return std::make_shared<Client>(
      std::make_shared<ClientBox>(ffi_new_client(public_project_id)));
}

Client::Client(std::shared_ptr<ClientBox> impl) : impl(std::move(impl)) {}

Client::~Client() {}

RunUploader Client::CreateRunBlocking() {
  auto run_uploader = (*impl)->ffi_create_run();
  return RunUploader{std::make_shared<RunUploaderBox>(std::move(run_uploader))};
}

RunStageUploader Client::DeserializeRunStage(const std::string &serialized) {
  auto uploader = (*impl)->ffi_deserialize_run_stage(serialized);
  return RunStageUploader{
      std::make_shared<RunStageUploaderBox>(std::move(uploader))};
}

} // namespace observation_tools
