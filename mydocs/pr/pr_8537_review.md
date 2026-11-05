# PR #8537 리뷰 — fix(note): 주석 구분선 종류를 한/글처럼 테두리 선 표로 HWPX에 쓰고 읽는다

## 최종 판정

**머지 보류 — 셀 첫 줄 간격 보정 제품의 전쪽 Native/fresh WASM·전체 회귀·보호 문서 최종 검증 진행 중.**

원 PR head와 메인터너 보정 제품을 구분한다. 원격 comment·approve·push·merge·close는 실행하지 않았다. 검증 완료 뒤 판정과 증적을 갱신한다.

## 접수 정보

- 원 PR: [PR #8537](https://github.com/edwardkim/rhwp/pull/8537), 작성자 `semanticist21`, reviewer `jangster77`.
- 원 head: `6c2369bd25c28456b01e629f66c18c7c60176fc7`. 2026-11-05 재조회에서 검토 시작 때와 같고 Open·Non-Draft다.
- 고정 base: `170b886f167b705b6e630c5b34bb638d92fae0a9`(PR #8542 포함).
- 현재 리베이스 제품: `71e5e0ae2a72a6b8f51682d131077bd4df44994f`. 리베이스 전 셀 첫 줄 간격 보정 제품은 `2de033b4e42be406a01ea0b9053a4531247e147c`다. 직전 행 조각 보정 제품은 `a314d7a24d3234305abff43149f575853db31105`다. 아래 완료된 R2 측정 제품은 `dc39658f812d22c961ddd0fbc8e71c7ec7a52e5b`이며 새 후보 측정으로 바꾸어 쓰지 않는다.
- 통합 브랜치: `review/integration-20261105-semanticist21-open`.

## 원 주장과 실제 검증 범위

실행 트리거: 19구역 주석 구분선 HWP/HWPX 상호 저장.

독립 기대 동작: 파선·점선의 코드와 이름, 물결선·입체선 종류 보존.

원본 열기·모델 저장 검사와 실제 편집·저장·한컴 Print 시각 증거를 구분한다. 보충 입력은 원 PR 원본을 검증한 것으로 표현하지 않는다.

이 PR의 `line-types.hwp/.hwpx`는 PR #8536의 `note-formats.hwp/.hwpx`와 원본·독립 PDF·compat의 SHA가 같다. 아래는 같은 바이트의 전쪽 측정 재사용이며 유사 문서 대체가 아니다.

## 체리픽과 메인터너 보정

| 원 commit | 170b 리베이스 후 적용 commit |
| --- | --- |
| `2469c0b13ad4acf290d20bb721b93320766d73d4` | `a738b8e78d5e3c25e4bc3055ea1f804180df50e7` |
| `6c2369bd25c28456b01e629f66c18c7c60176fc7` | `47be5c360acdb66c5db93fc15cb1bec11459c54c` |

공통 보정의 원인·소비 경로·리베이스 충돌 해소는 [통합 분석](assets/semanticist21_20261105/integration-plan.md)에 기록했다.

## 검증 입력과 전쪽 결과

아래 Native·fresh WASM은 R2 제품 측정이다. 수용 판정은 최종 보정 후보 검증까지 완료한 뒤 내린다.

| 입력·용도 | 쪽수 | Native 최저 raw·TSV | fresh WASM 최저 raw·TSV | 독립 Print PDF |
| --- | ---: | --- | --- | --- |
| [note-formats.hwp](../../tests/fixtures/number_shapes/note-formats.hwp) — HWP 각주·쪽 번호 모양 | 19 | [99.98%](assets/semanticist21_20261105/validation-r2/native/8536-note-formats-hwp-2022.tsv) | [99.98%](assets/semanticist21_20261105/validation-r2/wasm/8536-note-formats-hwp-2022.tsv) | [note-formats-hancom-mac-12.30.pdf](../../pdf/number_shapes/note-formats-hancom-mac-12.30.pdf) |
| [note-formats.hwpx](../../tests/fixtures/number_shapes/note-formats.hwpx) — HWPX 각주·쪽 번호 모양 | 19 | [99.98%](assets/semanticist21_20261105/validation-r2/native/8536-note-formats-hwpx-2022.tsv) | [99.98%](assets/semanticist21_20261105/validation-r2/wasm/8536-note-formats-hwpx-2022.tsv) | [note-formats-hancom-mac-12.30.pdf](../../pdf/number_shapes/note-formats-hancom-mac-12.30.pdf) |

입력의 실행 경로·정식 보존 경로·SHA-256·크기·역할은 [입력 대응표](assets/semanticist21_20261105/validation-inputs.json)에 있다. 새 정식 보존 파일은 최종 제출 전에 실제 commit 포함과 실행 바이트 동일성을 확인한다.

## 주장에 대응하는 페이지 증적

아래 PNG는 같은 R2 측정에서 나온 원본 PNG를 합성했다. 점수와 직접 확대 판독을 구분하며, 최종 후보에서 달라진 영역은 새 증적으로 교체한다.

- `8536-note-formats-hwpx-2022` 2쪽: [review](assets/semanticist21_20261105/validation-r2/png/8536-note-formats-hwpx-2022/review_002.png) · [overlay](assets/semanticist21_20261105/validation-r2/png/8536-note-formats-hwpx-2022/overlay_002.png).
- `8536-note-formats-hwpx-2022` 3쪽: [review](assets/semanticist21_20261105/validation-r2/png/8536-note-formats-hwpx-2022/review_003.png) · [overlay](assets/semanticist21_20261105/validation-r2/png/8536-note-formats-hwpx-2022/overlay_003.png).
- `8536-note-formats-hwpx-2022` 12쪽: [review](assets/semanticist21_20261105/validation-r2/png/8536-note-formats-hwpx-2022/review_012.png) · [overlay](assets/semanticist21_20261105/validation-r2/png/8536-note-formats-hwpx-2022/overlay_012.png).

## 회귀와 조판 원칙 준수

- [tests/cases/note_separator_line_types.rs](../../tests/cases/note_separator_line_types.rs)

- R2 제품의 최소 글꼴 집중 검사는 35모듈·169건 PASS이며 새 각주 관계 검사 두 건도 PASS다. 전체 nextest 통과와 구분한다.
- R2 전체 nextest는 행 조각 제품 보정이 필요해 컴파일 중 종료 코드 130으로 중단했으며 최종 후보에서 다시 실행한다.
- 새 base PR #8640의 실제 46상태·124쪽을 포함한 R2 Native 104사례·262쪽은 모두 90% 이상이다. 최저 91.36%, 새 base 대비 2%p 초과 하락 0쪽, 리베이스 전 136쪽 표시값 하락 0쪽이다.
- 현재 진행 중인 fresh WASM·기존 PR #8389/#8397 보호 문서·편람 필수 gate·Rust lint·Native Skia·필요한 브라우저 검증을 완료한 뒤 최종 수용 여부를 갱신한다. 웹폰트 시간 제한 검사는 사용자 지시에 따라 충돌이 없으므로 반복하지 않는다.
- 원본 단위·참조 보존 계약과 실물 배치 좌표 고정을 구분한다. 새 회귀·기존 좌표 핀은 실제 Native/fresh WASM·확대 판독 조건에 맞춰 점검하고 첫머리 근거를 갱신한다.

## Merge 후 처리 계획

통합 PR의 검증·CI·작업지시자 승인 후 원 PR별 해결 범위와 같은 원본·페이지의 증적을 연결한다. 한국어 존댓말의 기존 comment 수정안을 준비하고, 원격 게시·close·merge는 승인 범위에 따라 별도로 수행한다.
