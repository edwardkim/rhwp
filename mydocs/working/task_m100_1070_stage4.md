# Stage 4 보고서 — Task #1070: 회귀 가드 + 최종 검증

- 브랜치: `local/task1070`
- 신규 파일: `tests/issue_1070_tac_table_post_text_overflow.rs`

## 회귀 가드 (공개 픽스처 3)
재현 3파일 모두 tracked 공개 샘플 → SVG 좌표 기반 가드 작성.
- 검증 방식: 영향 페이지(page 2) 렌더 → `<text>` 최대 y 가 물리 페이지 높이(1122.48px)
  이내인지 단언. 결함 시 후속 본문이 y≈1490px(페이지 초과)로 렌더 → 단언 실패.
- 테스트: `gibu_dalryepum_page2` / `hwpx_h_02_page2` / `haewoe_jikjeop_tuja_page2` — **3/3 통과**.
- 임계값 근거: 수정 전 max_y≈1490px > 1122 (실패) / 수정 후 max_y≈1069.7px < 1122 (통과).

## 최종 검증
- **재현 3파일**: 기부·답례품 472→해소(표 잔여 5.6), hwpx-h-02 348→7.4, 해외직접투자 348→7.4.
  - 구조 정합: dump-pages `PartialParagraph pi=25 lines=0..2 → 1..2`(표줄 제외).
- **전수 sweep**(samples hwp/hwpx, overflow 97파일): baseline 3057 lines / 382815px →
  **3043 lines / 376707px** (−14 lines, −6108px, 회귀 0·개선).
- **cargo test --release**: lib **1324 passed** + 통합 전부 ok(신규 3 포함), FAILED 0.
- **골든 SVG 8/8**, clippy clean, fmt clean(typeset.rs + 신규 테스트).

## 결론
HWPX TAC 표 line0 + 본문줄 post-text 표줄 포함 결함 해소. 회귀 가드 3 추가. 비회귀·골든 0.
최종 보고서 → 작업지시자 한컴 시각 판정 → PR.
