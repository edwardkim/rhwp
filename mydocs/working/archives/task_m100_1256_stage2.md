# Stage 2 보고서 — Task #1256: 핵심 수정(safe_vpos_backtrack 덮어쓰기) + 회귀 정리

- 이슈: edwardkim/rhwp#1256 · 브랜치: `local/task1256` (from `devel`)
- 구현계획서: `task_m100_1256_impl.md`

## 1. 적용 수정 (path-1) — `src/renderer/height_cursor.rs` `vpos_adjust`

단일 줄 prev(빈 separator)로 끝나는 미주 제목 경계에서, typeset 이 주입한 between-notes(7mm)
trailing 이 `y_offset` 에 이미 포함되는데 `applied`/`safe_vpos_backtrack` 이 `end_y`(절대 vpos,
7mm 미포함)로 덮어써 제목을 ~20.4px 위로 끌어올리는 버그를 수정.

```rust
let injected_between_notes =
    self.endnote_between_notes_hu > 0 && seg.line_spacing >= self.endnote_between_notes_hu;
if injected_between_notes && compact_endnote_question_title
    && !vpos_rewind && !prev_is_multiline && end_y < y_offset {
    // y_offset(7mm 포함) 유지 + 내린 만큼 vpos base 이동(후속 항목 desync 방지)
    let restored_hu = ((y_offset - end_y) / self.dpi * 7200.0).round() as i32;
    if restored_hu > 0 { /* is_page_path ? page_base : lazy_base -= restored_hu */ }
    return y_offset;
}
```

- 다줄 prev(문22)는 `y_offset` 이 7mm 을 못 가지는 별개 경로 → 기존 #1246 rescue(+prev_ls) 유지.
- base-shift 동반이 핵심(과거 +132px overflow 회귀 #1246/#1082 의 본질).

## 2. 검증 결과

**한컴 PDF 정합 (96dpi):**
- 문6→문7 = **287.0px** (한컴 PDF 287px와 정확 일치). 전 영향 15건 7mm 복원.
- 영향 15건 = 문6·7·12·16·18·19·20·21·23·25·26·27·28(공통+선택과목).

**회귀:**
- 페이지 수 불변: 3-09 .hwp/.hwpx **23/23**, 미주사이20 **24**, 구분선아래20 **23**, 3-11 **21**.
- 오버플로우 0: 전 페이지 max content-y ≤ 1089.8(기존 envelope 내), shifted 페이지(9/13/14/18)도
  범위 내. 3-11 #1189 오버플로우 테스트 통과.
- **`cargo test` 전체 1961 passed, 0 failed.**

## 3. #1209 테스트 갱신 (작업지시자 승인)

`issue_1209_2022_sep_page10_question12_uses_safe_vpos_backtrack` 는 문12 를 cram 위치(390–406px)
로 단언("한컴/PDF 흐름")했으나, **한컴 PDF(10쪽)는 문12) 위에 7mm 갭이 명확히 존재**(문11 "k=9"
다음 빈 줄). PDF·작업지시자 보고와 모순이므로 PDF 기준으로 정정:
- 함수명 → `issue_1256_2022_sep_page10_question12_keeps_between_notes_gap`
- q12_y 단언 `390..=406` → `410..=426`(실측 418.7). 나머지 단언(수식 x 중앙정렬 402.5,
  꼬리-수식 간격 18.0≤20, q13_y 721.6≤724)은 #1209 그대로 통과(내부 간격 불변).

## 4. 제외(후속 분리) — 문5 등 컬럼-하단 케이스

문5(9쪽, 컬럼 하단 lazy_base<0 SKIP 경로의 task-1189 cram)는 cram 해제 시 **3-11월 10쪽 7.6px
오버플로우 회귀**(제목은 들어가나 후속 풀이 꼬리가 컬럼 초과). `fits_with_full_gap` 가드 불충분.
→ path-2 되돌림, **별도 이슈 #1257** 로 분리(= #1184 절대-vpos 재설계 영역).

## 5. 변경 파일
- `src/renderer/height_cursor.rs` (path-1, +32/-7)
- `tests/issue_1139_inline_picture_duplicate.rs` (#1209→#1256 갱신)
- (cargo fmt: 변경 파일만)

---
승인 요청: Stage 2 결과 확인 후 Stage 3(한컴 정합 시각 검증 보강 + 회귀 전수 표) 진행해도 될까요?
