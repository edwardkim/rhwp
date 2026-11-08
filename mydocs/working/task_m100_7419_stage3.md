# #7419 Stage 3 — 전체 검증과 제출 판정

## 전체 회귀에서 검출한 Native 소유권 회귀의 보정

`fafd0d378` 전체 회귀에서 가상융합 시행령(`issue6776/78494…hwpx`)의 셀 쪽 밖2줄,
off-canvas2건과 `maintainer_nested_table_lines::missing_stored_rows_do_not_assume_one_inline_row`의
Native NO_LS 대조군 실패를 검출했다. 확장자는 hwpx지만 이 입력의 실제 magic은 HWP5 CFB다.
동일 입력의 고정 devel4662 CLI는74쪽/셀 쪽 밖0, 후보는75쪽/쪽 밖2다. 현재28·46쪽의
pi248·442가 각각18.6/10.5px 용지 밖으로 나갔다. 기존 이슈 이관이나 기대값 변경으로 숨기지 않는다.

원인은 셀 전용 측정 투영과 별개로 일반 문단 프레임의 NO_LS 빈 host/다중 TAC 수용도 넓힌 것이다.
일반/Native의 기존 `보이는 글자+단일 TAC` 수용은 유지하고, 현대 NO_LS 셀의 측정 소유자가
실측 높이 목록을 전달했을 때만 빈 host/다중 TAC 줄과 프레임 좌표를 발행하도록 고친다.
생산한 프레임에 실측 표 높이를 함께 넣어 측정·셀 유닛·whole/partial paint가 같은 결과를 읽는다.
기존 정상 행 소유·내용 보존 검사는 그대로 두고, 실패3개와 신규12개를 먼저 다시 실행한다.
진단 명령·전후 원시 로그는 `output/pr-review/pr7433-current-20261016/overflow6776-*`,
원인 기록은 같은 경로의 `regression-cause-and-fix.md`에 보존한다. 보정 후 결과는 아직 미검증이다.

제품 코드 `a08cf5cc6`, 관계 회귀 `fd8ed2451`.
아래 추가 보정 전의 기록은 Stage1·Stage2에 있다. 이 단계에서는 분할 경로 대조로 제품 코드도 추가 보정했으므로 최종 source의 시각·전체 게이트를 다시 수행한다.

## 원본 코퍼스 등록

원 PR에서 새로 넣은 `samples/issue7419/`의2개 원본은 독립 Print1쪽과 일치하며 전쪽 양쪽100%다.
이2개만 `oracle_page_count_baseline.tsv`에1/1로 신규 등록했다. 기존 허용치를 변경하지 않는다.
원본 XML은 수동 작성이며 `lastSavedWith` 표기는2024지만 실제 한컴 생성 출처로 인정하지 않는다.
페이지 기대값은 해당 바이트에 대한 새 한컴2024 Print에서 정했다. 입력·PDF SHA는 Stage1에 있다.
추가 반례7개는 `tests/fixtures/`의 목적별 입력이며 Stage2의 직접 내용·쪽 소유 검사가 담당한다.

`layout-anomaly --json`의 기본 네 방향 overflow는 중첩 원본에서 base/head 모두1이다.
이는 오른쪽4.7733px이며 본문 바닥(`body_overflow_baseline`)의 집계 대상이 아니다. 다른 래칫의
기존 수치를 이 값으로 추정하거나 허용치를 추가하지 않았다. 전체 회귀의 실제 결과로 확인한다.

## 실행 중인 필수 검증

`output/pr-review/pr7433-current-20261016/validate.py`에서 Cargo 잡8개, 고정 공용 target으로
fmt·native/WASM clippy·workspace build/all-target clippy·suite manifest, release-test 전체,
Native Skia3종을 순서대로 실행한다. Cargo 계열 명령은 겹치지 않는다. 명령·head·exit·시간은
`{lint,full,native}-validation.json`, 원시 결과는 각 이름의 log에 남긴다.

시작 전 원격 devel도 다시 확인했고 `375aff29498166260a22796d54e67f508c84aada`로 변경이 없었다.
따라서 불필요한 rebase는 수행하지 않았다. PR 직전에는 다시 확인한다.

전체 회귀·lint·Native Skia의 완료 결과와 PR gate 판정은 실제 종료 뒤 추가한다.

## 분할 경로 대조에서 추가 보정한 범위

Stage1은 원본·선언 높이·TAC 전체 이월을 확인한 시점의 기록이다. 후속 코드 대조에서 분할 셀의
`CellComposedStore` eager/lazy는 다른 조판 호출을 사용하고 있었으므로, 셀 컷과 실제 조각 배치가
`compose_cell_unit_paragraph`를 함께 사용하도록 바꿨다. 다음은 같은 입력의 직접 반례다.

| 입력 | 독립 Print 계약 | 보호할 경계 |
| --- | --- | --- |
| `partial-nested-width-wrap.hwpx` | 2쪽. 첫 자식은1쪽, 둘째는2쪽, 후속 본문은2쪽 | 줄 소속0/8 UTF-16, 시작·끝 컷, 실제 수용 높이와 내용 순서 |
| `partial-nested-wrapper.hwpx` | 2쪽. 부모 제목은1쪽, wrapper 전체와 두 자식·뒤 본문은2쪽 | 한 쪽에 들어가는 TAC wrapper의 통째 이월, 여러 자식 보존·종료 |

두 입력은 폭 부족 원본에서 용지45000HU, 부모 표를 비TAC로 바꾸고 후속 본문을 붙였다.
둘째는 그 두 자식을1행1열 wrapper에 넣었다. 표·문단 id는 충돌하지 않게 바꾸고 실제 wrapper
셀 주소를(row0,col0)으로 확인했다. 초기 ZIP 제작 오류 및 rowAddr1인 잘못된 초안은
`invalid-input-proof/`에 분리했고 검증 근거에서 제외했다. 잘못된 초안 변환 job2개는 failed로
끝났으며 정정 입력의 새 Print job `cd8d7b6f-d27e-48fd-a7cd-91d0d7da19dd`는 succeeded였다.
서버 endpoint/token은 출력하지 않았다.

### 현재 결과의 생산과 소비

- `CellComposedStore`의 양쪽 구성이 공통 셀 조판을 호출한다. 원래 문단과 분할 줄 창은 보존한다.
- 여러 TAC를 소유한 현재 프레임의 각 물리 줄이 `CellUnit.vis_start/vis_end`로 전달된다.
  같은 줄의 두 표 높이를 합산하지 않고 다른 줄의 표를 하나의 atomic 합으로 바꾸지 않는다.
  `advance_row_cut_for_physical_band`의 실제 수용 유닛과 원래 줄 창을 paint가 함께 소비한다.
- 부모의 단일 TAC wrapper는 독립 Print처럼 통째 이월한다. 내부 두 줄이 존재한다는 이유로
  부모를 분할 가능으로 승격하지 않는다. 그 가정을 실제 Print에 따라 제거했다.
- `ComposedParagraph::control_line_indices`가 저장 줄 또는 현재 프레임의 원시 슬롯 소유를
  발행한다. partial의 줄별 폭, 선택 창의 개체 보존, 인라인 pen과 `resolved_table_left`가
  이를 소비한다. 이 pen 뒤의 재앵커 선택까지 직접 확인했다.
- `layout_paragraph_in_frame_impl`은 들여쓰기 표시를 반환용 LineSeg뿐 아니라 확정할 실제
  RowSegment.source_tag에도 보존한다. 프레임을 다시 읽는 소비자가 내어쓰기 후속 줄의
  가로 원점을 잃지 않는다. 보정 도중 기존 완료 좌표 검사가 이 누락을 FAIL로 잡았고
  반환 사본만 고치는 대신 생산자를 수정했다.
- 상세 `MeasuredCell`의 NO_LS HWPX 가로 문단도 같은 현재 프레임을 쓴다. line_heights,
  para_line_counts, 자기 프레임의 분할 줄 수가 실제 셀 컷과 대응한다. HWP5·legacy·실제 저장 줄의
  상세 원장은 이 변경의 대상이 아니다. 이미 흡수한 본체를 원본 합성 높이로 다시 더하지 않도록
  미흡수 높이 보정도 현재 프레임 문단으로 수행한다.
- `sole_nested_table_in_paragraph`는 단일 표와 배치 상자가 없는 마커만 허용한다. 측정·전체 배치·
  partial wrapper·typeset의 해제 후보가 이 계약을 사용한다.1×1 모양만으로 첫 표를 고르면
  두 번째 자식이 사라지므로, 여러 배치 소유자가 있으면 wrapper 셀을 유지한다.
  기존 단일 표+폭 중립 마커의 허용 계약은 정적 대조했고 전체 회귀에서도 확인한다.

기존 컷의 rowspan/저장 프레임/캡션/각주 알고리즘을 별도로 재작성하지 않는다. 새 입력은
rowspan·캡션·각주가 없으며 그 조합의 신규 시각 일치 주장을 하지 않는다. 분할 뒤 원문 전체와
후속 본문, 마지막 자식의 소유·중복·누락과 불필요한 빈 쪽까지 직접 비교한다.

추가 구현으로 Stage3 최초 lint 묶음의 workspace build를 중단했다. 그 전 fmt/native/WASM clippy의
성공은 당시 source의 기록이며 새 최종 source의 성공으로 재사용하지 않는다. 중단 exit -2는
제품 테스트 실패나 완료로 계산하지 않고 최종 source에서 필수 묶음을 다시 실행한다.

## 원본 식별자와 최신 기준 확인

재조판 `paragraph_with_frame_geometry`는 줄 좌표·높이를 읽기 위한 임시 뷰다.
`cell_units_cache`는 원본 `Cell` 주소를 key로 유지하므로 그 복사본의 Table/Cell을 넘기지 않는다.
whole 셀 배치, partial 조각 배치, 셀 유닛의 실제 개체 순회는 원본 controls를 유지하고,
줄 상자·슬롯 소속·pen은 재조판 프레임을 소비한다. 모델 식별자와 줄 좌표의 계약을 구분했다.

추가2개 반례를 포함한 Native/fresh WASM 11문서·15쪽 모두100%를 확인한 후 분할 관계2개와
그4쪽의 독립 Print 제목/후속 본문 ±2px 검사를 보완했다. 원본 식별자 보완 직후 Native 전쪽을
다시 산출했다. fresh WASM과 전체 Rust의 최종 결과는 리베이스 후 실행 결과로 별도 고정한다.

원격 head가 `375aff294`에서 `4662c069e`(#7913)로 변경됐다. 현재 보정을 커밋한 뒤
리베이스하며, 새 기준 producer와 후보 producer를 다시 비교한다. 사용자의 최신 지시에 따라
PR 생성은 검증 결과와 본문을 준비한 뒤 별도 승인을 받아 진행한다.

현재 기준의 page-count gate도 유지한다. 이전 base375aff는 `page-height-28000`에서2쪽/Print3쪽,
`partial-nested-width-wrap`·`partial-nested-wrapper`에서 각각1쪽/Print2쪽이었다.
의도한 복구라도 base TSV 누락·쪽수 차이를 승인으로 바꾸지 않는다. 새 base의 동일 조건을
확인한 후 `visual_verification_governance.md`의 전쪽 일대일 비교 판정에 기록한다.

### 여백 앵커 회귀의 검출과 보정

원본 식별자 보완만 적용했을 때 `nested-outer-margins` Native가93.64997%로 낮아졌고
관련12개 중 여백 관계·완료 쪽 ±2px 검사2개가 실패했다. 새 기대값을 만들지 않고 원인을 고쳤다.
실제 표는 원본 주소를 유지하되, `table_band_hu`와 바깥여백을 포함한 줄 높이 비교에는
`FrameGeometry::inline_table_height_hu`의 현재 본체 높이를 소비하도록 수정했다.
`expanded-targeted-tests.log`와 `frame-height-final-regression-proof.log`는 같은 검사 실행 파일로
수정 전10통과/2실패 → 수정 후12통과/0실패를 대조한다. 최종 시각 증거는 새 source에서 전쪽 재산출한다.

## 리베이스 후 최종 시각 판정

기준 `4662c069e557e928ed7968d061121fde90b1f027` 위로 충돌 없이 리베이스했다.
제품·회귀 소스 SHA는 `fafd0d37813a61af76fbe4f0e0a4bcf5f3c64783`이다. 이전 코드
`a08cf5cc6`→`fab2e2cfd`, 관계 검사 `fd8ed2451`→`11dccf763`, 최종 보정
`578d1376d`→`fafd0d378`은 리베이스 전후 대응이며 과거 실행 로그의 SHA를 소급 교체하지 않는다.

같은 입력·한컴 Print·96dpi·compat2024·Print profile·고정 HBATANG/HDOTUM 글꼴을 사용했다.
각 소스의 Native와 fresh WASM producer는 `final-builds.json`의 별도 빌드로 만들었고
`final-producer-provenance.json`에 파일 SHA를 고정했다. sweep manifest의 git_head는 실행 작업트리이며,
기준 producer의 source SHA를 후보 작업트리 SHA로 해석하지 않는다.

- 최종 Native: 11문서·15쪽, 최저100%, 미달·누락0, 모든 Print 쪽수 일치.
- 최종 fresh WASM: 같은11문서·15쪽, 최저100%, 미달·누락0, 모든 Print 쪽수 일치.
- 새 PNG15쪽을 Native/독립 Print/fresh WASM 같은 영역에서 직접 확인했다. 여백·이월·wrapper·
  후속 본문은 같은 확대율에서도 대조했고 대표 review 및 standalone overlay도 확인했다.
  같은 줄/개행/폭 부족, 줄·표 순서, 마지막 자식과 뒤 본문, 작은 용지3쪽의 소유가 일치한다.
- 100%는2px 관용 실루엣 보조값이다. 실제 래스터 윤곽은 backend마다 다르므로 엄격 픽셀100%를
  주장하지 않는다. 해당 출력에서 남은 줄바꿈·포함·누락·겹침·여백 결함은 관찰하지 못했다.
  이 입력에는 수식·캡션·각주가 없어 그 조합의 새로운 시각 일치 주장을 하지 않는다.
- 관계 검사11개와 px 검사1개(11문서의 완료14쪽)를 합한12개를 같은 진단 실행 파일로 대조했다.
  새 devel0통과/12실패 → 분할 보정 전9통과/3실패 → 최종 CLI12통과/0실패다.
  실패는 실제 셀 포함·쪽수·제어문 소유·제목 원점 assertion이며 빌드/환경 실패가 아니다.
  최종 source의 전체 Cargo 실행 결과는 아래 필수 검증 완료 기록과 구분한다.

### devel 전쪽 일대일 gate

`compare_final.py`는 Decimal로0.01%p 하락을 비교하고 입력/PDF/글꼴 해시·backend·profile·
compat 조건도 대조한다. 비교 가능한8문서의 양쪽8쪽에서 하락0쪽, 조건 불일치0건이다.
나머지3문서는 아래 쪽수 복구 때문에 base TSV가 없다. 현행 지침과 사용자가 고른
전체 일대일 기준을 유지하므로 `re_review_required`이며 PR 생성·승인·통합을 보류한다.

| 입력 | devel Native/WASM | 최종 Native/WASM | 독립 Print |
| --- | --- | --- | --- |
| page-height-28000 | 2/2 | 3/3 | 3 |
| partial-nested-width-wrap | 1/1 | 2/2 | 2 |
| partial-nested-wrapper | 1/1 | 2/2 | 2 |

같은 쪽의 수치 하락을 검출한 결과와, 위 쪽수 복구의 대응 증거 부족을 혼동하지 않는다.
모든 후보 쪽이100%여도 이 별도 gate를 승인으로 바꾸지 않았다. 원본·독립 PDF·정상 회귀를
제거하거나 기준을 바꿔 PR 조건을 회피하지 않는다.

### 추가 입력의 고정 바이트

| 입력/PDF | SHA-256 |
| --- | --- |
| `tests/fixtures/issue7419-boundaries/partial-nested-width-wrap.hwpx` | `500fb3d32f015808c3c4c4d5304fdd561e33ec233bbae7e6cdd536911c55d096` |
| `pdf/issue7419/partial-nested-width-wrap-print-2024.pdf` | `c4a8e326e03254df9a5df2767cef4bc8e2bb1420371d4b19d3cc3a7ad623441f` |
| `tests/fixtures/issue7419-boundaries/partial-nested-wrapper.hwpx` | `95e9b90b1b2fa47e51566235e210291e093f886254ae2d82571c715574c7dd61` |
| `pdf/issue7419/partial-nested-wrapper-print-2024.pdf` | `19fc1d70763dda29100f79c6bb5571f3285f25be1d9af20704d4b6be6e35167c` |

## Native 소유권 보정 뒤 실행 기록

최종 source `59d87766f96a72b8502a49aff31c66dd16b9742c`의 실패3개+신규12개 집중 검사는
15/15통과했다(`regression-fix-focused.log`). 실패 assertion·baseline·fixture는 바꾸지 않았다.
가상융합 시행령은74쪽으로 돌아왔으며 고정 devel과 전74쪽 render tree JSON이 바이트 동일하다
(`regression-fix/decree-tree-comparison.json`). 전체 시각 보정 완료 문서라는 주장은 하지 않는다.

새 source의 Native/fresh WASM11문서15쪽은 양쪽 최저100%·미달/누락0·Print 쪽수 일치다.
전15쪽과 여백/이월/wrapper/뒤 본문 확대 판독, 대표 review·overlay를 다시 확인했다.
이전 source PNG를 최신 결과로 바꾸어 기록하지 않았다. 최종 producer 해시, TSV와 직접 판독은
`regression-fix/{producer-provenance,final-direct-read,final-devel-comparison}.json`에 있다.
비교 가능한8문서는 양쪽 하락0, 쪽수 복구3문서는 기존 전쪽 일대일 gate가 계속 보류다.

Studio 원본2개의 Canvas2D·CanvasKit compat/default software Print6캡처를 실행했다.
초기 응답6개의 WASM SHA는 모두 `b4ead97873e707606b0351ae792b8f71926d4497aa2731826a9ca069862e115d`로
fresh package와 일치한다. 최초 관측의 inspector body cache 축출은 별도 로그에 보존하고,
관측 버퍼를 늘린 새 실제 캡처에서 신원을 확인했다. replay hard violation0이다.
backend 간 report-only 엄격 픽셀 비교4건은 불일치이며 gate 통과로 표시하지 않는다.
같은 좌표의 독립 PDF 내용 영역과 직접 대조했고 줄·표 소속/겹침 결함은 관찰하지 못했다.
이 입력에는 별도 DOM 그림이 없으며 화면 안내선/래스터 윤곽 차이와 본문 배치를 구분했다.

최종 lint6종(fmt·host/WASM Clippy·workspace build/all-target Clippy·suite manifest)은 모두 통과했다.
release-test 전체는 시작됐으며 Native Skia3종은 그 뒤에 실행한다.
두 묶음은 `regression-fix/pipeline.log`와 각 validation.json의 실제 exit로 갱신한다.
[개별 review](../pr/archives/pr_7433_review.md)와 [현재 시각 자산](../pr/assets/pr_7433_maintainer/README.md)에 연결한다.

## 추가 혼재 경계의 직접 실패 — 최종 후보 재보정 필요

한컴2024가 실제 저장한 HWP/HWPX를 만든 뒤 부모 host와 첫 자식의 LineSeg만 제거했다.
둘째 자식의 실제 저장 줄은 그대로 유지한다. 두 자식 줄을 모두 제거한 대조군도 만들었다.
두 변형의 독립 Print는 모두2쪽이며, 쪽별 PDF PNG가 바이트 동일하다.
59d87766f에서 전부 NO_LS인 대조군은100%/100%, 저장 줄 혼재 반례는55.80081%/68.31124%다.
혼재 반례에서 첫 자식이1쪽에 나오지 않고2쪽의 두 자식이 겹치는 것을 직접 확인했다.
이는 이전11문서의100%와 별도의 새 실패이며, 그 통과로 면제하지 않는다.
`FrameGeometry`가 NoLS 자식만 소유하여, 같은 프레임 안의 저장 줄 자식이 셀 유닛에서 따로
예약되는 생산/소비 불일치다. 실측 프레임을 만들 때 모든 TAC 형제를 함께 소유하게 보정한다.
이 검사를 추가하기 전 양쪽90% 이상과 직접 판독을 다시 확보한다. 생성 과정·SHA·한컴job·PDF·
수정 전 PNG는 ignored `output/pr-review/pr7433-current-20261016/regression-fix/mixed-storage/`에 보존한다.
59d의 최종 전체 빌드는 이 결함 수정 전에 중단했다. 외부 pipeline exit130이며 테스트 summary가
없으므로 전체 회귀의 성공/실패 건수로 기록하지 않는다. lint6종과 기존15개 집중 성공은 당시
source의 증거로 남기고 다음 source에서 필요한 검증을 다시 한다. Native Skia3종은 미실행이다.
