# PR #8571 리뷰 — feat(header-footer): 칸 경로로 머리말·꼬리말 표를 만들고 고친다

## 최종 판정

**메인터너 보정 후 수용 가능 — 원 PR 주장 범위의 로컬 검증 완료.**

원본별 Native/fresh WASM 전쪽 TSV와 같은 주장 페이지의 review·overlay를 아래에 연결했다. [통합 PR #8659](https://github.com/edwardkim/rhwp/pull/8659)의 정확한 code head CI가 성공했다. [통합 리뷰](pr_8659_review.md)에 실행 URL을 기록했으며, trailing head 확인·merge·원 PR close는 후속 조건이다.

## 접수 정보

- 원 PR: [PR #8571](https://github.com/edwardkim/rhwp/pull/8571), 작성자 `semanticist21`, reviewer `jangster77`.
- 원 head: `dfef7b709773f56535ca1da123bb837eaad85700`. 2026-11-06 재조회에서 검토 시작 때와 같고 Open·Non-Draft다.
- 고정 base: `170b886f167b705b6e630c5b34bb638d92fae0a9`(PR #8542 포함).
- 현재 제품 source: Rust `86709ac16711820e61baa3b87e2c4ddaac23573e` · Studio `5feccb130e0f695de0046d994c060130fa3e928e`. 아래 표는 R6 새 Native·fresh WASM 실제 측정이다. R5 과거 증거는 출처를 구분해 보존한다.
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

공통 보정의 원인·소비 경로·리베이스 충돌 해소는 [통합 분석](../assets/semanticist21_20261105/integration-plan.md)에 기록했다.

## 검증 입력과 전쪽 결과

제품 R6의 실제 전체 페이지 측정이다. 두 출력 경로와 독립 Print PDF의 전체 쪽수가 일치하며,
모든 페이지가 90% 이상이다. 96 DPI·Print 프로필·색상 경계 보정 전 2px raw를 사용했다.

| 같은 원본·용도 | 전체 쪽수 | Native 최저 raw·TSV | fresh WASM 최저 raw·TSV | 독립 Print PDF |
| --- | ---: | --- | --- | --- |
| [header-table.hwp](../../../tests/fixtures/semanticist21_integration_review/pr8571/header-table.hwp) — 새 후보의 실제 공개 API 생성·저장 후 한컴 Print | 1 | [100.0%](../assets/semanticist21_20261105/validation-r6/tsv/api-8571-header-table-hwp-native.tsv) | [100.0%](../assets/semanticist21_20261105/validation-r6/tsv/api-8571-header-table-hwp-wasm.tsv) | [PDF](../../../pdf/semanticist21-20261105/pr8571/current-d3e6-header-table-hwp-2020.pdf) |
| [header-table.hwpx](../../../tests/fixtures/semanticist21_integration_review/pr8571/header-table.hwpx) — 새 후보의 실제 공개 API 생성·저장 후 한컴 Print | 1 | [100.0%](../assets/semanticist21_20261105/validation-r6/tsv/api-8571-header-table-hwpx-native.tsv) | [100.0%](../assets/semanticist21_20261105/validation-r6/tsv/api-8571-header-table-hwpx-wasm.tsv) | [PDF](../../../pdf/semanticist21-20261105/pr8571/current-d3e6-header-table-hwpx-2020.pdf) |
| [non-inline-header-table.hwp](../../../tests/fixtures/semanticist21_integration_review/pr8571/non-inline-header-table.hwp) — 새 후보의 실제 공개 API 생성·저장 후 한컴 Print | 1 | [100.0%](../assets/semanticist21_20261105/validation-r6/tsv/api-8571-non-inline-header-table-hwp-native.tsv) | [100.0%](../assets/semanticist21_20261105/validation-r6/tsv/api-8571-non-inline-header-table-hwp-wasm.tsv) | [PDF](../../../pdf/semanticist21-20261105/pr8571/current-d3e6-non-inline-header-table-hwp-2020.pdf) |
| [non-inline-header-table.hwpx](../../../tests/fixtures/semanticist21_integration_review/pr8571/non-inline-header-table.hwpx) — 새 후보의 실제 공개 API 생성·저장 후 한컴 Print | 1 | [100.0%](../assets/semanticist21_20261105/validation-r6/tsv/api-8571-non-inline-header-table-hwpx-native.tsv) | [100.0%](../assets/semanticist21_20261105/validation-r6/tsv/api-8571-non-inline-header-table-hwpx-wasm.tsv) | [PDF](../../../pdf/semanticist21-20261105/pr8571/current-d3e6-non-inline-header-table-hwpx-2020.pdf) |
| [header-tac-explicit-break.hwpx](../../../tests/fixtures/semanticist21_integration_review/pr8571/header-tac-explicit-break.hwpx) — 합성 경계: header-tac-explicit-break | 1 | [100.0%](../assets/semanticist21_20261105/validation-r6/tsv/header-tac-explicit-break-native.tsv) | [100.0%](../assets/semanticist21_20261105/validation-r6/tsv/header-tac-explicit-break-wasm.tsv) | [PDF](../../../pdf/semanticist21-20261105/pr8571/header-tac-explicit-break-boundary-2020.pdf) |
| [header-tac-mixed-text.hwpx](../../../tests/fixtures/semanticist21_integration_review/pr8571/header-tac-mixed-text.hwpx) — 합성 경계: header-tac-mixed-text | 1 | [100.0%](../assets/semanticist21_20261105/validation-r6/tsv/header-tac-mixed-text-native.tsv) | [100.0%](../assets/semanticist21_20261105/validation-r6/tsv/header-tac-mixed-text-wasm.tsv) | [PDF](../../../pdf/semanticist21-20261105/pr8571/header-tac-mixed-text-boundary-2020.pdf) |
| [header-tac-pair-fit.hwpx](../../../tests/fixtures/semanticist21_integration_review/pr8571/header-tac-pair-fit.hwpx) — 합성 경계: header-tac-pair-fit | 1 | [100.0%](../assets/semanticist21_20261105/validation-r6/tsv/header-tac-pair-fit-native.tsv) | [100.0%](../assets/semanticist21_20261105/validation-r6/tsv/header-tac-pair-fit-wasm.tsv) | [PDF](../../../pdf/semanticist21-20261105/pr8571/header-tac-pair-fit-boundary-2020.pdf) |
| [header-tac-pair-wrap.hwpx](../../../tests/fixtures/semanticist21_integration_review/pr8571/header-tac-pair-wrap.hwpx) — 합성 경계: header-tac-pair-wrap | 1 | [100.0%](../assets/semanticist21_20261105/validation-r6/tsv/header-tac-pair-wrap-native.tsv) | [100.0%](../assets/semanticist21_20261105/validation-r6/tsv/header-tac-pair-wrap-wasm.tsv) | [PDF](../../../pdf/semanticist21-20261105/pr8571/header-tac-pair-wrap-boundary-2020.pdf) |
| [header-stale.hwpx](../../../tests/fixtures/semanticist21_integration_review/pr8571/height-contract/header-stale.hwpx) — 미측정에서 공통 측정으로 전환한 머리말·꼬리말의 셀·개체 선언 높이 계약 | 1 | [100.0%](../assets/semanticist21_20261105/validation-r6/tsv/api-8571-height-contract-header-stale-native.tsv) | [100.0%](../assets/semanticist21_20261105/validation-r6/tsv/api-8571-height-contract-header-stale-wasm.tsv) | [PDF](../../../pdf/semanticist21-20261105/pr8571/height-contract/header-stale-2020.pdf) |
| [header-near.hwpx](../../../tests/fixtures/semanticist21_integration_review/pr8571/height-contract/header-near.hwpx) — 미측정에서 공통 측정으로 전환한 머리말·꼬리말의 셀·개체 선언 높이 계약 | 1 | [100.0%](../assets/semanticist21_20261105/validation-r6/tsv/api-8571-height-contract-header-near-native.tsv) | [100.0%](../assets/semanticist21_20261105/validation-r6/tsv/api-8571-height-contract-header-near-wasm.tsv) | [PDF](../../../pdf/semanticist21-20261105/pr8571/height-contract/header-near-2020.pdf) |
| [header-grown.hwpx](../../../tests/fixtures/semanticist21_integration_review/pr8571/height-contract/header-grown.hwpx) — 미측정에서 공통 측정으로 전환한 머리말·꼬리말의 셀·개체 선언 높이 계약 | 1 | [100.0%](../assets/semanticist21_20261105/validation-r6/tsv/api-8571-height-contract-header-grown-native.tsv) | [100.0%](../assets/semanticist21_20261105/validation-r6/tsv/api-8571-height-contract-header-grown-wasm.tsv) | [PDF](../../../pdf/semanticist21-20261105/pr8571/height-contract/header-grown-2020.pdf) |
| [footer-stale.hwpx](../../../tests/fixtures/semanticist21_integration_review/pr8571/height-contract/footer-stale.hwpx) — 미측정에서 공통 측정으로 전환한 머리말·꼬리말의 셀·개체 선언 높이 계약 | 1 | [100.0%](../assets/semanticist21_20261105/validation-r6/tsv/api-8571-height-contract-footer-stale-native.tsv) | [100.0%](../assets/semanticist21_20261105/validation-r6/tsv/api-8571-height-contract-footer-stale-wasm.tsv) | [PDF](../../../pdf/semanticist21-20261105/pr8571/height-contract/footer-stale-2020.pdf) |
| [footer-near-complete.hwpx](../../../tests/fixtures/semanticist21_integration_review/pr8571/height-contract/footer-near-complete.hwpx) — 미측정에서 공통 측정으로 전환한 머리말·꼬리말의 셀·개체 선언 높이 계약 | 1 | [100.0%](../assets/semanticist21_20261105/validation-r6/tsv/api-8571-height-contract-footer-near-complete-native.tsv) | [100.0%](../assets/semanticist21_20261105/validation-r6/tsv/api-8571-height-contract-footer-near-complete-wasm.tsv) | [PDF](../../../pdf/semanticist21-20261105/pr8571/height-contract/footer-near-complete-2020.pdf) |
| [footer-grown.hwpx](../../../tests/fixtures/semanticist21_integration_review/pr8571/height-contract/footer-grown.hwpx) — 미측정에서 공통 측정으로 전환한 머리말·꼬리말의 셀·개체 선언 높이 계약 | 1 | [100.0%](../assets/semanticist21_20261105/validation-r6/tsv/api-8571-height-contract-footer-grown-native.tsv) | [100.0%](../assets/semanticist21_20261105/validation-r6/tsv/api-8571-height-contract-footer-grown-wasm.tsv) | [PDF](../../../pdf/semanticist21-20261105/pr8571/height-contract/footer-grown-2020.pdf) |

[원본·PDF SHA·producer·정밀 raw](../assets/semanticist21_20261105/validation-r6/measurements.md), [측정 명령·환경](../assets/semanticist21_20261105/validation-r6/README.md)를 연결한다. 원 PR 원본·실제 API 저장본·보충 대조군의 용도를 구분한다.

## 주장에 대응하는 페이지 증적

같은 원본·독립 PDF·페이지의 실제 PNG다. 직접 판독과 실제 API 검증을 구분한다. 일부 Native 패널은 현재 RHWP/PDF PNG 쌍의 SHA가 이전 판독 쌍과 각각 같음을 확인해 재사용했다.

### api-8571-header-table-hwpx · 1쪽

실제 머리말 셀 경로 API가 생성한 표의 배치와 본문 보존

**native raw 100.0%** — 실제 API 저장본의 hp·hq 셀과 머리말 표 외곽·중앙 경계를 독립 Print와 대조했다.

![native 같은 원본 1쪽 review](../assets/semanticist21_20261105/validation-r6/png/pr8571/native/api-8571-header-table-hwpx/review_001.png)

![native 같은 원본 1쪽 overlay](../assets/semanticist21_20261105/validation-r6/png/pr8571/native/api-8571-header-table-hwpx/overlay_001.png)

**wasm raw 100.0%** — 실제 API 저장본 머리말의 hp·hq 셀과 표 외곽·중앙 경계 원점을 대조했다.

![wasm 같은 원본 1쪽 review](../assets/semanticist21_20261105/validation-r6/png/pr8571/wasm/api-8571-header-table-hwpx/review_001.png)

![wasm 같은 원본 1쪽 overlay](../assets/semanticist21_20261105/validation-r6/png/pr8571/wasm/api-8571-header-table-hwpx/overlay_001.png)

### header-tac-pair-wrap · 1쪽

같은 머리말 문단에서 가용 너비를 넘은 표 두 개의 줄바꿈

**native raw 100.0%** — 머리말 가용 폭을 넘긴 두 표가 각각 다른 줄을 차지하며 격자와 본문 시작이 Print와 맞는다.

![native 같은 원본 1쪽 review](../assets/semanticist21_20261105/validation-r6/png/pr8571/native/header-tac-pair-wrap/review_001.png)

![native 같은 원본 1쪽 overlay](../assets/semanticist21_20261105/validation-r6/png/pr8571/native/header-tac-pair-wrap/overlay_001.png)

**wasm raw 100.0%** — 머리말 첫 표·둘째 표가 가용 너비에 따라 서로 다른 줄에 놓이고 겹치지 않으며 독립 Print와 대응한다.

![wasm 같은 원본 1쪽 review](../assets/semanticist21_20261105/validation-r6/png/pr8571/wasm/header-tac-pair-wrap/review_001.png)

![wasm 같은 원본 1쪽 overlay](../assets/semanticist21_20261105/validation-r6/png/pr8571/wasm/header-tac-pair-wrap/overlay_001.png)

## 회귀와 조판 원칙 준수

- [tests/cases/header_footer_table_paths.rs](../../../tests/cases/header_footer_table_paths.rs)

- 현재 전쪽 검증은 Native 128입력·301쪽 최저 91.81401%, fresh WASM 128입력·301쪽 최저 91.17272%다. 같은 새 base의 각 301쪽 대비 하락 0쪽이다. [현재 증거](../assets/semanticist21_20261105/validation-r6/README.md)를 연결한다.
- 실제 API 12그룹·브라우저 동작·TypeScript/production build·생성 타입 export와 소비 검사는 [API](../assets/semanticist21_20261105/validation-r6/api/README.md)·[UI](../assets/semanticist21_20261105/validation-r6/ui/README.md)에 연결했다. 원래 R5 전체 Rust 12,735건을 R6 새 실행으로 표현하지 않는다.
- 최소 공개 글꼴 5개에서 현재 전체 Rust 12,740건 PASS·기존 ignored 45건, 실제 목록 누락·중복 0이다. 최종 편람·lint·Skia·release·doctest도 완료했다.
- 실제 assertion을 확인해 관계형·고정형을 구분하고 27개 파일 첫머리를 실제 측정값으로 갱신했다. 원본 단위·비율 계약은 유지하며, [17개 완료 페이지](../assets/semanticist21_20261105/validation-r6/qualification.md)는 양쪽 98% 이상과 독립 확대 판독 근거로 기존 고정 좌표를 유지한다.
- 원래 웹폰트 수명 검사는 관련 충돌이 없어 재사용했으나, 최종 Studio 기호 공급 보정은 로딩 경로를 바꾸므로 로더 수명 14건과 실제 브라우저 7시나리오를 추가 실행해 모두 통과했다.

## Merge 후 contributor PR comment 계획

통합 PR의 실제 CI URL·code/trailing head와 확정 merge SHA를 기록한 뒤 원 PR에 한국어 존댓말로 결과를 게시한다. 기존 comment가 있으면 해당 comment를 수정하고 API로 다시 확인한다. 원 기여의 해결 범위와 위 공통 메인터너 보정을 구분한다.

- 실제 확인 페이지: api-8571-header-table-hwpx p1, header-tac-pair-wrap p1.
- Native/fresh WASM의 위 같은 원본·쪽 review와 overlay를 사용한다. 수치와 직접 판독 결론·한계는 본문 관측을 그대로 반영한다.
- 대표 이미지 안정 경로: `mydocs/pr/assets/semanticist21_20261105/validation-r6/png/pr8571/<backend>/<key>/<review 또는 overlay>_NNN.png`. 같은 PR의 추가 완료 페이지는 [패널 목록](../assets/semanticist21_20261105/validation-r6/panels.md)을 따른다.
- URL은 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/` 뒤에 위 실제 경로를 붙인다. devel 포함 확인 후 `--body-file` 게시·API 재조회한다.
- 문서 비교는 [Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을 따른다. 원 PR·관련 이슈 close는 같은 해결 범위의 완료와 포함 관계를 확인한 뒤 처리한다.

## 기존 표 높이 회귀의 독립 재검토

원 해결 `007cd428c7`의 미측정 경로 임시 2배 가정과 PR #8571의 실제 공통 측정을 대조했다. [독립 Print·같은 원본의 양쪽 TSV·대표 review/overlay](../assets/semanticist21_20261105/hf-height-contract/README.md)는 여섯 TAC 입력과 정상 비-TAC 두 형식이 각각 전 1쪽 100%임을 입증한다. 쪽 밖에서 하단이 잘리던 보충 입력은 완전한 표가 보이는 별도 대조군으로 구분했다. 기존 잘못된 200px/357.24px 핀은 원본 최소 크기·내용 포함·영역 소속 관계로 정정했고 최소 글꼴 4건 PASS를 확인했다. 원 법제처 실물 문서의 검증으로 확대하지 않는다.
