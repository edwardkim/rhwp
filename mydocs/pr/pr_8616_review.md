# PR #8616 리뷰 — fix: 덧말을 그리고 글자 겹치기 14모양을 한컴처럼 그리며 두 컨트롤을 넣고 고치는 API를 둔다 (#8019)

## 최종 판정

**머지 보류 — 셀 첫 줄 간격 보정 제품의 전쪽 Native/fresh WASM·전체 회귀·보호 문서 최종 검증 진행 중.**

원 PR head와 메인터너 보정 제품을 구분한다. 원격 comment·approve·push·merge·close는 실행하지 않았다. 검증 완료 뒤 판정과 증적을 갱신한다.

## 접수 정보

- 원 PR: [PR #8616](https://github.com/edwardkim/rhwp/pull/8616), 작성자 `semanticist21`, reviewer `jangster77`.
- 원 head: `5768ed5323238c5b1e1027eb0e0dcba6c62e26d8`. 2026-11-05 재조회에서 검토 시작 때와 같고 Open·Non-Draft다.
- 고정 base: `170b886f167b705b6e630c5b34bb638d92fae0a9`(PR #8542 포함).
- 현재 리베이스 제품: `71e5e0ae2a72a6b8f51682d131077bd4df44994f`. 리베이스 전 셀 첫 줄 간격 보정 제품은 `2de033b4e42be406a01ea0b9053a4531247e147c`다. 직전 행 조각 보정 제품은 `a314d7a24d3234305abff43149f575853db31105`다. 아래 완료된 R2 측정 제품은 `dc39658f812d22c961ddd0fbc8e71c7ec7a52e5b`이며 새 후보 측정으로 바꾸어 쓰지 않는다.
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

아래 Native·fresh WASM은 R2 제품 측정이다. 수용 판정은 최종 보정 후보 검증까지 완료한 뒤 내린다.

| 입력·용도 | 쪽수 | Native 최저 raw·TSV | fresh WASM 최저 raw·TSV | 독립 Print PDF |
| --- | ---: | --- | --- | --- |
| [hancom-mac-shapes.hwp](../../samples/char_overlap/hancom-mac-shapes.hwp) — 글자 겹치기 14모양·PUA 숫자 | 1 | [99.21%](assets/semanticist21_20261105/validation-r2/native/8616-hancom-mac-shapes-hwp-2022.tsv) | 측정 진행 중 또는 실행 오류 재측정 필요 | [hancom-mac-shapes.pdf](../../samples/char_overlap/hancom-mac-shapes.pdf) |
| [hancom-mac-shapes.hwpx](../../samples/char_overlap/hancom-mac-shapes.hwpx) — HWPX 글자 겹치기 14모양 | 1 | [100.00%](assets/semanticist21_20261105/validation-r2/native/8616-hancom-mac-shapes-hwpx-2022.tsv) | 측정 진행 중 또는 실행 오류 재측정 필요 | [hancom-mac-shapes.pdf](../../samples/char_overlap/hancom-mac-shapes.pdf) |
| [hancom-mac-dutmal.hwp](../../samples/dutmal/hancom-mac-dutmal.hwp) — 덧말 본말·덧말과 기준선 | 1 | [100.00%](assets/semanticist21_20261105/validation-r2/native/8616-hancom-mac-dutmal-hwp-2022.tsv) | 측정 진행 중 또는 실행 오류 재측정 필요 | [hancom-mac-dutmal.pdf](../../samples/dutmal/hancom-mac-dutmal.pdf) |
| [hancom-mac-dutmal.hwpx](../../samples/dutmal/hancom-mac-dutmal.hwpx) — HWPX 덧말 본말·덧말 | 1 | [100.00%](assets/semanticist21_20261105/validation-r2/native/8616-hancom-mac-dutmal-hwpx-2022.tsv) | 측정 진행 중 또는 실행 오류 재측정 필요 | [hancom-mac-dutmal-hwpx-2020.pdf](../../pdf/semanticist21-20261105/pr8616/hancom-mac-dutmal-hwpx-2020.pdf) |
| [hancom-mac-shapes.hwp](../../samples/char_overlap/hancom-mac-shapes.hwp) — 글자 겹치기 14모양·PUA 숫자 | 1 | [99.27%](assets/semanticist21_20261105/validation-r2/native/8616-hancom-mac-shapes-hwp-2024.tsv) | 측정 진행 중 또는 실행 오류 재측정 필요 | [hancom-mac-shapes.pdf](../../samples/char_overlap/hancom-mac-shapes.pdf) |
| [hancom-mac-shapes.hwpx](../../samples/char_overlap/hancom-mac-shapes.hwpx) — HWPX 글자 겹치기 14모양 | 1 | [100.00%](assets/semanticist21_20261105/validation-r2/native/8616-hancom-mac-shapes-hwpx-2024.tsv) | 측정 진행 중 또는 실행 오류 재측정 필요 | [hancom-mac-shapes.pdf](../../samples/char_overlap/hancom-mac-shapes.pdf) |
| [hancom-mac-dutmal.hwp](../../samples/dutmal/hancom-mac-dutmal.hwp) — 덧말 본말·덧말과 기준선 | 1 | [100.00%](assets/semanticist21_20261105/validation-r2/native/8616-hancom-mac-dutmal-hwp-2024.tsv) | 측정 진행 중 또는 실행 오류 재측정 필요 | [hancom-mac-dutmal.pdf](../../samples/dutmal/hancom-mac-dutmal.pdf) |
| [36389301_결재문서본문_직장훈련계획_덧말.hwpx](../../samples/hwpx/opengov/36389301_결재문서본문_직장훈련계획_덧말.hwpx) — 원 PR의 결재 문서 덧말 본말·덧말 표시 | 2 | [100.00%](assets/semanticist21_20261105/validation-r2/native/8616-opengov-ruby.tsv) | 측정 진행 중 또는 실행 오류 재측정 필요 | [opengov-36389301-ruby-2020.pdf](../../pdf/semanticist21-20261105/pr8616/opengov-36389301-ruby-2020.pdf) |

입력의 실행 경로·정식 보존 경로·SHA-256·크기·역할은 [입력 대응표](assets/semanticist21_20261105/validation-inputs.json)에 있다. 새 정식 보존 파일은 최종 제출 전에 실제 commit 포함과 실행 바이트 동일성을 확인한다.

## 주장에 대응하는 페이지 증적

아래 PNG는 같은 R2 측정에서 나온 원본 PNG를 합성했다. 점수와 직접 확대 판독을 구분하며, 최종 후보에서 달라진 영역은 새 증적으로 교체한다.

- `8616-hancom-mac-shapes-hwpx-2022` 1쪽: [review](assets/semanticist21_20261105/validation-r2/png/8616-hancom-mac-shapes-hwpx-2022/review_001.png) · [overlay](assets/semanticist21_20261105/validation-r2/png/8616-hancom-mac-shapes-hwpx-2022/overlay_001.png).
- `8616-hancom-mac-dutmal-hwpx-2022` 1쪽: [review](assets/semanticist21_20261105/validation-r2/png/8616-hancom-mac-dutmal-hwpx-2022/review_001.png) · [overlay](assets/semanticist21_20261105/validation-r2/png/8616-hancom-mac-dutmal-hwpx-2022/overlay_001.png).
- `8616-opengov-ruby` 1쪽: [review](assets/semanticist21_20261105/validation-r2/png/8616-opengov-ruby/review_001.png) · [overlay](assets/semanticist21_20261105/validation-r2/png/8616-opengov-ruby/overlay_001.png).

## 회귀와 조판 원칙 준수

- [tests/cases/char_overlap_caret_axis.rs](../../tests/cases/char_overlap_caret_axis.rs)
- [tests/cases/char_overlap_hancom_shapes.rs](../../tests/cases/char_overlap_hancom_shapes.rs)
- [tests/cases/compose_controls_insert.rs](../../tests/cases/compose_controls_insert.rs)
- [tests/cases/ruby_hancom_render.rs](../../tests/cases/ruby_hancom_render.rs)

- R2 제품의 최소 글꼴 집중 검사는 35모듈·169건 PASS이며 새 각주 관계 검사 두 건도 PASS다. 전체 nextest 통과와 구분한다.
- R2 전체 nextest는 행 조각 제품 보정이 필요해 컴파일 중 종료 코드 130으로 중단했으며 최종 후보에서 다시 실행한다.
- 새 base PR #8640의 실제 46상태·124쪽을 포함한 R2 Native 104사례·262쪽은 모두 90% 이상이다. 최저 91.36%, 새 base 대비 2%p 초과 하락 0쪽, 리베이스 전 136쪽 표시값 하락 0쪽이다.
- 현재 진행 중인 fresh WASM·기존 PR #8389/#8397 보호 문서·편람 필수 gate·Rust lint·Native Skia·필요한 브라우저 검증을 완료한 뒤 최종 수용 여부를 갱신한다. 웹폰트 시간 제한 검사는 사용자 지시에 따라 충돌이 없으므로 반복하지 않는다.
- 원본 단위·참조 보존 계약과 실물 배치 좌표 고정을 구분한다. 새 회귀·기존 좌표 핀은 실제 Native/fresh WASM·확대 판독 조건에 맞춰 점검하고 첫머리 근거를 갱신한다.

## Merge 후 처리 계획

통합 PR의 검증·CI·작업지시자 승인 후 원 PR별 해결 범위와 같은 원본·페이지의 증적을 연결한다. 한국어 존댓말의 기존 comment 수정안을 준비하고, 원격 게시·close·merge는 승인 범위에 따라 별도로 수행한다.
