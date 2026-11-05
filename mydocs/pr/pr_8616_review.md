# PR #8616 리뷰 — fix: 덧말을 그리고 글자 겹치기 14모양을 한컴처럼 그리며 두 컨트롤을 넣고 고치는 API를 둔다 (#8019)

## 최종 판정

**머지 보류 — R5 통합 제품의 최종 검증 진행 중.**

아래 입력의 Native/fresh WASM·직접 판독과 전체 필수 게이트를 완료한 뒤 최종 수용 판정을 갱신한다. 원격 comment·approve·push·merge·close는 실행하지 않았다.

## 접수 정보

- 원 PR: [PR #8616](https://github.com/edwardkim/rhwp/pull/8616), 작성자 `semanticist21`, reviewer `jangster77`.
- 원 head: `5768ed5323238c5b1e1027eb0e0dcba6c62e26d8`. 2026-11-06 재조회에서 검토 시작 때와 같고 Open·Non-Draft다.
- 고정 base: `170b886f167b705b6e630c5b34bb638d92fae0a9`(PR #8542 포함).
- 현재 제품 source: `71e5e0ae2a72a6b8f51682d131077bd4df44994f`. 아래 표는 R5 실제 측정이며, 리베이스 전 R2/R3/R4 결과는 통합 분석과 기존 증적에서 구분한다.
- 통합 브랜치: `review/integration-20261105-semanticist21-open`.

## 원 주장과 실제 검증 범위

실행 트리거: 덧말·14개 글자 겹치기 그리기 및 본문·셀·글상자·각주·미주 API 편집.

독립 기대 동작: 본말·덧말·테두리·PUA 숫자·전각 폭·줄 높이·캐럿·저장 보존.

원본 열기·모델 저장 검사와 실제 편집·저장·한컴 Print 시각 증거를 구분한다. 보충 입력은 원 PR 원본을 검증한 것으로 표현하지 않는다.

## 체리픽과 메인터너 보정

| 원 commit | 170b 리베이스 후 적용 commit |
| --- | --- |
| `70b3a3a937930e5752a271ed1d18c8efbf7cde14` | `2fb1103d8195261593769fef5255cc9f5a1aaee2` |
| `d0bddfa86f8f17a2dc2acefcd025484d8e7942d4` | `bb9341a4b2767d7be2412df014cf5b512ce447ea` |
| `28213394d0df5c1f96d417f46088d8a4dd318350` | `6417de28bcdcc0cfb6a1932074fee3aa4b55a3da` |
| `c469db2ce8128f016b70a8268fb2cb646457e15c` | `85da706892ad62e64fa4da08f6ae00660969f7eb` |
| `5768ed5323238c5b1e1027eb0e0dcba6c62e26d8` | `c4896bca1470d8555131b20574601c49a287fc05` |

공통 보정의 원인·소비 경로·리베이스 충돌 해소는 [통합 분석](assets/semanticist21_20261105/integration-plan.md)에 기록했다.

## 검증 입력과 전쪽 결과

아래는 제품 `71e5e0ae2a72a6b8f51682d131077bd4df44994f`의 실제 R5 측정이다. Native·fresh WASM을 구분하고 아직 끝나지 않은 행은 진행 중으로 표시한다. 최종 수용은 전체 필수 검증 뒤 판정한다.

| 같은 원본·용도 | 전체 쪽수 | Native 최저 raw·TSV | fresh WASM 최저 raw·TSV | 독립 Print PDF |
| --- | ---: | --- | --- | --- |
| [hancom-mac-shapes.hwp](../../samples/char_overlap/hancom-mac-shapes.hwp) — 글자 겹치기 14모양·PUA 숫자 | 1 | [99.2128%](assets/semanticist21_20261105/validation-r5/tsv/8616-hancom-mac-shapes-hwp-2022-native.tsv) | 진행 중 | [PDF](../../samples/char_overlap/hancom-mac-shapes.pdf) |
| [hancom-mac-shapes.hwpx](../../samples/char_overlap/hancom-mac-shapes.hwpx) — HWPX 글자 겹치기 14모양 | 1 | [100.0%](assets/semanticist21_20261105/validation-r5/tsv/8616-hancom-mac-shapes-hwpx-2022-native.tsv) | 진행 중 | [PDF](../../samples/char_overlap/hancom-mac-shapes.pdf) |
| [hancom-mac-dutmal.hwp](../../samples/dutmal/hancom-mac-dutmal.hwp) — 덧말 본말·덧말과 기준선 | 1 | [100.0%](assets/semanticist21_20261105/validation-r5/tsv/8616-hancom-mac-dutmal-hwp-2022-native.tsv) | 진행 중 | [PDF](../../samples/dutmal/hancom-mac-dutmal.pdf) |
| [hancom-mac-dutmal.hwpx](../../samples/dutmal/hancom-mac-dutmal.hwpx) — HWPX 덧말 본말·덧말 | 1 | [100.0%](assets/semanticist21_20261105/validation-r5/tsv/8616-hancom-mac-dutmal-hwpx-2022-native.tsv) | 진행 중 | [PDF](../../pdf/semanticist21-20261105/pr8616/hancom-mac-dutmal-hwpx-2020.pdf) |
| [hancom-mac-shapes.hwp](../../samples/char_overlap/hancom-mac-shapes.hwp) — 글자 겹치기 14모양·PUA 숫자 | 1 | [99.27308%](assets/semanticist21_20261105/validation-r5/tsv/8616-hancom-mac-shapes-hwp-2024-native.tsv) | 진행 중 | [PDF](../../samples/char_overlap/hancom-mac-shapes.pdf) |
| [hancom-mac-shapes.hwpx](../../samples/char_overlap/hancom-mac-shapes.hwpx) — HWPX 글자 겹치기 14모양 | 1 | [100.0%](assets/semanticist21_20261105/validation-r5/tsv/8616-hancom-mac-shapes-hwpx-2024-native.tsv) | 진행 중 | [PDF](../../samples/char_overlap/hancom-mac-shapes.pdf) |
| [hancom-mac-dutmal.hwp](../../samples/dutmal/hancom-mac-dutmal.hwp) — 덧말 본말·덧말과 기준선 | 1 | [100.0%](assets/semanticist21_20261105/validation-r5/tsv/8616-hancom-mac-dutmal-hwp-2024-native.tsv) | 진행 중 | [PDF](../../samples/dutmal/hancom-mac-dutmal.pdf) |
| [36389301_결재문서본문_직장훈련계획_덧말.hwpx](../../samples/hwpx/opengov/36389301_결재문서본문_직장훈련계획_덧말.hwpx) — 원 PR의 결재 문서 덧말 본말·덧말 표시 | 2 | [100.0%](assets/semanticist21_20261105/validation-r5/tsv/8616-opengov-ruby-native.tsv) | 진행 중 | [PDF](../../pdf/semanticist21-20261105/pr8616/opengov-36389301-ruby-2020.pdf) |
| [nonbmp-endnote-false.hwp](../../tests/fixtures/char_overlap_caret_axis/nonbmp-endnote-false.hwp) — 같은 검사 파일의 기존 ±2px 캐럿 핀을 유지하는 완료 페이지의 98% 조건 확인. 원 PR의 대표 문서로 대체하지 않음 | 1 | [100.0%](assets/semanticist21_20261105/validation-r5/tsv/8616-existing-caret-false-hwp-native.tsv) | 진행 중 | [PDF](../../pdf/issue7819_char_overlap_caret_axis/nonbmp-endnote-false-hwp-2020.pdf) |
| [nonbmp-endnote-false.hwpx](../../tests/fixtures/char_overlap_caret_axis/nonbmp-endnote-false.hwpx) — 같은 검사 파일의 기존 ±2px 캐럿 핀을 유지하는 완료 페이지의 98% 조건 확인. 원 PR의 대표 문서로 대체하지 않음 | 1 | [100.0%](assets/semanticist21_20261105/validation-r5/tsv/8616-existing-caret-false-hwpx-native.tsv) | 진행 중 | [PDF](../../pdf/issue7819_char_overlap_caret_axis/nonbmp-endnote-false-hwpx-2020.pdf) |
| [nonbmp-endnote-true.hwp](../../tests/fixtures/char_overlap_caret_axis/nonbmp-endnote-true.hwp) — 같은 검사 파일의 기존 ±2px 캐럿 핀을 유지하는 완료 페이지의 98% 조건 확인. 원 PR의 대표 문서로 대체하지 않음 | 1 | [100.0%](assets/semanticist21_20261105/validation-r5/tsv/8616-existing-caret-true-hwp-native.tsv) | 진행 중 | [PDF](../../pdf/issue7819_char_overlap_caret_axis/nonbmp-endnote-true-hwp-2020.pdf) |
| [nonbmp-endnote-true.hwpx](../../tests/fixtures/char_overlap_caret_axis/nonbmp-endnote-true.hwpx) — 같은 검사 파일의 기존 ±2px 캐럿 핀을 유지하는 완료 페이지의 98% 조건 확인. 원 PR의 대표 문서로 대체하지 않음 | 1 | [100.0%](assets/semanticist21_20261105/validation-r5/tsv/8616-existing-caret-true-hwpx-native.tsv) | 진행 중 | [PDF](../../pdf/issue7819_char_overlap_caret_axis/nonbmp-endnote-true-hwpx-2020.pdf) |

[원본·PDF SHA와 보존 경로](assets/semanticist21_20261105/validation-r5/inputs.json), [페이지별 정밀 raw·쪽수·producer](assets/semanticist21_20261105/validation-r5/measurements.json)를 연결한다. 실제 실행 파일과 커밋 파일의 동일성을 확인했으며, 다른 문서의 높은 수치로 미달 쪽을 상쇄하지 않는다.

## 주장에 대응하는 페이지 증적

다음 이미지는 같은 R5 측정의 실제 PNG로 현재 Visual Sweep 함수를 실행해 만들었다. 원본과 PDF를 다시 출력하지 않았고, 해당 입력·쪽의 raw가 TSV와 같은지 확인했다. 직접 판독 완료 여부와 전체 통과 판정은 별도로 기록한다.

### native · 8616-hancom-mac-shapes-hwp-2022 · 1쪽

원 PR 한컴 저장본의 14개 겹침 모양·PUA 숫자와 전각 폭

![같은 원본 1쪽 review](assets/semanticist21_20261105/validation-r5/png/pr8616/native/8616-hancom-mac-shapes-hwp-2022/review_001.png)

![같은 원본 1쪽 overlay](assets/semanticist21_20261105/validation-r5/png/pr8616/native/8616-hancom-mac-shapes-hwp-2022/overlay_001.png)

### native · 8616-hancom-mac-dutmal-hwpx-2022 · 1쪽

원 PR의 본말·덧말 실제 표시

![같은 원본 1쪽 review](assets/semanticist21_20261105/validation-r5/png/pr8616/native/8616-hancom-mac-dutmal-hwpx-2022/review_001.png)

![같은 원본 1쪽 overlay](assets/semanticist21_20261105/validation-r5/png/pr8616/native/8616-hancom-mac-dutmal-hwpx-2022/overlay_001.png)

## 회귀와 조판 원칙 준수

- [tests/cases/char_overlap_caret_axis.rs](../../tests/cases/char_overlap_caret_axis.rs)
- [tests/cases/char_overlap_hancom_shapes.rs](../../tests/cases/char_overlap_hancom_shapes.rs)
- [tests/cases/compose_controls_insert.rs](../../tests/cases/compose_controls_insert.rs)
- [tests/cases/ruby_hancom_render.rs](../../tests/cases/ruby_hancom_render.rs)

- [최소 글꼴 집중 회귀](assets/semanticist21_20261105/validation-r5/focused-regression.txt)는 184건 PASS다. 전체 Rust 회귀 완료로 대신하지 않는다.
- R5 Native 전체는 128개 입력·301쪽, 최저 91.81%다. [전체 Rust 목록 대조](assets/semanticist21_20261105/validation-r5/full-rust-inventory-audit.json)에서 최소 글꼴 5개·55개 binary·12,735건 PASS·45건 ignored를 확인했고 누락·중복은 없다. [실제 브라우저·생성 타입 검증](assets/semanticist21_20261105/validation-r5/ui/README.md)도 완료했다. fresh WASM·새 base 대비 비교·보호 PR #8389/#8397·편람 게이트·lint·Native Skia의 최종 결과는 계속 통합한다.
- 원본 단위·참조 보존과 실물 배치의 절대 좌표 고정을 구분한다. 완료 페이지의 Native/fresh WASM 양쪽 raw 98% 이상과 독립 PDF 확대 판독을 확인한 핀만 유지하며, 회귀 첫머리를 실제 측정 근거로 갱신한다.
- 리베이스 전 R2/R3의 성공 기록과 현재 R5를 구분한다. source·측정 조건이 같은 완전한 증거만 재사용하고 현재 제품의 실패를 과거 통과로 덮지 않는다.
- 웹폰트 시간 제한·오프라인 검사는 관련 충돌이 없으므로 작업지시자 지시에 따라 반복하지 않는다.

## Merge 후 처리 계획

통합 PR의 검증·CI·작업지시자 승인 후 원 PR별 해결 범위와 같은 원본·페이지의 증적을 연결한다. 한국어 존댓말의 기존 comment 수정안을 준비하고, 원격 게시·close·merge는 승인 범위에 따라 별도로 수행한다.
