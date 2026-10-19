# #7998 채움 띠의 테두리 편집 소유

- 승인: stage18 `0d637b5c4` 수용 뒤 다음 절편 진행 승인.
- 원인: `table_detection::boundaryObservations`는 실제 얇은 사각 fill을 `fill_band`로 관측하고
  선택 격자의 `sourcePathIndices`에 연결하지만, `table_paint::partition`은 stroke만 편집
  소유로 보존한다. 실제 변 변경은 원래 fill 조각을 남기므로 새 native 변과 겹친다.
- 첫 경계: 실제 단일 Rectangle·불투명 fill·무 stroke/변환의 띠 중 선택된 격자의 경계이며
  같은 축/구간의 다른 경계와 역할이 경합하지 않는 경우를 연결한다. 넓은 배경·가림 fill과
  여러 축을 한 번에 채우는 compound를 bbox로 같은 경계라고 인증하지 않는다.
- 같은 축의 다른 원본 선을 덮는 fill은 색상과 무관하게 편집 소유를 부여하지 않는다.
  기존 전체 높이 stroke 위에 올린 fill, 넓은 배경과 중첩 자식 표를 반례로 대조한다.
- 생산 `PathContent`/실제 경계 관측 → 선택 격자 → 물리 셀/페이지 조각 → 기존 소유 sidecar
  → native setter의 교체 closure와 실제 command undo/redo·양 저장을 추적한다. 높이·줄/측정·
  pagination은 바꾸지 않는다. 다축 compound는 각각의 변과 접점을 독립적으로 분해해야 하므로
  이번 첫 fill 띠 왕복과 구분하며 R1의 잔여 항목에 유지한다.
- 독립 원본 PDF 연산자로 만든 7개 입력과 기존 fill 가림·raster 경계·실물을 대조한다.
  양 편집 저장본은 같은 편집을 한 한컴 Print, Native/fresh Docker WASM 전쪽과 직접 판독으로
  확인한다. 90% 조건·직접 판정 전에는 새 rendering 회귀/fixture를 등록하지 않는다.
- 증거: ignored `output/pr-review/issue7998/p4-fill-border/` 및 같은 이름의 저장소 밖 증적 폴더.
  전체 회귀/lint·최신 devel 통합·Worker/package는 P5에 유지한다.

## 첫 구현의 검증과 배경 표현 보완 요청

첫 구현은 실제 API에서 원래 채움 조각 4→4/undo 없음 FAIL을 4→3/undo 있음 PASS로
보정했다. Native API의 2종×4동작·양 저장·원문 undo와 실제 Studio command 3종의
문자 삽입/삭제 뒤 undo/redo·양 저장 재열기도 통과했다. Native/fresh Docker WASM 각
41쪽에서 미달·누락·쪽수 차이가 없으며, 편집 Print 최저는 99.07124%/97.87336%다.
새 정식 검사 5건과 기존 집중 46건도 이 첫 구현에서 통과했다. 이는 최종 수용 판정이 아니다.

2026-10-19 메인테이너가 `fill-background-all`의 셀 안 글상자 처리가 의도인지 질문했다.
실제 저장본의 `BEFORE CHILD`/`AFTER CHILD`와 A–D는 셀의 Paragraph이며, 파란 면에는
`drawText`가 없다. 원본은 첫 셀 안쪽 5pt의 한 사각형이지만 기존 물리 분할은 부모 6개·
자식 4개의 Rectangle으로 저장한다. frozen stage18/현재 원본 재가져오기의 XML을 대조해
이 10조각과 셀 문단이 동일함을 확인했다(`background-structure-comparison.json`).

메인테이너는 **“배경 표현 추가 보완 필요”**로 판정했다. 따라서 이 41쪽의 점수·정식
검사 통과를 절편 완료로 보고하지 않는다. 원본 한 면의 개체 신원과 셀 채우기의 편집
계약을 구분하고, 표준 셀 전체 채움과 부분 채움 계약을 비교했다. 다축 compound나
R2로 넘어가지 않으며 이 경계를 보정한 새 source의 영향 검증과 직접 판정을 이어간다.

## 배경 표현 계약의 추가 승인과 구현 중 기록

메인테이너는 사각형 대신 셀 채우기 속성으로 편집하도록 지시한 뒤, 실제 native CellFill
시험본과 독립 한컴 Print를 확인할 수 있는 비교 자료를 바탕으로 **“표준 셀 전체 채움·5pt
차이 수용”**을 승인했다. 따라서 부분 채움 IR을 신설하지 않고 기존 Cell BorderFill을 쓴다.
해당 원본의 5pt inset이 셀 전체로 확장되는 것은 승인된 의미 변환이며 원본과 같은 기하라고
보고하지 않는다. 비교 기준 원본 PDF도 고치지 않는다.

`table_paint::partition`에서 실제 단일 불투명 Rectangle/원본 path를 확인하고, 경계 기여자는
제외한다. 유일한 내부 셀과 그 실제 하위 셀들의 span/image 참조, 모든 소유 내용의 포함,
원본 z-order를 확인해 한 배경을 승격한다. 일부 강조·앞쪽 도형·경합 배경·빈 inset 도형은
이 근거를 만족하지 않아 source paint를 유지한다. 실패한 표 가족의 원자적 fallback과 기존
연산 예산은 유지한다. 이 후보는 source path→선택 candidate/cell로 진단 JSON에 기록한다.

생산 `PaintPlan.cell_fills` → `structured_lowering`의 셀별 BorderFill 복제/중복제거 → 기존
`render_cell_background`/`border_fill_background_nodes`가 실제 셀 영역을 소비한다. 공유 스타일을
직접 수정하지 않아 다른 셀을 채우지 않는다. 텍스트/중첩 표·줄 높이·pagination은 바꾸지 않는다.
실제 API·Studio `SetCellBorderFillCommand`의 색 변경/지우기/문자 역편집 뒤 undo·redo·양 저장과
한컴 Print를 검증한다. Native/fresh Docker WASM 전쪽과 직접 판정은 새 source에서 진행한다.

증거 하위 폴더 `native-cell-fill-final/`에 첫 구현과 구분해 저장한다. 변경 전 실제 엔진의
`before-fill-contract.log`는 첫 셀 FillType=None으로 실패했다(환경/빌드 실패가 아니다).
앞선 41쪽과 5+46건의 결과는 앞선 source에 한정하며 이번 변경의 최종 증거로 재사용하지 않는다.

## 최종 배경 보정과 검증

- 실제 첫 셀은 `FillType=None`에서 source RGB의 Solid 셀 채움으로 바뀌었다. 원래 파란
  사각형 10조각은 0개이며, 부모 셀 문단은 `BEFORE CHILD`/빈 child host/`AFTER CHILD`,
  자식은 실제 2×2 Table과 A–D를 유지한다. 기존 Paragraph/CharShape/Cell geometry 서명도
  같다. `standard-hwpx-cell-fill-proof.json`은 사설 도형 투영이 아닌 정상 `borderFillIDRef`와
  `winBrush`를 확인한다. 초록 변경은 #20B050, 지우기는 채움 없음으로 정상 저장된다.
- 겹치는 두 배경의 첫 면을 승격하던 반례를 보완했다. `before-competing-guard.log`의
  잘못된 native fill 1개 FAIL은 최종 `negative-guards.log`의 0개/원문 20조각 보존 PASS로
  바뀌었다. 일부 강조는 2표와 원문 그림을 유지하고, 앞쪽 가림은 기존 가족 전체의 positional
  fallback(0 semantic table)을 유지한다. 모호한 면을 셀 채움으로 주장하지 않는다.
- 실제 native API의 원본/초록/없음, 테두리 2종×4동작, 양 저장·원문 undo를 통과했다.
  실제 Studio 명령 5종은 X 삽입/삭제 뒤 undo·redo·양 저장 재열기와 snapshot 0개 회수를
  통과했다(`studio-result.json`). 채움만 바꾸는 명령은 글자 역편집이 무효화한 LineSeg 캐시를
  되돌리는 명령이 아니므로 이 경우에는 복원된 셀 속성·내용 관계와 실제 저장 출력으로
  검사한다. 문자 편집 없는 채움 undo는 원본 HWPX bytes도 복원한다. 테두리 paint 명령의
  기존 원문 byte 복원 계약은 유지한다. undo/redo 뒤 HWPX 4개도 별도 시각 비교했다.
- HWPX `pattern_type`의 무늬 없음 표기는 API 0과 읽기 -1이 달라질 수 있다. 색·채움 유형·
  alpha·무늬 없음의 실제 계약을 검사하고 이 표기 차이를 색상 손실로 주장하지 않는다.
- Native/API 편집 12파일 모두 한컴 2020 11.0.0.9136에서 열기·인쇄 성공이다. method 0,
  `hancom2020_pdf_driver_one_up`이며 입력/출력 SHA와 작업 ID를 `print-sources.json`·
  `print-hwp-sources.json`에 보존했다. 최종 producer의 저장 bytes가 같은 12입력 SHA임을
  확인해 이미 끝난 Print를 재사용했다. 큐 제한으로 거절된 미등록 요청만 나중에 제출했고,
  생성된 작업을 중복 시작하지 않았다.

| 경로 | 원본 21문서/32쪽 최저 | API 저장 12파일/12쪽 최저 | 실제 Studio 저장·undo/redo 14파일/14쪽 최저 |
|---|---:|---:|---:|
| Native Skia | 96.41335% | 99.05384% | 99.05384% |
| fresh Docker WASM | 96.40682% | 97.83224% | 97.83224% |

각 backend 58쪽 전부 산출했으며 90% 미만·미측정·누락·독립 PDF와의 쪽수 차이가 없다.
94개 case manifest의 완료 목록과 `pr_review_gate.document_page_counts`를 실제 확인했다.
실루엣 전용 자동 gate는 `not_evaluated`이며 이를 자동 승인으로 바꾸지 않았다. 메인테이너가
2026-10-19 비교 화면과 실제 새 Studio 탭을 확인하고 **시각·편집 구조 통과**로 판정했다.

source의 `fill-background`와 같은 구조인 기존 `nested-periodic-background`는 같은 전체
셀 채움 계약을 적용해 각 100%에서 약 96.4%로 변한다. 승인된 5pt 확장을 원인으로 기록하며
devel 무하락이라고 보고하지 않는다. 첫 stage19의 나머지 source 27쪽 중 Native 보고서 8쪽은
1픽셀의 채널값 1 차이만 있고 raw 99.80715%는 같다. 원본 PNG·확대 전후를 비교 화면에
남겼다. 원래 Helvetica의 한컴 대체 글꼴과 가는 선 농도 차이는 별도로 표시했고 이번에
글꼴이 완전히 같아졌다고 주장하지 않는다. 내용·자식 표·다른 셀과 변의 보존을 직접 대조했다.

최종 전쪽 TSV는 `native/visual/*/silhouette.tsv`와 `wasm/visual/*/silhouette.tsv`, 요약은
`final-validation-summary.json`, source와 producer는 `final-evidence.json`에 연결했다.
실제 WASM 19,559,644 bytes의 SHA-256은
`f8f149aaa0394de9e583616de2af743d40ad32167edbddce8e79970a1b46a6de`이며
7793의 실제 CDP 4탭에서 같은 해시와 1쪽 표시를 확인했다. 메인 Studio 7700은 변경하지 않았다.
Native/fresh WASM은 같은 runtime source manifest SHA-256 `5d841b98697c99efae54a9b567992cade6a0c4a010208080bb7f440ef44f64d9`에 대응한다.

비교 패키지는 `E:\tmp\issue7998-fill-border-review\native-cell-fill-final\index.html`이다.
양 저장·독립 Print·Native/WASM PNG·동일 DPI 원점 overlay·전쪽 TSV와 강조/가림/경합 반례를
포함한다. 표준 CellFill 자체는 한컴 호환이며, 기존 source 테두리의 편집 소유 sidecar가
한컴의 source 도형 편집 계약을 확장하지 않는다는 기존 한계는 유지한다.

## 정식 집중 회귀와 남은 경계

새 공개 입력 7개는 PDF 연산자와 `generate_sources.py --output <dir>`로 재생성한다. 표준
셀 채움과 source 테두리 교체를 구분하고 관계형 검사 7건으로 등록한다. 기존 ownership
검사 1곳은 배경 사각형 수만 요구하던 조건을 실제 native 셀 채움과 양 저장의 보존으로
갱신했다. 내용 그림의 source provenance와 테두리 paint 소유 검사는 유지한다. 이 갱신은
동일 원본의 전쪽 Native/fresh WASM 90% 이상·직접 판독과 명시적 전체 셀 채움 수용에 근거한다.
정확한 좌표·baseline/golden·래칫을 완화한 것이 아니다.

기존 정식 함수 46건을 묶은 집중 harness는 최종 갱신 뒤 모두 통과했다
(`existing-guards-final.log`). 신규 7건도 최종 Native 집중 harness에서 통과했다. Cargo 자동
수집 경로의 정식 실행은 `regression_suite_019`에서 새 7건이 실제 수집·통과했다(`formal-fill-tests.log`).
기존 변경 함수는 `regression_suite_023`에 계속 등록되어 있으며 위 46건의 실제 함수 실행에
포함됐다. 019로 기존 모듈을 조회한 0건 실행은 검증 건수에 포함하지 않는다.
전체 회귀/lint·Worker/package·최신 devel 통합은 P5에 남으며 이번 절편에서 반복하지 않는다.
다축 compound closure·새 행열의 source paint 상속/삭제·복원·부분 병합 이웃 및 R2도 남는다.
`parser_allocation_bounds_proven=false`를 유지하며 P4 전체/비신뢰 입력 배포 완료를 선언하지 않는다.
원격 push·PR·GitHub comment는 수행하지 않았다.

수정 Rust의 rustfmt 검사와 `git diff --check`도 통과했다. 공개 7 PDF·7 source 기록의
바이트 동일 재생성을 확인했다(`public-fixtures.json`). 전체 검사를 반복하지 않았고 이 절편의
집중 검사 **53개 함수(신규 7+기존 46)**로 범위를 구분한다. 최종 직접 판정 뒤 runtime source는
변경하지 않았으며, 추가로 빌드·전쪽 캡처를 반복할 사유가 없어 이미 완료된 증거를 보존했다.
