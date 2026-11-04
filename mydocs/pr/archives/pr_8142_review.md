# PR #8142 리뷰 — 병합 칸의 안쪽 행 종료

> **2026-10-29 통합 후보 검증:** 아래 과거 결과는 각 측정 당시 head의 기록이다. 한국어 가정통신문은 과거와 같은 글꼴·PDF 래스터 조건에서 Native 두 쪽의 렌더 PNG와 점수가 이전과 동일하다. 현재 시스템 글꼴 조건과 fresh WASM은 이 재현 결과와 구분한다. [동일 조건 근거](../../working/assets/review_integration_20261026/validation/post-rebase-8328/newsletter-historical-font-recovery.json). 통합 최종 판정과 남은 조건은 [통합 계획](../../working/review_integration_20261026_document_updates_plan.md)을 따른다.

## 최종 판정

**통합 후보 최종 판정 보류 — 병합 칸의 안쪽 행 종료의 관련 결과는 아래에 기록했다.**
원 head의 단독 실행이나 원격 승인을 뜻하지 않는다. 원 변경을 누적 적용한 아래 통합 후보에 대한 판정이며,
통합 PR의 최신 head CI와 검토가 완료되면 작업지시자가 승인한 머지와 후속 처리를 진행한다. 작업지시자가 승인한 통합 PR은 남은 검사와 미해결 기준을 처리한 뒤 제출한다.

## 접수와 적용

| 항목 | 값 |
| --- | --- |
| 원 PR·작성자·base | [#8142](https://github.com/edwardkim/rhwp/pull/8142) / planet6897 / devel |
| 체리픽 당시 원 head | `e1abc4c7f149268ea63995eeedeb02cfcc92e5cd` |
| 2026-10-29 재조회 원 head | `500f5b8825f0b5cc7c4409b04d2a53e0cbdb2ed8`; [head·본문 대조](../../working/assets/review_integration_20261026/validation/post-rebase-8328/source-pr-head-body-20261029.json), [리베이스 patch 검토](../../working/assets/review_integration_20261026/validation/post-rebase-8328/source-pr-paired-range-diff-review.md) |
| 원 PR의 이슈 연결 | 원 PR 자동 닫기 연결 없음; 통합 PR의 자동 닫기로 전환하지 않음 |
| 원 head CI 참고값 | SUCCESS; 통합 후보 CI와 구분 |
| 검증한 통합 code/test head | `2e902abacf328bccdd7780606666ac3b1c5c1ce9` |
| 시각 production source | `ae8c413f84a01a4cc4591357056183709f21b150` |
| 고정 기준 devel | `faa885e8b54ffc7cdf4196cd43a3103f362499a2` |
| 실제 적용·중복 | 3개 적용 / 0개 patch 중복 제외 |
| reviewer·조회 시점 상태 | jangster77 지정 유지 / OPEN / mergeable=MERGEABLE; 머지 전 재조회 필요 |

브랜치는 `review/integration-20261026-document-updates`다. [적용 순서·SHA·단계 기록](<../../../mydocs/pr/archives/pr_8142_review_impl.md>)과
[원 head·reviewer·댓글·스레드 재조회](<../../../mydocs/working/assets/review_integration_20261026/validation/source-pr-preflight.json>), [통합 계획·실패별 선행 분석·실제 소비 경로](<../../../mydocs/working/review_integration_20261026_document_updates_plan.md>)을 연결한다.

## 변경과 실제 검증 범위

병합 칸 내부 행의 실제 내용·캡션과 남은 쪽 예산을 대조한다. 내용이 수용되는 안쪽 행은 쪽 끝에서 닫고, 실제 요구 높이와 부분 배치가 다른 상자를 소비하지 않도록 한다.

원 PR 제목: fix(typeset): 병합 칸 안쪽 행은 내용이 들어가면 쪽 끝에서 끝낸다

주요 구현 경로: [row.rs](<../../../src/renderer/typeset/table/scan/row.rs>), [row_step.rs](<../../../src/renderer/typeset/table/scan/runner/row_step.rs>).

보호 검사: [captioned_float_table_stored_frame.rs](<../../../tests/cases/captioned_float_table_stored_frame.rs>), [rowspan_band_fits_remaining_page.rs](<../../../tests/cases/rowspan_band_fits_remaining_page.rs>). 전체 nextest 또는 해당 Native/Studio 실행의 실제 결과에 포함된다.

이 PR에 별도 단독 보정은 추가하지 않았다. 누적 통합에 필요한 컴파일 호환 보정과 다른 PR의 공통 경로 보정은 통합 후보에 포함된다.

## 검증 입력과 결과

[전체 로컬 실행·명령·exit·검증 SHA](<../../../mydocs/working/assets/review_integration_20261026/validation/final-rust-validation.json>)에 fmt·세 Clippy·workspace build·고정 base 정책·전체 nextest·Native 실행 결과를 보존한다.
Studio 단위 검사는 2,607개 통과·기존 skip 2개이며 실제 브라우저와 fresh 최적화 WASM도 실행했다. [production source 동일성](<../../../mydocs/working/assets/review_integration_20261026/validation/production-source-continuity.json>)과 [WASM 빌드 출처](<../../../mydocs/working/assets/review_integration_20261026/validation/final-wasm-build.json>)로 검증 head를 구분한다.

| 입력 조합 | 전체 쪽수 | Native 최저 raw | fresh WASM 최저 raw |
| --- | ---: | ---: | ---: |
| caption-negative-offset | 2 | [97.66789%](<../../../mydocs/working/assets/review_integration_20261026/validation/tsv/native/caption-negative-offset.tsv>) | [97.62499%](<../../../mydocs/working/assets/review_integration_20261026/validation/tsv/wasm/caption-negative-offset.tsv>) |
| rowspan-band-fit | 9 | [97.83595%](<../../../mydocs/working/assets/review_integration_20261026/validation/tsv/native/rowspan-band-fit.tsv>) | [97.51391%](<../../../mydocs/working/assets/review_integration_20261026/validation/tsv/wasm/rowspan-band-fit.tsv>) |
| rowspan-header-original | 52 | [98.39081%](<../../../mydocs/working/assets/review_integration_20261026/validation/tsv/native/rowspan-header-original.tsv>) | [98.37914%](<../../../mydocs/working/assets/review_integration_20261026/validation/tsv/wasm/rowspan-header-original.tsv>) |

해당 조합은 모두 독립 PDF의 전쪽과 대응하며 90% 미만·후보 쪽수 불일치·누락이 없다. 실제 경로·입력/PDF SHA·명령은 [전쪽 실행 원장](<../../../mydocs/working/assets/review_integration_20261026/validation/visual-validation.json>),
[커밋 blob과 실제 입력의 해시 감사](<../../../mydocs/working/assets/review_integration_20261026/validation/tracked-input-audit.json>), [실제 글꼴·PDF 래스터 출처](<../../../mydocs/working/assets/review_integration_20261026/validation/font-and-raster-provenance.json>)에 있다.

[동일 조건 base/head 판정](<../../../mydocs/working/assets/review_integration_20261026/validation/base-head-comparison.json>)에서 Native·WASM 각각 64개 조합·910개 기준 쪽을 비교했고 2%p 이상 하락은 0쪽이다. 미세 하락 여부·최대 하락은 원장의 실제 값을 따른다.
기준의 공식 사양 83쪽/71쪽과 cellnotes 12쪽/11쪽 오류는 기준 실패로 보존하고 모든 기준 쪽을 별도 진단 비교했다.

## 2026-10-28 해당 PR 쪽별 재검증

원 PR 입력 `rowspan-band-fit.hwp` 9쪽은 Native·fresh WASM 각각 100.00000%, 전9쪽 최저 97.84306% / 97.50188%다. 이어지는 표의 독립 Print 위·아래 경계 좌표를 고정 검사로 보호한다.

측정 원본과 기준 PDF의 SHA 및 모든 쪽 수치는 [Native 원장](<../../../mydocs/working/assets/review_integration_20261026/validation/post-rebase-8328/original-floatfix-native-comparison.json>)과 [fresh WASM 원장](<../../../mydocs/working/assets/review_integration_20261026/validation/post-rebase-8328/original-floatfix-wasm-comparison.json>)에 있다. [변경 검사 정상 실행 기록](<../../../mydocs/working/assets/review_integration_20261026/validation/post-rebase-8328/targeted-coordinate-tests.json>)은 전체 회귀 재실행이 아니다.

## 조판 원칙 준수 판정

| 항목 | 판정과 근거 |
| --- | --- |
| 제출 전 로컬 검증 | 완료된 공통 검증은 [선택 회귀 110개](../../working/assets/review_integration_20261026/validation/post-rebase-8328/fontless-branch-regression.json), [후속 회귀 24개](../../working/assets/review_integration_20261026/validation/post-rebase-8328/pr8184-after-targeted-nextest.json), [Rust lint·build](../../working/assets/review_integration_20261026/validation/post-rebase-8328/pr-local-rust-gates-20261029.json)과 [시각 도구·CI 구문](../../working/assets/review_integration_20261026/validation/post-rebase-8328/prepr-visual-ci-checks.json)에 기록한다. 전체 Rust 회귀는 작업지시자 지시에 따라 재실행하지 않았고 원격 CI는 별도다. |
| 기존 해결의 선행 분석 | 충족 — [통합 계획·실패별 선행 분석·실제 소비 경로](<../../../mydocs/working/review_integration_20261026_document_updates_plan.md>); 원 이슈·원 해결·보정 SHA를 실패별로 추적 |
| 구현 근거와 일반성 | 충족 — 위 실제 소비 경로와 독립 기대값. 문서 ID 보정·출력 은폐 없이 공통 결과 사용 |
| 측정·배치 일관성 | 충족 — 공통 계산·저장 줄·프레임 결과의 실제 소비와 전쪽 Native/WASM 출력 대조 |
| 리베이스 기능 보존 | 충족 — #8116 새 base의 15개 계약·4개 문서와 기존 17개 계약·전쪽을 함께 검증. 한 충돌에서 두 수용 조건을 보존했으며 커밋 대조와 실행 원장에 기록 |
| 최종 출력 소비 | 충족 — 실제 Native PNG·fresh WASM Canvas, 적용되는 CanvasKit 경로를 구분 |
| 분할·이어받기 | 충족 — 실제 컷·요구/예약 높이·배치·후속 소유를 관련 회귀와 독립 전쪽 Print로 대조 |
| 줄 소속과 점유 높이 | 충족 — 원본 LineSeg/개체 줄·공통 flow 결과·실제 문단/개체 소유 대조 |
| 독립성·반례 | 충족 — 실제 원본·독립 Print 또는 원본 payload/외부 경로·편집 대조군. 임시 스크립트를 정식 회귀로 대신하지 않음 |
| 기준값 변경 | 비해당 — 이 PR의 baseline·golden·허용치를 완화하지 않음 |
| 회귀 추가 선행 조건 | 충족 — 관련 입력 전쪽 Native와 해당 보강 단계의 fresh WASM 최저 90% 이상 선행 확인. 보강 회귀는 소유·순서·내용 포함 관계이며 최종 최적화 WASM도 별도 완료 |
| 수식·간격 확대 판독 | 충족 — 관련 변경·최저 쪽과 독립 Print의 직접 비교. 원 PR 잔여 차이·정책 fallback은 별도 공개 |
| 구현 주장과 검증 범위 | 충족 — 고정 source와 실제 결과를 연결. 원 head 단독 CI·정량 성능·미실행 경로로 범위 확장하지 않음 |

## 시각 증적과 남은 조건

[대표 비교·overlay](<../../../mydocs/working/assets/review_integration_20261026/validation/review-panels.json>)와 [전쪽·확대 증거 목록](<../../../mydocs/working/assets/review_integration_20261026/validation/README.md>)을 연결한다. 시각 지표는 색상 경계 보정 전 raw 실루엣이며 글꼴 예외로 gate를 우회하지 않았다.

원 fork head는 변경하지 않았다. 실제 편람 수정 #8268·충돌 #8161은 제외하며 추가 PR을 체리픽하지 않는다.

## Merge 후 contributor comment 계획

현재는 게시·close·merge를 실행하지 않는다. 추후 승인된 통합 머지 뒤 최종 merge SHA·CI URL과
Visual Sweep 정본·SHA 고정 raw 이미지·원 기여와 보정 범위·남은 차이를 정중한 한국어로 안내한다.
작업지시자가 승인한 후속 처리로 본문 파일을 게시하고 API에서 실제 내용과 링크를 재확인한다.

## 원 PR이 직접 판독한 8·9쪽

원 PR은 병합 셀이 끝나는 8쪽과 이어지는 9쪽을 직접 비교했다. 기여자 고정 커밋과 동일한 [8쪽 Native review](../../report/rowspan-band-fit/rowspan_band_fit_p008_head_review.png), [8쪽 overlay](../../report/rowspan-band-fit/rowspan_band_fit_p008_head_overlay.png), [9쪽 수정 전](../../report/rowspan-band-fit/rowspan_band_fit_p009_base_review.png), [9쪽 Native](../../report/rowspan-band-fit/rowspan_band_fit_p009_head_review.png), [9쪽 WASM](../../report/rowspan-band-fit/rowspan_band_fit_p009_wasm_review.png)을 연결한다.

별도 통합 후보의 기존 PDF 래스터 기준 8쪽은 Native 97.84306% / WASM 97.50188%, 9쪽은 양쪽 100.00000%다. 후속 표 보정 전후 두 쪽의 렌더 PNG는 각 경로에서 같았다. [원본·기준·이미지와 쪽별 해시](../../working/assets/review_integration_20261026/validation/post-rebase-8328/pr8142-source-pages8-9.json). 원 PR 이미지 실행과 통합 후보 측정은 구분한다.
