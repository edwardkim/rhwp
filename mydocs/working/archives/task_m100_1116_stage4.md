# Task #1116 Stage 4 보고서 — Web Canvas/Skia 탭 리더 보정

- 이슈: [edwardkim/rhwp#1116](https://github.com/edwardkim/rhwp/issues/1116)
- 브랜치: `local/task1116`
- 작성일: 2026-05-25
- 상태: SVG 보정 로직을 Web Canvas/Skia 렌더 경로까지 반영

## 1. 배경

Stage 3에서 CLI SVG의 목차 탭 리더 뒤넘김은 줄었지만, 작업지시자가 확인하는 화면은 `http://localhost:7700`의 rhwp-studio Web Canvas 렌더 경로다.

따라서 `src/renderer/svg.rs`만 고치면 브라우저 화면에서는 페이지 번호 뒤까지 이어지는 탭 리더가 그대로 남을 수 있다.

## 2. 구현

공통 헬퍼 추가:

```text
src/renderer/mod.rs::clamp_tab_leader_end_x
```

역할:

- 탭 리더가 같은 run 안의 실제 내용 글자 앞에서 끝나도록 보정.
- 예: `...\t15` 구조에서 점선/실선 leader가 `15` 뒤까지 넘어가지 않게 제한.

반영 경로:

- `src/renderer/svg.rs`
- `src/renderer/web_canvas.rs`
- `src/renderer/skia/text_replay.rs`

기존 layout 단계 보정:

- cross-run right tab에서 직전 run의 모든 tab leader를 후속 텍스트 앞까지 제한.
- line-node 확정 직전에 같은 줄의 후속 TextRun 시작점 기준으로 leader end를 재제한.
- `extract_tab_leaders_with_extended`에서 같은 run의 탭 뒤 실제 content x를 end 후보로 사용.

## 3. 3mm 격자 옵션

CLI SVG 디버그 비교용으로 다음 형식을 추가했다.

```bash
target/debug/rhwp export-svg samples/hwp3-sample16-hwp5.hwp \
  -o output/poc/render-spacing/hwp3-sample16-hwp5-page3-grid-3mm \
  -p 2 \
  --show-grid=3mm \
  --debug-overlay \
  --show-control-codes
```

검증값:

```text
SVG pattern width=11.3386 height=11.3386
```

## 4. 브라우저 경로 확인

WASM 갱신:

```bash
wasm-pack build --target web --dev
```

Headless Chrome에서 rhwp-studio와 같은 `window.__wasm.renderPageToCanvas()` 경로로 렌더:

```text
pageCount = 64
page2 canvas = 793x1122, nonWhite = 64048
page3 canvas = 793x1122, nonWhite = 142190
```

산출물:

```text
output/debug/task1116/browser-canvas/page2.png
output/debug/task1116/browser-canvas/page3.png
```

## 5. 검증

통과:

```bash
cargo test --test issue_1116 -- --nocapture
cargo build --bin rhwp
cargo test --test issue_874_ktx_toc_page_number_right_align -- --nocapture
cargo test --test issue_1086 -- --nocapture
cargo test --test issue_1105 -- --nocapture
cargo test --test issue_630 -- --nocapture
cargo test --features native-skia skia --lib
cargo fmt --all -- --check
git diff --check
wasm-pack build --target web --dev
```

참고:

- `cargo test --features native-skia skia --lib`에서 기존 warning 6개가 출력됐지만 실패는 없다.
- #874 테스트에서 기존 `LAYOUT_OVERFLOW` 진단 1건이 출력됐지만 테스트는 통과했다.

## 6. 남은 판정

이번 단계는 탭 리더가 페이지 번호 뒤로 넘어가는 렌더링 결함을 SVG/Web Canvas/Skia에 함께 반영한 것이다.

작업지시자 3mm 한컴 캡처와의 최종 정합은 별도 판정이 필요하다.

- p2: 목차 전체 x/y, 들여쓰기, 페이지 번호 열 위치.
- p3: 본문 4개 문단과 체크 항목의 누적 높이.
- 브라우저에서 최신 wasm 반영 확인 시 강력 새로고침 또는 새 탭 사용 권장.
