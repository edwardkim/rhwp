---
kind: snapshot
status: active
canonical: mydocs/pr/archives/pr_7225_review.md
last_verified: 2026-09-17
---

# PR #7225 검토

## 최종 판정

**승인 — 요청한 머지 보류 사유 해소(검토 변경 범위).** 156676190의 3→4쪽 회귀를 Native/fresh WASM 모두 3쪽으로 복구하고 목표 경계와 정상 대조군을 검증했다. 전체 회귀·Skia·lint·fresh WASM·Visual Sweep을 완료했다. 원격 CI/merge 승인과 문서 전체 PDF 일치 판정은 별개다.

## 메인터너 보정 (2026-09-17)

음수 Percent 간격을 Fixed 겹침 해제와 구분하고, 합성 개체 줄에 표 상하 바깥여백을 포함했다. pre-flush는 실제 TAC 배치와 같은 소유 줄 상자를 사용해 바깥여백을 두 번 예약하지 않는다. TAC 뒤 전진에는 음수 간격도 반영하며, 쪽 위 간격 판독에는 정규화 전 원본 vpos를 사용한다.

156676190은 수정 전 Native/fresh WASM 4쪽에서 최종 Native/fresh WASM 모두 3쪽으로 복구했다. 첫 본문 y≈371.8, 첫 쪽 사진 표 y≈811, 2쪽 첫 본문 y≈114.47의 독립 PDF 좌표를 정식 회귀 검사에 추가했다. 양수 간격까지 일괄 변경한 중간 시도에서 결재문서5→6·셀 여백 문서6→7을 검출했으므로 그 변경을 철회했고, 기존 저장 구역의 양수 간격 호환 동작을 보존했다. 최종 Native/fresh WASM 대조 문서는 각각 5·6쪽이다.

Native overlay 1–3쪽에서 사진 누락·다음 쪽 이월과 2쪽20px 오차가 해소됐다. 3쪽의 세 표 위치는 base와 동일하며(상단98.2/순서도604.6/사진799.1px), 제목 글꼴 굵기·줄 폭·사진 배율과 PDF의 잔차는 남는다. 3쪽 사진 배율 잔차는 source 그림 폭15462/15329/16156 HU가 셀 안쪽 폭14004 HU보다 큰데 기존 셀 그림 경로가 폭을186.7px로 축소하는 데서 발생한다(`layout/table_layout.rs`의 `pic_w.min(inner_area.width)` 경로). 기준 PDF의 실제 그림 폭은206.13/204.21/215.24px로 source 선언 폭과 대응한다. 이번 #7217의 그림 자체 inMargin은 이3개 모두0이므로 그 변경의 효과가 아니다. 별도 셀 그림 확대·clipping 계약을 건드리지 않았고, 그 잔차를 해소했다고 보고하지 않는다. 이 보정은 페이지 회귀 및 예약 계약의 해결 범위이며 문서 전체 시각 일치로 확대하지 않는다. 최종 집중 검사·fresh WASM·전체 검증은 아래 공통 기록을 따른다.

### 보정 후 증적

렌더링/UI 검증 코드 head는 `54c24ebddb1a578786a6eb082c40c493dcde07f1`이다. 이후 `f94dece59`는 테스트의 동등한 역방향 탐색 보정이며 fmt·전체 target Clippy·해당 2개 테스트를 재검증했다. [최종 공통 검증](pr_7210_review.md#메인터너-보정-최종-검증)에 실행 범위와 결과를 모았다.

- [maintainer_trim_counter_wasm_compare_001.png](../assets/pr7225_review/maintainer_trim_counter_wasm_compare_001.png)
- [maintainer_trim_counter_wasm_overlay_001.png](../assets/pr7225_review/maintainer_trim_counter_wasm_overlay_001.png)
- [maintainer_trim_counter_wasm_review_001.png](../assets/pr7225_review/maintainer_trim_counter_wasm_review_001.png)
- [maintainer_trim_counter_wasm_compare_002.png](../assets/pr7225_review/maintainer_trim_counter_wasm_compare_002.png)
- [maintainer_trim_counter_wasm_overlay_002.png](../assets/pr7225_review/maintainer_trim_counter_wasm_overlay_002.png)
- [maintainer_trim_counter_wasm_review_002.png](../assets/pr7225_review/maintainer_trim_counter_wasm_review_002.png)
- [maintainer_trim_counter_wasm_compare_003.png](../assets/pr7225_review/maintainer_trim_counter_wasm_compare_003.png)
- [maintainer_trim_counter_wasm_overlay_003.png](../assets/pr7225_review/maintainer_trim_counter_wasm_overlay_003.png)
- [maintainer_trim_counter_wasm_review_003.png](../assets/pr7225_review/maintainer_trim_counter_wasm_review_003.png)
- [maintainer_consulting_wasm_compare_004.png](../assets/pr7225_review/maintainer_consulting_wasm_compare_004.png)
- [maintainer_consulting_wasm_overlay_004.png](../assets/pr7225_review/maintainer_consulting_wasm_overlay_004.png)
- [maintainer_consulting_wasm_review_004.png](../assets/pr7225_review/maintainer_consulting_wasm_review_004.png)
- [maintainer_balance_wasm_compare_004.png](../assets/pr7225_review/maintainer_balance_wasm_compare_004.png)
- [maintainer_balance_wasm_overlay_004.png](../assets/pr7225_review/maintainer_balance_wasm_overlay_004.png)
- [maintainer_balance_wasm_review_004.png](../assets/pr7225_review/maintainer_balance_wasm_review_004.png)
- [maintainer_balance_wasm_compare_005.png](../assets/pr7225_review/maintainer_balance_wasm_compare_005.png)
- [maintainer_balance_wasm_overlay_005.png](../assets/pr7225_review/maintainer_balance_wasm_overlay_005.png)
- [maintainer_balance_wasm_review_005.png](../assets/pr7225_review/maintainer_balance_wasm_review_005.png)
- [maintainer_balance_wasm_compare_006.png](../assets/pr7225_review/maintainer_balance_wasm_compare_006.png)
- [maintainer_balance_wasm_overlay_006.png](../assets/pr7225_review/maintainer_balance_wasm_overlay_006.png)
- [maintainer_balance_wasm_review_006.png](../assets/pr7225_review/maintainer_balance_wasm_review_006.png)
- [maintainer_trim_rhwp9_pdf8_wasm_compare.png](../assets/pr7225_review/maintainer_trim_rhwp9_pdf8_wasm_compare.png)
- [maintainer_trim_rhwp9_pdf8_wasm_overlay.png](../assets/pr7225_review/maintainer_trim_rhwp9_pdf8_wasm_overlay.png)
- [maintainer_trim_rhwp9_pdf8_wasm_review.png](../assets/pr7225_review/maintainer_trim_rhwp9_pdf8_wasm_review.png)
- [maintainer_trim_rhwp10_pdf9_wasm_compare.png](../assets/pr7225_review/maintainer_trim_rhwp10_pdf9_wasm_compare.png)
- [maintainer_trim_rhwp10_pdf9_wasm_overlay.png](../assets/pr7225_review/maintainer_trim_rhwp10_pdf9_wasm_overlay.png)
- [maintainer_trim_rhwp10_pdf9_wasm_review.png](../assets/pr7225_review/maintainer_trim_rhwp10_pdf9_wasm_review.png)

원래 #7196 입력은 최종 11쪽 / 기준 PDF 10쪽이다(보정 전 rhwp 12쪽). 내용을 다시 추적한 비교는 **rhwp 9쪽↔PDF 8쪽, rhwp 10쪽↔PDF 9쪽**이다. 앞쪽 끝의 “감사합니다”와 다음 쪽 “붙임3” 시작, 표 외곽·후속 내용을 직접 대조했다. 과거 10↔8 / 11↔9 PNG는 아래 수정 전 증거로만 보존하며 최종 판정에 재사용하지 않는다. 목표 경계의 불필요한 앞당김은 해소했지만 문서 전체 쪽수 일치를 주장하지 않는다.

### 보정 전 판정과 증거

**머지 보류** — 목표 경계는 개선됐지만 156676190에서 3→4쪽 회귀와 첫 페이지 그림의 다음 쪽 이동이 재현된다.

**이 아래의 코드 위치·수치·보류 판정·미실행 설명과 기존 PNG는 초기 검토 `cd074a4da`의 이력이다. 현재 판정은 문서 상단과 보정 후 증적을 따른다.**

[원 PR #7225](https://github.com/edwardkim/rhwp/pull/7225): 수정: 다음 경계에서 철회될 문단 간격 트림을 조판이 하지 않는다 (#7196)
관련 [이슈 #7196](https://github.com/edwardkim/rhwp/issues/7196).
이 판정은 아래 변경 범위의 로컬 검토 결과이며 GitHub APPROVE 제출·원격 merge와 구분한다.

## Head·통합 계보·CI

- 원 head `cf0c06f846ba37e6a96aba80a48cdc817b5b8894`, base `devel`. 검토자는 `jangster77`이다.
- source `cf0c06f846ba37e6a96aba80a48cdc817b5b8894` → applied `003dabe99e7e57e365c5b8d4a9544a115906a778`
- 통합 branch `codex/planet-review-20260917`, code head `cd074a4da`, fixture head `6600d48b2`.
- 확인한 성공 check/workflow: [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/35184654175/job/105084006101), [CI](https://github.com/edwardkim/rhwp/actions/runs/35184654271/job/105084006599), [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/35184653818/job/105084005163), [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/35184654168/job/105084006123), [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/35184654174/job/105084006183), [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/35184653836/job/105084004371), [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/35185783057). SKIPPED job은 검사 성공으로 계산하지 않는다.
- [공통 실행·전체 계보](pr_7210_review.md#통합-검토-공통-실행-기록), [처리 계획](../pr_7225_review_impl.md).

## 코드 경로와 독립 실행 증거

next_boundary_reverts_spacing_trim 조건으로 다음 vpos snap이 취소될 때 줄간격 트림도 보류한다. 여러 fit/advance 소비 지점에 반영했으나 저장 줄 정보가 없는 inline object 호스트의 후속 흐름은 다른 계약을 필요로 한다.

한컴2020 PDF=3쪽, base fcbd00e0f Native=3쪽, 통합 Native/fresh WASM=4쪽. 첫 쪽 기숙사 사진이 통합 출력에서는 다음 쪽으로 이동했다. 목표 156760012는 rhwp10↔PDF8, rhwp11↔PDF9로 의미상 대응시켜 별도 overlay도 생성했다. 목표 한쪽의 성공만으로 반례를 무시하지 않았다.

관련 실행: **issue_7196_page_top_spacing_trim_restore 1개 및 실제 3쪽 대조 문서**. Rust 전체 focused 34개 / Studio 1755개 통과.
원 PR의 수정 전 FAIL 기록은 작성자 증거이며 이번 reviewer가 소스 rollback으로 재실행한 것으로 세지 않는다.
reviewer가 비교한 base는 공통 기록의 실제 Native binary다.

## 남은 차이·보류 해제 또는 merge 전 조건

실행으로 재현된 회귀다. 작성자가 #7224로 분리했지만 새 회귀를 이슈 등록만으로 수용하지 않는다. 원문 156676190과 동일 입력 Hancom PDF를 검토 commit에 포함했다. 목표 문서 전체도 12/10쪽 차이가 남는다.

저장 줄 없는 inline object host의 점유·전진 계산을 보정해 156676190 3쪽 및 1쪽 사진/후속 내용이 PDF 위치를 유지하게 한다. #7196 목표 경계를 보존하는 정식 반례 테스트와 overlay를 추가한다.

## 공통 조판 원칙 준수 검토

| 항목 | 판정 | 근거 |
| --- | --- | --- |
| 구현 근거와 일반성 | 충족 | 특정 파일 ID 분기 없이 문서/셀/줄 속성을 사용한다. 적용 범위와 비적용 대조군을 위에 구분했다. |
| 측정·배치 일관성 | 충족 | next_boundary_reverts_spacing_trim 조건으로 다음 vpos snap이 취소될 때 줄간격 트림도 보류한다. 여러 fit/advance 소비 지점에 반영했으나 저장 줄 정보가 없는 inline object 호스트의 후속 흐름은 다른 계약을 필요로 한다. |
| 분할·이어받기 계약 | 미충족 | 컷/예약/paint의 동일성 또는 새 페이지 경계 회귀가 해소되지 않았다. |
| 줄 소속과 점유 높이 | 미충족 | 실제 줄/그림/표의 대상 의미와 검사 범위는 위 실행 증거 참조. 해당하지 않는 편집 UI에 조판 사례 전수를 요구하지 않는다. |
| 사례와 증거의 독립성 | 충족 | 공개 원문과 별도 한컴 PDF, actual Studio 입력 또는 정상 대조군 사용. 잔차를 숨기지 않았다. |
| 기준값 변경 | 비해당 | baseline/golden을 재생성하지 않았으며 실패를 허용치 증가로 해소하지 않았다. |
| 주장과 검증 범위 | 충족 | 실행 검출 결함, 코드상 우려, 미검증, 기존 차이를 구분했다. 전체 회귀·원격 CI 완료를 주장하지 않는다. |

## Visual Sweep 입력·직접 확인 범위

DPI 96, fresh WASM 및 Native. overlay 색상은 rhwp만 있는 차이 빨강 / PDF만 있는 차이 파랑 / 양쪽 내용의 색상 차이 주황 / 허용값 이내 회색이다. 자동 일치율은 보조값이며 승인 기준 자체가 아니다.

| 입력 | 기준 PDF | 직접 비교한 쪽 |
| --- | --- | --- |
| [tests/fixtures/planet_review_20260917/156676190_[교육부 02-27(목) 조간보도자료] 2026년 국립대 임대형 민자사업(BTL) 기숙사 추진.hwpx](../../../tests/fixtures/planet_review_20260917/156676190_%5B%EA%B5%90%EC%9C%A1%EB%B6%80%2002-27%28%EB%AA%A9%29%20%EC%A1%B0%EA%B0%84%EB%B3%B4%EB%8F%84%EC%9E%90%EB%A3%8C%5D%202026%EB%85%84%20%EA%B5%AD%EB%A6%BD%EB%8C%80%20%EC%9E%84%EB%8C%80%ED%98%95%20%EB%AF%BC%EC%9E%90%EC%82%AC%EC%97%85%28BTL%29%20%EA%B8%B0%EC%88%99%EC%82%AC%20%EC%B6%94%EC%A7%84.hwpx) | [pdf/planet-review-20260917/156676190-2020.pdf](../../../pdf/planet-review-20260917/156676190-2020.pdf) | 1-3 |
| [samples/issue7196/156760012_page_top_spacing_trim.hwpx](../../../samples/issue7196/156760012_page_top_spacing_trim.hwpx) | [samples/issue7196/156760012_page_top_spacing_trim-2024.pdf](../../../samples/issue7196/156760012_page_top_spacing_trim-2024.pdf) | rhwp10↔PDF8, rhwp11↔PDF9 (번호 불일치를 명시) |

입력·PDF는 이미 Git에 존재하는 경로를 재사용했고 새로 추가한 3입력/3PDF는 `6600d48b2`에서 추적한다.

### 보존한 비교 PNG

- [trim_counter_wasm_compare_001.png](../assets/pr7225_review/trim_counter_wasm_compare_001.png)
- [trim_counter_wasm_compare_002.png](../assets/pr7225_review/trim_counter_wasm_compare_002.png)
- [trim_counter_wasm_compare_003.png](../assets/pr7225_review/trim_counter_wasm_compare_003.png)
- [trim_counter_wasm_overlay_001.png](../assets/pr7225_review/trim_counter_wasm_overlay_001.png)
- [trim_counter_wasm_overlay_002.png](../assets/pr7225_review/trim_counter_wasm_overlay_002.png)
- [trim_counter_wasm_overlay_003.png](../assets/pr7225_review/trim_counter_wasm_overlay_003.png)
- [trim_counter_wasm_review_001.png](../assets/pr7225_review/trim_counter_wasm_review_001.png)
- [trim_counter_wasm_review_002.png](../assets/pr7225_review/trim_counter_wasm_review_002.png)
- [trim_counter_wasm_review_003.png](../assets/pr7225_review/trim_counter_wasm_review_003.png)
- [trim_counter_base_review_001.png](../assets/pr7225_review/trim_counter_base_review_001.png)
- [trim_counter_base_overlay_001.png](../assets/pr7225_review/trim_counter_base_overlay_001.png)
- [trim_counter_base_review_002.png](../assets/pr7225_review/trim_counter_base_review_002.png)
- [trim_counter_base_overlay_002.png](../assets/pr7225_review/trim_counter_base_overlay_002.png)
- [trim_counter_base_review_003.png](../assets/pr7225_review/trim_counter_base_review_003.png)
- [trim_counter_base_overlay_003.png](../assets/pr7225_review/trim_counter_base_overlay_003.png)
- [trim_rhwp10_pdf8_wasm_compare.png](../assets/pr7225_review/trim_rhwp10_pdf8_wasm_compare.png)
- [trim_rhwp10_pdf8_wasm_overlay.png](../assets/pr7225_review/trim_rhwp10_pdf8_wasm_overlay.png)
- [trim_rhwp10_pdf8_wasm_review.png](../assets/pr7225_review/trim_rhwp10_pdf8_wasm_review.png)
- [trim_rhwp11_pdf9_wasm_compare.png](../assets/pr7225_review/trim_rhwp11_pdf9_wasm_compare.png)
- [trim_rhwp11_pdf9_wasm_overlay.png](../assets/pr7225_review/trim_rhwp11_pdf9_wasm_overlay.png)
- [trim_rhwp11_pdf9_wasm_review.png](../assets/pr7225_review/trim_rhwp11_pdf9_wasm_review.png)

## Merge 후 contributor PR comment 계획

실제 최종 head CI와 통합 merge가 완료된 뒤 원 source PR에 한국어로 적용 commit·통합 PR·merge SHA·CI URL과 감사 인사를 남긴다.
이번 review는 아직 remote push/통합 PR/merge 단계가 아니다. 보류가 남으면 완료·이슈 종료 댓글을 게시하지 않는다.
[Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)을 연결하고,
대표 compare/review뿐 아니라 위 **standalone overlay**도 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-sha>/mydocs/pr/assets/...`로 본문에 직접 포함한다.
실제로 확인한 쪽·backend·개선 범위와 기존 차이를 함께 적는다. 다쪽 경계는 앞/뒤 쪽을 모두 포함하며 #7225는 156676190의 1–3쪽 및 추가 4쪽 해소 여부를 숨기지 않는다.
UTF-8 body 파일과 `--body-file`로 게시하고 한국어·이미지 URL·실제 head를 다시 확인한다. 관련 이슈의 남은 범위가 있으면 열린 상태를 유지한다.
