# PR #351 처리 결과 보고서

## PR 정보

| 항목 | 내용 |
|------|------|
| PR 번호 | [#351](https://github.com/edwardkim/rhwp/pull/351) |
| 작성자 | [@planet6897](https://github.com/planet6897) (Jaeuk Ryu) |
| 이슈 | [#347](https://github.com/edwardkim/rhwp/issues/347) |
| 처리 | **Merge (admin)** |
| 처리일 | 2026-04-26 |
| Merge commit | `34e329e` |

## 변경 요약

`samples/exam_eng.hwp` SVG 와 PDF 참조본 어긋남 6 영역 통합 수정.

### 6 단계

1. 표/그림 절대 좌표 (HorzRelTo::Page body_area 통일, HorzAlign::Right 부호, InFrontOfText push-down 분리, BehindText 그림 y_offset 분리)
2. TAC 그림/InFrontOfText 표 Z-order 보존 (p4 Q28)
3. 인라인 TAC 그림 x 좌표 (margin_left/indent 반영)
4. 확장 바탕쪽 is_extension 판별 (ext_flags 0x02 + apply_to 휴리스틱)
5. 셀 첫 줄 y 위치 (LineSeg.vpos 우선 — Q27 Center valign)
6. 셀 padding 우선순위·축소 보정 (Task #279 비대칭 케이스 호환)

### 변경 파일 (21)

- 코드 5: `body_text.rs` (+5/-3), `height_measurer.rs` (+16/-6), `layout.rs` (+87/-16), `picture_footnote.rs` (+6/-4), `table_layout.rs` (+63/-23)
- 골든 1: `tests/golden_svg/form-002/page-0.svg` (±572) — Stage 6 의 부수효과 (좌표 미세 시프트 ≈+0.88px)
- 문서 15: 수행/구현/stage1·2·3 보고서 + 최종 보고서 + 7 첨부 PNG + samples/exam_eng.pdf

## 메인테이너 검증

| 항목 | 결과 |
|------|------|
| `cargo build --release v0.7.6` | ✅ 26.70s |
| `cargo test --lib` | ✅ 1008 passed (1000 → +8 신규) |
| `cargo test --test svg_snapshot` | ✅ 6/6 (form-002 포함) |
| `cargo test --test issue_301` | ✅ z-table 가드 |
| `cargo clippy --lib -D warnings` | ✅ clean |
| `cargo check --target wasm32` | ✅ clean |
| 7 핵심 샘플 + form-002 페이지 수 | ✅ 무변화 |
| WASM Docker 빌드 + 작업지시자 시각 검증 | ✅ 통과 |

## 안내 사항

### 1. PR description 의 골든 표현 오류

PR description "골든 SVG 갱신 없음" 인데 실제로는 form-002/page-0.svg ±572 변경. Stage 6 (셀 padding 정책) 부수효과로 좌표 미세 시프트 — 의도된 변경이지만 description 정확성 안내.

### 2. 이슈 #345 (exam_eng 9 → 8 회귀) 미해결

본 PR 의 6 영역에 페이지 분할 알고리즘 변경 없으므로 페이지 수 8 그대로. 이슈 #345 는 OPEN 유지 (별도 작업).

## 외부 기여 가치

- 6 영역 통합 진단 + 단일 PR 처리 (PR #340/#341 와 같은 통합 패턴)
- 다단 문서 + 박스 콘텐츠 좌표 정합화 — Task #279 호환 의도 명시
- 자체 보강 (Stage 1 → 2 → 3) — 시각 검증 사이클로 height_measurer 동기화 추가 (08b8427)

## 작성자 누적 기여 (planet6897)

PR #303, #305, #308, #315, #320, #327, #341, #343, **#351** — typeset/layout/table/picture 영역 깊은 기여 누적.

## 참고 링크

- [PR #351](https://github.com/edwardkim/rhwp/pull/351)
- [감사 코멘트](https://github.com/edwardkim/rhwp/pull/351#issuecomment-4321992056)
- 이슈: [#347 closed](https://github.com/edwardkim/rhwp/issues/347), [#345 OPEN (페이지 회귀, 별도)](https://github.com/edwardkim/rhwp/issues/345)
