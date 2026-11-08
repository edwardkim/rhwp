# #7998 stage30 — 직접 역편집의 문단 원점 계약

2026-10-20 다음 절편 진행 승인을 받아 R2의 문자 삽입·직접 삭제 후 저장 줄 원점/기준선 경계를 조사한다. 기준은 `c4e98a947771aef381a18a18a34b36bc59d5c7b9`다. Snapshot undo/redo와 직접 역편집을 구분한다. stage29의 실제 셀 테두리 통과·B25 클리핑 농도 예외는 유지한다.

## 범위와 독립 근거

PDF의 실제 텍스트 행렬·baseline·셀 경계와 정상 한컴 편집/인쇄를 기준으로 문단 속성 → 저장 LineSeg → 편집 재조판 → 최종 배치를 추적한다. 기존 화면을 고정하기 위한 원점 복사나 임의 상수는 해결 근거로 삼지 않는다. 첫 경계는 공개 `structure-borders/native-pen.pdf`와 `horizontal-metrics/closed-large/wide-inferred.pdf`의 첫 셀이다. PDF·HWP·HWPX 경로를 각각 문자 삽입·직접 삭제하며 원문, 문단 속성과 실제 render tree를 비교한다.

## 수평 원점 범위의 수용

2026-10-20 메인테이너가 **“수평 원점 범위 시각·편집 구조 통과”**로 판정했다. 생산된 문단 속성은 그대로 유지하고 flat·중첩 편집의 공통 상자 소비를 보정했다. 정식 회귀 1건을 기존 자동 수집 module에 등록했다. 증거는 ignored `output/pr-review/issue7998/p4-paragraph-reflow-origin/` → `/mnt/e/tmp/issue7998-stage30-evidence/`에 보존한다. Native/fresh WASM·독립 출력·정식 집중 검증은 실제 영향에 맞춰 수행한다. 전체 suite·전체 lint·다음 절편·원격 push·PR 작업은 이번 착수 범위가 아니다. B24/R1/R2/P5 및 `parser_allocation_bounds_proven=false`는 유지한다.

## 현재 코드 재현과 1차 구현

`result.json`은 기준 HEAD의 실제 Native library로 PDF·HWP·HWPX × 앞/중간 삽입 12경로를 실행한 결과다. `native-pen`은 문단 margin_left=2000(2×HU)이 유지되는데 줄 column_start가 1000→0, baseline_distance가 950→850이다. `wide`도 column_start 1000→0·baseline 1140→1020이며 2개 저장 source 줄이 재조판 시 1줄로 바뀐다. 소스의 관측 줄을 그대로 복원하는 검사는 추가하지 않는다. 실제 단락의 BREAK 계약과 폭에 따른 재줄바꿈은 허용한다.

이번 첫 보정은 **수평 원점 계약**이다. 실제 문단 여백은 정상이며 producer의 누락이 아니다. `text_editing::reflow_cell_paragraph_with_metrics`와 깊이≥2 `reflow_cell_paragraph_by_path`가 안쪽 폭에서 여백만 뺀 후 `content_width_px`로 원점을 버렸다. 두 경로를 renderer가 이미 사용하는 `ParagraphBox::content_for_style`과 연결한다. 새 상자의 `effective`가 reflow 발행 column_start/segment_width와 inline frame을 함께 결정한다. flat 표/글상자/그림 캡션, HWPX full 후보와 split 호출도 같은 상자를 소비한다. indent/tag·목록 원점 비유도 처리·가용 폭 거절은 공통 helper의 기존 규칙을 사용한다. padding은 owner에서 한 번만 뺀다.

별도 기준선 차이는 PDF producer의 .95×font와 reflow의 .85×font 정책 차이다. 이번 수평 원점 수정으로 해결됐다고 보고하지 않는다. LineSeg를 되돌리거나 vpos를 임의 보정하지 않는다. pagination/continuation 컷/높이 예약 로직은 바꾸지 않는다.

## 검증 결과

- shared `target/pr-review` jobs14·release-test Native 빌드, canonical shared `rhwp` Docker WASM(codegen-units16)을 실행했다. Native `a2bccf676ad288568d9bfc1d4a19d07154ecb442c90c2f3f6472f39b12c3839f`, fresh WASM `0e8276d6a7a7fc596d75a83c66743849c8fb0112f209e11c1e383c50dff73b85`, JS glue `5671802fbb89ed060c3bab7a6f458a78833a7e5ca3e7df8e1699d45098cddeb6`다. root/backend 재컴파일을 확인했다. 빌드 뒤 네 호출의 줄 접기만 rustfmt로 정리했으며 `format-only.json`에 빌드 입력·현재 입력 SHA 및 동일 정규화 결과를 기록했다. 이를 제품 변경으로 보고 동일 검증을 반복하지 않았다.
- 12개 flat 현재-HEAD 재현의 줄 원점은 모두 수정 전 FAIL / 수정 후 PASS다(`horizontal-before-after.json`). 별도로 첫 셀·중첩 셀·내어쓰기·가운데 정렬·하단 셀의 7대상 × 3입력 경로 21개를 관측했다(`origin-boundary-summary.json`). 같은 앞 삽입 경계를 중복 합산해 독립 사례 수라고 부르지 않는다. 저장 줄 수/기준선 변화는 기록하되 통과 assertion으로 숨기지 않는다.
- 실제 Studio public WASM 21경로에서 삽입·직접 역삭제와 별도 Snapshot undo/redo를 실행했다. PDF 경로의 양 저장 바이트는 Native Print 입력과 같다. 84개 WASM 저장본은 두 형식의 실제 재파싱으로 원문·문단 여백에 맞는 원점을 확인했다(`studio-result.json`, `wasm-save-origin-contract.json`). 84개 독립 한컴 Print를 수행했다는 뜻은 아니다. JS 오류는 0이다.
- 7대상의 X 삽입·직접 역삭제 HWP/HWPX 28개 각각 immutable 입력·새 한컴 2020 Print를 확보했다. 입력 바이트/SHA·engine2020·11.0.0.9136·method0·one-up을 `print-contract.json`에 확인했다. 진단 harness가 HWP/HWPX의 root control index를 PDF와 같다고 가정한 부분은 실제 표 경로로 해석하도록 보정했으며 제품 실패로 집계하지 않는다.
- 기존 자동 수집 #7998 20 module 155건과 문단 상자·서식 #7407/#6639 12건, 기존 집중167건이 통과했다. 수용 후에는 등록한 새 회귀 1건만 같은 제품 library에 연결해 추가 실행했다(총168건의 범위별 증거). 기존 함수·제품 코드가 같아 167건/빌드/캡처를 반복하지 않았다. 새 CI 호출·suite·target은 추가하지 않았다. canonical manifest `--check`·변경 파일 rustfmt·diff check도 통과했다. 전체 suite/전체 lint/Worker·package/P5 검증을 실행한 결과는 아니다.
- Native 독립 비교는 31입력·31쪽 모두 90% 이상·누락/쪽수 차이 0이다. fresh WASM의 같은 31쪽도 완료했다. 최저는 Native **94.47080%**, fresh WASM **95.05463%**이고 두 backend 모두 미달·누락·전체 쪽수 차이 0이다. 전체 TSV·명령·입력/기준 SHA는 `{native,wasm}/results.json`에 기록한다. 대표 직접 판독에서는 셀 경계·글자 순서/포함·문단 정렬을 대조했다. 한컴 Helvetica 대체와 Native Arial 공급의 외형 차이 및 기존 기준선 차이는 남긴다.

## 이번 경계의 판정 한계

수평 저장 원점 범위의 메인테이너 직접 판독과 신규 정식 회귀 등록은 마쳤다. 새 회귀의 전체 함수에 대한 이전 Rust library 실행 제한은 아래에 구분한다. 일반 글상자·그림 캡션·split 특수 경로는 같은 상자 생산자를 소비하지만 새 독립 편집 Print를 이 절편에서 각각 실행한 것은 아니다. 해당 경로까지 완료로 확대하지 않는다. 기준선 .95/.85 차이와 source 줄의 재나눔, 두 미수용 glyph/공백 메트릭, B24/R1 일반화·P5는 유지한다. 보정 원점과 기존 renderer의 x 보완을 구분하며 화면 x 자체가 이동했던 버그라고 보고하지 않는다.

작업 과정의 첫 Docker 호출은 worktree 기본 project로 네트워크 주소 pool 부족에 실패해 컴파일하지 않았다. 그 호출이 만든 새 빈 volume 네 개는 container 참조가 없음을 확인해 제거하고 기존 shared project로 정상 빌드했다. 기존 target/cache/일반 사용자 profile은 정리하지 않았다.

## 직접 판독 수용과 남은 검증 범위

비교 화면은 `E:\tmp\issue7998-paragraph-origin-review\index.html`이다. 원본·같은 저장본 한컴 Print·Native/fresh WASM의 전쪽 TSV와 대표 compare/overlay/review를 연결했다. 최저 영역과 대체 글꼴·기준선 미세 차이도 숨기지 않았다. 실제 새 WASM SHA를 검증한 Studio 7793의 `issue7998-paragraph-origin-{flat,nested,indent,center}` 네 탭을 열었다. 7700과 기존 탭은 보존했다.

수평 원점 범위의 메인테이너 시각·편집 구조 판정은 **통과**다. `maintainer-disposition.json`에 실제 네 탭·fresh WASM SHA와 판정 범위를 기록했다. 기준선 차이의 수용이나 R2 전체 완료로 확대하지 않는다.

새 정식 회귀는 `tests/cases/issue_7998_pdf_table_inference.rs::pdf_cell_text_reflow_keeps_source_horizontal_box_through_edits_and_both_saves`다. 기존 공개 PDF의 독립 텍스트 행렬과 셀 경계(30−20pt, 48−40pt, 28−20pt)에서 원점을 정한다. source 좌표를 rhwp의 최종 px 배치 pin으로 쓰지 않는다. 7대상×PDF/HWP/HWPX에서 삽입·직접 삭제의 원문·source 원점·가용 폭 보존, 양 저장의 원문/쪽수/최종 paint 순서를 검사한다. 가운데 정렬 원점0·내어쓰기와 중첩 셀은 같은 검사 안에서 정상 대조한다. source 줄 수·baseline을 과거 값으로 고정하지 않는다. 자동 수집은 같은 module을 `regression_suite_007`에 배정하며 1563 source·6970 static attrs·28 suite+20 exception/48 target을 확인했다.

이전 frozen Native probe가 동일 API·독립 기대값의 1000→0 실패를 실제 실행으로 보존한다. 다만 새 Rust 함수 **전체**를 이전 frozen rlib에 재링크하려던 시도는 과거 의존 산출물 부재(E0463)로 실행하지 못했다. 이를 수정 전 새 함수 FAIL로 세지 않는다. 이 전체 함수의 이전 코드 실행은 미검증으로 남기며 동일 product의 현재 정식 함수 PASS와 구분한다. 과거 전체 빌드나 target 초기화로 검증을 반복하지 않았다.

## 로컬 절편 마감

등록된 새 함수는 `registered-origin.log`에서 **1 PASS / 155 filtered out**다. 기존 #7998 155건·상자/서식12건은 제품 코드가 같은 직전 실행을 재사용한다. 자동 수집 `--prepare`/`--check`, 변경 source/test의 rustfmt와 diff check를 통과했다. 제품 source·기준 PDF·전쪽 Native/fresh WASM·사용자 탭의 SHA 일치는 `source-runtime.json`, `source-runtime-registered.json`에 연결한다. 코드 변경 없는 재빌드/재캡처는 수행하지 않았다.

이번 마감은 수평 저장 원점의 로컬 수용·회귀 등록이다. 기준선 .95/.85, source 줄의 재나눔, 남은 두 메트릭과 B24/R1/P5는 유지한다. 전체 suite·전체 lint·Worker/package·원격 반영·PR 작업과 다음 절편은 실행하지 않았다. 전체 R2/P4/P5 완료나 제출 준비 완료로 보고하지 않는다.
