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
    name = "io_bazel_rules_kotlin",
    sha256 = "6cbd4e5768bdfae1598662e40272729ec9ece8b7bded8f0d2c81c8ff96dc139d",
    urls = [
        "https://github.com/bazelbuild/rules_kotlin/releases/download/v1.5.0-beta-4/rules_kotlin_release.tgz",
    ],
)

http_archive(
    name = "rules_foreign_cc",
    #sha256 = "73737d50f2908c77e431946cfba41bad7c2530b602efb974cdc3cbc5dcd068f0",
    sha256 = "eb9fa6adf5002054e6953afb5c40f5cc484a2a5c8dfc0b4e8b0ca1180275b040",
    #strip_prefix = "rules_foreign_cc-32e222aeff1220605b80b9c36377db27e7a76555",
    strip_prefix = "rules_foreign_cc-4f6bece436b11381c1952a7dd1e8aa8fb857fa24",
    urls = [
        #"https://github.com/bazelbuild/rules_foreign_cc/archive/32e222aeff1220605b80b9c36377db27e7a76555.tar.gz",
        "https://github.com/Dig-Doug/rules_foreign_cc/archive/4f6bece436b11381c1952a7dd1e8aa8fb857fa24.tar.gz",
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
    sha256 = "fe3af05eceeb4d51d73f96d087870edb8a42f4e317509c964dfeb61d99ad98fc",
    strip_prefix = "rules_rust-348c3e31667ad0e73d3e0b19b372061c14ee8d58",
    urls = [
        "https://github.com/bazelbuild/rules_rust/archive/348c3e31667ad0e73d3e0b19b372061c14ee8d58.tar.gz",
    ],
)

##########

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

load("@com_google_protobuf//:protobuf_deps.bzl", "protobuf_deps")

protobuf_deps()

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

load("@rules_rust//rust:repositories.bzl", "rust_register_toolchains", "rust_repositories")

RUST_VERSION = "1.58.1"

rust_repositories(
    edition = "2021",
    version = RUST_VERSION,
)

rust_register_toolchains()

load("@rules_rust//proto:repositories.bzl", "rust_proto_repositories")

rust_proto_repositories()

load("@rules_rust//proto:transitive_repositories.bzl", "rust_proto_transitive_repositories")

rust_proto_transitive_repositories()

load("//cargo:crates.bzl", "raze_fetch_remote_crates")

raze_fetch_remote_crates()

load("@rules_foreign_cc//foreign_cc:repositories.bzl", "rules_foreign_cc_dependencies")

rules_foreign_cc_dependencies()

load("//src/client/cpp:deps.bzl", "observation_tools_cpp_deps")

observation_tools_cpp_deps()

load("@com_github_nelhage_rules_boost//:boost/boost.bzl", "boost_deps")

boost_deps()
