FROM rust:latest

# wasm 타겟 및 wasm-pack 설치
# [#2233] wasm-pack 버전은 CI 와 단일 정책으로 고정한다. 버전 변경 시 여기와
# .github/actions/install-wasm-pack/action.yml 을 함께 갱신한다.
RUN rustup target add wasm32-unknown-unknown \
    && rustup component add clippy \
    && cargo install wasm-pack@0.15.0

# 호스트 사용자 UID/GID로 실행 (빌드 산출물 소유권 문제 방지)
ARG UID=1000
ARG GID=1000
# `builder` 그룹은 만들어지지 않을 수 있다. GID 가 이미 쓰이고 있으면
# `groupadd` 가 실패하고 `|| true` 가 그것을 삼킨다 — macOS 의 기본 GID 20 이
# 데비안에서는 `dialout` 이라 정확히 그 경우가 된다. 그래서 소유자를 그룹
# **이름**이 아니라 GID 로 지정한다.
RUN groupadd -g ${GID} builder 2>/dev/null || true \
    && useradd -m -u ${UID} -g ${GID} builder \
    && mkdir -p /home/builder/.cache/.wasm-pack \
    && chown -R builder:${GID} /home/builder

ENV CARGO_HOME=/home/builder/.cargo
RUN mkdir -p /home/builder/.cargo \
    && cp -r /usr/local/cargo/* /home/builder/.cargo/ \
    && chown -R builder:${GID} /home/builder/.cargo

USER builder
WORKDIR /app

# 기본 명령: 네이티브 빌드
CMD ["cargo", "build"]
