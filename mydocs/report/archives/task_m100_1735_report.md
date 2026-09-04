# 최종 결과보고서 — 방점(U+302E/U+302F) 렌더링 정합 (M100 #1735)

- **이슈**: [edwardkim/rhwp#1735](https://github.com/edwardkim/rhwp/issues/1735)
- **브랜치**: `local/task1735` (from `local/devel`)
- **분류**: bug / rendering
- **샘플**: `samples/unicode/각 항목에 명시되어 있는_유니코드.hwp` (+ 한컴 정답지 `…_유니코드.pdf`)

## 1. 문제
rhwp(스튜디오)에서 문서를 열면 맨 앞 방점 `〮`(U+302E)이 한컴의 독립 가운데 점(`·각`)과 달리 **dotted-circle(◌)+톤 점 결합 아티팩트**로 표기됨(작업지시자 스크린샷 확인).

## 2. 원인
U+302E/U+302F 는 유니코드 **결합문자**. 유효 base 없이(줄 시작·공백 뒤) 셰이핑되면 브라우저/엔진이 **dotted-circle(U+25CC)** placeholder 를 삽입하고 톤 점을 쌓는다. 한컴은 방점을 spacing 글리프로 취급해 아티팩트가 없다. 부차적으로 U+302E 는 메트릭 advance=0(결합 글리프)이라 폴백 휴리스틱에서 반각(0.5em)으로 분류됐다.

## 3. 해결
| 단계 | 파일 | 변경 |
|------|------|------|
| Stage 1 | `src/renderer/composer.rs` | `tone_mark_display` 신설, 렌더 확장에서 `U+302E→·(U+00B7)`, `U+302F→⁚(U+205A)` 치환(IR 불변). 전 렌더 표면 + paint/json 스튜디오 계약 동시 정합. |
| Stage 2 | `src/renderer/layout/text_measurement.rs` | `is_narrow_punctuation` 에 U+302E/U+302F 추가 → 측정 폭 narrow(0.3em)로 렌더와 정합. |

핵심: rhwp 는 U+00B7(·)을 폰트 비의존 `<circle>` 벡터(Task #257)로 CJK x-height 중앙에 렌더하므로, 방점을 ·로 치환하면 자동으로 한컴형 가운데 점이 된다.

## 4. 검증
- **시각**: `export-pdf` → `· 각 항목에 명시되어 있는`. 한컴 정답지 `·각 …` 의 가운데 점과 일치, dotted-circle 소멸.
- **테스트(TDD)**: `test_expand_tone_marks_to_spacing_dot`, `test_narrow_glyph_tone_marks` 추가.
- **게이트**: `cargo test --lib` 2046 passed/0 failed · `cargo clippy --lib` 클린 · `rustfmt --check`(수정 파일) 클린.

## 5. 오판 정정
초기 조사에서 "두 번째 문단 통째 누락" 가설이 있었으나, dump 상 문단 0.1(cc=1, text_len=0)은 **정상 빈 마지막 문단**(한컴도 미출력)으로 확인·폐기.

## 6. 범위/한계
- 범위: 방점 U+302E/U+302F 2자로 한정.
- U+302F(쌍방점) 는 현재 샘플에 없어 정답지 시각 검증 미실시(U+205A best-effort). 필요 시 후속 확인.
- 스튜디오(WASM) 반영은 WASM 재빌드 필요(렌더 로직은 native/WASM 공유이므로 재빌드 시 동일 정합).

## 7. 후속
- fork(origin) push → upstream `devel` PR(`Refs #1735`). 이슈 self-close/직접 merge 금지.
