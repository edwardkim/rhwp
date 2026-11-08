# PR #8184 리뷰 — 빈 호스트와 저장 표 프레임

> **2026-10-29 통합 후보 검증:** 아래 과거 결과는 각 측정 당시 head의 기록이다. 한국어 가정통신문은 과거와 같은 글꼴·PDF 래스터 조건에서 Native 두 쪽의 렌더 PNG와 점수가 이전과 동일하다. 현재 시스템 글꼴 조건과 fresh WASM은 이 재현 결과와 구분한다. [동일 조건 근거](../../working/assets/review_integration_20261026/validation/post-rebase-8328/newsletter-historical-font-recovery.json). 통합 최종 판정과 남은 조건은 [통합 계획](../../working/review_integration_20261026_document_updates_plan.md)을 따른다.

## 최종 판정

**통합 후보 최종 판정 보류 — 빈 호스트와 저장 표 프레임의 관련 결과는 아래에 기록했다.**
원 head의 단독 실행이나 원격 승인을 뜻하지 않는다. 원 변경을 누적 적용한 아래 통합 후보에 대한 판정이며,
통합 PR의 최신 head CI와 검토가 완료되면 작업지시자가 승인한 머지와 후속 처리를 진행한다. 작업지시자가 승인한 통합 PR은 남은 검사와 미해결 기준을 처리한 뒤 제출한다.

## 접수와 적용

| 항목 | 값 |
| --- | --- |
| 원 PR·작성자·base | [#8184](https://github.com/edwardkim/rhwp/pull/8184) / planet6897 / devel |
| 체리픽 당시 원 head | `567ff9d18ac1f7e702e94042eb38a03a44f95943` |
| 2026-10-29 재조회 원 head | `cc6d06077013daf6090021ab317c347dbd05b236`; [head·본문 대조](../../working/assets/review_integration_20261026/validation/post-rebase-8328/source-pr-head-body-20261029.json), [리베이스 patch 검토](../../working/assets/review_integration_20261026/validation/post-rebase-8328/source-pr-paired-range-diff-review.md) |
| 원 PR의 이슈 연결 | 원 PR 자동 닫기 연결 없음; 통합 PR의 자동 닫기로 전환하지 않음 |
| 원 head CI 참고값 | SUCCESS; 통합 후보 CI와 구분 |
| 검증한 통합 code/test head | `2e902abacf328bccdd7780606666ac3b1c5c1ce9` |
| 시각 production source | `ae8c413f84a01a4cc4591357056183709f21b150` |
| 고정 기준 devel | `faa885e8b54ffc7cdf4196cd43a3103f362499a2` |
| 실제 적용·중복 | 24개 적용 / 0개 patch 중복 제외 |
| reviewer·조회 시점 상태 | jangster77 지정 유지 / OPEN / mergeable=MERGEABLE; 머지 전 재조회 필요 |

브랜치는 `review/integration-20261026-document-updates`다. [적용 순서·SHA·단계 기록](<../../../mydocs/pr/archives/pr_8184_review_impl.md>)과
[원 head·reviewer·댓글·스레드 재조회](<../../../mydocs/working/assets/review_integration_20261026/validation/source-pr-preflight.json>), [통합 계획·실패별 선행 분석·실제 소비 경로](<../../../mydocs/working/review_integration_20261026_document_updates_plan.md>)을 연결한다.

## 변경과 실제 검증 범위

빈 공동 호스트가 소유한 첫 표 상자·바깥여백·캡션과 후속 저장 줄의 되감김으로 쪽을 확정한다. 표 분할 정책을 후속 호스트의 쪽 소유 판정으로 대신하지 않는다.

원 PR 제목: fix: #8149 빈 host 자리차지 표의 저장 상자·쪽 바닥 첫 조각 계약 (사양서 74→71쪽)

주요 구현 경로: [float_placement.rs](<../../../src/renderer/float_placement.rs>), [stored_empty_host_frame.rs](<../../../src/renderer/float_placement/stored_empty_host_frame.rs>), [layout.rs](<../../../src/renderer/layout.rs>), [paragraph_layout.rs](<../../../src/renderer/layout/paragraph_layout.rs>), [table_layout.rs](<../../../src/renderer/layout/table_layout.rs>), [typeset.rs](<../../../src/renderer/typeset.rs>), [empty_float.rs](<../../../src/renderer/typeset/controls/empty_float.rs>).

보호 검사: [issue_8149_empty_host_float_page_bottom.rs](<../../../tests/cases/issue_8149_empty_host_float_page_bottom.rs>). 전체 nextest 또는 해당 Native/Studio 실행의 실제 결과에 포함된다.

추가 보정: `a30b10d0967e98771dd8e90d6ad6748b2f14a52e`, `80e3cb7490e05294283dd45a545e41293bdfb4de`, `607c578e3b2cf1bca5c58a0ca0e679be97f23582`. 기여자의 원 변경과 구분하며 일반성·독립 근거·수정 전후는 통합 계획에 기록했다.

## 검증 입력과 결과

[전체 로컬 실행·명령·exit·검증 SHA](<../../../mydocs/working/assets/review_integration_20261026/validation/final-rust-validation.json>)에 fmt·세 Clippy·workspace build·고정 base 정책·전체 nextest·Native 실행 결과를 보존한다.
Studio 단위 검사는 2,607개 통과·기존 skip 2개이며 실제 브라우저와 fresh 최적화 WASM도 실행했다. [production source 동일성](<../../../mydocs/working/assets/review_integration_20261026/validation/production-source-continuity.json>)과 [WASM 빌드 출처](<../../../mydocs/working/assets/review_integration_20261026/validation/final-wasm-build.json>)로 검증 head를 구분한다.

| 입력 조합 | 전체 쪽수 | Native 최저 raw | fresh WASM 최저 raw |
| --- | ---: | ---: | ---: |
| civil-form | 2 | [100.00000%](<../../../mydocs/working/assets/review_integration_20261026/validation/tsv/native/civil-form.tsv>) | [100.00000%](<../../../mydocs/working/assets/review_integration_20261026/validation/tsv/wasm/civil-form.tsv>) |
| captioned-host-2020 | 3 | [100.00000%](<../../../mydocs/working/assets/review_integration_20261026/validation/tsv/native/captioned-host-2020.tsv>) | [100.00000%](<../../../mydocs/working/assets/review_integration_20261026/validation/tsv/wasm/captioned-host-2020.tsv>) |
| headerrow-2020 | 31 | [100.00000%](<../../../mydocs/working/assets/review_integration_20261026/validation/tsv/native/headerrow-2020.tsv>) | [100.00000%](<../../../mydocs/working/assets/review_integration_20261026/validation/tsv/wasm/headerrow-2020.tsv>) |
| reservefit-2020 | 65 | [100.00000%](<../../../mydocs/working/assets/review_integration_20261026/validation/tsv/native/reservefit-2020.tsv>) | [100.00000%](<../../../mydocs/working/assets/review_integration_20261026/validation/tsv/wasm/reservefit-2020.tsv>) |
| rowfit-2020 | 33 | [100.00000%](<../../../mydocs/working/assets/review_integration_20261026/validation/tsv/native/rowfit-2020.tsv>) | [100.00000%](<../../../mydocs/working/assets/review_integration_20261026/validation/tsv/wasm/rowfit-2020.tsv>) |
| hwp5-spec-official | 71 | [95.27413%](<../../../mydocs/working/assets/review_integration_20261026/validation/tsv/native/hwp5-spec-official.tsv>) | [95.17988%](<../../../mydocs/working/assets/review_integration_20261026/validation/tsv/wasm/hwp5-spec-official.tsv>) |
| hwpml-spec-2020 | 178 | [92.45586%](<../../../mydocs/working/assets/review_integration_20261026/validation/tsv/native/hwpml-spec-2020.tsv>) | [92.83164%](<../../../mydocs/working/assets/review_integration_20261026/validation/tsv/wasm/hwpml-spec-2020.tsv>) |

해당 조합은 모두 독립 PDF의 전쪽과 대응하며 90% 미만·후보 쪽수 불일치·누락이 없다. 실제 경로·입력/PDF SHA·명령은 [전쪽 실행 원장](<../../../mydocs/working/assets/review_integration_20261026/validation/visual-validation.json>),
[커밋 blob과 실제 입력의 해시 감사](<../../../mydocs/working/assets/review_integration_20261026/validation/tracked-input-audit.json>), [실제 글꼴·PDF 래스터 출처](<../../../mydocs/working/assets/review_integration_20261026/validation/font-and-raster-provenance.json>)에 있다.

[동일 조건 base/head 판정](<../../../mydocs/working/assets/review_integration_20261026/validation/base-head-comparison.json>)에서 Native·WASM 각각 64개 조합·910개 기준 쪽을 비교했고 2%p 이상 하락은 0쪽이다. 미세 하락 여부·최대 하락은 원장의 실제 값을 따른다.
기준의 공식 사양 83쪽/71쪽과 cellnotes 12쪽/11쪽 오류는 기준 실패로 보존하고 모든 기준 쪽을 별도 진단 비교했다.

[현재 Native 확대 판독과 기존 외양 차이](<../../../mydocs/working/assets/review_integration_20261026/validation/v14-native-reading-notes.json>)에서 점수 통과·프레임 보정과 남은 글꼴·작은 표식·도식 빗금 외양 차이를 구분했다. 이를 새 회귀나 해결 완료로 바꾸어 보고하지 않는다.

공식 사양은 [실패한 새 Print와 공식 PDF 재사용 근거](<../../../mydocs/working/assets/review_integration_20261026/validation/official-spec-reference-reuse.json>)를 따른다. 공식 원문 PDF·사양·정오표는 수정하지 않았다.
공식 사양 20·31쪽의 원 PR 잔여 간격·번호 차이는 [보정 전후 동일 PNG 해시](<../../../mydocs/working/assets/review_integration_20261026/validation/official-spec-known-residuals.json>)로 구분하고 해결했다고 주장하지 않는다.

## 2026-10-28 해당 PR 쪽별 재검증

원 PR의 `hwpspec.hwp` 23쪽은 Native 99.99389% / fresh WASM 99.79148%, 54쪽은 각각 99.99765%다. 전178쪽 최저는 92.45593% / 92.83271%다. 기존 공식 HWP5 사양 원본을 함께 검사하지만 서로 다른 문서의 쪽으로 합산하지 않는다.

측정 입력·기준 PDF SHA 및 전쪽 수치는 [Native 원장](<../../../mydocs/working/assets/review_integration_20261026/validation/post-rebase-8328/original-floatfix-native-comparison.json>)과 [fresh WASM 원장](<../../../mydocs/working/assets/review_integration_20261026/validation/post-rebase-8328/original-floatfix-wasm-comparison.json>)에서 각각 확인한다.

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

## 원 PR이 제시한 공식 사양서 쪽 재측정

후속 체리픽 뒤 원 PR이 제시한 공식 사양서 33·43·47쪽을 같은 글꼴 공급과 기존 PDF 래스터 기준으로 측정했다. 세 쪽 모두 Native·fresh WASM 각각 raw 100%다. 동일 PDF를 현재 방식으로 다시 래스터화한 측정은 양쪽 모두 33·43쪽 100%, 47쪽 99.98%다. 입력과 PDF SHA, 각 쪽의 출력·기준 래스터 SHA 및 두 결과를 [검증 증거](../../working/assets/review_integration_20261026/validation/post-rebase-8328/pr8184-spec-claims/evidence.json)에 기록했다. 추가로 공식 사양서 전 71쪽도 같은 기존 PDF 래스터로 재측정했다. Native·fresh WASM 각각 전쪽 점수가 후속 체리픽 전과 같아 0.1%p 이상 하락은 없으며 최저는 각각 95.31577% / 95.19516%다. [전쪽 비교·TSV](../../working/assets/review_integration_20261026/validation/post-rebase-8328/pr8184-spec-all/evidence.json).
