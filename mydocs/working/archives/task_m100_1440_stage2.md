# Task 1440 Stage 2 - 온새미로 6쪽 문항 박스 시각 차이 분석

## 목적

- `samples/[2027] 온새미로 1 본교재.hwp` 6쪽에서 한컴 PDF와 rhwp 렌더가 다르게 보이는 원인을 찾는다.
- 35쪽 그림 어울림 보정이 6쪽 지문 박스 문단의 LineSeg 처리에 잘못 전파되는지 확인한다.

## 입력 자료

- HWP: `samples/[2027] 온새미로 1 본교재.hwp`
- HWPX: `samples/[2027] 온새미로 1 본교재.hwpx`
- 한컴 PDF: `pdf/[2027] 온새미로 1 본교재-2024.pdf`

## 진행 계획

1. 한컴 PDF 6쪽과 rhwp HWP/HWPX 6쪽 SVG/PNG를 생성한다.
2. 6쪽 render tree에서 지문 박스 후보의 `RenderNodeType`, bbox, 선 스타일을 확인한다.
3. 원문 컨트롤/스타일에서 박스가 표, 도형, 문단 테두리, 글상자 중 무엇인지 추적한다.
4. 렌더 차이가 파서/모델/렌더 중 어디에서 생기는지 기록하고, 최소 수정 지점을 정한다.

## 분석 결과

- 6쪽 지문은 별도 표/도형이 아니라 문단 테두리 문단이다.
- HWP/HWPX 모두 문단 1.32의 ParaShape가 `margin_left=4000`, `indent=2000`, `border_fill_id=25`, `border_spacing=[850,850,850,850]`이고, LineSeg는 `cs=2000`, `sw=35468`이다.
- Stage 1에서 추가한 `precomputed_body_wrap_line`이 이 문단에도 적용되어 `effective_col_x += LineSeg.cs`가 한 번 더 들어갔다.
- 그 결과 첫 줄 x가 약 `255.6px`, 둘째 줄 x가 약 `242.3px`로 밀렸고, PDF bbox 기준 기대 위치인 약 `229px`/`216px`보다 오른쪽에 그려졌다.
- 35쪽 대상 문단 3.8은 앞 7줄이 `cs=850`, `sw=20999`, 뒤쪽 줄이 `cs=850`, `sw=36568`로 실제 그림 어울림 wrap zone과 일반 흐름 폭이 섞여 있어 계속 Stage 1 보정 대상이어야 한다.
- #604 문서의 기존 분석처럼 LineSeg `cs/sw` 단독 판정은 #547 passage box도 wrap zone으로 오인할 수 있으므로, anchor 메타데이터가 없는 fallback 보정은 같은 문단 안에서 좁은 줄과 넓은 줄이 섞인 precomputed picture-wrap 흐름으로 제한해야 한다.
- 한컴 설명서 기준 `문단 테두리 연결`은 두 개 이상의 문단에 대해 현재 문단과 이어지는 다음 문단들을 하나의 문단 테두리로 연결하는 설정이다.
- 파일 형식 표 44의 `ParaShape.attr1 bit 28`은 `문단 테두리 연결`, `bit 29`는 `문단 여백 무시`로 확인된다.
- HWPX 원본의 6쪽 지문 박스 ParaShape는 `<hh:border connect="1" ignoreMargin="1">`를 가진다. 기존 rhwp는 내부 attr1 값은 보존했지만 Studio 문단 모양 JSON/다이얼로그/수정 명령에서 이 값을 노출하지 않았고, HWPX 저장 시 `connect="0" ignoreMargin="0"`으로 고정 출력했다.
- 6쪽 지문 박스 테두리는 실선이 아니라 점선/파선 계열이다. 기존 렌더러는 네 면의 stroke가 같으면 `Rectangle` 하나로 최적화하면서 dash 정보를 잃어 실선처럼 출력했다.

## 수정

- `src/renderer/layout/paragraph_layout.rs`
  - `precomputed_body_wrap_line` 적용 조건에 문단 내부 LineSeg 폭 혼합 검사를 추가했다.
  - 모든 줄이 동일한 `column_start/segment_width` 패턴인 문단 테두리 박스는 anchor 없는 body wrap fallback 보정에서 제외했다.
  - 기존 #547 passage 박스 역시 LineSeg `cs/sw`만 보면 wrap zone으로 보이는 false-positive 케이스라 함께 회귀 차단 대상에 포함했다.
- `src/renderer/layout.rs`
  - 문단 테두리가 모두 실선일 때만 `Rectangle` stroke 최적화를 사용하도록 제한했다.
  - 점선/파선 문단 테두리는 면별 `LineNode`로 렌더해 `stroke-dasharray`가 SVG에 남도록 했다.
- `src/document_core/commands/formatting.rs`, `src/document_core/helpers.rs`, `src/model/style.rs`, `rhwp-studio/src/ui/para-shape-dialog.ts`
  - 문단 모양 속성 JSON과 Studio 다이얼로그에 `borderConnect`, `borderIgnoreMargin`을 연결했다.
- `src/serializer/hwpx/header.rs`
  - HWPX 저장 시 `<hh:border connect ignoreMargin>` 값을 ParaShape attr1 bit 28/29에서 출력하도록 수정했다.
- `mydocs/tech/한글문서파일형식_5.0_revision1.3.md`, `mydocs/tech/hwp_spec_errata.md`
  - 표 44의 bit 28 설명에 한컴 설명서의 `문단 테두리 연결` 의미를 보강했다.
  - HWPX `<hh:border connect ignoreMargin>`와 ParaShape attr1 bit 28/29 대응을 기록했다.

## 회귀 테스트

- `tests/issue_1440_onsamiro_picture_wrap.rs`
  - 기존 35쪽 그림 어울림 침범 방지 테스트 유지.
  - 6쪽 지문 박스 문단의 첫 줄/둘째 줄 x 좌표가 LineSeg `column_start` 이중 적용 위치로 밀리지 않는 테스트 추가.
  - 6쪽 지문 박스의 `문단 테두리 연결` bit 28 보존과 비실선 테두리 dash 렌더 보존 테스트 추가.

## 검증

- `cargo fmt --check` 통과.
- `cargo test --test issue_1440_onsamiro_picture_wrap` 통과. (4 tests)
- `cargo build --bin rhwp` 통과.
- 갱신 render tree 기준 6쪽 지문 문단 1.32:
  - 첫 줄 x=`229.0`
  - 둘째 줄 x=`215.6`
- 갱신 SVG 기준 6쪽 지문 박스 테두리:
  - HWP: `stroke-dasharray="2 2"`
  - HWPX: `stroke-dasharray="6 3"` (`type="DASH"` 원본 반영)

## 산출물

- 한컴 PDF raster: `mydocs/report/assets/task_m100_1440/stage2/hancom_pdf/page-06.png`
- rhwp HWP render: `mydocs/report/assets/task_m100_1440/stage2/rhwp_hwp_p06.png`
- rhwp HWPX render: `mydocs/report/assets/task_m100_1440/stage2/rhwp_hwpx_p06.png`
- rhwp SVG:
  - `mydocs/report/assets/task_m100_1440/stage2/rhwp_svg/hwp/[2027] 온새미로 1 본교재_006.svg`
  - `mydocs/report/assets/task_m100_1440/stage2/rhwp_svg/hwpx/[2027] 온새미로 1 본교재_006.svg`
