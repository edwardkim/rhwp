# PR #8571 리뷰 — feat(header-footer): 칸 경로로 머리말·꼬리말 표를 만들고 고친다

## 최종 판정

**머지 보류 — R5 통합 제품의 최종 검증 진행 중.**

아래 입력의 Native/fresh WASM·직접 판독과 전체 필수 게이트를 완료한 뒤 최종 수용 판정을 갱신한다. 원격 comment·approve·push·merge·close는 실행하지 않았다.

## 접수 정보

- 원 PR: [PR #8571](https://github.com/edwardkim/rhwp/pull/8571), 작성자 `semanticist21`, reviewer `jangster77`.
- 원 head: `dfef7b709773f56535ca1da123bb837eaad85700`. 2026-11-06 재조회에서 검토 시작 때와 같고 Open·Non-Draft다.
- 고정 base: `170b886f167b705b6e630c5b34bb638d92fae0a9`(PR #8542 포함).
- 현재 제품 source: `71e5e0ae2a72a6b8f51682d131077bd4df44994f`. 아래 표는 R5 실제 측정이며, 리베이스 전 R2/R3/R4 결과는 통합 분석과 기존 증적에서 구분한다.
- 통합 브랜치: `review/integration-20261105-semanticist21-open`.

## 원 주장과 실제 검증 범위

실행 트리거: 머리말·꼬리말 경로로 표 만들기·편집·변환·삭제.

독립 기대 동작: 셀 경로·캐럿·히트 테스트·글·표 소속과 저장 보존.

원본 열기·모델 저장 검사와 실제 편집·저장·한컴 Print 시각 증거를 구분한다. 보충 입력은 원 PR 원본을 검증한 것으로 표현하지 않는다.

## 체리픽과 메인터너 보정

| 원 commit | 170b 리베이스 후 적용 commit |
| --- | --- |
| `5d557479347c6218e4f5d991c1781a5461e751ad` | `b2913e92f528e4a163c7baef9038aaf005a69fe1` |
| `dfef7b709773f56535ca1da123bb837eaad85700` | `edcc9f1a477c505faa2214f1bdd4576ee5f397d7` |

공통 보정의 원인·소비 경로·리베이스 충돌 해소는 [통합 분석](assets/semanticist21_20261105/integration-plan.md)에 기록했다.

## 검증 입력과 전쪽 결과

아래는 제품 `71e5e0ae2a72a6b8f51682d131077bd4df44994f`의 실제 R5 측정이다. Native·fresh WASM을 구분하고 아직 끝나지 않은 행은 진행 중으로 표시한다. 최종 수용은 전체 필수 검증 뒤 판정한다.

| 같은 원본·용도 | 전체 쪽수 | Native 최저 raw·TSV | fresh WASM 최저 raw·TSV | 독립 Print PDF |
| --- | ---: | --- | --- | --- |
| [header-table.hwp](../../tests/fixtures/semanticist21_integration_review/pr8571/header-table.hwp) — 새 후보의 실제 공개 API 생성·저장 후 한컴 Print | 1 | [100.0%](assets/semanticist21_20261105/validation-r5/tsv/api-8571-header-table-hwp-native.tsv) | [100.0%](assets/semanticist21_20261105/validation-r5/tsv/api-8571-header-table-hwp-wasm.tsv) | [PDF](../../pdf/semanticist21-20261105/pr8571/current-d3e6-header-table-hwp-2020.pdf) |
| [header-table.hwpx](../../tests/fixtures/semanticist21_integration_review/pr8571/header-table.hwpx) — 새 후보의 실제 공개 API 생성·저장 후 한컴 Print | 1 | [100.0%](assets/semanticist21_20261105/validation-r5/tsv/api-8571-header-table-hwpx-native.tsv) | [100.0%](assets/semanticist21_20261105/validation-r5/tsv/api-8571-header-table-hwpx-wasm.tsv) | [PDF](../../pdf/semanticist21-20261105/pr8571/current-d3e6-header-table-hwpx-2020.pdf) |
| [non-inline-header-table.hwp](../../tests/fixtures/semanticist21_integration_review/pr8571/non-inline-header-table.hwp) — 새 후보의 실제 공개 API 생성·저장 후 한컴 Print | 1 | [100.0%](assets/semanticist21_20261105/validation-r5/tsv/api-8571-non-inline-header-table-hwp-native.tsv) | [100.0%](assets/semanticist21_20261105/validation-r5/tsv/api-8571-non-inline-header-table-hwp-wasm.tsv) | [PDF](../../pdf/semanticist21-20261105/pr8571/current-d3e6-non-inline-header-table-hwp-2020.pdf) |
| [non-inline-header-table.hwpx](../../tests/fixtures/semanticist21_integration_review/pr8571/non-inline-header-table.hwpx) — 새 후보의 실제 공개 API 생성·저장 후 한컴 Print | 1 | [100.0%](assets/semanticist21_20261105/validation-r5/tsv/api-8571-non-inline-header-table-hwpx-native.tsv) | [100.0%](assets/semanticist21_20261105/validation-r5/tsv/api-8571-non-inline-header-table-hwpx-wasm.tsv) | [PDF](../../pdf/semanticist21-20261105/pr8571/current-d3e6-non-inline-header-table-hwpx-2020.pdf) |
| [header-tac-explicit-break.hwpx](../../tests/fixtures/semanticist21_integration_review/pr8571/header-tac-explicit-break.hwpx) — 합성 경계: header-tac-explicit-break | 1 | [100.0%](assets/semanticist21_20261105/validation-r5/tsv/header-tac-explicit-break-native.tsv) | [100.0%](assets/semanticist21_20261105/validation-r5/tsv/header-tac-explicit-break-wasm.tsv) | [PDF](../../pdf/semanticist21-20261105/pr8571/header-tac-explicit-break-boundary-2020.pdf) |
| [header-tac-mixed-text.hwpx](../../tests/fixtures/semanticist21_integration_review/pr8571/header-tac-mixed-text.hwpx) — 합성 경계: header-tac-mixed-text | 1 | [100.0%](assets/semanticist21_20261105/validation-r5/tsv/header-tac-mixed-text-native.tsv) | [100.0%](assets/semanticist21_20261105/validation-r5/tsv/header-tac-mixed-text-wasm.tsv) | [PDF](../../pdf/semanticist21-20261105/pr8571/header-tac-mixed-text-boundary-2020.pdf) |
| [header-tac-pair-fit.hwpx](../../tests/fixtures/semanticist21_integration_review/pr8571/header-tac-pair-fit.hwpx) — 합성 경계: header-tac-pair-fit | 1 | [100.0%](assets/semanticist21_20261105/validation-r5/tsv/header-tac-pair-fit-native.tsv) | [100.0%](assets/semanticist21_20261105/validation-r5/tsv/header-tac-pair-fit-wasm.tsv) | [PDF](../../pdf/semanticist21-20261105/pr8571/header-tac-pair-fit-boundary-2020.pdf) |
| [header-tac-pair-wrap.hwpx](../../tests/fixtures/semanticist21_integration_review/pr8571/header-tac-pair-wrap.hwpx) — 합성 경계: header-tac-pair-wrap | 1 | [100.0%](assets/semanticist21_20261105/validation-r5/tsv/header-tac-pair-wrap-native.tsv) | 진행 중 | [PDF](../../pdf/semanticist21-20261105/pr8571/header-tac-pair-wrap-boundary-2020.pdf) |
| [header-stale.hwpx](../../tests/fixtures/semanticist21_integration_review/pr8571/height-contract/header-stale.hwpx) — 미측정에서 공통 측정으로 전환한 머리말·꼬리말의 셀·개체 선언 높이 계약 | 1 | [100.0%](assets/semanticist21_20261105/validation-r5/tsv/api-8571-height-contract-header-stale-native.tsv) | 진행 중 | [PDF](../../pdf/semanticist21-20261105/pr8571/height-contract/header-stale-2020.pdf) |
| [header-near.hwpx](../../tests/fixtures/semanticist21_integration_review/pr8571/height-contract/header-near.hwpx) — 미측정에서 공통 측정으로 전환한 머리말·꼬리말의 셀·개체 선언 높이 계약 | 1 | [100.0%](assets/semanticist21_20261105/validation-r5/tsv/api-8571-height-contract-header-near-native.tsv) | 진행 중 | [PDF](../../pdf/semanticist21-20261105/pr8571/height-contract/header-near-2020.pdf) |
| [header-grown.hwpx](../../tests/fixtures/semanticist21_integration_review/pr8571/height-contract/header-grown.hwpx) — 미측정에서 공통 측정으로 전환한 머리말·꼬리말의 셀·개체 선언 높이 계약 | 1 | [100.0%](assets/semanticist21_20261105/validation-r5/tsv/api-8571-height-contract-header-grown-native.tsv) | 진행 중 | [PDF](../../pdf/semanticist21-20261105/pr8571/height-contract/header-grown-2020.pdf) |
| [footer-stale.hwpx](../../tests/fixtures/semanticist21_integration_review/pr8571/height-contract/footer-stale.hwpx) — 미측정에서 공통 측정으로 전환한 머리말·꼬리말의 셀·개체 선언 높이 계약 | 1 | [100.0%](assets/semanticist21_20261105/validation-r5/tsv/api-8571-height-contract-footer-stale-native.tsv) | 진행 중 | [PDF](../../pdf/semanticist21-20261105/pr8571/height-contract/footer-stale-2020.pdf) |
| [footer-near-complete.hwpx](../../tests/fixtures/semanticist21_integration_review/pr8571/height-contract/footer-near-complete.hwpx) — 미측정에서 공통 측정으로 전환한 머리말·꼬리말의 셀·개체 선언 높이 계약 | 1 | [100.0%](assets/semanticist21_20261105/validation-r5/tsv/api-8571-height-contract-footer-near-complete-native.tsv) | 진행 중 | [PDF](../../pdf/semanticist21-20261105/pr8571/height-contract/footer-near-complete-2020.pdf) |
| [footer-grown.hwpx](../../tests/fixtures/semanticist21_integration_review/pr8571/height-contract/footer-grown.hwpx) — 미측정에서 공통 측정으로 전환한 머리말·꼬리말의 셀·개체 선언 높이 계약 | 1 | [100.0%](assets/semanticist21_20261105/validation-r5/tsv/api-8571-height-contract-footer-grown-native.tsv) | 진행 중 | [PDF](../../pdf/semanticist21-20261105/pr8571/height-contract/footer-grown-2020.pdf) |

[원본·PDF SHA와 보존 경로](assets/semanticist21_20261105/validation-r5/inputs.json), [페이지별 정밀 raw·쪽수·producer](assets/semanticist21_20261105/validation-r5/measurements.json)를 연결한다. 실제 실행 파일과 커밋 파일의 동일성을 확인했으며, 다른 문서의 높은 수치로 미달 쪽을 상쇄하지 않는다.

## 주장에 대응하는 페이지 증적

다음 이미지는 같은 R5 측정의 실제 PNG로 현재 Visual Sweep 함수를 실행해 만들었다. 원본과 PDF를 다시 출력하지 않았고, 해당 입력·쪽의 raw가 TSV와 같은지 확인했다. 직접 판독 완료 여부와 전체 통과 판정은 별도로 기록한다.

### native · api-8571-header-table-hwpx · 1쪽

실제 머리말 셀 경로 API가 생성한 표의 배치와 본문 보존

![같은 원본 1쪽 review](assets/semanticist21_20261105/validation-r5/png/pr8571/native/api-8571-header-table-hwpx/review_001.png)

![같은 원본 1쪽 overlay](assets/semanticist21_20261105/validation-r5/png/pr8571/native/api-8571-header-table-hwpx/overlay_001.png)

### wasm · api-8571-header-table-hwpx · 1쪽

실제 머리말 셀 경로 API가 생성한 표의 배치와 본문 보존

![같은 원본 1쪽 review](assets/semanticist21_20261105/validation-r5/png/pr8571/wasm/api-8571-header-table-hwpx/review_001.png)

![같은 원본 1쪽 overlay](assets/semanticist21_20261105/validation-r5/png/pr8571/wasm/api-8571-header-table-hwpx/overlay_001.png)

### native · header-tac-pair-wrap · 1쪽

같은 머리말 문단에서 가용 너비를 넘은 표 두 개의 줄바꿈

![같은 원본 1쪽 review](assets/semanticist21_20261105/validation-r5/png/pr8571/native/header-tac-pair-wrap/review_001.png)

![같은 원본 1쪽 overlay](assets/semanticist21_20261105/validation-r5/png/pr8571/native/header-tac-pair-wrap/overlay_001.png)

## 회귀와 조판 원칙 준수

- [tests/cases/header_footer_table_paths.rs](../../tests/cases/header_footer_table_paths.rs)

- [최소 글꼴 집중 회귀](assets/semanticist21_20261105/validation-r5/focused-regression.txt)는 184건 PASS다. 전체 Rust 회귀 완료로 대신하지 않는다.
- R5 Native 전체는 128개 입력·301쪽, 최저 91.81%다. [전체 Rust 목록 대조](assets/semanticist21_20261105/validation-r5/full-rust-inventory-audit.json)에서 최소 글꼴 5개·55개 binary·12,735건 PASS·45건 ignored를 확인했고 누락·중복은 없다. [실제 브라우저·생성 타입 검증](assets/semanticist21_20261105/validation-r5/ui/README.md)도 완료했다. fresh WASM·새 base 대비 비교·보호 PR #8389/#8397·편람 게이트·lint·Native Skia의 최종 결과는 계속 통합한다.
- 원본 단위·참조 보존과 실물 배치의 절대 좌표 고정을 구분한다. 완료 페이지의 Native/fresh WASM 양쪽 raw 98% 이상과 독립 PDF 확대 판독을 확인한 핀만 유지하며, 회귀 첫머리를 실제 측정 근거로 갱신한다.
- 리베이스 전 R2/R3의 성공 기록과 현재 R5를 구분한다. source·측정 조건이 같은 완전한 증거만 재사용하고 현재 제품의 실패를 과거 통과로 덮지 않는다.
- 웹폰트 시간 제한·오프라인 검사는 관련 충돌이 없으므로 작업지시자 지시에 따라 반복하지 않는다.

### 보정 근거

잘못된 기존 높이 가정을 정정한 근거와 합성 머리말·꼬리말 계약은 [높이 검증](assets/semanticist21_20261105/hf-height-contract/README.md)에 있다. 합성 계약의 통과와 실제 API 생성 문서의 시각 일치를 구분하며 현재 R5 행별 측정 상태는 위 표를 따른다.

## Merge 후 처리 계획

통합 PR의 검증·CI·작업지시자 승인 후 원 PR별 해결 범위와 같은 원본·페이지의 증적을 연결한다. 한국어 존댓말의 기존 comment 수정안을 준비하고, 원격 게시·close·merge는 승인 범위에 따라 별도로 수행한다.

## 기존 표 높이 회귀의 독립 재검토

원 해결 `007cd428c7`의 미측정 경로 임시 2배 가정과 PR #8571의 실제 공통 측정을 대조했다. [독립 Print·같은 원본의 양쪽 TSV·대표 review/overlay](assets/semanticist21_20261105/hf-height-contract/README.md)는 여섯 TAC 입력과 정상 비-TAC 두 형식이 각각 전 1쪽 100%임을 입증한다. 쪽 밖에서 하단이 잘리던 보충 입력은 완전한 표가 보이는 별도 대조군으로 구분했다. 기존 잘못된 200px/357.24px 핀은 원본 최소 크기·내용 포함·영역 소속 관계로 정정했고 최소 글꼴 4건 PASS를 확인했다. 원 법제처 실물 문서의 검증으로 확대하지 않는다.
