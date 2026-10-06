# Task #1718 Stage 2 — 소스 수정

## 변경
`src/renderer/layout/table_layout.rs` 2곳(`advance_row_cut` 5232, `advance_row_block_cut` 5357):

```rust
// before
&& units[j + 1..].iter().any(|unit| unit.empty_spacer);
// after ([Task #1718])
&& units[j + 1..].iter().all(|unit| unit.empty_spacer);
```

## 근거
`visible_tail_before_spacer` 의 의도는 "뒤가 **전부** spacer 인 진짜 꼬리줄"만 grace 로 보존하는 것.
`.any()` 는 뒤쪽 어딘가에 빈문단 spacer 가 하나라도 있으면 grace 를 줘서, 654문단 거대 셀의
연속 텍스트 중간에서도 avail+120px 오버플로를 수용 → over-fill. `.all()` 로 정정하면:
- 연속 텍스트(뒤에 가시라인) → grace 거부 → 정상 capacity break(한글 정합)
- 진짜 tail-before-spacer → grace 유지 → over-pagination 방지 케이스(byeolpyo1/4) 무회귀
