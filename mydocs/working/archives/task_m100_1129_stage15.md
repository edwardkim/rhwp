# Task #1129 Stage 15 - 쪽 클립 위치와 페이지 외곽 상하 여백 보정

- 이슈: [#1129](https://github.com/edwardkim/rhwp/issues/1129)
- 브랜치: `local/task_m100_1129`
- 일자: 2026-05-26

## 배경

Stage 14 이후 사용자 수동 비교에서 다음 문제가 남았다.

- 쪽 클립 위치가 한컴오피스와 다르다.
- 페이지 외곽선 기준 상단/하단 여백이 한컴오피스와 다르다.

첨부 비교 순서:

1. rhwp-studio
2. 한컴오피스

## 판단

Stage 14 자동 검증에서 `PageInfo.pageBorderTop/Bottom`은 대칭이었다.

- top: `20.955020788711096px`
- bottom: `20.955020788711096px`

그러나 실제 화면 비교에서는 페이지 테두리 위치가 한컴오피스와 다르다. 따라서 grid overlay만의 문제가 아니라 WASM 렌더의 페이지 테두리 기준과 `PageInfo`의 기준이 한컴과 맞는지 확인해야 한다.

## 검토 대상

- `src/document_core/queries/rendering.rs`의 `get_page_info_native()`
- `src/renderer/layout.rs`의 page border 렌더 위치 계산
- `PageBorderFill`의 `basis`, `spacing_top`, `spacing_bottom` 해석

## 수정 방향

- page border 렌더와 `PageInfo.pageBorder*`가 같은 좌표계를 쓰도록 맞춘다.
- 한컴오피스 기준과 다르게 상단이 좁고 하단이 넓어지는 원인을 실제 렌더 위치 계산에서 찾는다.
- grid overlay는 page border 기준을 따라가므로, WASM 기준을 고친 뒤 overlay 값을 다시 검증한다.

## 원인

`build_page_borders()`는 쪽 테두리를 실제로 그릴 때 `footer_inside`가 꺼져 있으면 하단선을 `body_area` 끝으로 올려서 그린다.

```rust
if !footer_inside {
    let footer_top = layout.body_area.y + layout.body_area.height;
    if by + bh > footer_top {
        bh = footer_top - by;
    }
}
```

하지만 `getPageInfo()`는 `pageBorderBottom`을 `PageBorderFill.spacing_bottom` 그대로 반환했다.

그 결과:

- 실제 렌더된 페이지 하단 외곽선은 위쪽에 있음
- grid overlay와 쪽 클립은 더 아래쪽 `spacing_bottom` 기준을 사용함
- 한컴오피스 대비 쪽 클립 위치와 외곽선 하단 여백이 어긋나 보임

## 수정 내용

- `src/document_core/queries/rendering.rs`
  - `get_page_info_native()`에서 `footer_inside`가 꺼진 경우 실제 렌더 하단선과 같은 `body_area` 끝 기준을 `pageBorderBottom`에 반영한다.
  - `PageInfo.pageBorderBottom`이 grid overlay/쪽 클립에서 실제 렌더된 외곽선 하단을 따르도록 맞췄다.

## 검증 계획

- Rust/Playwright로 `samples/hwp3-sample16-hwp5.hwp` 첫 페이지 `PageInfo` 기록
- 로컬 Playwright 기능 검증
  - `쪽/3mm/0,0`
  - overlay CSS와 page border 기준 기록
- `npm run build`
- `wasm-pack build --target web --out-dir pkg`
- `cargo fmt --all -- --check && git diff --check`

## 검증 결과

- `wasm-pack build --target web --out-dir pkg` 통과
  - 추적 산출물 변경 없음
- 로컬 Playwright 기능 검증 통과
  - 샘플: `samples/hwp3-sample16-hwp5.hwp`
  - 모달: `그대로 보기`
  - 설정: `쪽/3mm/0,0`
  - `PageInfo.pageBorderBottom`: `75.6`
  - `pageBorderTop`: `18.9`
  - overlay 수: `2`
  - clip corner overlay 수: `2`
  - `background-size`: `12.5714px 12.5714px`
  - `background-position`: `20.955px 20.955px`
  - `clip-path`: `inset(21.955px 21.955px 84.8201px)`
  - page border CSS px:
    - top: `20.955020788711096`
    - bottom: `83.82008315484438`
  - bottomY: `1266.1799168451557`
- `npm run build` 통과
  - 기존 큰 chunk 경고만 표시됨
- `cargo fmt --all -- --check && git diff --check` 통과
- `cargo test --lib` 통과
  - `1397 passed; 0 failed; 6 ignored`

## 대기

자동 검증 후 커밋하고, 최종 정합 여부는 작업지시자의 수동 비교를 기다린다.
