# Task M100-1251 Stage 2 완료 보고서

- **이슈**: [#1251](https://github.com/edwardkim/rhwp/issues/1251)
- **단계**: Stage 2 — `charming` 네이티브 SSR 스파이크
- **브랜치**: `task-1251-ole-chart`
- **작성일**: 2026-06-03

## 1. 변경 내용

네이티브 빌드에서만 `charming` SSR renderer를 사용할 수 있도록 target-specific dependency를 추가했다.

```toml
[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
charming = { version = "0.6", default-features = false, features = ["ssr"] }
```

추가 파일:

- `src/ole_chart/charming_renderer.rs`

추가 public API:

- `render_smoke_chart_svg(width, height)`
- `OleChartRenderError`

`render_smoke_chart_svg`는 최소 pie chart를 `charming::Chart`로 구성하고 `ImageRenderer::render`를 호출해 SVG 문자열을 반환한다.

참고: `Cargo.lock`는 이 저장소의 `.gitignore` 대상이므로 PR 변경 범위에 포함하지 않는다.

## 2. `charming` 적용 가능성

적용 가능 범위:

- 네이티브 `export-svg`/테스트 경로에서 `charming` `ImageRenderer`를 사용해 SVG 문자열을 생성할 수 있다.
- `charming 0.6.0`의 `ssr` feature는 native target에서 동작했고, smoke test에서 SVG 문자열을 안정적으로 반환했다.
- `Cargo.toml` target-specific dependency로 추가했기 때문에 `wasm32-unknown-unknown` 라이브러리 체크에는 포함되지 않았다.

제한:

- `charming`은 chart option builder와 renderer이며, HWP OLE `/Contents` 스트림 파서는 아니다.
- #1251 fixture의 `/Contents`는 legacy HWP chart binary로 보이며, `charming`을 사용하려면 먼저 rhwp 내부에서 `OleChart` IR로 파싱해야 한다.
- `wasm` feature는 DOM id 기반 renderer라 현재 `export-svg`의 문자열 생성 경로에는 직접 맞지 않는다.
- `ssr` feature는 `deno_core`/`serde_v8`/`v8` 의존성을 끌어와 native 빌드 의존성이 크게 증가한다.

참조한 upstream 문서:

- `charming` Cargo feature 정의: <https://docs.rs/crate/charming/latest/source/Cargo.toml.orig>
- `ImageRenderer`: <https://docs.rs/charming/latest/charming/renderer/image_renderer/struct.ImageRenderer.html>
- `WasmRenderer`: <https://docs.rs/charming/latest/charming/renderer/wasm_renderer/struct.WasmRenderer.html>

## 3. 테스트

신규 smoke test:

- `charming_ssr_smoke_renders_svg_string`

검증 내용:

- `render_smoke_chart_svg(420, 320)` 호출
- 반환값이 `<svg`로 시작함
- sample label `alpha`가 SVG에 포함됨

## 4. 실행 결과

```text
cargo fmt --check
```

통과.

```text
cargo test --test issue_1251_ole_chart_contents -- --nocapture
```

통과: 4 passed.

최초 실행은 sandbox DNS 제한으로 crates.io index 접근에 실패했고, 네트워크 허용 후 재실행하여 `charming v0.6.0`과 관련 dependency를 resolve했다.

```text
cargo test --lib ole_chart -- --nocapture
```

통과: 3 passed.

```text
cargo build
```

통과.

```text
cargo check --target wasm32-unknown-unknown --lib
```

통과. native-only `charming` dependency가 wasm library check를 깨지 않는 것을 확인했다.

## 5. 다음 단계

Stage 3에서는 `samples/143E433F503322BD33.hwp`의 `BinData #2` `/Contents`에서 실제 차트 데이터를 추출할 수 있는지 조사한다.

우선순위:

1. 차트 종류 후보
2. 카테고리 라벨 후보
3. 시리즈 값 후보
4. 시리즈 이름/범례 후보
5. 제목 후보

안정적인 구조를 특정하지 못하면 파싱 성공으로 처리하지 않고 오류 enum과 fallback 사유를 고정한다.
