# PR #8213 리뷰 — 중첩 표 높이와 물리 끝 프레임

> **2026-10-29 통합 후보 검증:** 아래 과거 결과는 각 측정 당시 head의 기록이다. 한국어 가정통신문은 과거와 같은 글꼴·PDF 래스터 조건에서 Native 두 쪽의 렌더 PNG와 점수가 이전과 동일하다. 현재 시스템 글꼴 조건과 fresh WASM은 이 재현 결과와 구분한다. [동일 조건 근거](../../working/assets/review_integration_20261026/validation/post-rebase-8328/newsletter-historical-font-recovery.json). 통합 최종 판정과 남은 조건은 [통합 계획](../../working/review_integration_20261026_document_updates_plan.md)을 따른다.

## 최종 판정

**통합 후보 최종 판정 보류 — 중첩 표 높이와 물리 끝 프레임의 관련 결과는 아래에 기록했다.**
원 head의 단독 실행이나 원격 승인을 뜻하지 않는다. 원 변경을 누적 적용한 아래 통합 후보에 대한 판정이며,
통합 PR의 최신 head CI와 검토가 완료되면 작업지시자가 승인한 머지와 후속 처리를 진행한다. 작업지시자가 승인한 통합 PR은 남은 검사와 미해결 기준을 처리한 뒤 제출한다.

## 접수와 적용

| 항목 | 값 |
| --- | --- |
| 원 PR·작성자·base | [#8213](https://github.com/edwardkim/rhwp/pull/8213) / planet6897 / devel |
| 체리픽 당시 원 head | `d9aab2292be0da4c71dc2c28665fd9ddc8c07a59` |
| 2026-10-29 재조회 원 head | `73d77a2a05f8d9523aedd0e53b22ed1faca14b8a`; [head·본문 대조](../../working/assets/review_integration_20261026/validation/post-rebase-8328/source-pr-head-body-20261029.json), [리베이스 patch 검토](../../working/assets/review_integration_20261026/validation/post-rebase-8328/source-pr-paired-range-diff-review.md) |
| 원 PR의 이슈 연결 | 원 PR 자동 닫기 연결 없음; 통합 PR의 자동 닫기로 전환하지 않음 |
| 원 head CI 참고값 | SUCCESS; 통합 후보 CI와 구분 |
| 검증한 통합 code/test head | `2e902abacf328bccdd7780606666ac3b1c5c1ce9` |
| 시각 production source | `ae8c413f84a01a4cc4591357056183709f21b150` |
| 고정 기준 devel | `faa885e8b54ffc7cdf4196cd43a3103f362499a2` |
| 실제 적용·중복 | 7개 적용 / 10개 patch 중복 제외 |
| reviewer·조회 시점 상태 | jangster77 지정 유지 / OPEN / mergeable=MERGEABLE; 머지 전 재조회 필요 |

브랜치는 `review/integration-20261026-document-updates`다. [적용 순서·SHA·단계 기록](<../../../mydocs/pr/archives/pr_8213_review_impl.md>)과
[원 head·reviewer·댓글·스레드 재조회](<../../../mydocs/working/assets/review_integration_20261026/validation/source-pr-preflight.json>), [통합 계획·실패별 선행 분석·실제 소비 경로](<../../../mydocs/working/review_integration_20261026_document_updates_plan.md>)을 연결한다.

## 변경과 실제 검증 범위

중첩 표 본체 높이·제목행·행 상자 판정의 반복 계산을 재사용한다. 편집 뒤 fresh render와 같아야 하며, 별도 보정은 저장 개체 줄과 셀의 남은 물리 상자를 같은 컷·예약·실제 배치에서 소비한다.

원 PR 제목: perf(table): 표 렌더의 중첩 표 높이·제목행 수·행 상자 판정 반복 제거 (#8212)

주요 구현 경로: [document.rs](<../../../src/document_core/commands/document.rs>), [raw_provenance.rs](<../../../src/model/raw_provenance.rs>), [digest_encoding.rs](<../../../src/model/raw_provenance/digest_encoding.rs>), [section_tail.rs](<../../../src/model/section_tail.rs>), [mod.rs](<../../../src/parser/mod.rs>), [stored_cell_reflow.inc.rs](<../../../src/renderer/composer/stored_cell_reflow.inc.rs>), [float_placement.rs](<../../../src/renderer/float_placement.rs>).

보호 검사: [nested_table_height_cache_contract.rs](<../../../tests/cases/nested_table_height_cache_contract.rs>). 전체 nextest 또는 해당 Native/Studio 실행의 실제 결과에 포함된다.

추가로 검사와 같은 중첩 셀 편집을 저장·재열기한 입력에 대해 한컴 2020 MCP Print와 18쪽을 비교했다. 2쪽 Native 58.54219% / fresh WASM 91.73536%, 3쪽 26.59257% / 26.64294%, 4쪽 46.02103% / 46.09464%이며 전쪽 최저는 3쪽이다. 편집 저장본 비교와 검사 자체의 메모리 내 SVG 캐시 동등성은 구분한다. 이 입력은 98% 미만이므로 해당 검사는 관계형을 유지한다. [편집본 출력·전쪽 수치](<../../../mydocs/pr/assets/pr8213-nested-cache-edit/evidence.json>).

추가 보정: `db096cb48ae1b113cbe9c0d3c6dce7986ac3e9df`, `dceccce7947325cb28f02501dabc9a46b2005cb6`, `1785fc69eb278c2aaba82a62a958af62c949e3f8`, `4816eda4121b317d6ade8a9bec1520518795a54f`, `358e256f92354a1a9686867a38f478a36adde734`, `607c578e3b2cf1bca5c58a0ca0e679be97f23582`, `8fd43e0cadc07674f2a7ad0d4489592381c7bdb1`, `eaa4918972d0d7354641c72b0bc8bf38d5954bed`, `92f41d036fe4cd9f04348a97e36d5e07b7d3cf0c`. 기여자의 원 변경과 구분하며 일반성·독립 근거·수정 전후는 통합 계획에 기록했다.

성능 판정은 반복 판정의 이동·캐시 무효화·저장 바이트와 출력 무회귀에 한정한다. 기여자의 장비별 가속률이나 비공개 편람의 수치를 재현했다고 주장하지 않는다.

## 검증 입력과 결과

[전체 로컬 실행·명령·exit·검증 SHA](<../../../mydocs/working/assets/review_integration_20261026/validation/final-rust-validation.json>)에 fmt·세 Clippy·workspace build·고정 base 정책·전체 nextest·Native 실행 결과를 보존한다.
Studio 단위 검사는 2,607개 통과·기존 skip 2개이며 실제 브라우저와 fresh 최적화 WASM도 실행했다. [production source 동일성](<../../../mydocs/working/assets/review_integration_20261026/validation/production-source-continuity.json>)과 [WASM 빌드 출처](<../../../mydocs/working/assets/review_integration_20261026/validation/final-wasm-build.json>)로 검증 head를 구분한다.

| 입력 조합 | 전체 쪽수 | Native 최저 raw | fresh WASM 최저 raw |
| --- | ---: | ---: | ---: |
| newsletter-chinese | 2 | [93.55035%](<../../../mydocs/working/assets/review_integration_20261026/validation/tsv/native/newsletter-chinese.tsv>) | [92.66758%](<../../../mydocs/working/assets/review_integration_20261026/validation/tsv/wasm/newsletter-chinese.tsv>) |
| nested-height-cache | 17 | [94.38594%](<../../../mydocs/working/assets/review_integration_20261026/validation/tsv/native/nested-height-cache.tsv>) | [94.22950%](<../../../mydocs/working/assets/review_integration_20261026/validation/tsv/wasm/nested-height-cache.tsv>) |
| newsletter-korean | 2 | [93.09499%](<../../../mydocs/working/assets/review_integration_20261026/validation/tsv/native/newsletter-korean.tsv>) | [91.58176%](<../../../mydocs/working/assets/review_integration_20261026/validation/tsv/wasm/newsletter-korean.tsv>) |
| endnotes-november | 21 | [90.81277%](<../../../mydocs/working/assets/review_integration_20261026/validation/tsv/native/endnotes-november.tsv>) | [91.32974%](<../../../mydocs/working/assets/review_integration_20261026/validation/tsv/wasm/endnotes-november.tsv>) |
| endnotes-september | 23 | [95.03445%](<../../../mydocs/working/assets/review_integration_20261026/validation/tsv/native/endnotes-september.tsv>) | [96.35945%](<../../../mydocs/working/assets/review_integration_20261026/validation/tsv/wasm/endnotes-september.tsv>) |

해당 조합은 모두 독립 PDF의 전쪽과 대응하며 90% 미만·후보 쪽수 불일치·누락이 없다. 실제 경로·입력/PDF SHA·명령은 [전쪽 실행 원장](<../../../mydocs/working/assets/review_integration_20261026/validation/visual-validation.json>),
[커밋 blob과 실제 입력의 해시 감사](<../../../mydocs/working/assets/review_integration_20261026/validation/tracked-input-audit.json>), [실제 글꼴·PDF 래스터 출처](<../../../mydocs/working/assets/review_integration_20261026/validation/font-and-raster-provenance.json>)에 있다.

[동일 조건 base/head 판정](<../../../mydocs/working/assets/review_integration_20261026/validation/base-head-comparison.json>)에서 Native·WASM 각각 64개 조합·910개 기준 쪽을 비교했고 2%p 이상 하락은 0쪽이다. 미세 하락 여부·최대 하락은 원장의 실제 값을 따른다.
기준의 공식 사양 83쪽/71쪽과 cellnotes 12쪽/11쪽 오류는 기준 실패로 보존하고 모든 기준 쪽을 별도 진단 비교했다.

중첩 표 원본의 새 Hancom 2020 Print를 사용했다. 11쪽 Native 98.82287%·WASM 98.83280% 확인 후 기존 잘못된 Top [1.88,3.76] 핀을 Center·내용 포함으로 정정했다. 실제 부모 clip 대조군을 보존한 5개가 통과했다. 초기 전체 검사에서 확인한 온새미로 쪽수·중첩 원점 중복·같은 물리 글줄 정렬도 공통 경로에서 보정했다. civil 원본과 top margin만 바꾼 독립 Print 쌍은 뒤 표가 −1.9173px 이동해 기존 0 이동 기대값의 오류를 입증했으며 허용치를 유지한 채 원본 여백과의 관계로 정정했다. [수정 전 6 FAIL / 수정 후 6 PASS](<../../../mydocs/working/assets/review_integration_20261026/validation/source-frame-before-after-proof.json>) 및 [추가 정렬 수정 전 FAIL / 현재 7 PASS와 focused 33 PASS](<../../../mydocs/working/assets/review_integration_20261026/validation/v12-focused-proofs.json>)는 컴파일 실패·0개 수집을 재현으로 세지 않는다.

## 2026-10-28 해당 PR 쪽별 재검증

중첩 표 원본 `issue2007_nested_cell_pagination_42065.hwp` 11쪽은 Native 98.82265% / fresh WASM 98.87353%, 전17쪽 최저 94.47057% / 94.39749%다. 한국어 가정통신문 2쪽 Native는 직전 비교 93.09501%에서 92.81548%로 0.27953%p 하락하여 사용자 기준 0.1%p를 초과한다. 동일 환경에서 구·신 바이너리 출력이 같다는 진단은 원인 확정이나 기준 통과를 뜻하지 않는다.

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
| 기준값 변경 | 충족 — 잘못된 Top 핀을 원본 Center·내용 포함으로 정정, 실제 clip 대조군 유지 |
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

## 원 PR에 제시된 두 미주 문서

후속 #8184 보정 전 전쪽 측정에서 `3-11월_실전_통합_2022.hwp`는 21쪽이며 최저 21쪽 Native 90.80955% / fresh WASM 90.84255%, `3-09월_교육_통합_2022.hwp`는 23쪽이며 최저 20쪽 Native 95.06161% / fresh WASM 95.32108%다. 각 원본과 대응 한컴 2020 Print PDF의 SHA 및 전쪽 수치는 [Native 원장](../../working/assets/review_integration_20261026/validation/post-rebase-8328/original-floatfix-native-comparison.json)과 [WASM 원장](../../working/assets/review_integration_20261026/validation/post-rebase-8328/original-floatfix-wasm-comparison.json)에 기록했다. 이는 후속 보정 후의 새 측정을 뜻하지 않는다.

## 후속 표 보정 이후 출력 보존

후속 표 보정 이후 원 PR의 3-11월 문서 21쪽과 3-09월 문서 20쪽은 Native와 WASM 각각 보정 전 렌더 PNG와 같다. 같은 이전 PDF 래스터 기준 점수 차이는 0이며 [Native 전쪽 해시](../../working/assets/review_integration_20261026/validation/post-rebase-8328/pr8184-original-native-png-comparison.json), [WASM 전쪽 해시](../../working/assets/review_integration_20261026/validation/post-rebase-8328/pr8184-original-wasm-png-comparison.json)에서 확인한다.
