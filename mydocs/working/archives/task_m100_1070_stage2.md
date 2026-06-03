# Stage 2 보고서 — Task #1070: 설계 페이퍼 검증 (경계 모순 점검)

- 브랜치: `local/task1070`
- 소스 무변경(설계 검증). 적용 코드는 Stage 3 확정.

## 신규 arm (typeset.rs:2626~2630)
```rust
} else if table.common.treat_as_char && total_lines > pre_table_end_line + 1 {
    pre_table_end_line + 1
}
```
`tac_wrap_split`(2622) · `attr&0x01`(2624) **다음**, `is_last&&!is_first`(2631) **앞**.

## 발동 영역 (정밀)
신규 arm 은 **`treat_as_char && pre_end == 0 && total_lines > 1`** 일 때만 실효한다(아래 도출).
= "표가 첫 줄(line0)에 있고 그 뒤에 실제 본문 줄이 있는 HWPX TAC 표".

## 경계 모순 점검

| 케이스 | 선행 분기/조건 | 신규 arm 발동? | 동작 | 모순 |
|--------|----------------|----------------|------|------|
| tac_wrap_split (pre_end>0 && <total) | 2622 먼저 매칭 | ✗ (선행 arm) | 기존 | 없음 |
| HWP5 TAC (attr&0x01) | 2624 먼저 매칭 | ✗ | 기존 pre_end.max(1) | 없음 |
| HWPX TAC, pre_end==0, total>1 | 2626 매칭 | **✓** | post_start=1 (표줄 제외) | **타깃** |
| HWPX TAC, 단일줄 (total==1, pre_end==0) | total>pre_end+1=1 거짓 | ✗ | else→pre_end(0) 기존 | 없음(mel-001 불변) |
| vertical_offset>0 (pre_end=total) | total>total+1 거짓 | ✗ | else 기존 | 없음 |
| Square wrap 어울림 (비-TAC) | treat_as_char 거짓 | ✗ | 기존 | 없음 |
| 비-TAC 블록표 | treat_as_char 거짓 | ✗ | is_last&&!is_first / else | 없음 |
| page-larger TAC (분할) | post_start 은 split 후 post-text 만 관여 | 직교 | 분할 로직 불변 | 없음 |

- HWP5 attr&0x01 의 `pre_end.max(1)` 와 정합: pre_end==0 일 때 둘 다 1 → 표줄 제외 동일.
- 다중 HWPX TAC 표 문단: `tac_table_count`(attr&0x01 카운트, HWPX=0) `<=1` 로 should_add_post_text
  유지. 신규 arm 은 현재 표의 post_start 만 조정 → 전수 sweep 0 회귀로 실증.

## 비회귀 케이스 실증 (Stage 1 재인용)
sample11-hwpx 0→0, tac-img-02.hwp 8→8, mel-001 3→3 (단일줄 불변),
tac-img-02.hwpx 7→6 / sample16 35→15 / aift 5→4 (다행 케이스 개선).

## 결론
신규 arm 은 타깃 패턴(HWPX TAC 표 line0 + 본문줄)만 정밀 발동하며 tac_wrap_split / HWP5 /
단일줄 / vertical_offset / Square wrap / 비-TAC / page-larger 와 모순 없음. Stage 3 확정.
