workspace(
    name = "observation_tools_client",
)

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

http_archive(
    name = "com_google_protobuf",
    sha256 = "48d510f3e7ba3a9a4bb70dc304b5bee76f5d9501efed03261f93246dfc7149b3",
    strip_prefix = "protobuf-7c40b2df1fdf6f414c1c18c789715a9c948a0725",
    urls = [
        "https://github.com/protocolbuffers/protobuf/archive/7c40b2df1fdf6f414c1c18c789715a9c948a0725.tar.gz",
    ],
)

http_archive(
    name = "rules_proto",
    sha256 = "5d4cd6780634eb2ecafa091df8be8009d395f70a02f722e07e063883dd8af861",
    strip_prefix = "rules_proto-493169c1199dc21b9da860f7040a4502aa174676",
    urls = [
        "https://github.com/bazelbuild/rules_proto/archive/493169c1199dc21b9da860f7040a4502aa174676.tar.gz",
    ],
)

http_archive(
    name = "rules_rust",
    sha256 = "4a9cb4fda6ccd5b5ec393b2e944822a62e050c7c06f1ea41607f14c4fdec57a2",
    urls = ["https://github.com/bazelbuild/rules_rust/releases/download/0.25.1/rules_rust-v0.25.1.tar.gz"],
)

##########

load("@com_google_protobuf//:protobuf_deps.bzl", "protobuf_deps")

protobuf_deps()

load("@rules_rust//rust:repositories.bzl", "rust_repositories")

rust_repositories(
    edition = "2021",
    versions = [
        "1.71.0",
    ],
)

load("@rules_rust//crate_universe:repositories.bzl", "crate_universe_dependencies")

crate_universe_dependencies(bootstrap = True)

load("@rules_rust//crate_universe:defs.bzl", "crates_repository")

crates_repository(
    name = "crate_index",
    cargo_config = "//:.cargo/config.toml",
    cargo_lockfile = "//:Cargo.lock",
    generate_binaries = True,
    generator = "@cargo_bazel_bootstrap//:cargo-bazel",
    lockfile = "//:cargo-bazel-lock.json",
    manifests = [
        "//:Cargo.toml",
        "//:examples/Cargo.toml",
        "//src/api/artifacts:Cargo.toml",
        "//src/client/rust:Cargo.toml",
    ],
)

load("@crate_index//:defs.bzl", "crate_repositories")

crate_repositories()

load("@rules_rust//proto/protobuf:repositories.bzl", "rust_proto_protobuf_dependencies", "rust_proto_protobuf_register_toolchains")

rust_proto_protobuf_dependencies()

register_toolchains("//:proto-toolchain")
