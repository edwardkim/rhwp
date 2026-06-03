# Task #1129 Stage 16 - 쪽 외곽선과 격자 기준 좌표 일치

- 이슈: [#1129](https://github.com/edwardkim/rhwp/issues/1129)
- 브랜치: `local/task_m100_1129`
- 일자: 2026-05-26

## 배경

Stage 15 이후 사용자 수동 비교에서 다음 문제가 확인됐다.

- 페이지 상단 여백이 좁고 하단 여백이 넓다.
- 쪽 클립 위치가 정확하지 않다.
- 격자가 쪽 클립부터 시작해야 하는데 외곽선부터 시작한다.

첨부 비교 순서:

1. rhwp-studio
2. 한컴오피스

## 판단

Stage 15는 `getPageInfo().pageBorderBottom`을 실제 렌더 하단 기준으로 바꾸었다. 이 접근은 절반만 맞았다.

진짜 문제는 세 가지였다.

1. `src/renderer/layout.rs`는 `footer_inside=false`일 때 쪽 외곽선 하단을 본문 하단으로 잘라서 그렸다.
   - 결과: 상단 외곽선은 종이 기준 5mm 근처, 하단 외곽선은 본문 하단 근처가 되어 rhwp-studio에서 상단이 좁고 하단이 넓어졌다.
2. `getPageInfo().pageBorder*`의 `BodyBased` 계산이 렌더러 공식과 달랐다.
   - 렌더러는 body 기준일 때 본문에서 spacing을 바깥쪽으로 뺀다.
   - `getPageInfo()`는 spacing을 더하는 방향이었다.
3. HWP5/HWPX 파서가 문서의 쪽 테두리 기준값을 무시하고 항상 `PaperBased`로 강제했다.
   - HWP5 spec 표 136: `PAGE_BORDER_FILL.attr bit0`은 `0=본문 기준`, `1=종이 기준`.
   - HWPX 대응: `hp:pageBorderFill@textBorder`는 `CONTENT=본문 기준`, `PAPER=종이 기준`.
   - 한컴 UI의 `쪽 기준/종이 기준`은 이 쪽 테두리/배경 문맥의 기준이며, 일반 개체 위치의 `Page/Paper` 기준과 섞으면 안 된다.

따라서 한컴오피스 기준에 맞추려면 문서의 쪽 테두리 기준값을 보존하고, 쪽 외곽선 렌더링 공식과 `getPageInfo().pageBorder*` 공식이 같아야 한다. 단, 격자 보기의 `쪽` 기준은 쪽 테두리/배경 설정이 아니라 한컴 격자 설정의 쪽 클립 영역이므로 별도 좌표를 사용해야 한다.

## 수정 방향

- `src/renderer/layout.rs`
  - `footer_inside=false`일 때 하단 외곽선을 본문 하단으로 자르는 clip 제거.
  - 종이 기준 외곽선은 상단/하단 모두 `PageBorderFill.spacing` 기준을 유지.
- `src/document_core/queries/rendering.rs`
  - `getPageInfo().pageBorder*`를 렌더러와 같은 공식으로 계산.
  - `PaperBased`: 종이 가장자리에서 spacing만큼 안쪽.
  - `BodyBased`: 본문 영역에서 spacing만큼 바깥쪽.
- `src/parser/body_text.rs`
  - HWP5 `PAGE_BORDER_FILL.attr bit0`을 `PageBorderBasis`에 반영.
- `src/parser/hwpx/section.rs`
  - HWPX `pageBorderFill@textBorder`를 `PageBorderBasis`에 반영.
- `rhwp-studio/src/view/grid-overlay.ts`
  - `쪽` 기준 격자/쪽클립 overlay는 `pageBorder*`가 아니라 쪽 클립 영역을 사용.
  - 좌/우: `marginLeft`, `marginRight`
  - 상/하: `marginTop + marginHeader`, `marginBottom + marginFooter`
  - `clip-path`의 임의 1px padding 제거.

## 검증 계획

- 로컬 Playwright 기능 검증
  - `samples/hwp3-sample16-hwp5.hwp` 로드
  - `쪽/3mm/0,0` 설정
  - overlay CSS와 clip corner overlay 위치 기록
  - `PageInfo.pageBorderTop`과 `PageInfo.pageBorderBottom`이 같은지 확인
  - grid overlay가 `pageBorder*`가 아닌 쪽 클립 영역에서 시작하는지 확인
- `npm run build`
- `wasm-pack build --target web --out-dir pkg`
- `cargo fmt --all -- --check && git diff --check`
- `cargo test test_parse_page_border_fill -- --nocapture`
- `cargo test --lib`

## 검증 결과

- `cargo fmt --all -- --check && git diff --check`: 통과
- `wasm-pack build --target web --out-dir pkg`: 통과
- 로컬 Playwright 기능 검증: 통과
  - 샘플: `samples/hwp3-sample16-hwp5.hwp`
  - 페이지 수: 64
  - zoom: `1.1087312586619629`
  - `PageInfo.pageBorderTop`: `18.9`
  - `PageInfo.pageBorderBottom`: `18.9`
  - 쪽 클립 예상 inset
    - 좌: `56.7 * zoom = 62.8651px`
    - 상: `(37.8 + 37.8) * zoom = 83.8201px`
    - 하: `(37.8 + 37.8) * zoom = 83.8201px`
  - overlay `backgroundPosition`: `62.8651px 83.8201px`
  - overlay `clipPath`: `inset(83.8201px 62.8651px)`
  - 3mm 격자 `backgroundSize`: `12.5714px 12.5714px`
- `cargo test test_parse_page_border_fill -- --nocapture`: 통과
  - HWP5 `attr bit0=1` -> `PaperBased`
  - HWP5 `attr bit0=0` -> `BodyBased`
  - HWPX `textBorder=PAPER` -> `PaperBased`
  - HWPX `textBorder=CONTENT` -> `BodyBased`
- `npm run build`: 통과
  - 기존 chunk size warning만 발생.

## 대기

자동 검증 후 커밋하고, 최종 정합 여부는 작업지시자의 수동 비교를 기다린다.
