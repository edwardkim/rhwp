# Stage 3 보고서 — Task #1070: 구현 확정

- 브랜치: `local/task1070`
- 수정 파일: `src/renderer/typeset.rs` (1 곳, `place_table_with_text`)

## 변경 (typeset.rs:2624~2630)
`post_table_start` 산식에 HWPX TAC 표 전용 arm 추가:
```rust
} else if table.attr & 0x01 != 0 {
    pre_table_end_line.max(1)
} else if table.common.treat_as_char && total_lines > pre_table_end_line + 1 {
    // HWPX TAC 표(attr 비트0=0): 표줄(pre_table_end_line) 다음에 실제 본문 줄이
    // 있으면 표줄을 post-text 에서 제외(HWP5 attr&0x01 의 pre_end.max(1) 와 정합).
    // 단일 줄(표줄만)은 건드리지 않아 기존 동작 보존.
    pre_table_end_line + 1
} else if is_last_table && !is_first_table {
    0
} else {
    pre_table_end_line
};
```

## 단위 검증 (smoke)
- 재현 3파일: 기부·답례품 472→**5.6**, hwpx-h-02 348→**7.4**, 해외직접투자 348→**7.4** px.
- 회귀세트 6파일: sample11-hwpx 0, tac-img-02.hwpx **6**(←7), tac-img-02.hwp 8, sample16-hwp5
  **15**(←35), aift **4**(←5), mel-001 3 — 회귀 0.

## 빌드/품질
- cargo build --release OK, cargo test --release lib **1324 passed** + 골든 **8/8**.
- clippy clean, fmt clean (typeset.rs).

## 다음
Stage 4 — 공개 픽스처 회귀 가드(`tests/issue_1070_*.rs`) + 전수 sweep 최종 + 최종 보고서.
