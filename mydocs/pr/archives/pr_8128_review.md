# PR #8128 리뷰 — 이어지는 쪽의 저장 reset과 자기 각주

> **2026-10-29 통합 후보 검증:** 아래 과거 결과는 각 측정 당시 head의 기록이다. 한국어 가정통신문은 과거와 같은 글꼴·PDF 래스터 조건에서 Native 두 쪽의 렌더 PNG와 점수가 이전과 동일하다. 현재 시스템 글꼴 조건과 fresh WASM은 이 재현 결과와 구분한다. [동일 조건 근거](../../working/assets/review_integration_20261026/validation/post-rebase-8328/newsletter-historical-font-recovery.json). 통합 최종 판정과 남은 조건은 [통합 계획](../../working/review_integration_20261026_document_updates_plan.md)을 따른다.

## 최종 판정

**통합 후보 최종 판정 보류 — 이어지는 쪽의 저장 reset과 자기 각주의 관련 결과는 아래에 기록했다.**
원 head의 단독 실행이나 원격 승인을 뜻하지 않는다. 원 변경을 누적 적용한 아래 통합 후보에 대한 판정이며,
통합 PR의 최신 head CI와 검토가 완료되면 작업지시자가 승인한 머지와 후속 처리를 진행한다. 작업지시자가 승인한 통합 PR은 남은 검사와 미해결 기준을 처리한 뒤 제출한다.

## 접수와 적용

| 항목 | 값 |
| --- | --- |
| 원 PR·작성자·base | [#8128](https://github.com/edwardkim/rhwp/pull/8128) / planet6897 / devel |
| 체리픽 당시 원 head | `21c40f21c5ee019fdf1ee4e68bdfeed35d3ae26c` |
| 2026-10-29 재조회 원 head | `063763a3a374d32d2b74f031c1603c317334e661`; [head·본문 대조](../../working/assets/review_integration_20261026/validation/post-rebase-8328/source-pr-head-body-20261029.json), [리베이스 patch 검토](../../working/assets/review_integration_20261026/validation/post-rebase-8328/source-pr-paired-range-diff-review.md) |
| 원 PR의 이슈 연결 | 원 PR 자동 닫기 연결 없음; 통합 PR의 자동 닫기로 전환하지 않음 |
| 원 head CI 참고값 | SUCCESS; 통합 후보 CI와 구분 |
| 검증한 통합 code/test head | `2e902abacf328bccdd7780606666ac3b1c5c1ce9` |
| 시각 production source | `ae8c413f84a01a4cc4591357056183709f21b150` |
| 고정 기준 devel | `faa885e8b54ffc7cdf4196cd43a3103f362499a2` |
| 실제 적용·중복 | 3개 적용 / 0개 patch 중복 제외 |
| reviewer·조회 시점 상태 | jangster77 지정 유지 / OPEN / mergeable=MERGEABLE; 머지 전 재조회 필요 |

브랜치는 `review/integration-20261026-document-updates`다. [적용 순서·SHA·단계 기록](<../../../mydocs/pr/archives/pr_8128_review_impl.md>)과
[원 head·reviewer·댓글·스레드 재조회](<../../../mydocs/working/assets/review_integration_20261026/validation/source-pr-preflight.json>), [통합 계획·실패별 선행 분석·실제 소비 경로](<../../../mydocs/working/review_integration_20261026_document_updates_plan.md>)을 연결한다.

## 변경과 실제 검증 범위

이어지는 쪽의 다음 저장 위치 되감김을 경계로 소비하고, 각주는 원래 문단 번호만으로 앞 쪽에 귀속시키지 않고 실제 표시된 조각에 둔다.

원 PR 제목: fix(typeset): 이어지는 쪽에서도 같은 문단의 다음 저장 쪽 reset 을 경계로 쓰고 그 조각의 자기 각주를 표시 쪽에 둔다 (#8126)

주요 구현 경로: [body.rs](<../../../src/renderer/typeset/notes/footnotes/body.rs>), [paragraph.rs](<../../../src/renderer/typeset/paragraph.rs>), [scan.rs](<../../../src/renderer/typeset/paragraph/scan.rs>), [state.rs](<../../../src/renderer/typeset/state.rs>), [commands.rs](<../../../src/renderer/typeset/state/commands.rs>), [data.rs](<../../../src/renderer/typeset/state/data.rs>), [transition.rs](<../../../src/renderer/typeset/state/transition.rs>).

보호 검사: [continuation_stored_reset_own_notes.rs](<../../../tests/cases/continuation_stored_reset_own_notes.rs>). 전체 nextest 또는 해당 Native/Studio 실행의 실제 결과에 포함된다.

이 PR에 별도 단독 보정은 추가하지 않았다. 누적 통합에 필요한 컴파일 호환 보정과 다른 PR의 공통 경로 보정은 통합 후보에 포함된다.

## 검증 입력과 결과

[전체 로컬 실행·명령·exit·검증 SHA](<../../../mydocs/working/assets/review_integration_20261026/validation/final-rust-validation.json>)에 fmt·세 Clippy·workspace build·고정 base 정책·전체 nextest·Native 실행 결과를 보존한다.
Studio 단위 검사는 2,607개 통과·기존 skip 2개이며 실제 브라우저와 fresh 최적화 WASM도 실행했다. [production source 동일성](<../../../mydocs/working/assets/review_integration_20261026/validation/production-source-continuity.json>)과 [WASM 빌드 출처](<../../../mydocs/working/assets/review_integration_20261026/validation/final-wasm-build.json>)로 검증 head를 구분한다.

| 입력 조합 | 전체 쪽수 | Native 최저 raw | fresh WASM 최저 raw |
| --- | ---: | ---: | ---: |
| cont_reset_F20_L70_N0 | 3 | [100.00000%](<../../../mydocs/working/assets/review_integration_20261026/validation/tsv/native/cont_reset_F20_L70_N0.tsv>) | [99.99070%](<../../../mydocs/working/assets/review_integration_20261026/validation/tsv/wasm/cont_reset_F20_L70_N0.tsv>) |
| cont_reset_F20_L70_N4 | 3 | [99.99760%](<../../../mydocs/working/assets/review_integration_20261026/validation/tsv/native/cont_reset_F20_L70_N4.tsv>) | [99.99070%](<../../../mydocs/working/assets/review_integration_20261026/validation/tsv/wasm/cont_reset_F20_L70_N4.tsv>) |
| cont_reset_F20_L70_N6 | 3 | [99.99840%](<../../../mydocs/working/assets/review_integration_20261026/validation/tsv/native/cont_reset_F20_L70_N6.tsv>) | [99.98502%](<../../../mydocs/working/assets/review_integration_20261026/validation/tsv/wasm/cont_reset_F20_L70_N6.tsv>) |

해당 조합은 모두 독립 PDF의 전쪽과 대응하며 90% 미만·후보 쪽수 불일치·누락이 없다. 실제 경로·입력/PDF SHA·명령은 [전쪽 실행 원장](<../../../mydocs/working/assets/review_integration_20261026/validation/visual-validation.json>),
[커밋 blob과 실제 입력의 해시 감사](<../../../mydocs/working/assets/review_integration_20261026/validation/tracked-input-audit.json>), [실제 글꼴·PDF 래스터 출처](<../../../mydocs/working/assets/review_integration_20261026/validation/font-and-raster-provenance.json>)에 있다.

[동일 조건 base/head 판정](<../../../mydocs/working/assets/review_integration_20261026/validation/base-head-comparison.json>)에서 Native·WASM 각각 64개 조합·910개 기준 쪽을 비교했고 2%p 이상 하락은 0쪽이다. 미세 하락 여부·최대 하락은 원장의 실제 값을 따른다.
기준의 공식 사양 83쪽/71쪽과 cellnotes 12쪽/11쪽 오류는 기준 실패로 보존하고 모든 기준 쪽을 별도 진단 비교했다.

## 2026-10-28 해당 PR 쪽별 재검증

원 PR의 `cont_reset_F20_L70_N0.hwp`, `N4.hwp`, `N6.hwp` 2쪽은 각각 Native/WASM 100.00000% / 100.00000%, 99.99600% / 99.99287%, 100.00000% / 99.99127%다. 세 문서 모두 전3쪽이며, 2쪽 본문 마지막 글줄을 독립 Print 좌표와 대조한다.

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

## 원 PR이 직접 판독한 3쪽

원 PR은 N4의 3쪽에서 수정 전·후 본문과 후행 문단을 비교하고 N6의 3쪽도 확인했다. 기여자 고정 커밋과 동일한 [N4 수정 전](../../report/continuation_stored_reset_own_notes/cont_reset_N4_p003_base_review.png), [N4 수정 후 Native](../../report/continuation_stored_reset_own_notes/cont_reset_N4_p003_head_review.png), [N4 수정 후 WASM](../../report/continuation_stored_reset_own_notes/cont_reset_N4_p003_wasm_review.png), [N6 Native overlay](../../report/continuation_stored_reset_own_notes/cont_reset_N6_p003_head_overlay.png)를 연결한다.

별도 통합 후보에서 N0·N4·N6의 3쪽은 Native·WASM 모두 기존 PDF 래스터 기준 raw 100%였고, 후속 표 보정 전후 렌더 PNG가 각각 같다. [원본·PDF·이미지와 쪽별 해시](../../working/assets/review_integration_20261026/validation/post-rebase-8328/pr8128-source-page3.json). 원 PR 이미지의 실행과 통합 후보 측정은 구분한다.
