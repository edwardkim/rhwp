FROM rust:latest

# wasm 타겟 및 wasm-pack 설치
RUN rustup target add wasm32-unknown-unknown \
    && rustup component add clippy \
    && cargo install wasm-pack@0.15.0

# 호스트 사용자 UID/GID로 실행 (빌드 산출물 소유권 문제 방지)
ARG UID=1000
ARG GID=1000
RUN groupadd -g ${GID} builder 2>/dev/null || true \
    && useradd -m -u ${UID} -g ${GID} builder \
    && mkdir -p /home/builder/.cache/.wasm-pack \
    && chown -R builder:builder /home/builder

ENV CARGO_HOME=/home/builder/.cargo
RUN mkdir -p /home/builder/.cargo \
    && cp -r /usr/local/cargo/* /home/builder/.cargo/ \
    && chown -R builder:builder /home/builder/.cargo

USER builder
WORKDIR /app

# 기본 명령: 네이티브 빌드
CMD ["cargo", "build"]
