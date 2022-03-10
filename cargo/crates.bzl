"""
@generated
cargo-raze generated Bazel file.

DO NOT EDIT! Replaced on runs of cargo-raze
"""

load("@bazel_tools//tools/build_defs/repo:git.bzl", "new_git_repository")  # buildifier: disable=load
load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")  # buildifier: disable=load
load("@bazel_tools//tools/build_defs/repo:utils.bzl", "maybe")  # buildifier: disable=load

def raze_fetch_remote_crates():
    """This function defines a collection of repos and should be called in a WORKSPACE file"""
    maybe(
        http_archive,
        name = "raze__aho_corasick__0_7_18",
        url = "https://crates.io/api/v1/crates/aho-corasick/0.7.18/download",
        type = "tar.gz",
        sha256 = "1e37cfd5e7657ada45f742d6e99ca5788580b5c529dc78faf11ece6dc702656f",
        strip_prefix = "aho-corasick-0.7.18",
        build_file = Label("//cargo/remote:BUILD.aho-corasick-0.7.18.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__async_mutex__1_4_0",
        url = "https://crates.io/api/v1/crates/async-mutex/1.4.0/download",
        type = "tar.gz",
        sha256 = "479db852db25d9dbf6204e6cb6253698f175c15726470f78af0d918e99d6156e",
        strip_prefix = "async-mutex-1.4.0",
        build_file = Label("//cargo/remote:BUILD.async-mutex-1.4.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__async_rwlock__1_3_0",
        url = "https://crates.io/api/v1/crates/async-rwlock/1.3.0/download",
        type = "tar.gz",
        sha256 = "261803dcc39ba9e72760ba6e16d0199b1eef9fc44e81bffabbebb9f5aea3906c",
        strip_prefix = "async-rwlock-1.3.0",
        build_file = Label("//cargo/remote:BUILD.async-rwlock-1.3.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__async_trait__0_1_52",
        url = "https://crates.io/api/v1/crates/async-trait/0.1.52/download",
        type = "tar.gz",
        sha256 = "061a7acccaa286c011ddc30970520b98fa40e00c9d644633fb26b5fc63a265e3",
        strip_prefix = "async-trait-0.1.52",
        build_file = Label("//cargo/remote:BUILD.async-trait-0.1.52.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__atty__0_2_14",
        url = "https://crates.io/api/v1/crates/atty/0.2.14/download",
        type = "tar.gz",
        sha256 = "d9b39be18770d11421cdb1b9947a45dd3f37e93092cbf377614828a319d5fee8",
        strip_prefix = "atty-0.2.14",
        build_file = Label("//cargo/remote:BUILD.atty-0.2.14.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__autocfg__1_1_0",
        url = "https://crates.io/api/v1/crates/autocfg/1.1.0/download",
        type = "tar.gz",
        sha256 = "d468802bab17cbc0cc575e9b053f41e72aa36bfa6b7f55e3529ffa43161b97fa",
        strip_prefix = "autocfg-1.1.0",
        build_file = Label("//cargo/remote:BUILD.autocfg-1.1.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__backoff__0_4_0",
        url = "https://crates.io/api/v1/crates/backoff/0.4.0/download",
        type = "tar.gz",
        sha256 = "b62ddb9cb1ec0a098ad4bbf9344d0713fa193ae1a80af55febcff2627b6a00c1",
        strip_prefix = "backoff-0.4.0",
        build_file = Label("//cargo/remote:BUILD.backoff-0.4.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__base64__0_13_0",
        url = "https://crates.io/api/v1/crates/base64/0.13.0/download",
        type = "tar.gz",
        sha256 = "904dfeac50f3cdaba28fc6f57fdcddb75f49ed61346676a78c4ffe55877802fd",
        strip_prefix = "base64-0.13.0",
        build_file = Label("//cargo/remote:BUILD.base64-0.13.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__bitflags__1_3_2",
        url = "https://crates.io/api/v1/crates/bitflags/1.3.2/download",
        type = "tar.gz",
        sha256 = "bef38d45163c2f1dde094a7dfd33ccf595c92905c8f8f4fdc18d06fb1037718a",
        strip_prefix = "bitflags-1.3.2",
        build_file = Label("//cargo/remote:BUILD.bitflags-1.3.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__bs58__0_4_0",
        url = "https://crates.io/api/v1/crates/bs58/0.4.0/download",
        type = "tar.gz",
        sha256 = "771fe0050b883fcc3ea2359b1a96bcfbc090b7116eae7c3c512c7a083fdf23d3",
        strip_prefix = "bs58-0.4.0",
        build_file = Label("//cargo/remote:BUILD.bs58-0.4.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__bumpalo__3_9_1",
        url = "https://crates.io/api/v1/crates/bumpalo/3.9.1/download",
        type = "tar.gz",
        sha256 = "a4a45a46ab1f2412e53d3a0ade76ffad2025804294569aae387231a0cd6e0899",
        strip_prefix = "bumpalo-3.9.1",
        build_file = Label("//cargo/remote:BUILD.bumpalo-3.9.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__bytes__1_1_0",
        url = "https://crates.io/api/v1/crates/bytes/1.1.0/download",
        type = "tar.gz",
        sha256 = "c4872d67bab6358e59559027aa3b9157c53d9358c51423c17554809a8858e0f8",
        strip_prefix = "bytes-1.1.0",
        build_file = Label("//cargo/remote:BUILD.bytes-1.1.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__cached__0_30_0",
        url = "https://crates.io/api/v1/crates/cached/0.30.0/download",
        type = "tar.gz",
        sha256 = "af4dfac631a8e77b2f327f7852bb6172771f5279c4512efe79fad6067b37be3d",
        strip_prefix = "cached-0.30.0",
        build_file = Label("//cargo/remote:BUILD.cached-0.30.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__cached_proc_macro__0_9_0",
        url = "https://crates.io/api/v1/crates/cached_proc_macro/0.9.0/download",
        type = "tar.gz",
        sha256 = "725f434d6da2814b989bd905c62ca28a9383feff7440210dc279665fbbbc9511",
        strip_prefix = "cached_proc_macro-0.9.0",
        build_file = Label("//cargo/remote:BUILD.cached_proc_macro-0.9.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__cached_proc_macro_types__0_1_0",
        url = "https://crates.io/api/v1/crates/cached_proc_macro_types/0.1.0/download",
        type = "tar.gz",
        sha256 = "3a4f925191b4367301851c6d99b09890311d74b0d43f274c0b34c86d308a3663",
        strip_prefix = "cached_proc_macro_types-0.1.0",
        build_file = Label("//cargo/remote:BUILD.cached_proc_macro_types-0.1.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__cc__1_0_73",
        url = "https://crates.io/api/v1/crates/cc/1.0.73/download",
        type = "tar.gz",
        sha256 = "2fff2a6927b3bb87f9595d67196a70493f627687a71d87a0d692242c33f58c11",
        strip_prefix = "cc-1.0.73",
        build_file = Label("//cargo/remote:BUILD.cc-1.0.73.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__cfg_if__1_0_0",
        url = "https://crates.io/api/v1/crates/cfg-if/1.0.0/download",
        type = "tar.gz",
        sha256 = "baf1de4339761588bc0619e3cbc0120ee582ebb74b53b4efbf79117bd2da40fd",
        strip_prefix = "cfg-if-1.0.0",
        build_file = Label("//cargo/remote:BUILD.cfg-if-1.0.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__chrono__0_4_19",
        url = "https://crates.io/api/v1/crates/chrono/0.4.19/download",
        type = "tar.gz",
        sha256 = "670ad68c9088c2a963aaa298cb369688cf3f9465ce5e2d4ca10e6e0098a1ce73",
        strip_prefix = "chrono-0.4.19",
        build_file = Label("//cargo/remote:BUILD.chrono-0.4.19.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__core_foundation__0_9_3",
        url = "https://crates.io/api/v1/crates/core-foundation/0.9.3/download",
        type = "tar.gz",
        sha256 = "194a7a9e6de53fa55116934067c844d9d749312f75c6f6d0980e8c252f8c2146",
        strip_prefix = "core-foundation-0.9.3",
        build_file = Label("//cargo/remote:BUILD.core-foundation-0.9.3.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__core_foundation_sys__0_8_3",
        url = "https://crates.io/api/v1/crates/core-foundation-sys/0.8.3/download",
        type = "tar.gz",
        sha256 = "5827cebf4670468b8772dd191856768aedcb1b0278a04f989f7766351917b9dc",
        strip_prefix = "core-foundation-sys-0.8.3",
        build_file = Label("//cargo/remote:BUILD.core-foundation-sys-0.8.3.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__cxx__1_0_66",
        url = "https://crates.io/api/v1/crates/cxx/1.0.66/download",
        type = "tar.gz",
        sha256 = "ce2295fe8865279f404147e9b2328e5af0ad11a2c016e58c13acfd48a07d8a55",
        strip_prefix = "cxx-1.0.66",
        build_file = Label("//cargo/remote:BUILD.cxx-1.0.66.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__cxxbridge_flags__1_0_66",
        url = "https://crates.io/api/v1/crates/cxxbridge-flags/1.0.66/download",
        type = "tar.gz",
        sha256 = "0a670224c6686471df12560a0b97a08145082e70bd38e2b0b5383b79e46c3da7",
        strip_prefix = "cxxbridge-flags-1.0.66",
        build_file = Label("//cargo/remote:BUILD.cxxbridge-flags-1.0.66.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__cxxbridge_macro__1_0_66",
        url = "https://crates.io/api/v1/crates/cxxbridge-macro/1.0.66/download",
        type = "tar.gz",
        sha256 = "7b700096ca0dece28d9535fdb17ab784a8ae155d7f29d39c273643e6292c9620",
        strip_prefix = "cxxbridge-macro-1.0.66",
        build_file = Label("//cargo/remote:BUILD.cxxbridge-macro-1.0.66.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__darling__0_12_4",
        url = "https://crates.io/api/v1/crates/darling/0.12.4/download",
        type = "tar.gz",
        sha256 = "5f2c43f534ea4b0b049015d00269734195e6d3f0f6635cb692251aca6f9f8b3c",
        strip_prefix = "darling-0.12.4",
        build_file = Label("//cargo/remote:BUILD.darling-0.12.4.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__darling__0_13_1",
        url = "https://crates.io/api/v1/crates/darling/0.13.1/download",
        type = "tar.gz",
        sha256 = "d0d720b8683f8dd83c65155f0530560cba68cd2bf395f6513a483caee57ff7f4",
        strip_prefix = "darling-0.13.1",
        build_file = Label("//cargo/remote:BUILD.darling-0.13.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__darling_core__0_12_4",
        url = "https://crates.io/api/v1/crates/darling_core/0.12.4/download",
        type = "tar.gz",
        sha256 = "8e91455b86830a1c21799d94524df0845183fa55bafd9aa137b01c7d1065fa36",
        strip_prefix = "darling_core-0.12.4",
        build_file = Label("//cargo/remote:BUILD.darling_core-0.12.4.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__darling_core__0_13_1",
        url = "https://crates.io/api/v1/crates/darling_core/0.13.1/download",
        type = "tar.gz",
        sha256 = "7a340f241d2ceed1deb47ae36c4144b2707ec7dd0b649f894cb39bb595986324",
        strip_prefix = "darling_core-0.13.1",
        build_file = Label("//cargo/remote:BUILD.darling_core-0.13.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__darling_macro__0_12_4",
        url = "https://crates.io/api/v1/crates/darling_macro/0.12.4/download",
        type = "tar.gz",
        sha256 = "29b5acf0dea37a7f66f7b25d2c5e93fd46f8f6968b1a5d7a3e02e97768afc95a",
        strip_prefix = "darling_macro-0.12.4",
        build_file = Label("//cargo/remote:BUILD.darling_macro-0.12.4.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__darling_macro__0_13_1",
        url = "https://crates.io/api/v1/crates/darling_macro/0.13.1/download",
        type = "tar.gz",
        sha256 = "72c41b3b7352feb3211a0d743dc5700a4e3b60f51bd2b368892d1e0f9a95f44b",
        strip_prefix = "darling_macro-0.13.1",
        build_file = Label("//cargo/remote:BUILD.darling_macro-0.13.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__derive_builder__0_10_2",
        url = "https://crates.io/api/v1/crates/derive_builder/0.10.2/download",
        type = "tar.gz",
        sha256 = "d13202debe11181040ae9063d739fa32cfcaaebe2275fe387703460ae2365b30",
        strip_prefix = "derive_builder-0.10.2",
        build_file = Label("//cargo/remote:BUILD.derive_builder-0.10.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__derive_builder_core__0_10_2",
        url = "https://crates.io/api/v1/crates/derive_builder_core/0.10.2/download",
        type = "tar.gz",
        sha256 = "66e616858f6187ed828df7c64a6d71720d83767a7f19740b2d1b6fe6327b36e5",
        strip_prefix = "derive_builder_core-0.10.2",
        build_file = Label("//cargo/remote:BUILD.derive_builder_core-0.10.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__derive_builder_macro__0_10_2",
        url = "https://crates.io/api/v1/crates/derive_builder_macro/0.10.2/download",
        type = "tar.gz",
        sha256 = "58a94ace95092c5acb1e97a7e846b310cfbd499652f72297da7493f618a98d73",
        strip_prefix = "derive_builder_macro-0.10.2",
        build_file = Label("//cargo/remote:BUILD.derive_builder_macro-0.10.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__either__1_6_1",
        url = "https://crates.io/api/v1/crates/either/1.6.1/download",
        type = "tar.gz",
        sha256 = "e78d4f1cc4ae33bbfc157ed5d5a5ef3bc29227303d595861deb238fcec4e9457",
        strip_prefix = "either-1.6.1",
        build_file = Label("//cargo/remote:BUILD.either-1.6.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__encoding_rs__0_8_30",
        url = "https://crates.io/api/v1/crates/encoding_rs/0.8.30/download",
        type = "tar.gz",
        sha256 = "7896dc8abb250ffdda33912550faa54c88ec8b998dec0b2c55ab224921ce11df",
        strip_prefix = "encoding_rs-0.8.30",
        build_file = Label("//cargo/remote:BUILD.encoding_rs-0.8.30.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__env_logger__0_8_4",
        url = "https://crates.io/api/v1/crates/env_logger/0.8.4/download",
        type = "tar.gz",
        sha256 = "a19187fea3ac7e84da7dacf48de0c45d63c6a76f9490dae389aead16c243fce3",
        strip_prefix = "env_logger-0.8.4",
        build_file = Label("//cargo/remote:BUILD.env_logger-0.8.4.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__event_listener__2_5_2",
        url = "https://crates.io/api/v1/crates/event-listener/2.5.2/download",
        type = "tar.gz",
        sha256 = "77f3309417938f28bf8228fcff79a4a37103981e3e186d2ccd19c74b38f4eb71",
        strip_prefix = "event-listener-2.5.2",
        build_file = Label("//cargo/remote:BUILD.event-listener-2.5.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__fastrand__1_7_0",
        url = "https://crates.io/api/v1/crates/fastrand/1.7.0/download",
        type = "tar.gz",
        sha256 = "c3fcf0cee53519c866c09b5de1f6c56ff9d647101f81c1964fa632e148896cdf",
        strip_prefix = "fastrand-1.7.0",
        build_file = Label("//cargo/remote:BUILD.fastrand-1.7.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__fnv__1_0_7",
        url = "https://crates.io/api/v1/crates/fnv/1.0.7/download",
        type = "tar.gz",
        sha256 = "3f9eec918d3f24069decb9af1554cad7c880e2da24a9afd88aca000531ab82c1",
        strip_prefix = "fnv-1.0.7",
        build_file = Label("//cargo/remote:BUILD.fnv-1.0.7.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__foreign_types__0_3_2",
        url = "https://crates.io/api/v1/crates/foreign-types/0.3.2/download",
        type = "tar.gz",
        sha256 = "f6f339eb8adc052cd2ca78910fda869aefa38d22d5cb648e6485e4d3fc06f3b1",
        strip_prefix = "foreign-types-0.3.2",
        build_file = Label("//cargo/remote:BUILD.foreign-types-0.3.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__foreign_types_shared__0_1_1",
        url = "https://crates.io/api/v1/crates/foreign-types-shared/0.1.1/download",
        type = "tar.gz",
        sha256 = "00b0228411908ca8685dba7fc2cdd70ec9990a6e753e89b6ac91a84c40fbaf4b",
        strip_prefix = "foreign-types-shared-0.1.1",
        build_file = Label("//cargo/remote:BUILD.foreign-types-shared-0.1.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__form_urlencoded__1_0_1",
        url = "https://crates.io/api/v1/crates/form_urlencoded/1.0.1/download",
        type = "tar.gz",
        sha256 = "5fc25a87fa4fd2094bffb06925852034d90a17f0d1e05197d4956d3555752191",
        strip_prefix = "form_urlencoded-1.0.1",
        build_file = Label("//cargo/remote:BUILD.form_urlencoded-1.0.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__futures__0_3_21",
        url = "https://crates.io/api/v1/crates/futures/0.3.21/download",
        type = "tar.gz",
        sha256 = "f73fe65f54d1e12b726f517d3e2135ca3125a437b6d998caf1962961f7172d9e",
        strip_prefix = "futures-0.3.21",
        build_file = Label("//cargo/remote:BUILD.futures-0.3.21.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__futures_channel__0_3_21",
        url = "https://crates.io/api/v1/crates/futures-channel/0.3.21/download",
        type = "tar.gz",
        sha256 = "c3083ce4b914124575708913bca19bfe887522d6e2e6d0952943f5eac4a74010",
        strip_prefix = "futures-channel-0.3.21",
        build_file = Label("//cargo/remote:BUILD.futures-channel-0.3.21.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__futures_core__0_3_21",
        url = "https://crates.io/api/v1/crates/futures-core/0.3.21/download",
        type = "tar.gz",
        sha256 = "0c09fd04b7e4073ac7156a9539b57a484a8ea920f79c7c675d05d289ab6110d3",
        strip_prefix = "futures-core-0.3.21",
        build_file = Label("//cargo/remote:BUILD.futures-core-0.3.21.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__futures_executor__0_3_21",
        url = "https://crates.io/api/v1/crates/futures-executor/0.3.21/download",
        type = "tar.gz",
        sha256 = "9420b90cfa29e327d0429f19be13e7ddb68fa1cccb09d65e5706b8c7a749b8a6",
        strip_prefix = "futures-executor-0.3.21",
        build_file = Label("//cargo/remote:BUILD.futures-executor-0.3.21.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__futures_io__0_3_21",
        url = "https://crates.io/api/v1/crates/futures-io/0.3.21/download",
        type = "tar.gz",
        sha256 = "fc4045962a5a5e935ee2fdedaa4e08284547402885ab326734432bed5d12966b",
        strip_prefix = "futures-io-0.3.21",
        build_file = Label("//cargo/remote:BUILD.futures-io-0.3.21.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__futures_macro__0_3_21",
        url = "https://crates.io/api/v1/crates/futures-macro/0.3.21/download",
        type = "tar.gz",
        sha256 = "33c1e13800337f4d4d7a316bf45a567dbcb6ffe087f16424852d97e97a91f512",
        strip_prefix = "futures-macro-0.3.21",
        build_file = Label("//cargo/remote:BUILD.futures-macro-0.3.21.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__futures_sink__0_3_21",
        url = "https://crates.io/api/v1/crates/futures-sink/0.3.21/download",
        type = "tar.gz",
        sha256 = "21163e139fa306126e6eedaf49ecdb4588f939600f0b1e770f4205ee4b7fa868",
        strip_prefix = "futures-sink-0.3.21",
        build_file = Label("//cargo/remote:BUILD.futures-sink-0.3.21.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__futures_task__0_3_21",
        url = "https://crates.io/api/v1/crates/futures-task/0.3.21/download",
        type = "tar.gz",
        sha256 = "57c66a976bf5909d801bbef33416c41372779507e7a6b3a5e25e4749c58f776a",
        strip_prefix = "futures-task-0.3.21",
        build_file = Label("//cargo/remote:BUILD.futures-task-0.3.21.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__futures_util__0_3_21",
        url = "https://crates.io/api/v1/crates/futures-util/0.3.21/download",
        type = "tar.gz",
        sha256 = "d8b7abd5d659d9b90c8cba917f6ec750a74e2dc23902ef9cd4cc8c8b22e6036a",
        strip_prefix = "futures-util-0.3.21",
        build_file = Label("//cargo/remote:BUILD.futures-util-0.3.21.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__getrandom__0_2_5",
        url = "https://crates.io/api/v1/crates/getrandom/0.2.5/download",
        type = "tar.gz",
        sha256 = "d39cd93900197114fa1fcb7ae84ca742095eed9442088988ae74fa744e930e77",
        strip_prefix = "getrandom-0.2.5",
        build_file = Label("//cargo/remote:BUILD.getrandom-0.2.5.bazel"),
    )

    maybe(
        new_git_repository,
        name = "raze__google_cloud_auth__0_1_0",
        remote = "https://github.com/googleapis/google-cloud-rust",
        commit = "b82d4462900db232b14029f69766e5016743fcc4",
        build_file = Label("//cargo/remote:BUILD.google-cloud-auth-0.1.0.bazel"),
        init_submodules = True,
    )

    maybe(
        http_archive,
        name = "raze__h2__0_3_11",
        url = "https://crates.io/api/v1/crates/h2/0.3.11/download",
        type = "tar.gz",
        sha256 = "d9f1f717ddc7b2ba36df7e871fd88db79326551d3d6f1fc406fbfd28b582ff8e",
        strip_prefix = "h2-0.3.11",
        build_file = Label("//cargo/remote:BUILD.h2-0.3.11.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__hashbrown__0_11_2",
        url = "https://crates.io/api/v1/crates/hashbrown/0.11.2/download",
        type = "tar.gz",
        sha256 = "ab5ef0d4909ef3724cc8cce6ccc8572c5c817592e9285f5464f8e86f8bd3726e",
        strip_prefix = "hashbrown-0.11.2",
        build_file = Label("//cargo/remote:BUILD.hashbrown-0.11.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__hermit_abi__0_1_19",
        url = "https://crates.io/api/v1/crates/hermit-abi/0.1.19/download",
        type = "tar.gz",
        sha256 = "62b467343b94ba476dcb2500d242dadbb39557df889310ac77c5d99100aaac33",
        strip_prefix = "hermit-abi-0.1.19",
        build_file = Label("//cargo/remote:BUILD.hermit-abi-0.1.19.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__http__0_2_6",
        url = "https://crates.io/api/v1/crates/http/0.2.6/download",
        type = "tar.gz",
        sha256 = "31f4c6746584866f0feabcc69893c5b51beef3831656a968ed7ae254cdc4fd03",
        strip_prefix = "http-0.2.6",
        build_file = Label("//cargo/remote:BUILD.http-0.2.6.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__http_body__0_4_4",
        url = "https://crates.io/api/v1/crates/http-body/0.4.4/download",
        type = "tar.gz",
        sha256 = "1ff4f84919677303da5f147645dbea6b1881f368d03ac84e1dc09031ebd7b2c6",
        strip_prefix = "http-body-0.4.4",
        build_file = Label("//cargo/remote:BUILD.http-body-0.4.4.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__httparse__1_6_0",
        url = "https://crates.io/api/v1/crates/httparse/1.6.0/download",
        type = "tar.gz",
        sha256 = "9100414882e15fb7feccb4897e5f0ff0ff1ca7d1a86a23208ada4d7a18e6c6c4",
        strip_prefix = "httparse-1.6.0",
        build_file = Label("//cargo/remote:BUILD.httparse-1.6.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__httpdate__1_0_2",
        url = "https://crates.io/api/v1/crates/httpdate/1.0.2/download",
        type = "tar.gz",
        sha256 = "c4a1e36c821dbe04574f602848a19f742f4fb3c98d40449f11bcad18d6b17421",
        strip_prefix = "httpdate-1.0.2",
        build_file = Label("//cargo/remote:BUILD.httpdate-1.0.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__humantime__2_1_0",
        url = "https://crates.io/api/v1/crates/humantime/2.1.0/download",
        type = "tar.gz",
        sha256 = "9a3a5bfb195931eeb336b2a7b4d761daec841b97f947d34394601737a7bba5e4",
        strip_prefix = "humantime-2.1.0",
        build_file = Label("//cargo/remote:BUILD.humantime-2.1.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__hyper__0_14_17",
        url = "https://crates.io/api/v1/crates/hyper/0.14.17/download",
        type = "tar.gz",
        sha256 = "043f0e083e9901b6cc658a77d1eb86f4fc650bbb977a4337dd63192826aa85dd",
        strip_prefix = "hyper-0.14.17",
        build_file = Label("//cargo/remote:BUILD.hyper-0.14.17.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__hyper_tls__0_5_0",
        url = "https://crates.io/api/v1/crates/hyper-tls/0.5.0/download",
        type = "tar.gz",
        sha256 = "d6183ddfa99b85da61a140bea0efc93fdf56ceaa041b37d553518030827f9905",
        strip_prefix = "hyper-tls-0.5.0",
        build_file = Label("//cargo/remote:BUILD.hyper-tls-0.5.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__ident_case__1_0_1",
        url = "https://crates.io/api/v1/crates/ident_case/1.0.1/download",
        type = "tar.gz",
        sha256 = "b9e0384b61958566e926dc50660321d12159025e767c18e043daf26b70104c39",
        strip_prefix = "ident_case-1.0.1",
        build_file = Label("//cargo/remote:BUILD.ident_case-1.0.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__idna__0_2_3",
        url = "https://crates.io/api/v1/crates/idna/0.2.3/download",
        type = "tar.gz",
        sha256 = "418a0a6fab821475f634efe3ccc45c013f742efe03d853e8d3355d5cb850ecf8",
        strip_prefix = "idna-0.2.3",
        build_file = Label("//cargo/remote:BUILD.idna-0.2.3.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__indexmap__1_8_0",
        url = "https://crates.io/api/v1/crates/indexmap/1.8.0/download",
        type = "tar.gz",
        sha256 = "282a6247722caba404c065016bbfa522806e51714c34f5dfc3e4a3a46fcb4223",
        strip_prefix = "indexmap-1.8.0",
        build_file = Label("//cargo/remote:BUILD.indexmap-1.8.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__indoc__0_3_6",
        url = "https://crates.io/api/v1/crates/indoc/0.3.6/download",
        type = "tar.gz",
        sha256 = "47741a8bc60fb26eb8d6e0238bbb26d8575ff623fdc97b1a2c00c050b9684ed8",
        strip_prefix = "indoc-0.3.6",
        build_file = Label("//cargo/remote:BUILD.indoc-0.3.6.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__indoc_impl__0_3_6",
        url = "https://crates.io/api/v1/crates/indoc-impl/0.3.6/download",
        type = "tar.gz",
        sha256 = "ce046d161f000fffde5f432a0d034d0341dc152643b2598ed5bfce44c4f3a8f0",
        strip_prefix = "indoc-impl-0.3.6",
        build_file = Label("//cargo/remote:BUILD.indoc-impl-0.3.6.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__instant__0_1_12",
        url = "https://crates.io/api/v1/crates/instant/0.1.12/download",
        type = "tar.gz",
        sha256 = "7a5bbe824c507c5da5956355e86a746d82e0e1464f65d862cc5e71da70e94b2c",
        strip_prefix = "instant-0.1.12",
        build_file = Label("//cargo/remote:BUILD.instant-0.1.12.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__ipnet__2_4_0",
        url = "https://crates.io/api/v1/crates/ipnet/2.4.0/download",
        type = "tar.gz",
        sha256 = "35e70ee094dc02fd9c13fdad4940090f22dbd6ac7c9e7094a46cf0232a50bc7c",
        strip_prefix = "ipnet-2.4.0",
        build_file = Label("//cargo/remote:BUILD.ipnet-2.4.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__itertools__0_10_3",
        url = "https://crates.io/api/v1/crates/itertools/0.10.3/download",
        type = "tar.gz",
        sha256 = "a9a9d19fa1e79b6215ff29b9d6880b706147f16e9b1dbb1e4e5947b5b02bc5e3",
        strip_prefix = "itertools-0.10.3",
        build_file = Label("//cargo/remote:BUILD.itertools-0.10.3.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__itoa__1_0_1",
        url = "https://crates.io/api/v1/crates/itoa/1.0.1/download",
        type = "tar.gz",
        sha256 = "1aab8fc367588b89dcee83ab0fd66b72b50b72fa1904d7095045ace2b0c81c35",
        strip_prefix = "itoa-1.0.1",
        build_file = Label("//cargo/remote:BUILD.itoa-1.0.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__js_sys__0_3_56",
        url = "https://crates.io/api/v1/crates/js-sys/0.3.56/download",
        type = "tar.gz",
        sha256 = "a38fc24e30fd564ce974c02bf1d337caddff65be6cc4735a1f7eab22a7440f04",
        strip_prefix = "js-sys-0.3.56",
        build_file = Label("//cargo/remote:BUILD.js-sys-0.3.56.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__lazy_static__1_4_0",
        url = "https://crates.io/api/v1/crates/lazy_static/1.4.0/download",
        type = "tar.gz",
        sha256 = "e2abad23fbc42b3700f2f279844dc832adb2b2eb069b2df918f455c4e18cc646",
        strip_prefix = "lazy_static-1.4.0",
        build_file = Label("//cargo/remote:BUILD.lazy_static-1.4.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__libc__0_2_119",
        url = "https://crates.io/api/v1/crates/libc/0.2.119/download",
        type = "tar.gz",
        sha256 = "1bf2e165bb3457c8e098ea76f3e3bc9db55f87aa90d52d0e6be741470916aaa4",
        strip_prefix = "libc-0.2.119",
        build_file = Label("//cargo/remote:BUILD.libc-0.2.119.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__link_cplusplus__1_0_6",
        url = "https://crates.io/api/v1/crates/link-cplusplus/1.0.6/download",
        type = "tar.gz",
        sha256 = "f8cae2cd7ba2f3f63938b9c724475dfb7b9861b545a90324476324ed21dbc8c8",
        strip_prefix = "link-cplusplus-1.0.6",
        build_file = Label("//cargo/remote:BUILD.link-cplusplus-1.0.6.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__lock_api__0_4_6",
        url = "https://crates.io/api/v1/crates/lock_api/0.4.6/download",
        type = "tar.gz",
        sha256 = "88943dd7ef4a2e5a4bfa2753aaab3013e34ce2533d1996fb18ef591e315e2b3b",
        strip_prefix = "lock_api-0.4.6",
        build_file = Label("//cargo/remote:BUILD.lock_api-0.4.6.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__log__0_4_14",
        url = "https://crates.io/api/v1/crates/log/0.4.14/download",
        type = "tar.gz",
        sha256 = "51b9bbe6c47d51fc3e1a9b945965946b4c44142ab8792c50835a980d362c2710",
        strip_prefix = "log-0.4.14",
        build_file = Label("//cargo/remote:BUILD.log-0.4.14.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__matches__0_1_9",
        url = "https://crates.io/api/v1/crates/matches/0.1.9/download",
        type = "tar.gz",
        sha256 = "a3e378b66a060d48947b590737b30a1be76706c8dd7b8ba0f2fe3989c68a853f",
        strip_prefix = "matches-0.1.9",
        build_file = Label("//cargo/remote:BUILD.matches-0.1.9.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__memchr__2_4_1",
        url = "https://crates.io/api/v1/crates/memchr/2.4.1/download",
        type = "tar.gz",
        sha256 = "308cc39be01b73d0d18f82a0e7b2a3df85245f84af96fdddc5d202d27e47b86a",
        strip_prefix = "memchr-2.4.1",
        build_file = Label("//cargo/remote:BUILD.memchr-2.4.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__mime__0_3_16",
        url = "https://crates.io/api/v1/crates/mime/0.3.16/download",
        type = "tar.gz",
        sha256 = "2a60c7ce501c71e03a9c9c0d35b861413ae925bd979cc7a4e30d060069aaac8d",
        strip_prefix = "mime-0.3.16",
        build_file = Label("//cargo/remote:BUILD.mime-0.3.16.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__mime_guess__2_0_4",
        url = "https://crates.io/api/v1/crates/mime_guess/2.0.4/download",
        type = "tar.gz",
        sha256 = "4192263c238a5f0d0c6bfd21f336a313a4ce1c450542449ca191bb657b4642ef",
        strip_prefix = "mime_guess-2.0.4",
        build_file = Label("//cargo/remote:BUILD.mime_guess-2.0.4.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__mio__0_8_0",
        url = "https://crates.io/api/v1/crates/mio/0.8.0/download",
        type = "tar.gz",
        sha256 = "ba272f85fa0b41fc91872be579b3bbe0f56b792aa361a380eb669469f68dafb2",
        strip_prefix = "mio-0.8.0",
        build_file = Label("//cargo/remote:BUILD.mio-0.8.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__miow__0_3_7",
        url = "https://crates.io/api/v1/crates/miow/0.3.7/download",
        type = "tar.gz",
        sha256 = "b9f1c5b025cda876f66ef43a113f91ebc9f4ccef34843000e0adf6ebbab84e21",
        strip_prefix = "miow-0.3.7",
        build_file = Label("//cargo/remote:BUILD.miow-0.3.7.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__native_tls__0_2_8",
        url = "https://crates.io/api/v1/crates/native-tls/0.2.8/download",
        type = "tar.gz",
        sha256 = "48ba9f7719b5a0f42f338907614285fb5fd70e53858141f69898a1fb7203b24d",
        strip_prefix = "native-tls-0.2.8",
        build_file = Label("//cargo/remote:BUILD.native-tls-0.2.8.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__ntapi__0_3_7",
        url = "https://crates.io/api/v1/crates/ntapi/0.3.7/download",
        type = "tar.gz",
        sha256 = "c28774a7fd2fbb4f0babd8237ce554b73af68021b5f695a3cebd6c59bac0980f",
        strip_prefix = "ntapi-0.3.7",
        build_file = Label("//cargo/remote:BUILD.ntapi-0.3.7.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__num_integer__0_1_44",
        url = "https://crates.io/api/v1/crates/num-integer/0.1.44/download",
        type = "tar.gz",
        sha256 = "d2cc698a63b549a70bc047073d2949cce27cd1c7b0a4a862d08a8031bc2801db",
        strip_prefix = "num-integer-0.1.44",
        build_file = Label("//cargo/remote:BUILD.num-integer-0.1.44.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__num_traits__0_2_14",
        url = "https://crates.io/api/v1/crates/num-traits/0.2.14/download",
        type = "tar.gz",
        sha256 = "9a64b1ec5cda2586e284722486d802acf1f7dbdc623e2bfc57e65ca1cd099290",
        strip_prefix = "num-traits-0.2.14",
        build_file = Label("//cargo/remote:BUILD.num-traits-0.2.14.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__num_cpus__1_13_1",
        url = "https://crates.io/api/v1/crates/num_cpus/1.13.1/download",
        type = "tar.gz",
        sha256 = "19e64526ebdee182341572e50e9ad03965aa510cd94427a4549448f285e957a1",
        strip_prefix = "num_cpus-1.13.1",
        build_file = Label("//cargo/remote:BUILD.num_cpus-1.13.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__once_cell__1_10_0",
        url = "https://crates.io/api/v1/crates/once_cell/1.10.0/download",
        type = "tar.gz",
        sha256 = "87f3e037eac156d1775da914196f0f37741a274155e34a0b7e427c35d2a2ecb9",
        strip_prefix = "once_cell-1.10.0",
        build_file = Label("//cargo/remote:BUILD.once_cell-1.10.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__openssl__0_10_38",
        url = "https://crates.io/api/v1/crates/openssl/0.10.38/download",
        type = "tar.gz",
        sha256 = "0c7ae222234c30df141154f159066c5093ff73b63204dcda7121eb082fc56a95",
        strip_prefix = "openssl-0.10.38",
        build_file = Label("//cargo/remote:BUILD.openssl-0.10.38.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__openssl_probe__0_1_5",
        url = "https://crates.io/api/v1/crates/openssl-probe/0.1.5/download",
        type = "tar.gz",
        sha256 = "ff011a302c396a5197692431fc1948019154afc178baf7d8e37367442a4601cf",
        strip_prefix = "openssl-probe-0.1.5",
        build_file = Label("//cargo/remote:BUILD.openssl-probe-0.1.5.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__openssl_sys__0_9_72",
        url = "https://crates.io/api/v1/crates/openssl-sys/0.9.72/download",
        type = "tar.gz",
        sha256 = "7e46109c383602735fa0a2e48dd2b7c892b048e1bf69e5c3b1d804b7d9c203cb",
        strip_prefix = "openssl-sys-0.9.72",
        build_file = Label("//cargo/remote:BUILD.openssl-sys-0.9.72.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__parking_lot__0_11_2",
        url = "https://crates.io/api/v1/crates/parking_lot/0.11.2/download",
        type = "tar.gz",
        sha256 = "7d17b78036a60663b797adeaee46f5c9dfebb86948d1255007a1d6be0271ff99",
        strip_prefix = "parking_lot-0.11.2",
        build_file = Label("//cargo/remote:BUILD.parking_lot-0.11.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__parking_lot__0_12_0",
        url = "https://crates.io/api/v1/crates/parking_lot/0.12.0/download",
        type = "tar.gz",
        sha256 = "87f5ec2493a61ac0506c0f4199f99070cbe83857b0337006a30f3e6719b8ef58",
        strip_prefix = "parking_lot-0.12.0",
        build_file = Label("//cargo/remote:BUILD.parking_lot-0.12.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__parking_lot_core__0_8_5",
        url = "https://crates.io/api/v1/crates/parking_lot_core/0.8.5/download",
        type = "tar.gz",
        sha256 = "d76e8e1493bcac0d2766c42737f34458f1c8c50c0d23bcb24ea953affb273216",
        strip_prefix = "parking_lot_core-0.8.5",
        build_file = Label("//cargo/remote:BUILD.parking_lot_core-0.8.5.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__parking_lot_core__0_9_1",
        url = "https://crates.io/api/v1/crates/parking_lot_core/0.9.1/download",
        type = "tar.gz",
        sha256 = "28141e0cc4143da2443301914478dc976a61ffdb3f043058310c70df2fed8954",
        strip_prefix = "parking_lot_core-0.9.1",
        build_file = Label("//cargo/remote:BUILD.parking_lot_core-0.9.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__paste__0_1_18",
        url = "https://crates.io/api/v1/crates/paste/0.1.18/download",
        type = "tar.gz",
        sha256 = "45ca20c77d80be666aef2b45486da86238fabe33e38306bd3118fe4af33fa880",
        strip_prefix = "paste-0.1.18",
        build_file = Label("//cargo/remote:BUILD.paste-0.1.18.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__paste_impl__0_1_18",
        url = "https://crates.io/api/v1/crates/paste-impl/0.1.18/download",
        type = "tar.gz",
        sha256 = "d95a7db200b97ef370c8e6de0088252f7e0dfff7d047a28528e47456c0fc98b6",
        strip_prefix = "paste-impl-0.1.18",
        build_file = Label("//cargo/remote:BUILD.paste-impl-0.1.18.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__percent_encoding__2_1_0",
        url = "https://crates.io/api/v1/crates/percent-encoding/2.1.0/download",
        type = "tar.gz",
        sha256 = "d4fd5641d01c8f18a23da7b6fe29298ff4b55afcccdf78973b24cf3175fee32e",
        strip_prefix = "percent-encoding-2.1.0",
        build_file = Label("//cargo/remote:BUILD.percent-encoding-2.1.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__pin_project_lite__0_2_8",
        url = "https://crates.io/api/v1/crates/pin-project-lite/0.2.8/download",
        type = "tar.gz",
        sha256 = "e280fbe77cc62c91527259e9442153f4688736748d24660126286329742b4c6c",
        strip_prefix = "pin-project-lite-0.2.8",
        build_file = Label("//cargo/remote:BUILD.pin-project-lite-0.2.8.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__pin_utils__0_1_0",
        url = "https://crates.io/api/v1/crates/pin-utils/0.1.0/download",
        type = "tar.gz",
        sha256 = "8b870d8c151b6f2fb93e84a13146138f05d02ed11c7e7c54f8826aaaf7c9f184",
        strip_prefix = "pin-utils-0.1.0",
        build_file = Label("//cargo/remote:BUILD.pin-utils-0.1.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__pkg_config__0_3_24",
        url = "https://crates.io/api/v1/crates/pkg-config/0.3.24/download",
        type = "tar.gz",
        sha256 = "58893f751c9b0412871a09abd62ecd2a00298c6c83befa223ef98c52aef40cbe",
        strip_prefix = "pkg-config-0.3.24",
        build_file = Label("//cargo/remote:BUILD.pkg-config-0.3.24.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__ppv_lite86__0_2_16",
        url = "https://crates.io/api/v1/crates/ppv-lite86/0.2.16/download",
        type = "tar.gz",
        sha256 = "eb9f9e6e233e5c4a35559a617bf40a4ec447db2e84c20b55a6f83167b7e57872",
        strip_prefix = "ppv-lite86-0.2.16",
        build_file = Label("//cargo/remote:BUILD.ppv-lite86-0.2.16.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__proc_macro_hack__0_5_19",
        url = "https://crates.io/api/v1/crates/proc-macro-hack/0.5.19/download",
        type = "tar.gz",
        sha256 = "dbf0c48bc1d91375ae5c3cd81e3722dff1abcf81a30960240640d223f59fe0e5",
        strip_prefix = "proc-macro-hack-0.5.19",
        build_file = Label("//cargo/remote:BUILD.proc-macro-hack-0.5.19.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__proc_macro2__1_0_36",
        url = "https://crates.io/api/v1/crates/proc-macro2/1.0.36/download",
        type = "tar.gz",
        sha256 = "c7342d5883fbccae1cc37a2353b09c87c9b0f3afd73f5fb9bba687a1f733b029",
        strip_prefix = "proc-macro2-1.0.36",
        build_file = Label("//cargo/remote:BUILD.proc-macro2-1.0.36.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__protobuf__2_8_2",
        url = "https://crates.io/api/v1/crates/protobuf/2.8.2/download",
        type = "tar.gz",
        sha256 = "70731852eec72c56d11226c8a5f96ad5058a3dab73647ca5f7ee351e464f2571",
        strip_prefix = "protobuf-2.8.2",
        build_file = Label("//cargo/remote:BUILD.protobuf-2.8.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__protobuf_codegen__2_8_2",
        url = "https://crates.io/api/v1/crates/protobuf-codegen/2.8.2/download",
        type = "tar.gz",
        sha256 = "3d74b9cbbf2ac9a7169c85a3714ec16c51ee9ec7cfd511549527e9a7df720795",
        strip_prefix = "protobuf-codegen-2.8.2",
        build_file = Label("//cargo/remote:BUILD.protobuf-codegen-2.8.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__pyo3__0_15_1",
        url = "https://crates.io/api/v1/crates/pyo3/0.15.1/download",
        type = "tar.gz",
        sha256 = "7cf01dbf1c05af0a14c7779ed6f3aa9deac9c3419606ac9de537a2d649005720",
        strip_prefix = "pyo3-0.15.1",
        build_file = Label("//cargo/remote:BUILD.pyo3-0.15.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__pyo3_build_config__0_15_1",
        url = "https://crates.io/api/v1/crates/pyo3-build-config/0.15.1/download",
        type = "tar.gz",
        sha256 = "dbf9e4d128bfbddc898ad3409900080d8d5095c379632fbbfbb9c8cfb1fb852b",
        strip_prefix = "pyo3-build-config-0.15.1",
        build_file = Label("//cargo/remote:BUILD.pyo3-build-config-0.15.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__pyo3_macros__0_15_1",
        url = "https://crates.io/api/v1/crates/pyo3-macros/0.15.1/download",
        type = "tar.gz",
        sha256 = "67701eb32b1f9a9722b4bc54b548ff9d7ebfded011c12daece7b9063be1fd755",
        strip_prefix = "pyo3-macros-0.15.1",
        build_file = Label("//cargo/remote:BUILD.pyo3-macros-0.15.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__pyo3_macros_backend__0_15_1",
        url = "https://crates.io/api/v1/crates/pyo3-macros-backend/0.15.1/download",
        type = "tar.gz",
        sha256 = "f44f09e825ee49a105f2c7b23ebee50886a9aee0746f4dd5a704138a64b0218a",
        strip_prefix = "pyo3-macros-backend-0.15.1",
        build_file = Label("//cargo/remote:BUILD.pyo3-macros-backend-0.15.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__quote__1_0_15",
        url = "https://crates.io/api/v1/crates/quote/1.0.15/download",
        type = "tar.gz",
        sha256 = "864d3e96a899863136fc6e99f3d7cae289dafe43bf2c5ac19b70df7210c0a145",
        strip_prefix = "quote-1.0.15",
        build_file = Label("//cargo/remote:BUILD.quote-1.0.15.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__rand__0_8_5",
        url = "https://crates.io/api/v1/crates/rand/0.8.5/download",
        type = "tar.gz",
        sha256 = "34af8d1a0e25924bc5b7c43c079c942339d8f0a8b57c39049bef581b46327404",
        strip_prefix = "rand-0.8.5",
        build_file = Label("//cargo/remote:BUILD.rand-0.8.5.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__rand_chacha__0_3_1",
        url = "https://crates.io/api/v1/crates/rand_chacha/0.3.1/download",
        type = "tar.gz",
        sha256 = "e6c10a63a0fa32252be49d21e7709d4d4baf8d231c2dbce1eaa8141b9b127d88",
        strip_prefix = "rand_chacha-0.3.1",
        build_file = Label("//cargo/remote:BUILD.rand_chacha-0.3.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__rand_core__0_6_3",
        url = "https://crates.io/api/v1/crates/rand_core/0.6.3/download",
        type = "tar.gz",
        sha256 = "d34f1408f55294453790c48b2f1ebbb1c5b4b7563eb1f418bcfcfdbb06ebb4e7",
        strip_prefix = "rand_core-0.6.3",
        build_file = Label("//cargo/remote:BUILD.rand_core-0.6.3.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__redox_syscall__0_2_11",
        url = "https://crates.io/api/v1/crates/redox_syscall/0.2.11/download",
        type = "tar.gz",
        sha256 = "8380fe0152551244f0747b1bf41737e0f8a74f97a14ccefd1148187271634f3c",
        strip_prefix = "redox_syscall-0.2.11",
        build_file = Label("//cargo/remote:BUILD.redox_syscall-0.2.11.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__regex__1_5_5",
        url = "https://crates.io/api/v1/crates/regex/1.5.5/download",
        type = "tar.gz",
        sha256 = "1a11647b6b25ff05a515cb92c365cec08801e83423a235b51e231e1808747286",
        strip_prefix = "regex-1.5.5",
        build_file = Label("//cargo/remote:BUILD.regex-1.5.5.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__regex_syntax__0_6_25",
        url = "https://crates.io/api/v1/crates/regex-syntax/0.6.25/download",
        type = "tar.gz",
        sha256 = "f497285884f3fcff424ffc933e56d7cbca511def0c9831a7f9b5f6153e3cc89b",
        strip_prefix = "regex-syntax-0.6.25",
        build_file = Label("//cargo/remote:BUILD.regex-syntax-0.6.25.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__remove_dir_all__0_5_3",
        url = "https://crates.io/api/v1/crates/remove_dir_all/0.5.3/download",
        type = "tar.gz",
        sha256 = "3acd125665422973a33ac9d3dd2df85edad0f4ae9b00dafb1a05e43a9f5ef8e7",
        strip_prefix = "remove_dir_all-0.5.3",
        build_file = Label("//cargo/remote:BUILD.remove_dir_all-0.5.3.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__reqwest__0_11_9",
        url = "https://crates.io/api/v1/crates/reqwest/0.11.9/download",
        type = "tar.gz",
        sha256 = "87f242f1488a539a79bac6dbe7c8609ae43b7914b7736210f239a37cccb32525",
        strip_prefix = "reqwest-0.11.9",
        build_file = Label("//cargo/remote:BUILD.reqwest-0.11.9.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__ring__0_16_20",
        url = "https://crates.io/api/v1/crates/ring/0.16.20/download",
        type = "tar.gz",
        sha256 = "3053cf52e236a3ed746dfc745aa9cacf1b791d846bdaf412f60a8d7d6e17c8fc",
        strip_prefix = "ring-0.16.20",
        build_file = Label("//cargo/remote:BUILD.ring-0.16.20.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__rustls__0_20_4",
        url = "https://crates.io/api/v1/crates/rustls/0.20.4/download",
        type = "tar.gz",
        sha256 = "4fbfeb8d0ddb84706bc597a5574ab8912817c52a397f819e5b614e2265206921",
        strip_prefix = "rustls-0.20.4",
        build_file = Label("//cargo/remote:BUILD.rustls-0.20.4.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__rustls_pemfile__0_2_1",
        url = "https://crates.io/api/v1/crates/rustls-pemfile/0.2.1/download",
        type = "tar.gz",
        sha256 = "5eebeaeb360c87bfb72e84abdb3447159c0eaececf1bef2aecd65a8be949d1c9",
        strip_prefix = "rustls-pemfile-0.2.1",
        build_file = Label("//cargo/remote:BUILD.rustls-pemfile-0.2.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__ryu__1_0_9",
        url = "https://crates.io/api/v1/crates/ryu/1.0.9/download",
        type = "tar.gz",
        sha256 = "73b4b750c782965c211b42f022f59af1fbceabdd026623714f104152f1ec149f",
        strip_prefix = "ryu-1.0.9",
        build_file = Label("//cargo/remote:BUILD.ryu-1.0.9.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__schannel__0_1_19",
        url = "https://crates.io/api/v1/crates/schannel/0.1.19/download",
        type = "tar.gz",
        sha256 = "8f05ba609c234e60bee0d547fe94a4c7e9da733d1c962cf6e59efa4cd9c8bc75",
        strip_prefix = "schannel-0.1.19",
        build_file = Label("//cargo/remote:BUILD.schannel-0.1.19.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__scopeguard__1_1_0",
        url = "https://crates.io/api/v1/crates/scopeguard/1.1.0/download",
        type = "tar.gz",
        sha256 = "d29ab0c6d3fc0ee92fe66e2d99f700eab17a8d57d1c1d3b748380fb20baa78cd",
        strip_prefix = "scopeguard-1.1.0",
        build_file = Label("//cargo/remote:BUILD.scopeguard-1.1.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__sct__0_7_0",
        url = "https://crates.io/api/v1/crates/sct/0.7.0/download",
        type = "tar.gz",
        sha256 = "d53dcdb7c9f8158937a7981b48accfd39a43af418591a5d008c7b22b5e1b7ca4",
        strip_prefix = "sct-0.7.0",
        build_file = Label("//cargo/remote:BUILD.sct-0.7.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__security_framework__2_6_1",
        url = "https://crates.io/api/v1/crates/security-framework/2.6.1/download",
        type = "tar.gz",
        sha256 = "2dc14f172faf8a0194a3aded622712b0de276821addc574fa54fc0a1167e10dc",
        strip_prefix = "security-framework-2.6.1",
        build_file = Label("//cargo/remote:BUILD.security-framework-2.6.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__security_framework_sys__2_6_1",
        url = "https://crates.io/api/v1/crates/security-framework-sys/2.6.1/download",
        type = "tar.gz",
        sha256 = "0160a13a177a45bfb43ce71c01580998474f556ad854dcbca936dd2841a5c556",
        strip_prefix = "security-framework-sys-2.6.1",
        build_file = Label("//cargo/remote:BUILD.security-framework-sys-2.6.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__serde__1_0_136",
        url = "https://crates.io/api/v1/crates/serde/1.0.136/download",
        type = "tar.gz",
        sha256 = "ce31e24b01e1e524df96f1c2fdd054405f8d7376249a5110886fb4b658484789",
        strip_prefix = "serde-1.0.136",
        build_file = Label("//cargo/remote:BUILD.serde-1.0.136.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__serde_derive__1_0_136",
        url = "https://crates.io/api/v1/crates/serde_derive/1.0.136/download",
        type = "tar.gz",
        sha256 = "08597e7152fcd306f41838ed3e37be9eaeed2b61c42e2117266a554fab4662f9",
        strip_prefix = "serde_derive-1.0.136",
        build_file = Label("//cargo/remote:BUILD.serde_derive-1.0.136.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__serde_json__1_0_79",
        url = "https://crates.io/api/v1/crates/serde_json/1.0.79/download",
        type = "tar.gz",
        sha256 = "8e8d9fa5c3b304765ce1fd9c4c8a3de2c8db365a5b91be52f186efc675681d95",
        strip_prefix = "serde_json-1.0.79",
        build_file = Label("//cargo/remote:BUILD.serde_json-1.0.79.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__serde_urlencoded__0_7_1",
        url = "https://crates.io/api/v1/crates/serde_urlencoded/0.7.1/download",
        type = "tar.gz",
        sha256 = "d3491c14715ca2294c4d6a88f15e84739788c1d030eed8c110436aafdaa2f3fd",
        strip_prefix = "serde_urlencoded-0.7.1",
        build_file = Label("//cargo/remote:BUILD.serde_urlencoded-0.7.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__signal_hook_registry__1_4_0",
        url = "https://crates.io/api/v1/crates/signal-hook-registry/1.4.0/download",
        type = "tar.gz",
        sha256 = "e51e73328dc4ac0c7ccbda3a494dfa03df1de2f46018127f60c693f2648455b0",
        strip_prefix = "signal-hook-registry-1.4.0",
        build_file = Label("//cargo/remote:BUILD.signal-hook-registry-1.4.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__slab__0_4_5",
        url = "https://crates.io/api/v1/crates/slab/0.4.5/download",
        type = "tar.gz",
        sha256 = "9def91fd1e018fe007022791f865d0ccc9b3a0d5001e01aabb8b40e46000afb5",
        strip_prefix = "slab-0.4.5",
        build_file = Label("//cargo/remote:BUILD.slab-0.4.5.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__smallvec__1_8_0",
        url = "https://crates.io/api/v1/crates/smallvec/1.8.0/download",
        type = "tar.gz",
        sha256 = "f2dd574626839106c320a323308629dcb1acfc96e32a8cba364ddc61ac23ee83",
        strip_prefix = "smallvec-1.8.0",
        build_file = Label("//cargo/remote:BUILD.smallvec-1.8.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__socket2__0_4_4",
        url = "https://crates.io/api/v1/crates/socket2/0.4.4/download",
        type = "tar.gz",
        sha256 = "66d72b759436ae32898a2af0a14218dbf55efde3feeb170eb623637db85ee1e0",
        strip_prefix = "socket2-0.4.4",
        build_file = Label("//cargo/remote:BUILD.socket2-0.4.4.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__spin__0_5_2",
        url = "https://crates.io/api/v1/crates/spin/0.5.2/download",
        type = "tar.gz",
        sha256 = "6e63cff320ae2c57904679ba7cb63280a3dc4613885beafb148ee7bf9aa9042d",
        strip_prefix = "spin-0.5.2",
        build_file = Label("//cargo/remote:BUILD.spin-0.5.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__strsim__0_10_0",
        url = "https://crates.io/api/v1/crates/strsim/0.10.0/download",
        type = "tar.gz",
        sha256 = "73473c0e59e6d5812c5dfe2a064a6444949f089e20eec9a2e5506596494e4623",
        strip_prefix = "strsim-0.10.0",
        build_file = Label("//cargo/remote:BUILD.strsim-0.10.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__syn__1_0_86",
        url = "https://crates.io/api/v1/crates/syn/1.0.86/download",
        type = "tar.gz",
        sha256 = "8a65b3f4ffa0092e9887669db0eae07941f023991ab58ea44da8fe8e2d511c6b",
        strip_prefix = "syn-1.0.86",
        build_file = Label("//cargo/remote:BUILD.syn-1.0.86.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__tempfile__3_3_0",
        url = "https://crates.io/api/v1/crates/tempfile/3.3.0/download",
        type = "tar.gz",
        sha256 = "5cdb1ef4eaeeaddc8fbd371e5017057064af0911902ef36b39801f67cc6d79e4",
        strip_prefix = "tempfile-3.3.0",
        build_file = Label("//cargo/remote:BUILD.tempfile-3.3.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__termcolor__1_1_3",
        url = "https://crates.io/api/v1/crates/termcolor/1.1.3/download",
        type = "tar.gz",
        sha256 = "bab24d30b911b2376f3a13cc2cd443142f0c81dda04c118693e35b3835757755",
        strip_prefix = "termcolor-1.1.3",
        build_file = Label("//cargo/remote:BUILD.termcolor-1.1.3.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__thiserror__1_0_30",
        url = "https://crates.io/api/v1/crates/thiserror/1.0.30/download",
        type = "tar.gz",
        sha256 = "854babe52e4df1653706b98fcfc05843010039b406875930a70e4d9644e5c417",
        strip_prefix = "thiserror-1.0.30",
        build_file = Label("//cargo/remote:BUILD.thiserror-1.0.30.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__thiserror_impl__1_0_30",
        url = "https://crates.io/api/v1/crates/thiserror-impl/1.0.30/download",
        type = "tar.gz",
        sha256 = "aa32fd3f627f367fe16f893e2597ae3c05020f8bba2666a4e6ea73d377e5714b",
        strip_prefix = "thiserror-impl-1.0.30",
        build_file = Label("//cargo/remote:BUILD.thiserror-impl-1.0.30.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__time__0_1_43",
        url = "https://crates.io/api/v1/crates/time/0.1.43/download",
        type = "tar.gz",
        sha256 = "ca8a50ef2360fbd1eeb0ecd46795a87a19024eb4b53c5dc916ca1fd95fe62438",
        strip_prefix = "time-0.1.43",
        build_file = Label("//cargo/remote:BUILD.time-0.1.43.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__tinyvec__1_5_1",
        url = "https://crates.io/api/v1/crates/tinyvec/1.5.1/download",
        type = "tar.gz",
        sha256 = "2c1c1d5a42b6245520c249549ec267180beaffcc0615401ac8e31853d4b6d8d2",
        strip_prefix = "tinyvec-1.5.1",
        build_file = Label("//cargo/remote:BUILD.tinyvec-1.5.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__tinyvec_macros__0_1_0",
        url = "https://crates.io/api/v1/crates/tinyvec_macros/0.1.0/download",
        type = "tar.gz",
        sha256 = "cda74da7e1a664f795bb1f8a87ec406fb89a02522cf6e50620d016add6dbbf5c",
        strip_prefix = "tinyvec_macros-0.1.0",
        build_file = Label("//cargo/remote:BUILD.tinyvec_macros-0.1.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__tokio__1_17_0",
        url = "https://crates.io/api/v1/crates/tokio/1.17.0/download",
        type = "tar.gz",
        sha256 = "2af73ac49756f3f7c01172e34a23e5d0216f6c32333757c2c61feb2bbff5a5ee",
        strip_prefix = "tokio-1.17.0",
        build_file = Label("//cargo/remote:BUILD.tokio-1.17.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__tokio_macros__1_7_0",
        url = "https://crates.io/api/v1/crates/tokio-macros/1.7.0/download",
        type = "tar.gz",
        sha256 = "b557f72f448c511a979e2564e55d74e6c4432fc96ff4f6241bc6bded342643b7",
        strip_prefix = "tokio-macros-1.7.0",
        build_file = Label("//cargo/remote:BUILD.tokio-macros-1.7.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__tokio_native_tls__0_3_0",
        url = "https://crates.io/api/v1/crates/tokio-native-tls/0.3.0/download",
        type = "tar.gz",
        sha256 = "f7d995660bd2b7f8c1568414c1126076c13fbb725c40112dc0120b78eb9b717b",
        strip_prefix = "tokio-native-tls-0.3.0",
        build_file = Label("//cargo/remote:BUILD.tokio-native-tls-0.3.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__tokio_util__0_6_9",
        url = "https://crates.io/api/v1/crates/tokio-util/0.6.9/download",
        type = "tar.gz",
        sha256 = "9e99e1983e5d376cd8eb4b66604d2e99e79f5bd988c3055891dcd8c9e2604cc0",
        strip_prefix = "tokio-util-0.6.9",
        build_file = Label("//cargo/remote:BUILD.tokio-util-0.6.9.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__tower_service__0_3_1",
        url = "https://crates.io/api/v1/crates/tower-service/0.3.1/download",
        type = "tar.gz",
        sha256 = "360dfd1d6d30e05fda32ace2c8c70e9c0a9da713275777f5a4dbb8a1893930c6",
        strip_prefix = "tower-service-0.3.1",
        build_file = Label("//cargo/remote:BUILD.tower-service-0.3.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__tracing__0_1_31",
        url = "https://crates.io/api/v1/crates/tracing/0.1.31/download",
        type = "tar.gz",
        sha256 = "f6c650a8ef0cd2dd93736f033d21cbd1224c5a967aa0c258d00fcf7dafef9b9f",
        strip_prefix = "tracing-0.1.31",
        build_file = Label("//cargo/remote:BUILD.tracing-0.1.31.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__tracing_core__0_1_23",
        url = "https://crates.io/api/v1/crates/tracing-core/0.1.23/download",
        type = "tar.gz",
        sha256 = "aa31669fa42c09c34d94d8165dd2012e8ff3c66aca50f3bb226b68f216f2706c",
        strip_prefix = "tracing-core-0.1.23",
        build_file = Label("//cargo/remote:BUILD.tracing-core-0.1.23.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__try_lock__0_2_3",
        url = "https://crates.io/api/v1/crates/try-lock/0.2.3/download",
        type = "tar.gz",
        sha256 = "59547bce71d9c38b83d9c0e92b6066c4253371f15005def0c30d9657f50c7642",
        strip_prefix = "try-lock-0.2.3",
        build_file = Label("//cargo/remote:BUILD.try-lock-0.2.3.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__unicase__2_6_0",
        url = "https://crates.io/api/v1/crates/unicase/2.6.0/download",
        type = "tar.gz",
        sha256 = "50f37be617794602aabbeee0be4f259dc1778fabe05e2d67ee8f79326d5cb4f6",
        strip_prefix = "unicase-2.6.0",
        build_file = Label("//cargo/remote:BUILD.unicase-2.6.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__unicode_bidi__0_3_7",
        url = "https://crates.io/api/v1/crates/unicode-bidi/0.3.7/download",
        type = "tar.gz",
        sha256 = "1a01404663e3db436ed2746d9fefef640d868edae3cceb81c3b8d5732fda678f",
        strip_prefix = "unicode-bidi-0.3.7",
        build_file = Label("//cargo/remote:BUILD.unicode-bidi-0.3.7.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__unicode_normalization__0_1_19",
        url = "https://crates.io/api/v1/crates/unicode-normalization/0.1.19/download",
        type = "tar.gz",
        sha256 = "d54590932941a9e9266f0832deed84ebe1bf2e4c9e4a3554d393d18f5e854bf9",
        strip_prefix = "unicode-normalization-0.1.19",
        build_file = Label("//cargo/remote:BUILD.unicode-normalization-0.1.19.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__unicode_xid__0_2_2",
        url = "https://crates.io/api/v1/crates/unicode-xid/0.2.2/download",
        type = "tar.gz",
        sha256 = "8ccb82d61f80a663efe1f787a51b16b5a51e3314d6ac365b08639f52387b33f3",
        strip_prefix = "unicode-xid-0.2.2",
        build_file = Label("//cargo/remote:BUILD.unicode-xid-0.2.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__unindent__0_1_8",
        url = "https://crates.io/api/v1/crates/unindent/0.1.8/download",
        type = "tar.gz",
        sha256 = "514672a55d7380da379785a4d70ca8386c8883ff7eaae877be4d2081cebe73d8",
        strip_prefix = "unindent-0.1.8",
        build_file = Label("//cargo/remote:BUILD.unindent-0.1.8.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__untrusted__0_7_1",
        url = "https://crates.io/api/v1/crates/untrusted/0.7.1/download",
        type = "tar.gz",
        sha256 = "a156c684c91ea7d62626509bce3cb4e1d9ed5c4d978f7b4352658f96a4c26b4a",
        strip_prefix = "untrusted-0.7.1",
        build_file = Label("//cargo/remote:BUILD.untrusted-0.7.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__url__2_2_2",
        url = "https://crates.io/api/v1/crates/url/2.2.2/download",
        type = "tar.gz",
        sha256 = "a507c383b2d33b5fc35d1861e77e6b383d158b2da5e14fe51b83dfedf6fd578c",
        strip_prefix = "url-2.2.2",
        build_file = Label("//cargo/remote:BUILD.url-2.2.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__uuid__0_8_2",
        url = "https://crates.io/api/v1/crates/uuid/0.8.2/download",
        type = "tar.gz",
        sha256 = "bc5cf98d8186244414c848017f0e2676b3fcb46807f6668a97dfe67359a3c4b7",
        strip_prefix = "uuid-0.8.2",
        build_file = Label("//cargo/remote:BUILD.uuid-0.8.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__vcpkg__0_2_15",
        url = "https://crates.io/api/v1/crates/vcpkg/0.2.15/download",
        type = "tar.gz",
        sha256 = "accd4ea62f7bb7a82fe23066fb0957d48ef677f6eeb8215f372f52e48bb32426",
        strip_prefix = "vcpkg-0.2.15",
        build_file = Label("//cargo/remote:BUILD.vcpkg-0.2.15.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__version_check__0_9_4",
        url = "https://crates.io/api/v1/crates/version_check/0.9.4/download",
        type = "tar.gz",
        sha256 = "49874b5167b65d7193b8aba1567f5c7d93d001cafc34600cee003eda787e483f",
        strip_prefix = "version_check-0.9.4",
        build_file = Label("//cargo/remote:BUILD.version_check-0.9.4.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__want__0_3_0",
        url = "https://crates.io/api/v1/crates/want/0.3.0/download",
        type = "tar.gz",
        sha256 = "1ce8a968cb1cd110d136ff8b819a556d6fb6d919363c61534f6860c7eb172ba0",
        strip_prefix = "want-0.3.0",
        build_file = Label("//cargo/remote:BUILD.want-0.3.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__wasi__0_10_2_wasi_snapshot_preview1",
        url = "https://crates.io/api/v1/crates/wasi/0.10.2+wasi-snapshot-preview1/download",
        type = "tar.gz",
        sha256 = "fd6fbd9a79829dd1ad0cc20627bf1ed606756a7f77edff7b66b7064f9cb327c6",
        strip_prefix = "wasi-0.10.2+wasi-snapshot-preview1",
        build_file = Label("//cargo/remote:BUILD.wasi-0.10.2+wasi-snapshot-preview1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__wasm_bindgen__0_2_79",
        url = "https://crates.io/api/v1/crates/wasm-bindgen/0.2.79/download",
        type = "tar.gz",
        sha256 = "25f1af7423d8588a3d840681122e72e6a24ddbcb3f0ec385cac0d12d24256c06",
        strip_prefix = "wasm-bindgen-0.2.79",
        build_file = Label("//cargo/remote:BUILD.wasm-bindgen-0.2.79.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__wasm_bindgen_backend__0_2_79",
        url = "https://crates.io/api/v1/crates/wasm-bindgen-backend/0.2.79/download",
        type = "tar.gz",
        sha256 = "8b21c0df030f5a177f3cba22e9bc4322695ec43e7257d865302900290bcdedca",
        strip_prefix = "wasm-bindgen-backend-0.2.79",
        build_file = Label("//cargo/remote:BUILD.wasm-bindgen-backend-0.2.79.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__wasm_bindgen_futures__0_4_29",
        url = "https://crates.io/api/v1/crates/wasm-bindgen-futures/0.4.29/download",
        type = "tar.gz",
        sha256 = "2eb6ec270a31b1d3c7e266b999739109abce8b6c87e4b31fcfcd788b65267395",
        strip_prefix = "wasm-bindgen-futures-0.4.29",
        build_file = Label("//cargo/remote:BUILD.wasm-bindgen-futures-0.4.29.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__wasm_bindgen_macro__0_2_79",
        url = "https://crates.io/api/v1/crates/wasm-bindgen-macro/0.2.79/download",
        type = "tar.gz",
        sha256 = "2f4203d69e40a52ee523b2529a773d5ffc1dc0071801c87b3d270b471b80ed01",
        strip_prefix = "wasm-bindgen-macro-0.2.79",
        build_file = Label("//cargo/remote:BUILD.wasm-bindgen-macro-0.2.79.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__wasm_bindgen_macro_support__0_2_79",
        url = "https://crates.io/api/v1/crates/wasm-bindgen-macro-support/0.2.79/download",
        type = "tar.gz",
        sha256 = "bfa8a30d46208db204854cadbb5d4baf5fcf8071ba5bf48190c3e59937962ebc",
        strip_prefix = "wasm-bindgen-macro-support-0.2.79",
        build_file = Label("//cargo/remote:BUILD.wasm-bindgen-macro-support-0.2.79.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__wasm_bindgen_shared__0_2_79",
        url = "https://crates.io/api/v1/crates/wasm-bindgen-shared/0.2.79/download",
        type = "tar.gz",
        sha256 = "3d958d035c4438e28c70e4321a2911302f10135ce78a9c7834c0cab4123d06a2",
        strip_prefix = "wasm-bindgen-shared-0.2.79",
        build_file = Label("//cargo/remote:BUILD.wasm-bindgen-shared-0.2.79.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__web_sys__0_3_56",
        url = "https://crates.io/api/v1/crates/web-sys/0.3.56/download",
        type = "tar.gz",
        sha256 = "c060b319f29dd25724f09a2ba1418f142f539b2be99fbf4d2d5a8f7330afb8eb",
        strip_prefix = "web-sys-0.3.56",
        build_file = Label("//cargo/remote:BUILD.web-sys-0.3.56.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__webpki__0_22_0",
        url = "https://crates.io/api/v1/crates/webpki/0.22.0/download",
        type = "tar.gz",
        sha256 = "f095d78192e208183081cc07bc5515ef55216397af48b873e5edcd72637fa1bd",
        strip_prefix = "webpki-0.22.0",
        build_file = Label("//cargo/remote:BUILD.webpki-0.22.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__winapi__0_3_9",
        url = "https://crates.io/api/v1/crates/winapi/0.3.9/download",
        type = "tar.gz",
        sha256 = "5c839a674fcd7a98952e593242ea400abe93992746761e38641405d28b00f419",
        strip_prefix = "winapi-0.3.9",
        build_file = Label("//cargo/remote:BUILD.winapi-0.3.9.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__winapi_i686_pc_windows_gnu__0_4_0",
        url = "https://crates.io/api/v1/crates/winapi-i686-pc-windows-gnu/0.4.0/download",
        type = "tar.gz",
        sha256 = "ac3b87c63620426dd9b991e5ce0329eff545bccbbb34f3be09ff6fb6ab51b7b6",
        strip_prefix = "winapi-i686-pc-windows-gnu-0.4.0",
        build_file = Label("//cargo/remote:BUILD.winapi-i686-pc-windows-gnu-0.4.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__winapi_util__0_1_5",
        url = "https://crates.io/api/v1/crates/winapi-util/0.1.5/download",
        type = "tar.gz",
        sha256 = "70ec6ce85bb158151cae5e5c87f95a8e97d2c0c4b001223f33a334e3ce5de178",
        strip_prefix = "winapi-util-0.1.5",
        build_file = Label("//cargo/remote:BUILD.winapi-util-0.1.5.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__winapi_x86_64_pc_windows_gnu__0_4_0",
        url = "https://crates.io/api/v1/crates/winapi-x86_64-pc-windows-gnu/0.4.0/download",
        type = "tar.gz",
        sha256 = "712e227841d057c1ee1cd2fb22fa7e5a5461ae8e48fa2ca79ec42cfc1931183f",
        strip_prefix = "winapi-x86_64-pc-windows-gnu-0.4.0",
        build_file = Label("//cargo/remote:BUILD.winapi-x86_64-pc-windows-gnu-0.4.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__windows_sys__0_32_0",
        url = "https://crates.io/api/v1/crates/windows-sys/0.32.0/download",
        type = "tar.gz",
        sha256 = "3df6e476185f92a12c072be4a189a0210dcdcf512a1891d6dff9edb874deadc6",
        strip_prefix = "windows-sys-0.32.0",
        build_file = Label("//cargo/remote:BUILD.windows-sys-0.32.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__windows_aarch64_msvc__0_32_0",
        url = "https://crates.io/api/v1/crates/windows_aarch64_msvc/0.32.0/download",
        type = "tar.gz",
        sha256 = "d8e92753b1c443191654ec532f14c199742964a061be25d77d7a96f09db20bf5",
        strip_prefix = "windows_aarch64_msvc-0.32.0",
        build_file = Label("//cargo/remote:BUILD.windows_aarch64_msvc-0.32.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__windows_i686_gnu__0_32_0",
        url = "https://crates.io/api/v1/crates/windows_i686_gnu/0.32.0/download",
        type = "tar.gz",
        sha256 = "6a711c68811799e017b6038e0922cb27a5e2f43a2ddb609fe0b6f3eeda9de615",
        strip_prefix = "windows_i686_gnu-0.32.0",
        build_file = Label("//cargo/remote:BUILD.windows_i686_gnu-0.32.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__windows_i686_msvc__0_32_0",
        url = "https://crates.io/api/v1/crates/windows_i686_msvc/0.32.0/download",
        type = "tar.gz",
        sha256 = "146c11bb1a02615db74680b32a68e2d61f553cc24c4eb5b4ca10311740e44172",
        strip_prefix = "windows_i686_msvc-0.32.0",
        build_file = Label("//cargo/remote:BUILD.windows_i686_msvc-0.32.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__windows_x86_64_gnu__0_32_0",
        url = "https://crates.io/api/v1/crates/windows_x86_64_gnu/0.32.0/download",
        type = "tar.gz",
        sha256 = "c912b12f7454c6620635bbff3450962753834be2a594819bd5e945af18ec64bc",
        strip_prefix = "windows_x86_64_gnu-0.32.0",
        build_file = Label("//cargo/remote:BUILD.windows_x86_64_gnu-0.32.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__windows_x86_64_msvc__0_32_0",
        url = "https://crates.io/api/v1/crates/windows_x86_64_msvc/0.32.0/download",
        type = "tar.gz",
        sha256 = "504a2476202769977a040c6364301a3f65d0cc9e3fb08600b2bda150a0488316",
        strip_prefix = "windows_x86_64_msvc-0.32.0",
        build_file = Label("//cargo/remote:BUILD.windows_x86_64_msvc-0.32.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__winreg__0_7_0",
        url = "https://crates.io/api/v1/crates/winreg/0.7.0/download",
        type = "tar.gz",
        sha256 = "0120db82e8a1e0b9fb3345a539c478767c0048d842860994d96113d5b667bd69",
        strip_prefix = "winreg-0.7.0",
        build_file = Label("//cargo/remote:BUILD.winreg-0.7.0.bazel"),
    )
