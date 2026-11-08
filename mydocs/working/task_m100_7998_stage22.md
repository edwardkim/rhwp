# #7998 임의 굵기 실선의 행·열 상속 — 첫 비표준 경계

- 시작 SHA `cc46e1817`. 2026-10-19 다음 절편 진행 승인.
- 위반: 표준 BorderFill 굵기와 동등하지 않은 원본 경계는 구조 편집이 거절된다. 임의 굵기를
  최근접 표준 굵기로 바꾸지 않고, 실제 연속 실선의 잉크와 편집 소유를 보존해야 한다.
- 이번 작은 경계는 완전한 직사각 격자의 모든 변이 같은 굵기·색의 불투명 연속 실선인 경우다.
  원본 PNG witness/현재 crop·셀/페이지 소유 → 전 구간 물리 잉크 증명 → 동일 굵기의 정수
  HWPUNIT 경계 띠 → 실제 model 행·열 편집 → 셀 내부와 페이지 바깥 반쪽 재생성 → 기존
  Rectangle 공통 paint·HWP/HWPX 및 CellBorderPaint 왕복을 추적한다. 문단/자식은 복제하지 않는다.
- PDF Butt 선의 직선 띠는 같은 색의 불투명 면 합집합과 동등하다. PDF point→HWPUNIT의
  1HU 이내 양자화만 허용하며 표준 굵기 bucket으로 근사하지 않는다. 측정·본문 높이·줄 구성·
  pagination은 변경하지 않는다. 새 경계는 원래 속성 증명 후 만드는 편집 결과이며 bbox가
  비슷한 그림에 편집 권한을 주지 않는다.
- 표준 동등 경계는 stage21 경로를 유지한다. dash/위상·alpha·가림/틈·서로 다른 pen·병합/
  spacing/지역 축은 이번 일반 상속 증명 밖이며 무변경 거절을 유지한다. 이 경계를 모두
  지원했다고 표시하지 않는다. R2 문자 역삭제 원점과 P5도 남긴다.
- 작은 수정 전 FAIL/수정 후 PASS, 실제 새 셀/본문·자식 보존·삽입/삭제·undo/redo·양 저장,
  한컴 편집 Print와 Native/fresh WASM 전쪽 TSV·직접 판독 후 정식 회귀를 등록한다.
  기존 증거가 같은 제품/같은 입력이면 재사용하고 제품 변경의 영향 범위는 새로 캡처한다.
- 증거는 ignored `output/pr-review/issue7998/p4-border-width/`(외부 E 드라이브)다.
  전체 회귀/lint·Worker/package·devel 통합은 P5이며 이번 절편 승인으로 원격 반영하지 않는다.
  `parser_allocation_bounds_proven=false`를 유지한다.

## 선행 진단과 소비 경계 보완

- `before.log`: stage21 실행 파일에서 0.5pt 비표준 pen의 행 삽입이 미지원 오류로 실패했다.
  빌드 오류는 이 결함 재현으로 세지 않는다. 첫 보정의 `after-fixed-probe.log`는 네 방향 삽입,
  행열 삭제·양 저장·저장 후 재편집과 snapshot 원본 복원을 확인했다.
- PDF의 선언 50HU pen은 실제 정수 crop 투영에서 49HU로 관측됐고, 홀수 51HU pen은
  51HU로 남았다. 이미 정한 1HU 양자화 범위 안에서 비교한다. `contracts-final.log`는 양
  저장 형식 각각 세 번의 열 삽입/삭제에도 이 폭이 누적 변화하지 않는 것을 확인했다.
  근접 표준 굵기 bucket으로 바꾼 결과가 아니다. HWP 저장에 따른 resource ID/raw 캐시
  정규화를 원문 손실로 오인하지 않고, 같은 재열기 상태 전후의 자식 원문/속성을 대조한다.
- 첫 probe의 재열기 후 API 호출은 source control index를 재사용해 실패했다. HWP 파서가
  복원한 실제 table index를 찾아 다시 실행했다. 이 진단 도구 가정과 제품 오류를 구분한다.
- 기존 표준 경계의 Native 저장본 12개는 stage21과 바이트가 같다. 첫 제품의 실제 Studio
  7동작/14개 저장본도 같은 Native API 출력과 바이트가 같다. 중간 source의 검증 결과는
  `before-frame-guard/`에 분리하며 최종 source의 결과로 보고하지 않는다.
- 실제 좌표 소비는 `layout/table_cell_content.rs`의 `text_y_start → para_y_before_compose →
  shape_y`와 `layout/shape_layout.rs::layout_shape_object_with_group_origin`의 Rectangle
  `render_x/y/w/h → RectangleNode`다. 가운데/아래 셀 정렬, 후속 문단 또는 다른 개체 정렬은
  셀 위쪽을 원점으로 삼는 이번 proof와 다르다. `frame-before.log`의 수정 전 무변경 거절
  실패를 확인하고 새 비표준 상속 경로가 이 미증명 원점을 거절하도록 보완했다.
  일반 native 표의 기존 편집과 stage21 표준 동등 경로를 이 보완으로 바꾸지 않는다.
- body glyph/LineSeg 재조판은 R2로 남긴다. 실제 SnapshotCommand의 undo는 원본, redo는
  undo 직전 상태를 복원하며, 문자 직접 역삭제의 원점 자체가 해결됐다는 증거가 아니다.
- 한컴 자동 Print는 첫 HWP가 실행 중이고 다른 작업은 대기열에 있다. 같은 입력을 중복
  제출하지 않는다. 사용자에게 `E:\tmp\issue7998-border-width-review\edited\`의 14개
  HWP/HWPX의 정상 열기·직접 Print를 요청했다. 인쇄 기준 부족은 미검증이며 기존 원본
  PDF 점수로 편집본의 한컴 호환성이나 시각 완료를 대신하지 않는다.

## HWP 도형 헤더 결함과 독립 한컴 확인

- 메인테이너의 첫 직접 확인은 **HWP 7개 손상 / HWPX 7개 정상 열기·인쇄**였다.
  rhwp 자체 재열기 통과를 한컴 호환성으로 대신할 수 없다는 반례다. 잘못된 HWP와 그
  입력으로 제출한 자동 작업 기록은 `before-hwp-header-jobs/`에 분리해 보존했다.
- 새 Rectangle의 `drawing.shape_attr.ctrl_id`에 원본 파싱용 ID를 직접 넣었지만
  `is_two_ctrl_id`는 기본 false였다. `serializer/control.rs::write_shape_component_base`는
  이 경우 원본 헤더 보존 경로를 선택해, 최상위 도형의 필수 이중 ID 대신 단일 ID를 썼다.
  새 도형은 source ID를 비워 writer가 최상위/그룹 자식 문맥으로 ID 개수를 결정하게 했다.
  serializer나 기존 원본 보존 규칙은 변경하지 않았다.
- 헤더만 바로잡은 진단 HWP의 정상 열림을 먼저 확인했고, 최종 코드로 재생성한 **HWP
  7개 모두 정상 열림·PDF 인쇄**를 메인테이너가 확인했다. HWPX 7개는 수정 전후 입력
  바이트가 같아 앞선 직접 Print를 재사용했다. 최종 HWP용 PDF는 사용자 파일명의
  `*-fixed.pdf`를 각 형식과 대응시켰다. `manual-print-sources.json`에 14개 입력/PDF의
  SHA·Creator·Producer·페이지 크기와 직접 인쇄 출처를 고정했다.
- `hwp-headers-before.log`는 단일 ID의 의도한 assertion 실패, `hwp-headers-after.log`는
  7개 모두 이중 ID 통과다. 삽입본 각각 34개, 삭제본 각각 14개의 소유 경계 도형을 검사했다.
  앞선 `contracts-final.log` 대신 최종 source의 폭·색/반복 검증은
  `contracts-hwp-fixed.log`, 71개 고유 기존 함수는 `existing-guards-hwp-fixed.log`다.
- source와 인쇄 용지는 각각 600×720pt / A4 595×841pt다. 앞뒤 문장의 물리 위치에 배율
  변화가 없음을 확인했다. canonical 비교는 동일 (0,0)에 흰 여백을 채울 뿐 정렬·축척을
  맞추기 위해 출력을 이동하거나 변형하지 않는다.

## 최종 source의 전쪽 검증 — 시각·편집 구조 통과

2026-10-19 메인테이너가 위 절편에 **“시각·편집 구조 통과”**로 판정했다. 공유 가로선의
농도 차이를 명시한 확대 화면을 포함한 판정이며, 해당 차이가 사라졌다고 보고하지 않는다.
정상 열림 판정과 이번 시각·편집 구조 판정을 구분해 기록한다. R2·일반 paint 상속·P5까지
포괄하는 승인은 아니다.

| 경계 | 최종 결과·근거 |
|---|---|
| 원본 2개 + 편집 저장본 14개 + 무변경 거절 원본 3개 | Native/fresh Docker WASM 각각 **19입력·19쪽**, 최저 **96.81666% / 95.99244%**. 미달·측정 불가·누락·쪽수 차이 0. 각 manifest의 전체 쪽수와 PNG/TSV를 확인했다. `final-validation-summary.json`. |
| 편집 저장본의 독립 한컴 Print만 | HWP/HWPX 각각 7개. Native 최저 **97.32593%**, WASM **95.99244%**. 파란 편집본은 Native **99.16448%**, WASM **98.04565%**다. `native-final-edits.log`, `wasm-final-edits.log`. |
| 실제 Studio | 최종 WASM에서 **7개 SnapshotCommand**·문자 삽입/역삭제 뒤 snapshot undo/redo·양 저장·재열기·1쪽 표시를 확인했다. 최초 구조 편집 저장본 14개는 Native API와 바이트가 같다. 동일 입력을 추가 14쪽으로 중복 합산하지 않는다. `studio-result.json`, `actual-identical-inputs.json`. |
| 기존 표준 경계 | stage21의 표준 경계 저장본 12개는 최종 Native와 바이트가 같다. `standard-final-identical.json`. 기존 71개 고유 정식 함수는 **71 passed / 0 failed / 1 filtered**다. 새 범위로 지원하게 된 compound-solid를 포함한 이전 거절 함수 한 개는 제외했다. 미실행을 통과로 세지 않으며, phase/alpha 거절과 신규 양성 검사를 분리하는 정식 변경은 직접 판정 뒤에 한다. |
| 구조·원본 속성 | 50/51HU 선언의 관측 폭·색과 양 저장 각각 3번의 열 삽입/삭제에 누적 폭 변화 없음. 첫 자식 표·본문 보존, 새 셀에 본문/자식 복제 없음, 오래된 crop·원점 미증명과 mixed pen의 무변경 거절을 진단했다. `contracts-hwp-fixed.log`, `frame-after.log`, `reject.log`, `guards-final.log`. |

최종 Native SHA-256은
`80637199c30e58cfaad676a1a1a414a49f991f193d41e0b8efe63631543035f0`,
fresh Docker WASM은 **19,579,585바이트**,
`bdd35f4c3d558706d8da8b2f1fafb6bfac05beade47a1c013bee7f88f07c996b`다.
JS glue는 기존 `5671802fbb89ed060c3bab7a6f458a78833a7e5ca3e7df8e1699d45098cddeb6`와
같다. 최종 product 3파일의 SHA는 `source-runtime.json`에 고정했다. 수치 검증 뒤 제품
코드를 바꾸지 않았고, 같은 빌드·성공 캡처를 반복하지 않았다. 실제 Studio 4개 새 탭에서
같은 WASM SHA·1쪽을 확인했다(`user-cdp-tabs.json`).

canonical compare·standalone overlay·review는 Native/WASM 각각 파란 행 삽입·오른쪽 열
삽입·행 삭제에 생성했다. 새 셀 네 변과 자식 표, 앞뒤 문장의 포함·연결·내용 보존을
대조하고 접점 4배 확대도 만들었다. **공유 가로선이 한컴 Print보다 Native/WASM에서 옅게
보이는 농도 차이**는 점수로 면제하지 않고 직접 판정 대상으로 표시했다. 검정 Helvetica의
한컴 대체 글꼴과 이 선 농도 차이는 별개다. 위치를 이동하거나 실루엣 평균으로 덮지 않는다.

이동 가능한 비교 화면은
`E:\tmp\issue7998-border-width-review\index.html`이다. 최종 저장본은 그 폴더의
`final-saved/`, 각각의 독립 Print는 `final-print/`, 전체 TSV/PNG는 `visual/`, 대표
review/overlay는 `reviews/`, 확대는 `zoom/`에 상대 경로로 연결했다. `edited/`의 첫
손상본을 최종 파일로 오인하지 않도록 분리했다. 기존 파일은 삭제하지 않았다.

`--silhouette-only`의 gate `not_evaluated`를 자동 승인으로 해석하지 않는다. 위 메인테이너의
직접 판정 뒤 정식 회귀 8함수를 `tests/cases/issue_7998_pdf_retained_border_structure.rs`에
연결했다. 동일 범위의 공개 입력 3개·독립 Print 14개와 재생성기·출처는
`pdf/issue7998/retained-border-structure/`다. 재생성한 입력은 수용된 시각 입력과 바이트가 같다.
실행 로그와 중간 JSON은 ignored 증거 경로에만 보존한다.

기존 `unsupported_pen_phase_and_alpha_reject_without_mutating_document`는 점선·반투명
거절만 보호하는 `unsupported_phase_and_alpha_reject_without_mutating_document`로 바꿨다.
지원하게 된 동일 pen 실선의 거절 기대값을 없애고 새 양성 검사로 대체하는 계약 확장이다.
phase/alpha·mixed pen의 원자적 거절을 완화하지 않으며, 실행 실패를 기준값 갱신으로
감춘 변경이 아니다. 정식 집중 실행의 최종 결과와 로컬 커밋은 아래에 연결한다.

### 정식 회귀 등록과 단계 종료

- 자동 수집 `--prepare`가 신규 원본을 `regression_suite_012`에 배정했고, 그 실제 Cargo
  target에서 **8 passed / 0 failed**다(`formal-prepare.log`, `formal-tests.log`). 새 셀의
  비표준 경계·본문/자식 미복제, 삽입·삭제의 내용 보존, 양 저장과 HWP 이중 ID, 원본 단위의
  폭·색/반복 편집, 공유 변 교체의 양쪽 소유 제거, snapshot 복원, mixed pen/잘못된 index와
  미증명 원점의 무변경 거절을 보호한다. 절대 px·전체 SVG 해시는 고정하지 않았다.
- 변경한 점선·반투명 거절 함수만 기존 `regression_suite_026`에서 **1 passed / 0 failed**로
  확인했다(`formal-existing-rejection.log`). 그대로인 71개 집중 결과를 재사용하므로 총
  **80개 고유 함수**다. 진단 실행·반복 loop·같은 검사의 재실행을 별도 건수로 더하지 않는다.
- `before.log`, `hwp-headers-before.log`, `frame-before.log`는 각각 미지원 구조 동작,
  실제 손상 헤더, 미증명 원점 수용을 검출한 수정 전의 작은 실행 증거다. 새 정식 8함수
  전부를 과거 코드에서 다시 실행했다는 주장은 하지 않는다. 같은 결함 경계를 보호하는
  정식 항목과 최종 PASS를 연결하며 빌드·환경 실패를 결함 재현으로 세지 않는다.
- 기존 표준 경계·원본 정상 검사는 보존했다. 새 구조/저장 검사는 문단/자식 소유와 내용,
  경계의 원본 폭·색/현재 소유를 확인하며, 실제 위치·농도는 위 독립 PDF와 Native/fresh
  WASM 및 직접 판정으로 검증한다. 메타데이터 개수만으로 실제 위치의 정확성을 주장하지 않는다.
- 재생성기 3개 입력·독립 Print 14개를 SHA와 대조했고 `git diff --check`를 통과했다.
  자동 생성 harness/manifest는 ignored 로컬 파생 상태이며 이 단계 커밋에 포함하지 않는다.
  제품 3파일은 수용된 source SHA와 같아 빌드·시각 캡처를 재실행하지 않았다.

재현 명령은 다음과 같다. 빌드 target은 기존 공용 경로 한 곳을 사용했다.

```bash
python3 pdf/issue7998/retained-border-structure/generate.py
node scripts/rust-test-suite-manifest.mjs --prepare
CARGO_BUILD_JOBS=14 cargo test --locked --profile release-test \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review --features native-skia \
  --test regression_suite_012 issue_7998_pdf_retained_border_structure -- --test-threads=4
CARGO_BUILD_JOBS=14 cargo test --locked --profile release-test \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review --features native-skia \
  --test regression_suite_026 \
  issue_7998_pdf_border_structure::unsupported_phase_and_alpha_reject_without_mutating_document \
  -- --test-threads=4
```

이번 소스·정식 회귀·독립 기준·계획/보고 기록을 같은 로컬 단계 커밋으로 묶는다.
R1의 다른 스타일·가림/부분 격자와 좌표 원점, R2 및 P5는 계속 남는다. P4 전체 완료나
PR 준비 완료로 보고하지 않으며 `parser_allocation_bounds_proven=false`다. 다음 절편,
전체 회귀/lint·Worker/package·최신 devel 통합·remote push/PR 변경은 진행하지 않았다.
