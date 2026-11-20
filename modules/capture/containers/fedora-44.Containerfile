# What kitokey-capture is built with for Fedora 44 (D27): that system's PipeWire and ALSA headers,
# clang, cmake, make and a C++ compiler — libflac-sys builds libFLAC with them: FLAC's `project()`
# names no language, so CMake asks for C and C++ (D87, #1059) —, Rust as this repository uses it. Read by scripts/release.sh.
FROM registry.fedoraproject.org/fedora:44
RUN dnf install -y -q gcc gcc-c++ make cmake pkgconf-pkg-config clang clang-devel pipewire-devel alsa-lib-devel && dnf clean all
ENV RUSTUP_HOME=/opt/rustup CARGO_HOME=/opt/cargo PATH=/opt/cargo/bin:$PATH
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain 1.99.0 \
    && chmod -R a+rwX /opt/cargo /opt/rustup
