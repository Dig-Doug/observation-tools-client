workspace(
    name = "observation_tools_client",
)

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

http_archive(
    name = "bazel_skylib",
    sha256 = "66ffd9315665bfaafc96b52278f57c7e2dd09f5ede279ea6d39b2be471e7e3aa",
    urls = [
        "https://mirror.bazel.build/github.com/bazelbuild/bazel-skylib/releases/download/1.4.2/bazel-skylib-1.4.2.tar.gz",
        "https://github.com/bazelbuild/bazel-skylib/releases/download/1.4.2/bazel-skylib-1.4.2.tar.gz",
    ],
)

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
    sha256 = "a761d54e49db06f863468e6bba4a13252b1bd499e8f706da65e279b3bcbc5c52",
    urls = ["https://github.com/bazelbuild/rules_rust/releases/download/0.36.2/rules_rust-v0.36.2.tar.gz"],
)

##########

load("@com_google_protobuf//:protobuf_deps.bzl", "protobuf_deps")

protobuf_deps()

load("@bazel_skylib//:workspace.bzl", "bazel_skylib_workspace")

bazel_skylib_workspace()

load("@rules_rust//rust:repositories.bzl", "rust_repositories")

rust_repositories(
    edition = "2021",
    versions = [
        "1.75.0",
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
        "//src/client:Cargo.toml",
    ],
)

load("@crate_index//:defs.bzl", "crate_repositories")

crate_repositories()

load("@rules_rust//proto/protobuf:repositories.bzl", "rust_proto_protobuf_dependencies", "rust_proto_protobuf_register_toolchains")

rust_proto_protobuf_dependencies()

register_toolchains("//:proto-toolchain")
