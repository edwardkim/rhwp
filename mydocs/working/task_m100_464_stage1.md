# Task #464 단계 1 보고서 — 원인 정확 진단

**이슈**: #464
**브랜치**: `local/task464_v2`
**선행 task**: #459 (col 0 → col 1 fix)

---

## 1. 결함 재현

```bash
./target/release/rhwp export-svg samples/exam_kor.hwp -p 14 -o /tmp/exam_kor_p15/
```

LAYOUT_OVERFLOW 로그:
```
LAYOUT_OVERFLOW_DRAW: section=1 pi=103 line=3 y=1429.5 col_bottom=1422.9 overflow=6.5px
LAYOUT_OVERFLOW_DRAW: section=1 pi=103 line=4 y=1454.0 col_bottom=1422.9 overflow=31.0px
LAYOUT_OVERFLOW_DRAW: section=1 pi=103 line=5 y=1478.5 col_bottom=1422.9 overflow=55.5px
LAYOUT_OVERFLOW_DRAW: section=1 pi=104 line=0 y=1493.8 col_bottom=1422.9 overflow=70.9px
LAYOUT_OVERFLOW_DRAW: section=1 pi=104 line=1~4 y=1518.3~1591.8 overflow=95.4~168.9px
```

→ pi=103 line 3 부터 overflow 시작.

## 2. dump-pages 분석 — vpos-reset 인코딩 확인

```
페이지 15 (global_idx=14, section=1)
  단 1 (col 1):
    ...
    pi=101  vpos=65141..74331  "2017년 즈음에..."
    pi=102  vpos=76169..83521  "지속 가능 항공유란..."
    pi=103  vpos=85359..3676 [vpos-reset@line3]  "지속 가능 항공유는..."
    pi=104  vpos=5514..12866   "이러한 장점 때문에..."
    PartialParagraph pi=105 lines=0..1 vpos=14704..16542
  
페이지 16 (global_idx=15, section=1):
    PartialParagraph pi=105 lines=1..5 vpos=16542..22056
```

- pi=103 lines 0~2 vpos 누적 (85359~88347)
- pi=103 line 3 vpos 리셋 (85359 → 3676) — HWP 인코딩상 "여기서 단 경계" 의미
- pi=104, pi=105 reset 후 vpos (5514, 14704) 로 누적

PDF 정답: pi=103 lines 0..3 만 col 1 page 15 표시, lines 3.. + pi=104, pi=105 는 page 16 col 0 으로 이동.

rhwp 현재 동작: vpos-reset 무시하고 모두 col 1 page 15 에 누적 → overflow.

## 3. 결함 코드 위치 — `typeset.rs:904`

```rust
let col_breaks = if st.col_count > 1 && st.current_column == 0 {
    Self::detect_column_breaks_in_paragraph(para)
} else {
    vec![0]
};

if col_breaks.len() > 1 {
    self.typeset_multicolumn_paragraph(st, para_idx, para, fmt, &col_breaks);
    return;
}
```

**`st.current_column == 0` 가드**: col 0 만 col_break 감지. col 1 (마지막 단) 의 paragraph 내 vpos-reset 은 무시 → typeset_multicolumn_paragraph 미호출 → 모두 col 1 에 누적.

## 4. `typeset_multicolumn_paragraph` 의 col 1 처리 한계 — `typeset.rs:1975~1981`

```rust
// 마지막 단이 아니면 다음 단으로 flush
if bi + 1 < col_breaks.len() {
    st.flush_column();
    if st.current_column + 1 < st.col_count {
        st.current_column += 1;
        st.current_height = 0.0;
    }
}
```

가드를 제거해 col 1 에서 호출되어도, 코드는 "다음 단" 으로만 이동. col 1 (마지막 단) 에서 col_break 발생 시 다음 단이 없으니 **페이지 break 처리가 누락**됨.

→ 수정 시 (a) 가드 완화 + (b) col 1 col_break 시 페이지 break 트리거 두 가지 모두 필요.

## 5. 동기 코드 — `pagination/engine.rs:610` (별도 엔진)

수행계획서 명시. 같은 패턴 — `current_column == 0` 가드. 동일하게 수정 필요.

```bash
grep -n "current_column == 0\|detect_column_breaks" src/renderer/pagination/engine.rs
→ 597-610 부근
```

## 6. Task #459 와의 관계

Task #459 가 `on_first_multicolumn_page` 가드는 제거 → col 0 → col 1 단 경계 정정. 그러나 `current_column == 0` 가드는 그대로 → col 1 → 다음 페이지 단 경계 미해소.

본 task 는 #459 의 후속 — 잔존 가드 제거.

## 7. 가설

수정 적용 시 예상 동작:

1. pi=103 col 1 layout 시작 — `col_breaks = [0, 3]` 감지
2. lines 0..3 을 PartialParagraph 로 col 1 page 15 에 발행 → 정상
3. `bi+1 < col_breaks.len()` (true, col_breaks 개수 2) → flush_column
4. `current_column + 1 < col_count` (false, 마지막 단) → **페이지 break 트리거**
5. 새 페이지 col 0 으로 이동
6. lines 3.. 을 PartialParagraph 로 col 0 page 16 에 발행
7. pi=104, pi=105 도 col 0 page 16 에 순차 layout

→ overflow 사라짐 + page 분리 정상 + page 16 col 0 시작.

## 8. 위험 / 회귀

메모리 `feedback_essential_fix_regression_risk.md`:
- 다단 + vpos-reset 처리는 광범위 영향. exam_eng/math/science/social 등 다단 샘플 회귀 검증 필수.
- pagination/engine.rs 와 typeset.rs 두 엔진 일관성 필요.

#459 가 일부 케이스 fix 했고 본 task 는 잔존 케이스 — 안전한 룰 정합 변경 (`feedback_rule_not_heuristic.md` 적용 — col_break 감지는 룰).

## 9. 단계 2 입력

- 결함 위치: `src/renderer/typeset.rs:904` (가드) + `:1975-1981` (페이지 break 누락)
- 동기 코드: `src/renderer/pagination/engine.rs:597-610`
- 수정 방향: (a) 가드 완화 + (b) col_count 도달 시 페이지 break 트리거
- 검증 계획 (수행계획서):
  - exam_kor.hwp 15p → 16p 페이지 분리
  - LAYOUT_OVERFLOW 16건 (Task #459 후 잔존) 추가 해소
  - exam_kor.hwp 페이지 수 (현재 20) 변동 0
  - 다단 샘플 (exam_eng/math/science/social) 회귀 0
  - cargo test 1069 통과

## 10. 단계 2 진입

자동승인 모드. 단계 2 (구현 계획서) 작성 후 단계 3+ 구현 진입.
