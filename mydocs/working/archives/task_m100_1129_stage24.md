# Task m100 #1129 Stage 24

## 문제

Stage 23 이후에도 `hwp3-sample16-hwp5.hwp` 첫 페이지 상단에서 한컴오피스와 rhwp-studio 표시가 다르다.

- 쪽 기준 외곽선과 좌상단 로고 사이의 시각적 간격이 아직 다름
- 3mm 격자 표시가 한컴오피스와 같은 개수/범위인지 별도 계측 필요

## 확인 방향

- 같은 샘플의 저장 좌표, 쪽 테두리/배경 설정, 본문 영역, 렌더 SVG 좌표를 다시 비교한다.
- 3mm 격자는 쪽 클립 기준 표시 영역의 가로/세로 점 개수를 산출해 한컴 기준과 같은지 확인한다.
- 원인이 외곽선 렌더 위치인지, grid overlay 기준/clip인지, 객체 좌표 보정인지 분리한다.

## 확인 결과

- `hwp3-sample16-hwp5.hwp` 1페이지 저장 값
  - 용지: `210mm x 297mm`
  - 여백: 좌/우 `15mm`, 상/하 `10mm`, 머리말/꼬리말 `10mm`
  - 쪽 클립 영역: 가로 `180mm`, 세로 `257mm`
- HWP3 원본 포맷에는 HWP5/HWPX처럼 `종이 기준`/`쪽 기준` 선택 필드가 없다.
  - HWP3 문서 정보의 `테두리 간격`은 "쪽 테두리와 본문 간격"이다.
  - 따라서 HWP3 파서는 page border를 항상 `쪽 기준(Page/BodyBased)`으로 정규화해야 한다.
- 3mm 점 격자 개수
  - 한컴처럼 쪽 클립 시작점부터 점 중심을 두면 가로 `61개` (`0, 3, ..., 180mm`), 세로 `86개`
  - 기존 rhwp-studio CSS는 `radial-gradient` 기본 중심점 때문에 첫 점이 기준점보다 `1.5mm` 안쪽에서 시작해 가로 `60개`, 세로 `86개`가 됨

## 수정

- HWP3 page border 정규화를 `hwp3_page_border_fill()` 함수로 분리하고 항상 `Page/BodyBased`를 반환하도록 수정한다.
- 샘플 파일명을 직접 박은 테스트 대신 `hwp3_page_border_fill()` 단위 테스트로 HWP3 계약을 검증한다.
- 점 격자(`dots`)일 때만 background position을 반 칸(`1.5mm`) 바깥으로 보정한다.
- 가로선/세로선/가로세로선 격자는 선 자체가 타일 시작점에 있으므로 기존 기준을 유지한다.

## 검증 계획

- `samples/hwp3-sample16-hwp5.hwp` 1페이지 SVG 재생성
- SVG 좌표에서 로고 bbox, 쪽 기준 외곽선, 쪽 클립 코너 좌표 확인
- rhwp-studio grid overlay의 3mm spacing과 clip 영역 기준 점 개수 산출
- 필요한 경우 코드 수정 후:
  - `cargo test test_hwp3_page_border_fill_is_always_page_basis --lib`
  - `cargo test page_border_fill_sample_basis_matches_hancom_ui --lib`
  - `wasm-pack build --target web --out-dir pkg`
  - `npm run build`
  - `cargo fmt --all -- --check`
  - `git diff --check`

## 검증 결과

- `samples/hwp3-sample16-hwp5.hwp` 기준 3mm 점 격자 산출
  - 수정 전: 가로 `60개`, 세로 `86개`
  - 수정 후: 가로 `61개`, 세로 `86개`
- `RHWP_DEBUG_PAGE_BORDER=1 cargo run --quiet --bin rhwp -- export-svg samples/hwp3-sample16.hwp -p 0 -o output/poc/task1129_stage24_hwp3`
  - `paper_based=false`, `attr=0x00000001`, `spacing(L=1420,R=1420,T=1420,B=1420)` 확인
- `cargo test test_hwp3_page_border_fill_is_always_page_basis --lib`: 통과
- 샘플 파일명을 직접 박은 `samples/hwp3-sample16.hwp` assert는 제거하고, HWP3 page border 정규화 함수 단위 테스트로 대체
- `npm run build` (`rhwp-studio`): 통과
- `wasm-pack build --target web --out-dir pkg`: 통과
- `cargo test page_border_fill_sample_basis_matches_hancom_ui --lib`: 통과
- `cargo fmt --all -- --check`: 통과
- `git diff --check`: 통과
