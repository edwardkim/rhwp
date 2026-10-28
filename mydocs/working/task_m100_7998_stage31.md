# #7998 stage31 — 가져오기·직접 역편집의 기준선 계약

2026-10-20 다음 작업 진행 승인을 받아 R2의 PDF 가져오기와 편집 재조판 사이 기준선 차이를 처리한다. 기준은 `d7b267ff96424fe3db5e2795d200585e26271160`이며 stage30 수평 원점의 사용자 통과와 정식 회귀는 유지한다.

## 규칙·독립 근거와 범위

PDF Tm/폰트 크기·셀 경계의 관측 기준선을 정답으로 사용한다. 새 줄 메트릭은 기존 공통 `frame_metrics_for_line`의 HWPUNIT 결과를 소비하고, source 기준선에서 해당 baseline 거리를 뺀 줄 상단을 발행한다. .95를 .85로 별도 복사하거나 저장 좌표/LineSeg를 편집 후 되돌리는 방식은 사용하지 않는다. 소유 문단의 첫 줄·중첩 셀·본문 글상자·혼합 기준선과 10.5pt의 반올림을 먼저 검증한다. source 두 줄을 편집 후 같은 개수로 고정하지 않으며, 문단 BREAK의 일반 재줄바꿈과 기준선 보존을 구분한다.

독립 기존 공개 입력과 10.5pt 변형의 생성 방식·입력 SHA는 ignored `output/pr-review/issue7998/p4-paragraph-baseline/` → `/mnt/e/tmp/issue7998-stage31-evidence/`에 보존한다. 변형은 기존 Arial 입력의 Tf만 바꾸고 source Tm/표 경계를 유지한 PDF이며 rhwp 기대 좌표를 넣은 fixture가 아니다. 시각 선행 조건 전에는 새 fixture/회귀를 정식 등록하지 않는다.

## 실제 소비 경로

- 생산: `structured_lowering::paragraph`의 줄 상단·line_height/text_height/baseline와 FlowItem bottom.
- 높이/순서: 셀 flow가 같은 top/bottom으로 spacing_before/after·첫 vpos를 만들고, body frame도 같은 결과로 shape 원점/높이·문단 vpos를 만든다.
- 편집: `reflow_cell_paragraph_with_metrics`/`reflow_cell_paragraph_by_path` → composer의 make_line_seg → font_size_to_line_height/baseline_distance_hwp. 최초 줄 vpos는 보존하며 뒤 문단 사다리는 실제 재계산한다.
- paint: compose_paragraph → `layout_composed_paragraph`의 stored baseline → corrected_line_baseline_for_source/ensure_min_baseline. 마지막 보정은 section source-metric eligibility와 최소 .8em 조건이며 일반 .85em 줄은 덮어쓰지 않는다. script_draw_metrics는 각 CharShape의 별도 source 이동을 적용한다. 셀·body의 source anchor도 대조한다.

현재 코드의 실제 실패를 보존한 뒤 공통 메트릭을 소비하도록 제품의 producer와 삭제 후 정규화 무효화 두 파일을 수정했다. Native 빌드와 좁은 계약은 통과했고 fresh WASM·독립 Print·전쪽 비교를 진행 중이다. 전체 회귀·P5·원격 작업·다음 절편은 이번 경계에 포함하지 않는다. B24/R1, 남은 R2 두 메트릭 및 `parser_allocation_bounds_proven=false`를 유지한다.

## 수정 전 실행과 구현

`before-contract.log`는 실제 기준 코드에서 같은 새 회귀 후보가 **FAIL**한 로그다. 첫 문단의 source Tm 기준 4500HU가 직접 삽입·삭제 뒤 4400HU로 올라갔다. 별도 model 비교도 baseline_distance 950→850을 검출한다. 최초 실패를 library/빌드 오류로 세지 않았고, 실행 파일·검사 source·입력·기준 head·library SHA를 `before/identity.json`과 함께 고정했다.

생산자는 각 줄의 HWP 폰트 크기를 먼저 확정해 공통 `frame_metrics_for_line`을 72DPI(pt 좌표계)로 호출한다. 반환한 line_height/text_height/baseline_distance와 그 baseline에서 역산한 row top을 같은 결과로 사용한다. 셀 flow의 gap/첫 vpos와 body frame의 top/height는 해당 FlowItem top/bottom을 소비한다. source CharShape의 첨자/별도 위치와 원래 텍스트 순서는 바꾸지 않는다. 일반 compositor·편집기·저장기의 줄 메트릭 정책을 별도 PDF 분기로 덮어쓰지 않는다. 기존 관측 줄간격의 표현 제한을 이번에 일반화했다고 주장하지 않는다.

## 실제 화면 소비에서 찾은 추가 결함과 범위 분리

본문 HWPX는 삽입 결과를 실제 RenderTree로 그린 뒤 직접 삭제하면 IR text에는 X가
없는데 정규화 section projection에는 X가 남았다. 삭제는 deferred 모드에서만 section
revision을 올려 immediate pagination이 이전 projection을 재사용했다. 삽입의 기존 정책과
동일하게 양쪽 모드에서 필요한 section을 무효화하고, deferred 모드에서만 즉시 재계산한다.
`baseline-only/ghost-contract.log`에 보정 전 실제 glyph 순서 FAIL을 보존했다. 이전 stage30
WASM `0e8276d6…`에서도 같은 입력으로 stale X와 X 없는 모델 저장본을 분리해 확인했다
(`baseline-only/old-wasm-ghost.json`, `model-vs-stale-paint.json`). 최종 Native의 같은 실제
삽입 paint/삭제 검사와 양 저장 재열기는 PASS다.

원래 negative 기준선 검사는 cell/pdf의 4500→4400HU로 실패했다. 이후 검사에서 body의
독립 PDF MediaBox를 260pt로 바로잡아 source Tm 기준선을 5000HU로 수정하고, 삽입
중간 paint를 추가했다. 최초 실패 실행/소스와 현재 검사 소스의 차이를 보존하며 전체 검사
바이트가 동일했다고 주장하지 않는다(`source-runtime.json`).

본문 무편집 저장본의 별도 source 줄 보존 결함은 독립 Print에서 발견했다. 수정 전 basic도
28.21447%이며 한컴은 두 줄을 한 줄로 재구성한다. 현재 basic 27.64057%, script 11.99262%
(Native)인 HWP/HWPX 4개 결과는 그대로 미달로 남긴다. 메인테이너가 **“다음 R2 절편으로
분리”**를 지시했으며 backlog에 입력·출력과 후속 계약을 연결했다. 임의 강제 개행·그림 대체로
완료하지 않는다. 이 본문 입력의 새 정식 렌더링 회귀 등록은 보류하며 현재 전체 R2·P5와
PR 생성/승인 gate도 미완료다.

## 최종 제품 빌드·집중 검증과 현재 시각 범위

제품 source SHA·실행 명령·Native/WASM SHA는 `source-runtime.json`, 각 visual manifest와
실행 로그에 고정했다. 공유 target의 Native build는 2분 42초, Docker fresh WASM은
8분 22초이며 JS glue는 이전과 동일하다. 검증에 사용한 제품 파일 두 개는 빌드 후 변경하지
않았다. Native SHA `ae7bf12973e187aa90976306771869c64ec0fa62ed8eaa17ff7de64f84e9ab3d`,
WASM SHA `5ded683745ea71367ec103650bc4511d4c603d34622af797deaeb9bc48fe06cd`다.
Studio 7793의 새 네 탭에서 실제 이 WASM 로드를 확인했다(`human-review-tabs.json`).

- 기존 자동 수집 #7998 집중 20개 case 모듈 **156건 PASS**와 문단 여백 3개 모듈
  **12건 PASS**. 신규 기준선 후보는 임시 evidence 경로에서 검사하며 정식 등록과 구분한다.
- 일반·중첩 셀, 10.5pt와 중첩 10.5pt, 셀/본문 첨자, 본문 글상자: 실제 Native
  PDF/HWP/HWPX 21경로에서 첫 줄의 같은 글자·최종 `script_draw_metrics` 기준선이
  삽입 표시→직접 삭제→양 저장 재열기에서 보존된다. source .95/편집 .85 실패는
  공통 row metric을 발행해 해소했다. source 모든 줄 개수의 보존은 검사하지 않는다.
- 실제 fresh WASM은 보존 테두리 guard를 포함한 **24경로**에서 삽입 paint·직접 삭제·
  Snapshot undo/redo·양 저장을 확인했다. PDF 경로 저장본은 Native와 바이트 동일하다
  (`studio-result.json`). 원래 body HWPX의 X 잔상도 사라졌다.
- 독립 한컴 2020 Print **34건**과 입력 SHA가 같은 **14개 alias**로 저장본 48개를
  대조했다. 수정 전 본문 원인 분리 Print 1건은 별도다. 입력·Print SHA와 작업 ID,
  engine/method/one-up은 `remote-print/`, `print-aliases.json`, visual manifest에 남겼다.
- 원본 PDF 26개·37쪽과 저장본 48개·48쪽: Native/fresh WASM 각각 **74입력·85쪽**
  전체 TSV를 보존했다. 다음 R2로 분리한 본문 무편집 4입력을 제외한 **70입력·81쪽**의
  최저는 **Native 93.68165% / WASM 93.11957%**이며 누락·쪽수 차이는 없다.
  미달 4입력은 Native basic 27.64057%/script 11.99262%, WASM basic 27.86690%/script
  13.47150%다. 전체 visual gate의 `re_review_required`를 임의로 PASS로 바꾸지 않는다.
- 실제 기존 source 코드와 현재 코드의 Native/fresh WASM 전쪽 비교에서 최대 하락은
  Native 보고서 8쪽 **0.03257%p**, WASM 보고서 6쪽 **0.00892%p**다. 각각 99.77458%/
  99.74036%로 유지하며 차이 영역을 같은 배율로 확대했다. 원본 26개 `export-tables`의
  실제 표/셀/중첩/내용과 `export-text`는 수정 전후 동일하다(`semantic-comparison.json`).

새 10.5pt 대조군을 정식 등록하기 전에 PDF stream 길이를 실제 content 908바이트로
바로잡았다(초기 진단본 909는 구분자 LF를 포함). source Tf/Tm·CTM·경계는 동일하며
관련 PDF 1쪽의 수정 전후 Native/WASM을 다시 산출했다. 같은 source로 만든 HWP/HWPX
12개는 모두 초기 저장본과 바이트가 동일해 이미 확보한 Print를 그대로 재사용했다
(`input/length-qualification-result.json`). source SHA와 변경 과정을 provenance에 보존했다.

비교 화면: `E:\tmp\issue7998-paragraph-baseline-review\index.html`. 독립 Print·source
수정 전후·최종 Native/fresh WASM·같은 배율 확대와 본문 미완료 4개를 함께 표시한다.
직접 시각·편집 구조 판정과 셀 범위 정식 회귀를 아래 로컬 마감에 연결했다.
본문 미달 입력의 새 정식 회귀 등록·전체 R2/P5 완료·PR 등록은 보류한다.

## 직접 판독·정식 회귀와 로컬 마감

메인테이너는 새 네 Studio 탭과 비교 화면의 첫 줄 기준선 범위를 **“통과 판정입니다.”**로
수용했다(`maintainer-disposition.json`). 일반/중첩 첫 줄, 첨자의 실제 높이와 10.5pt의
관계, 본문 X 잔상 해소를 source/독립 Print/이전 실제 역편집/최종 Native·WASM으로
대조했다. 본문 Helvetica가 한컴 Print의 내장 Haansoft Batang으로 대체된 외형 차이는
비교 화면에 남겼다(`pdffonts print/body-restored-hwpx-2020.pdf`). report source의 최대
raw 하락 영역도 앞뒤 글줄·괘선과 함께 확대했고 큰 위치/누락/겹침 변화는 없었다.
본문 source 줄 보존의 미달을 이 직접 수용으로 통과시키지 않는다.

새 정식 회귀는 `tests/cases/issue_7998_pdf_edit_baselines.rs::
pdf_cell_first_line_baselines_survive_painted_inverse_and_both_saves`다. 시각 선행 조건을
충족한 일반/중첩/10.5pt/중첩10.5pt/첨자 셀의 **5대상 × PDF/HWP/HWPX**에서 실제 삽입
paint→직접 삭제→HWP/HWPX 재열기의 첫 줄 glyph 소속·문자 순서·최종 기준선 관계와
이웃을 포함한 전체 painted 내용 보존을 검사한다. source의 임의 절대 px, .85/.95 계산값,
모든 편집 뒤 줄 개수를 기대값으로 복제하지 않는다. 본문 무편집 입력은 새 회귀에
넣지 않았다. 공개 10.5pt fixture의 생성기·provenance와 qualified SHA를 함께 보존했다.

최초 frozen Native 실행은 같은 일반 셀·실제 API의 최종 glyph 관계에서 4500→4400HU
FAIL을 보존했다. 이후 정식 함수는 같은 관계에 전체 내용 보존을 보완하고, 직접 분리한
본문 입력/절대 source 위치 assertion을 제외했다. 정식 함수 전체를 과거 rlib에 다시 링크한
실행은 하지 않았으므로 이를 ‘동일 함수 전체의 수정 전 FAIL’로 쓰지 않는다. 실제 최초
관계 실패·cache 보정 전 glyph 순서 실패와 정식 새 함수의 현재 PASS를 구분한다.

새 정식 함수 **1 PASS**와 이전 제품이 같은 기존 **156 PASS + 여백 12 PASS**, 합계
**169 PASS**를 연결했다. 전체 169건을 다시 실행한 것으로 바꾸지 않는다. 자동 수집은 새
모듈을 `regression_suite_008`에 배정하며 `--prepare`/`--check`에서 1564 sources·
6971 static attrs·28 suites+20 exceptions/48 targets를 확인했다. 제품 source가
빌드/전쪽 비교 후 동일함과 새 fixture의 입력 바이트는 `source-runtime-registered.json`으로
확인했다. source/test rustfmt와 diff check를 통과했다. 새 검사·문서 등록만으로 제품을
재빌드하거나 기존 캡처를 반복하지 않았다.

코드 위치는 producer `structured_lowering.rs:414`, 셀 FlowItem/lead 소비 같은 파일
1115–1153, 본문 frame `structured_lowering/body.rs:127`, 공통 row metric
`composer/line_breaking.rs:3280/3300`, 편집 재조판 `text_editing.rs:2658/3045`, paint
`paragraph_layout.rs:6077`의 stored baseline 소비와 최소 baseline 분기를 대조했다.
즉 helper의 결과 뒤에 다른 앵커/최솟값 보완이 있는 실제 소비 경로까지 확인했다.

이번 절편은 로컬 기준선 수용·셀 범위 회귀 등록이다. 다음은 사용자 지시에 따라 분리한
본문 무편집 source 줄의 저장/재조판 계약이며, 남은 R2 두 메트릭·B24/R1·P5도 유지한다.
전체 suite·전체 lint·Worker/package·원격 push/PR·다음 절편은 실행하지 않았다.
`parser_allocation_bounds_proven=false`와 전체 visual `re_review_required`는 유지한다.
