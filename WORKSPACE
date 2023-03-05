workspace(
    name = "observation_tools_client",
)

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

http_archive(
    name = "aspect_rules_js",
    sha256 = "928ba25fa82cfe7983f89118677413dc74dbc5d0360fa969da07ff22a9306052",
    strip_prefix = "rules_js-1.15.1",
    url = "https://github.com/aspect-build/rules_js/archive/refs/tags/v1.15.1.tar.gz",
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
    name = "io_bazel_rules_kotlin",
    sha256 = "15afe2d727f0dba572e0ce58f1dac20aec1441422ca65f7c3f7671b47fd483bf",
    urls = [
        "https://github.com/bazelbuild/rules_kotlin/releases/download/v1.7.0/rules_kotlin_release.tgz",
    ],
)

http_archive(
    name = "rules_foreign_cc",
    sha256 = "8ff19dcd73150f6a2fe659d4467a4d3db4333cbf4708e2d4effb90bdf4d71f42",
    strip_prefix = "rules_foreign_cc-baeee718db3ce2c9c1861e74a5f56c3bab71e9e3",
    urls = [
        "https://github.com/bazelbuild/rules_foreign_cc/archive/baeee718db3ce2c9c1861e74a5f56c3bab71e9e3.tar.gz",
    ],
)

http_archive(
    name = "rules_jvm_external",
    sha256 = "2e8806a236baad8b65623afd93846f8eade0e2f74a3699adba2bdaf22a270c69",
    strip_prefix = "rules_jvm_external-d610f38add575692ae711a905822d65e126d96ae",
    url = "https://github.com/bazelbuild/rules_jvm_external/archive/d610f38add575692ae711a905822d65e126d96ae.tar.gz",
)

http_archive(
    name = "rules_java",
    sha256 = "9a72d1bade803e1913d1e0a6f8beb35786fa3e8e460c98a56d2054200b9f6c5e",
    strip_prefix = "rules_java-385292fcfd244186e5e5811122ed32cf214a9024",
    urls = [
        "https://github.com/bazelbuild/rules_java/archive/385292fcfd244186e5e5811122ed32cf214a9024.tar.gz",
    ],
)

http_archive(
    name = "rules_proto",
    sha256 = "bc12122a5ae4b517fa423ea03a8d82ea6352d5127ea48cb54bc324e8ab78493c",
    strip_prefix = "rules_proto-af6481970a34554c6942d993e194a9aed7987780",
    urls = [
        "https://github.com/bazelbuild/rules_proto/archive/af6481970a34554c6942d993e194a9aed7987780.tar.gz",
    ],
)

http_archive(
    name = "rules_rust",
    sha256 = "04c45f756b9c7bc9295200cf86b3d5151a5b18f3b8eca9509452de79d6156a4c",
    #sha256 = "981933c494648b834d4410890687164454c7634e99f71dd35d859ca6f0f9161b",
    strip_prefix = "rules_rust-a5853fd37053b65ee30ba4f8064b9db67c90d53f",
    #strip_prefix = "rules_rust-c8fcf45e71b89516674edc91cc81f0f739cdb0c0",
    urls = [
        "https://github.com/bazelbuild/rules_rust/archive/a5853fd37053b65ee30ba4f8064b9db67c90d53f.tar.gz",
        #"https://github.com/Dig-Doug/rules_rust/archive/c8fcf45e71b89516674edc91cc81f0f739cdb0c0.tar.gz",
    ],
)

##########

load("@com_google_protobuf//:protobuf_deps.bzl", "protobuf_deps")

protobuf_deps()

load("@rules_jvm_external//:defs.bzl", "maven_install")
load("@observation_tools_client//src/client/java:deps.bzl", "OBSERVATION_TOOLS_JAVA_CLIENT_DEPS")

# To update the pinned list of deps, use:
#  bazel run @unpinned_maven//:pin
maven_install(
    artifacts = OBSERVATION_TOOLS_JAVA_CLIENT_DEPS,
    fetch_sources = True,
    generate_compat_repositories = True,
    maven_install_json = "//:maven_install.json",
    override_targets = dict(
        {
            "com.google.protobuf:protobuf-java": "@com_google_protobuf//:protobuf_java",
            "com.google.protobuf:protobuf-java-util": "@com_google_protobuf//:protobuf_java_util",
            "com.google.protobuf:protobuf-javalite": "@com_google_protobuf_javalite//:protobuf_java_lite",
        }.items(),
    ),
    repositories = [
        "https://jcenter.bintray.com/",
        "https://maven.google.com",
        "https://repo1.maven.org/maven2",
    ],
    version_conflict_policy = "pinned",
)

load("@maven//:defs.bzl", "pinned_maven_install")

pinned_maven_install()

load("@maven//:compat.bzl", "compat_repositories")

compat_repositories()

load("@io_bazel_rules_kotlin//kotlin:repositories.bzl", "kotlin_repositories")

kotlin_repositories()

load("@io_bazel_rules_kotlin//kotlin:core.bzl", "kt_register_toolchains")

kt_register_toolchains()

bind(
    name = "guava",
    actual = "@maven//:com_google_guava_guava",
)

bind(
    name = "gson",
    actual = "@maven//:com_google_code_gson_gson",
)

bind(
    name = "error_prone_annotations",
    actual = "@maven//:com_google_errorprone_error_prone_annotations",
)

bind(
    name = "j2objc_annotations",
    actual = "@maven//:com_google_j2objc_j2objc_annotations",
)

bind(
    name = "jsr305",
    actual = "@maven//:com_google_code_findbugs_jsr305",
)

bind(
    name = "junit",
    actual = "@maven//:junit_junit",
)

bind(
    name = "easymock",
    actual = "@maven//:org_easymock_easymock",
)

bind(
    name = "truth",
    actual = "@maven//:com_google_truth_truth",
)

load("@rules_rust//rust:repositories.bzl", "rust_repositories")

RUST_VERSION = "1.67.0"

rust_repositories(
    edition = "2021",
    versions = [RUST_VERSION],
)

load("@rules_rust//crate_universe:repositories.bzl", "crate_universe_dependencies")

crate_universe_dependencies(bootstrap = True)

load("@rules_rust//crate_universe:defs.bzl", "crates_repository")

crates_repository(
    name = "crate_index",
    cargo_config = "//:.cargo/config.toml",
    cargo_lockfile = "//:Cargo.lock",
    generator = "@cargo_bazel_bootstrap//:cargo-bazel",
    lockfile = "//:cargo-bazel-lock.json",
    generate_binaries = True,
    manifests = [
        "//:Cargo.toml",
        "//src/api/artifacts:Cargo.toml",
        "//src/client/rust:Cargo.toml",
    ],
)

load("@crate_index//:defs.bzl", "crate_repositories")

crate_repositories()

load("@rules_rust//proto:repositories.bzl", "rust_proto_repositories")

rust_proto_repositories(register_default_toolchain = False)

register_toolchains("//:proto-toolchain")

load("@rules_rust//wasm_bindgen:repositories.bzl", "rust_wasm_bindgen_repositories")

rust_wasm_bindgen_repositories(register_default_toolchain = True)

load("@rules_foreign_cc//foreign_cc:repositories.bzl", "rules_foreign_cc_dependencies")

rules_foreign_cc_dependencies()

load("//src/client/cpp:deps.bzl", "observation_tools_cpp_deps")

observation_tools_cpp_deps()

load("@com_github_nelhage_rules_boost//:boost/boost.bzl", "boost_deps")

boost_deps()

load("@aspect_rules_js//js:repositories.bzl", "rules_js_dependencies")

rules_js_dependencies()

load("@rules_nodejs//nodejs:repositories.bzl", "DEFAULT_NODE_VERSION", "nodejs_register_toolchains")

nodejs_register_toolchains(
    name = "nodejs",
    node_version = DEFAULT_NODE_VERSION,
)

load("@aspect_rules_js//npm:npm_import.bzl", "npm_translate_lock")

npm_translate_lock(
    name = "npm",
    pnpm_lock = "//:pnpm-lock.yaml",
    verify_node_modules_ignored = "//:.bazelignore",
)

load("@npm//:repositories.bzl", "npm_repositories")

npm_repositories()