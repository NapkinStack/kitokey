# The MSVC runtime libFLAC is compiled with, the one the Rust build links: the static one (/MT)
# when Cargo builds with `+crt-static`, CMake's default otherwise. Named by .cargo/config.toml.
if("$ENV{CARGO_CFG_TARGET_FEATURE}" MATCHES "(^|,)crt-static(,|$)")
  set(CMAKE_MSVC_RUNTIME_LIBRARY "MultiThreaded")
endif()
