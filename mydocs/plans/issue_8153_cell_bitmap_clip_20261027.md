# #8153: 셀 배경의 물리 Print clip과 본문 clip 소유

## 원본과 독립 기대값

원본 두 편람과 기존 독립 2024 Print를 유지한다. 291·301쪽의 표 바탕 image351과
Q 셀 image352는 HWPX BorderFill 717/718의 TOTAL 그림 채움이다. 셀의 본문
소유/잘림과 그림 채움의 원본 창을 구분하며 HWP5 표·셀/BorderFill 속성을 유지한다.
PDF의 이미지를 원본 대용으로 넣거나 원본 clip·그림을 바꾸지 않는다.

source b3ad의 실제 두 그림 자원은 독립 PDF xref2137/2138과 각각 **RGB 전픽셀이 같다**.
이미지 paint 원점 y=200.7804605px는 독립 PDF 200.780009px와 맞고, 원래 물리
bitmap clip y=200.940445px도 유지된다. 그런데 Q 셀에는 상위 논리 TableCell
clip y=201.066667px가 더 있어 그림 배경의 첫 장치 행을 잘라 내는 경계가 있다.
이 값들은 원인 관측이며 제품에 넣을 위치 상수가 아니다.

## 실제 원인 분리와 공통 소비

두 원본의 291·301쪽, 네 조건의 수정 전 재생 RGB가 실제 Native source b3ad와
전픽셀 같음을 먼저 확인했다. 직접 셀 배경의 PrintBitmapFill 자식만 기존 내용
clip 밖에 놓고, 그림 자체의 물리 clip·입력 바이트·프레임·본문 clip은 그대로 두었다.
그 재생에서 네 조건 모두 미대응 20 → 0픽셀, raw 100%다. 제품 수정이나
fresh WASM 통과의 증거로 확대하지 않는다.

실제 소비는 `render_cell_background의 원본 배경 역할 → TableFillOrigin →
PagePaintContext.bitmap_fill_frame의 그림/물리 clip → build_node(TableCell)의
논리 내용 clip → Native/Canvas replay`다. 같은 프레임 helper를 거쳐도 뒤의
다른 clip이 덮어쓰면 결과 공유가 끝난 것이 아니므로 이 마지막 경계를 수정한다.
모든 셀 clip을 제거하거나 Native·Canvas에서 따로 좌표를 고치지 않는다.

## 기존 계약과 다음 검증

셀 wrapper 원형은 `6f973d6076e`의 layered paint tree이며, 명시적 ClipKind는
`55e406d723b`의 Task #1197 paint order 변경이다. 물리 PrintBitmapFill clip의
현행 도입은 `2d072afca0`이다. 실제 변경 코드와 관련 단계/검토 기록을 대조하고,
기존 내용 overflow·중첩 셀·분할 쪽 컷의 보호 목적을 확인한 뒤 구현한다.
특히 분할 viewport는 본문 내용 상자와 다른 물리 컷이므로 같은 배경 우회로
가정하지 않는다. 아직 정식 제품 코드는 변경하지 않았다.

공통 paint builder에서 원본 배경 역할과 자체 물리 clip을 확인해 두 소유를 분리하고,
내용 clip·source ID·내용 순서·누락/중복·기존 paint order는 유지한다. 작은 구조
계약과 실제 네 쪽의 before FAIL/after PASS, 주변 정상 배경과 분할/중첩 반례를
먼저 검증한다. 실제 Native/fresh WASM 출력과 전쪽 TSV·확대 판독을 마친 후
정식 `tests/cases/issue_8153_*.rs` 회귀에 포함한다. 파일별 제출 TSV와 대표 PNG,
최종 표준 최적화 WASM·전체 회귀/lint/Skia/Studio와 PR 준비는 완료되지 않았다.

ignored 증거는 `remaining-raw100-diagnostic/continuous-text-shadow/
p289-301-source-contract/`의 `actual-top-image-paint.log`,
`p291-actual-independent-image-comparison.json`, `p291-301-independent-top-images.json`,
`cell-background-replay/comparison.json`, `cell-background-replay-hwp/comparison.json`이다.
원본·PDF·폰트·측정 기준 변경, upstream 동기화와 원격 변경은 하지 않았다.
