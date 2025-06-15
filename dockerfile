FROM ubuntu:25.10

ENV RUST_HOME /usr/local/lib/rust
ENV RUSTUP_HOME ${RUST_HOME}/rustup
ENV CARGO_HOME ${RUST_HOME}/cargo
ENV PATH="${CARGO_HOME}/bin:${PATH}"

RUN echo "Asia/Tokyo" | tee /etc/timezone \
    && apt-get update \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
    tzdata \
    qemu-system-x86 \
    qemu-system-gui \
    curl \
    file \
    build-essential \
    netcat-openbsd \
    clang \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN mkdir $RUST_HOME \
    && chmod 0755 $RUST_HOME \
    && curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs > ${RUST_HOME}/rustup.sh \
    && chmod +x ${RUST_HOME}/rustup.sh \
    && ${RUST_HOME}/rustup.sh -y --default-toolchain nightly --no-modify-path \
    && . ${RUST_HOME}/cargo/env

RUN rustup target add x86_64-unknown-uefi

CMD []
ENTRYPOINT ["qemu-system-x86_64"]