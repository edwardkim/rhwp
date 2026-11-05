# PR #8526 리뷰 — feat(style): 새 스타일을 한/글처럼 고른 스타일 바로 뒤에 넣는다

## 최종 판정

**머지 보류 — R5 통합 제품의 최종 검증 진행 중.**

아래 입력의 Native/fresh WASM·직접 판독과 전체 필수 게이트를 완료한 뒤 최종 수용 판정을 갱신한다. 원격 comment·approve·push·merge·close는 실행하지 않았다.

## 접수 정보

- 원 PR: [PR #8526](https://github.com/edwardkim/rhwp/pull/8526), 작성자 `semanticist21`, reviewer `jangster77`.
- 원 head: `a4ce64e250512153d3db6e27d88bb9448f120ee2`. 2026-11-06 재조회에서 검토 시작 때와 같고 Open·Non-Draft다.
- 고정 base: `170b886f167b705b6e630c5b34bb638d92fae0a9`(PR #8542 포함).
- 현재 제품 source: `71e5e0ae2a72a6b8f51682d131077bd4df44994f`. 아래 표는 R5 실제 측정이며, 리베이스 전 R2/R3/R4 결과는 통합 분석과 기존 증적에서 구분한다.
- 통합 브랜치: `review/integration-20261105-semanticist21-open`.

## 원 주장과 실제 검증 범위

실행 트리거: 선택한 스타일 뒤에 새 스타일 삽입 후 HWP/HWPX 재열기.

독립 기대 동작: 새 ID 1, 기존 본문·각주·머리말·꼬리말·덧말·메모·복사본의 스타일 참조와 다음 스타일 보존.

원본 열기·모델 저장 검사와 실제 편집·저장·한컴 Print 시각 증거를 구분한다. 보충 입력은 원 PR 원본을 검증한 것으로 표현하지 않는다.

## 체리픽과 메인터너 보정

| 원 commit | 170b 리베이스 후 적용 commit |
| --- | --- |
| `e1fc9e35fb9a923feb0a138627175f1d11d1ab3f` | `0126791f8dbab8269d1515553651632e8641e41f` |
| `a4ce64e250512153d3db6e27d88bb9448f120ee2` | `b690582a73895f6c0a1a0b713330b367b9154bbe` |

공통 보정의 원인·소비 경로·리베이스 충돌 해소는 [통합 분석](assets/semanticist21_20261105/integration-plan.md)에 기록했다.

## 검증 입력과 전쪽 결과

아래는 제품 `71e5e0ae2a72a6b8f51682d131077bd4df44994f`의 실제 R5 측정이다. Native·fresh WASM을 구분하고 아직 끝나지 않은 행은 진행 중으로 표시한다. 최종 수용은 전체 필수 검증 뒤 판정한다.

| 같은 원본·용도 | 전체 쪽수 | Native 최저 raw·TSV | fresh WASM 최저 raw·TSV | 독립 Print PDF |
| --- | ---: | --- | --- | --- |
| [inserted-style.hwp](../../tests/fixtures/semanticist21_integration_review/pr8526/inserted-style.hwp) — 새 후보의 실제 공개 API 생성·저장 후 한컴 Print | 1 | [100.0%](assets/semanticist21_20261105/validation-r5/tsv/api-8526-inserted-style-hwp-native.tsv) | 진행 중 | [PDF](../../pdf/semanticist21-20261105/pr8526/inserted-style-hwp-2020.pdf) |
| [inserted-style.hwpx](../../tests/fixtures/semanticist21_integration_review/pr8526/inserted-style.hwpx) — 새 후보의 실제 공개 API 생성·저장 후 한컴 Print | 1 | [100.0%](assets/semanticist21_20261105/validation-r5/tsv/api-8526-inserted-style-hwpx-native.tsv) | 진행 중 | [PDF](../../pdf/semanticist21-20261105/pr8526/inserted-style-hwpx-2020.pdf) |

[원본·PDF SHA와 보존 경로](assets/semanticist21_20261105/validation-r5/inputs.json), [페이지별 정밀 raw·쪽수·producer](assets/semanticist21_20261105/validation-r5/measurements.json)를 연결한다. 실제 실행 파일과 커밋 파일의 동일성을 확인했으며, 다른 문서의 높은 수치로 미달 쪽을 상쇄하지 않는다.

## 주장에 대응하는 페이지 증적

다음 이미지는 같은 R5 측정의 실제 PNG로 현재 Visual Sweep 함수를 실행해 만들었다. 원본과 PDF를 다시 출력하지 않았고, 해당 입력·쪽의 raw가 TSV와 같은지 확인했다. 직접 판독 완료 여부와 전체 통과 판정은 별도로 기록한다.

### native · api-8526-inserted-style-hwpx · 1쪽

스타일 삽입 후 동일 본문 보존. 삽입 ID와 참조 보존의 직접 증거는 실제 API 기록이다.

![같은 원본 1쪽 review](assets/semanticist21_20261105/validation-r5/png/pr8526/native/api-8526-inserted-style-hwpx/review_001.png)

![같은 원본 1쪽 overlay](assets/semanticist21_20261105/validation-r5/png/pr8526/native/api-8526-inserted-style-hwpx/overlay_001.png)

## 회귀와 조판 원칙 준수

- [tests/cases/inserted_style_keeps_references.rs](../../tests/cases/inserted_style_keeps_references.rs)

- [최소 글꼴 집중 회귀](assets/semanticist21_20261105/validation-r5/focused-regression.txt)는 184건 PASS다. 전체 Rust 회귀 완료로 대신하지 않는다.
- R5 Native 전체는 128개 입력·301쪽, 최저 91.81%다. [전체 Rust 목록 대조](assets/semanticist21_20261105/validation-r5/full-rust-inventory-audit.json)에서 최소 글꼴 5개·55개 binary·12,735건 PASS·45건 ignored를 확인했고 누락·중복은 없다. [실제 브라우저·생성 타입 검증](assets/semanticist21_20261105/validation-r5/ui/README.md)도 완료했다. fresh WASM·새 base 대비 비교·보호 PR #8389/#8397·편람 게이트·lint·Native Skia의 최종 결과는 계속 통합한다.
- 원본 단위·참조 보존과 실물 배치의 절대 좌표 고정을 구분한다. 완료 페이지의 Native/fresh WASM 양쪽 raw 98% 이상과 독립 PDF 확대 판독을 확인한 핀만 유지하며, 회귀 첫머리를 실제 측정 근거로 갱신한다.
- 리베이스 전 R2/R3의 성공 기록과 현재 R5를 구분한다. source·측정 조건이 같은 완전한 증거만 재사용하고 현재 제품의 실패를 과거 통과로 덮지 않는다.
- 웹폰트 시간 제한·오프라인 검사는 관련 충돌이 없으므로 작업지시자 지시에 따라 반복하지 않는다.

### 보정 근거

스타일 생성 실패 반환과 UI 소비 보정의 독립 근거는 [스타일 반환 계약](assets/semanticist21_20261105/style-return-contract/README.md)에 있다. 현재 R5의 실제 모달·undo 브라우저 검증 결과는 완료 후 별도로 반영한다.

## Merge 후 처리 계획

통합 PR의 검증·CI·작업지시자 승인 후 원 PR별 해결 범위와 같은 원본·페이지의 증적을 연결한다. 한국어 존댓말의 기존 comment 수정안을 준비하고, 원격 게시·close·merge는 승인 범위에 따라 별도로 수행한다.

## 스타일 생성 실패 반환의 UI 소비 보정

PR #8526이 잘못된 삽입 위치를 -1로 거절하면서, 이전 Node 소스 검사의 ‘항상 push·항상 성공’ 가정이 맞지 않게 됐다. 원 해결 PR #3400·issue #3438의 보정 이력을 대조했다. 실제 생성·삽입·거절의 Rust 보호는 유지하고, UI에서 음수 ID로 모양 적용·성공 알림을 진행하지 않도록 보완했다. 실제 확인 메서드를 실행한 보정 전 2 FAIL / 보정 후 관련 14 PASS, 최적화 fresh WASM의 실제 -1·문서 보존·정상 삽입을 확인했다.

[분석·실행 근거](assets/semanticist21_20261105/style-return-contract/README.md). TypeScript·production build도 통과했으며 실제 browser 소비 검증과 전체 통합 게이트를 완료한 뒤 최종 판정을 갱신한다. 이 UI/API 검사에 다른 문서의 시각 수치를 붙이지 않는다.

현재 UI 전체 Node 검사는 2026-11-06에 2,723건 모두 통과했다. 실제 브라우저의 실패 모달 유지·성공 생성·undo 복원과 초기화 WASM 응답 SHA도 [소비 경계 증거](assets/semanticist21_20261105/style-return-contract/README.md)에 연결한다. 이 결과는 아직 실행 중인 새 조판 제품의 전쪽 시각 검증을 대신하지 않는다.
