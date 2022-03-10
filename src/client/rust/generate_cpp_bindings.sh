#!/bin/bash
mkdir -p src/client/rust/generated
cxxbridge src/client/rust/lib.rs > src/client/rust/generated/rust_cxx.cpp && cxxbridge --header src/client/rust/lib.rs > src/client/rust/generated/rust_cxx.h