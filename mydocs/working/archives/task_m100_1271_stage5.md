# Stage 5 보고 — Task M100-1271

## 범위

- 이슈: [#1271](https://github.com/edwardkim/rhwp/issues/1271)
- 단계: 전체 회귀 검증, WASM 갱신, rhwp-studio 시각검증 서버 실행

## Rust 회귀 검증

명령:

```text
cargo fmt --check
cargo test --lib
cargo test --tests
```

결과:

- `cargo fmt --check`: 통과
- `cargo test --lib`: 통과
  - `1538 passed; 0 failed; 6 ignored`
- `cargo test --tests`: 통과
  - 라이브러리 유닛 테스트와 전체 `tests/` 통합 테스트 통과
  - #1271 전용 테스트 `onsaemiro_front_matter_is_not_shifted_by_behind_text_table_fragment` 통과
  - 집중 회귀 테스트 `issue_1197_svg_object_zorder` 통과

## WASM 갱신

`rhwp-studio` 가 현재 Rust 수정본을 사용하도록 WASM 패키지를 갱신했다.

처음에는 저장소 문서의 권장 절차인 Docker WASM 빌드를 시도했다.

```text
docker-compose --env-file .env.docker run --rm wasm
```

결과:

```text
Cannot connect to the Docker daemon at unix:///var/run/docker.sock. Is the docker daemon running?
```

Docker daemon 이 실행 중이 아니어서, 시각검증 서버용 산출물 갱신은 로컬 `wasm-pack` 으로 수행했다.

```text
wasm-pack build --target web
```

결과:

```text
Your wasm pkg is ready to publish at /Users/melee/Documents/projects/forks/rhwp/pkg.
```

확인:

```text
pkg/rhwp_bg.wasm
Jun  3 18:55:53 2026
```

즉 `src/renderer/typeset.rs`, `src/document_core/queries/rendering.rs` 변경 이후의 WASM 이다.

## rhwp-studio 서버

명령:

```text
cd rhwp-studio
npm run dev -- --host 127.0.0.1 --port 7700
```

서버:

```text
http://127.0.0.1:7700/
```

대상 샘플 자동 로드 URL:

```text
http://127.0.0.1:7700/?url=/samples/hwpx/%5B2027%5D%20%EC%98%A8%EC%83%88%EB%AF%B8%EB%A1%9C%201%20%EB%B3%B8%EA%B5%90%EC%9E%AC.hwpx&filename=%5B2027%5D%20%EC%98%A8%EC%83%88%EB%AF%B8%EB%A1%9C%201%20%EB%B3%B8%EA%B5%90%EC%9E%AC.hwpx
```

응답 확인:

```text
curl -I http://127.0.0.1:7700/
curl -I "http://127.0.0.1:7700/samples/hwpx/%5B2027%5D%20%EC%98%A8%EC%83%88%EB%AF%B8%EB%A1%9C%201%20%EB%B3%B8%EA%B5%90%EC%9E%AC.hwpx"
```

결과:

- 루트 페이지: `HTTP/1.1 200 OK`
- 대상 HWPX 샘플: `HTTP/1.1 200 OK`

## 판단

- Stage 1-4 에서 추가/수정한 로직은 전체 Rust 회귀 검증을 통과했다.
- #1271 대상 샘플은 전용 통합 테스트와 Stage 4 구조/시각 검증 기준을 모두 만족한다.
- rhwp-studio 서버는 현재 수정본 WASM 으로 실행 중이며, 사용자가 직접 대상 샘플을 열어 시각검증할 수 있다.
- 이슈 클로즈는 작업지시자 승인 전에는 수행하지 않는다.
