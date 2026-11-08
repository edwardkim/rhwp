# 이슈 #7642 — 실패 회귀 원본의 시각 보정과 관계 검사

## 범위와 선행 결과

- [2단계](task_m100_7642_stage2.md)의 전체 회귀는 10,524건 중 60건 실패로 종료했다.
  실패 입력 HWP 18개·HWPX 6개와 합성 검사 6건을 구분한다. 기존 시험지 16개·수식 진단·
  #1139 파일 전체의 승인된 범위도 유지한다.
- 원본 파일을 바꾸거나 한컴 재저장 대조군을 원본 통과로 보고하지 않는다. 독립 Print의 입력 해시,
  저장 제품에 맞는 engine, 실제 PrintMethod 0 / one-up, 전체 쪽수와 출력 해시를 확인한다.
- code source 기준은 `a08f27f86`이며 `3d3418221`은 절차 문서, `68ac0ddf4`는 수식 dump helper와
  실행 기록이다. 첫 시각 진단은 이 생산 코드의 Native 바이너리와 fresh WASM package를 고정해 수행한다.

## 실행 순서

1. 실패 로그의 최초 assertion 대상과 실제 fixture 선언을 대조해 원본 목록을 확정한다.
   전수 partition에 속했다는 이유만으로 통과한 파일을 실패 목록에 넣지 않는다.
2. 해시·Print 출처가 확인된 동일 원본 PDF를 재사용하고, 자료가 부족한 파일은 MCP Print로 산출한다.
3. 전체 페이지 Native/fresh WASM TSV에서 최저값·90% 미만 쪽·쪽수 불일치를 확인한다.
   실패 쪽과 대표 경계의 review PNG를 직접 열고 사용자에게 표시한다.
4. 미달 입력은 위반 규칙과 측정·배치 소비 경로, 반례를 추적해 실제 출력을 먼저 개선한다.
   그림 세로 정렬·빈 개체 host의 저장 흐름·수식 메트릭·미주 분할 등 실행으로 확인한 원인을 구분한다.
5. 관련 모든 페이지 90% 이상, 전체 쪽수 일치, 수식·간격 확대 판독 뒤 독립 기대값에 따라 필요한
   px 선택·고정 검사를 소유·순서·보존·포함·겹침 관계로 변경한다. 정확한 기존 검사는 구현을 고친다.
6. 보정 전 FAIL / 보정 후 PASS와 정상 대조군을 확인하고, 영향 시각 증적과 필요한 전체 회귀·lint를
   최종 생산 코드로 재실행한다. 미달·미검증 상태에서는 기대값·golden·래칫 허용치를 갱신하지 않는다.

## 증적 경로

- 전체 회귀 로그·입력 대응 TSV: ignored `output/pr-review/issue7642/full-regression-20261008/`.
- 새 입력·Print 실행 근거·Native/fresh WASM 전쪽 TSV·진단: ignored
  `output/pr-review/issue7642/failed-inputs-20261008/`.
- 독립 PDF와 원본은 검증 후 저장소의 안정 경로에 포함하고, TSV·실행 로그·중간 JSON·nextrun은
  output에만 보존한다. 현재 새 PR 번호는 없으며 archive review·오늘할일 trailing 기록은
  self PR의 code head CI 성공 뒤 작성한다.

## p122의 줄 상자와 인쇄 크기 — 구현 전 관측

- 동일 원본의 MCP 2020 Print는 job `78b35e69-f32f-4226-b824-1de2bd905db2`, 3쪽으로 성공했다.
  2쪽 그림의 PDF 상단은 99.13104pt이며 본문 상단에 대응한다. 현재 Native 2쪽은
  56.23886%, 그림 y는 -1737.50244px로 실패했고 같은 review를 직접 표시했다.
- 원본 그림 문단의 저장 LineSeg는 높이 22238HU·기준선 18902HU다. 그림의 줄 구성 크기는
  composer의 `tac_controls`와 `line_breaking::inline_control_size_hwp`가 공통 선언 크기의
  HWPUNIT로 소비한다. 한편 paint의 `resolve_inline_picture_size`는 Paper 상대 크기를
  실제 용지에서 계산한다. 저장 줄의 기준선에서 이 확대된 잉크 높이의 어센트를 빼면서 원점이 바뀐다.
- 보정은 줄 구성에서 사용한 그림 상자 크기를 paint의 기준선 정렬에도 전달하는 것이다.
  인쇄 크기는 기존 상대 용지 크기로 출력한다. 원점 clamp·문서 ID 분기·현재 화면 좌표 고정은 쓰지 않는다.
  생산 지점 → `tac_controls`/줄 나눔의 소비 → 세 그림 배치 호출의 기준선 소비를 연결한다.
- 절대 크기의 국어 CENTER 그림 줄·영어 그림을 대조군으로 사용하고, 독립 Print로 p122 전체
  Native/fresh WASM 3쪽 및 그림 확대를 다시 확인한다. 관계 회귀 변경은 이 확인 뒤 수행한다.

## 실패 원본 전쪽 진단 결과

- 실패 원본 24개 모두에서 올바른 engine의 독립 Print를 확보했다. 13개는 입력·PDF 해시와
  Print 결과 계약을 확인해 재사용했고, 11개는 MCP로 새로 출력했다. Native와 fresh WASM의
  24개 순회 실행은 모두 exit 0으로 종료했다. 개별 Sweep은 미달 gate로 exit 1인 경우를
  포함하며, 쪽수 불일치 때문에 TSV가 없는 입력은 미검증이다.
- 두 경로 모두 `samples/3-10월_교육_통합_2022.hwp`만 18쪽 전체 최저 91.24244%다.
  다른 원본은 90% 미만 또는 쪽수 불일치다. 상세 입력별 결과는 ignored
  `output/pr-review/issue7642/failed-inputs-20261008/failed-input-visual-status.tsv`에 연결한다.
  점수 통과 입력도 수식·간격 확대 판독 전에는 기대값 변경을 승인하지 않는다.

## 상대 크기 그림의 독립 반례

- 원본 p122는 현재 크기와 common 선언값이 우연히 같아 단위 오류를 드러내지 못한다.
  원본을 보존한 채 진단 사본에서 저장 줄을 제거하고 높이를 용지 25%로 설정한 다음,
  MCP 2020으로 정상 저장했다. save job은 `f42dce62-4ce2-4448-a325-ffc58591ea63`이다.
- 한컴 정상 저장본은 common.height=2500(비율), shape_attr.current_height=21047HU,
  LineSeg.line_height/text_height=21047HU, baseline=17890HU로 산출됐다.
  따라서 상대 크기의 줄 구성에 common의 비율을 HWPUNIT으로 사용하는 기존 측정도 잘못됐다.
- 공통 생산자는 절대 크기의 common HWPUNIT과 상대 크기의 current HWPUNIT을 구분한다.
  이를 tac_controls의 너비, inline_control_size_hwp의 너비·높이,
  inline_control_line_height_hwp의 높이와 세 그림 배치 호출이 함께 소비한다.
  같은 helper를 호출한 뒤 별도로 줄 상자 크기를 다시 계산하는 경로도 대조한다.
- 첫 Native 시험 출력에서 p122 전쪽은 100%였고 2쪽 review를 직접 표시했다.
  상대 크기 반례로 생산자를 추가 보정했으므로 이 결과를 최종 검증으로 재사용하지 않는다.
  최종 Native·fresh WASM, 정상 절대 크기 대조군과 후속 문단 흐름을 재검증한다.

## 추가 보정의 Native 결과

- current HWPUNIT 생산자를 적용한 빌드는 exit 0, 5분 28초로 종료했다. 바이너리·5개 생산
  코드 해시는 ignored `picture-current-box-provenance.json`에 고정했다.
- 같은 p122 원본은 3쪽 모두 100%다. 정상 저장한 용지 25% 반례도 affect_line_spacing
  false/true 각각 3쪽 모두 100%다. 상대 값 2500을 줄 크기로 썼던 이전 시험 바이너리는
  false 반례 2쪽이 15.28551%였다. before/after 증거로 단위 오류를 분리했다.
- 정상 절대 크기 영어 8쪽은 최저 95.63340%, 국어 HWP 20쪽은 최저 90.61634%다.
  fresh WASM 빌드·전쪽 대조를 진행 중이며, 이 결과만으로 회귀 검사를 변경하지 않았다.
- 정식 적용 전 관계 검사 초안을 ignored output에서 실행했다. 같은 원본·같은 검사로
  a08 바이너리는 본문 상단과 그림 원점의 비율 관계에서 FAIL(exit 101), 보정 바이너리는
  3개 검사 PASS(exit 0)다. 고정 px 위치·영역 선택 대신 문단/개체 소유와 최종 SVG의
  용지 상대 비율을 확인하며, 저장 기준선의 정수 HWPUNIT 양자화만 무차원 비율로 허용한다.
  로그는 `p122-relation-diagnostic-before.log` / `p122-relation-diagnostic-after.log`다.
- 동일 진단 모델의 다음 문단을 같은 쪽에 놓아 직접 Print한 경우, 두 설정 모두 그림
  범위는 같지만 후속 문단 원점은 달랐다. 수동 모델의 current 크기가 Print 재계산 결과와
  다르므로 정상 저장 반례와 분리하며, 재조판 경로 통과 증거로 사용하지 않는다.

## 다른 실패 원본의 원인 추적

- HWP3 sample16 변환본 첫 문단은 Square 그림을 매단 빈 문단이며 저장 줄 높이 2000HU와
  줄 간격 1200HU, 다음 빈 문단의 vpos 3200HU를 갖는다. `dump-pages`의 해당 FullParagraph
  높이는 0이고, 실제 다음 줄 원점도 저장 줄 높이만큼 앞당겨졌다.
- 측정의 `typeset/paragraph/format.rs::resolve_line_metrics`는 빈 Square 줄을 0으로 만들고,
  paint의 `layout.rs`는 다음 문단이 가시 텍스트일 때만 저장 사다리를 질의한다.
  다음 문단이 실제 빈 줄인 경우를 가시 텍스트와 구분해 공유 소비 계약을 보정해야 한다.
  전체 64쪽의 남은 간격·배치 결함은 아직 해결되지 않았다.
- 임대차 양식은 1쪽 77.53780%, 2쪽 98.93925%, 3쪽 96.77251%다. 3쪽 회귀만 보고
  기대값을 바꾸지 않는다. 1쪽 review의 본문 위치·제목 글꼴 차이를 직접 확인했다.
  full 글꼴 공급 진단도 1쪽 77.63099%로 미달하므로 글꼴 예외로 배치 차이를 면제하지 않는다.

## 그림 상자 보정의 정식 회귀 적용

- 최종 Native/fresh WASM의 동일 입력 6개·57쪽은 모두 쪽수가 일치하고 90% 이상이다.
  p122·상대 크기 false/true는 각각 3쪽 모두 100%, 영어 8쪽은 최저 95.63340%,
  국어 HWP/HWPX 각 20쪽은 최저 90.61634%다. p122의 두 경로 PNG는 전쪽 SHA-256도 같다.
  TSV는 ignored `picture-current-box-wasm/results.json`과 각 Native 결과 경로에 연결한다.
- 현재 Native와 fresh WASM의 p122 review를 각각 생성·판독했고, 국어 CENTER 그림의 5쪽도
  직접 표시했다. 17쪽에는 기존 본문 글꼴·보기 영역의 차이가 남아 있으므로 점수만으로
  해당 문서 전체의 미세 간격 보정 완료를 선언하지 않는다.
- `tests/cases/p122_stored_vpos_page_break.rs`의 기존 px 고정 검사를 소유·정렬·용지 비율
  관계로 바꿨다. 정상 저장한 두 반례의 같은 계약을 검사하는 회귀 1건을 추가했다.
  실제 저장본 HWP와 독립 Print PDF는 `tests/fixtures/issue_7642/`에 함께 보존했다.
- `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review
  --test regression_suite_018 --test-threads 12 --no-fail-fast
  -E 'test(p122_stored_vpos_page_break)'`: exit 0, 4건 PASS, 207건은 필터로 미실행이다.
  prepare 뒤 해당 원본의 파생 suite가 019에서 018로 이동했다. 파생 파일은 커밋하지 않는다.
- 최종 검사 원본을 ignored 진단 바이너리로도 실행했다. 최초 a08은 p122의 원점 관계에서
  FAIL, raw-common 중간 구현은 정상 저장 반례에서 FAIL, 최종 구현은 4건 모두 PASS다.
  최종 소스의 포맷 검사와 base `b3c3047db575d146dff9d39098e6de98c4630b3e` 대비
  manifest 검사는 통과했다. 본 단계의 전체 회귀·필수 lint·PR·CI는 아직 완료되지 않았다.
- 새로 확보한 실패 원본 Print PDF 11개와 출처는 `pdf/issue7642-failed-regression-print/`에,
  대표 최신 PNG는 `mydocs/pr/assets/issue7642_inline_picture_box/`에 보존한다.
  변경된 공통 그림 메트릭의 영향 확인을 위해 실패 원본 24개의 Native/fresh WASM 전쪽
  재대조도 진행한다. 기준 a08의 60건 실패를 최종 head의 결과로 재사용하지 않는다.

## 재조판 경로의 추가 반례 — 보정 필요

- 정상 저장한 두 상대 크기 그림에서 LineSeg만 제거하고, 다음 문단을 같은 쪽의
  `다음문단`으로 바꾼 입력을 따로 Print했다. 한컴 2020 job은 false
  `24e59242-d2c0-4ec3-bc7d-21d118d8793e`, true `55405e59-d0d3-4c21-9157-0afa7b0929cf`다.
  이 입력은 정상 저장된 현재 크기를 보존하며, 앞선 수동 모델의 크기 불일치와 구분한다.
- 독립 Print의 두 입력에는 같은 그림이 보이고, true 설정의 후속 문단은 false보다
  아래에 놓인다. 현재 Native는 두 입력 모두 그림을 출력하지 않고 그림 문단 높이를
  비율값 2500HU로 소비한다. 이후 본문도 너무 일찍 나온다. 저장 경로의 100%를 이
  경로의 통과로 대신하지 않는다. 진단은 ignored `normalized-reflow-geometry/`에 보존한다.
- 실제 호출을 추적하면 `composer::recompose_stored_lines_in_frame_with_known_square_band`
  의 상위 gate가 NO_LS 글자처럼 그림만 있는 문단을 사양한다. 아래 frame은 지원하는
  경로인데 진입하지 못해 빈 줄 폴백이 그림을 누락한다. 또한
  `renderer::tac_object_flow_height_hu`는 아직 비율값을 높이로 읽으며,
  `affect_line_spacing`은 parser/serializer만 보존하고 renderer는 소비하지 않는다.
  해당 실제 호출 경로와 개체의 줄 소유·간격 생산을 추가 보정해야 한다.
- 실패 원본 재대조 중 Native failed-14는 쪽수가 이전과 같은 21쪽이다. Chrome의
  `/dev/null` 열기 실패로 TSV를 만들지 못한 환경 오류를 배치 회귀와 구분하고,
  순회가 끝난 뒤 해당 실행을 재산출한다. 미측정을 90% 통과로 처리하지 않는다.
