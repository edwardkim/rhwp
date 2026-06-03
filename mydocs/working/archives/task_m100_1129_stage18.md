# Task #1129 Stage 18 - 쪽 테두리 기준 샘플 재판정

## 배경

작업지시자가 HWP5 기준 샘플 두 개를 추가했다.

- `samples/종이기준.hwp`
- `samples/쪽기준.hwp`

`hwp3-sample16-hwp5.hwp`도 한컴오피스 기준으로는 쪽 기준으로 저장되어 있는데, rhwp-studio 대화상자는 종이 기준으로 표시했다.

## 조사 결과

`hwp5-inventory`로 첫 번째 `PAGE_BORDER_FILL` raw 값을 확인했다.

| 파일 | 첫 `PAGE_BORDER_FILL.attr` | HWPX `textBorder` | 한컴 UI 기준 |
|------|----------------------------|-------------------|--------------|
| `samples/종이기준.hwp` | `0x00000000` | `CONTENT` | 종이 기준 |
| `samples/쪽기준.hwp` | `0x00000001` | `PAPER` | 쪽 기준 |
| `samples/hwp3-sample16-hwp5.hwp` | `0x00000001` | - | 쪽 기준 |

현재 rhwp는 `attr bit0=1`을 `PageBorderBasis::PaperBased`로 해석해 UI/API에 `basis:"paper"`로 노출한다. 그래서 기준 샘플과 반대로 표시된다.

## 결론

한컴 스펙 문서의 `bit0 0=본문 기준, 1=종이 기준` 설명을 한컴오피스 UI의 `종이 기준/쪽 기준` 명칭으로 그대로 옮기면 틀린 결과가 나온다.

이번 단계에서는 저장 포맷 기준값과 렌더러의 외곽선 배치 계약을 분리한다.

- HWP5/HWPX 저장값 `attr bit0=0`, `textBorder=CONTENT` → UI `종이 기준`
- HWP5/HWPX 저장값 `attr bit0=1`, `textBorder=PAPER` → UI `쪽 기준`
- 렌더러 외곽선 paper-edge 정합은 별도 계약으로 유지한다.

## 구현 계획

1. `PageBorderFill`에 대화상자/저장 기준을 표현하는 별도 필드를 추가한다.
2. HWP5/HWPX 파서는 기준 샘플에 맞춰 UI 기준 필드를 채운다.
3. `getPageBorderFill`/`setPageBorderFill`은 UI 기준 필드를 사용한다.
4. 기존 외곽선 렌더링의 `basis` 계약은 유지해 시각 회귀를 줄인다.
5. 기준 샘플과 `hwp3-sample16-hwp5.hwp`의 API 반환값을 테스트로 고정한다.

## 검증 계획

- `cargo test page_border_fill_sample_basis_matches_hancom_ui --lib`
- `wasm-pack build --target web --out-dir pkg`
- `npm run build`
- 로컬 Playwright 기능 검증
  - `samples/종이기준.hwp` → `getPageBorderFill(0).basis === "paper"`
  - `samples/쪽기준.hwp` → `getPageBorderFill(0).basis === "page"`
  - `samples/hwp3-sample16-hwp5.hwp` → `getPageBorderFill(0).basis === "page"`
- `cargo test --lib`
- `cargo fmt --all -- --check && git diff --check`
