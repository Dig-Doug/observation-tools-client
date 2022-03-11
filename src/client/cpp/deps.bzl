load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
load("@bazel_tools//tools/build_defs/repo:utils.bzl", "maybe")

def observation_tools_cpp_deps():
    maybe(
        http_archive,
        name = "opencv",
        sha256 = "9ccb2192d7e8c03c58fee07051364d94ed7599363f3b0dce1c5e6cc11c1bb0ec",
        strip_prefix = "opencv-4.2.0",
        urls = [
            "https://github.com/opencv/opencv/archive/4.2.0.tar.gz",
        ],
        build_file = Label("//src/client/cpp:BUILD.opencv.bazel"),
    )
    maybe(
        http_archive,
        name = "cgal",
        build_file = Label("//src/client/cpp:BUILD.cgal.bazel"),
        sha256 = "1a4499f5df9fbe50a57761e79867aea73ed56deaec9be8249f6ba05a6d8dcdd9",
        strip_prefix = "cgal-314f86756457d947f9565ce6c41abf9604f93430",
        urls = [
            "https://github.com/Dig-Doug/cgal/archive/314f86756457d947f9565ce6c41abf9604f93430.tar.gz",
        ],
    )
    maybe(
        http_archive,
        name = "eigen",
        build_file = Label("//src/client/cpp:BUILD.eigen.bazel"),
        sha256 = "8586084f71f9bde545ee7fa6d00288b264a2b7ac3607b974e54d13e7162c1c72",
        strip_prefix = "eigen-3.4.0",
        urls = [
            "https://gitlab.com/libeigen/eigen/-/archive/3.4.0/eigen-3.4.0.tar.gz",
        ],
    )
    maybe(
        http_archive,
        name = "com_github_nelhage_rules_boost",
        sha256 = "c3264642c6f77a894c19432fed9b0c0d1ad156b56f8e32c13abac4c682bd0873",
        strip_prefix = "rules_boost-c8b9b4a75c4301778d2e256b8d72ce47a6c9a1a4",
        urls = [
            "https://github.com/nelhage/rules_boost/archive/c8b9b4a75c4301778d2e256b8d72ce47a6c9a1a4.tar.gz",
        ],
    )
