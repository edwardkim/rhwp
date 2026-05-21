# Stage 1 완료보고서 — Task M100 #1043

## 작업
중첩 표(1×1 wrapper) 외곽 테두리 borderFill lookup 의 off-by-one 정정.

## 변경 (`src/renderer/layout/table_layout.rs`)
1×1 wrapper 분기의 외곽 테두리 lookup 을 다른 모든 경로(일반 셀/표/zone)와 동일하게
`border_fill_id` 를 0-based 변환하도록 수정:

```rust
// before
styles.border_styles.get(cell.border_fill_id as usize)
// after
styles.border_styles.get((cell.border_fill_id as usize).saturating_sub(1))
```

`borderFillIDRef` 는 1-based, `border_styles` 는 0-based Vec 이므로 `-1` 이 필요하다.
주석으로 근거 명시. 좌표/렌더 로직 변경 없음.

## 검증 (예비)
- `cargo build` 통과.
- `export-svg -p 7`: 조직도 외곽 박스의 **전폭 연속 가로선** 생성 확인
  (y=615.6, x 75.6→672.7 = body 좌→우 전구간). 기존엔 박스 사이 gap 으로 끊겨 외곽선 부재였음.

정식 회귀 테스트/전체 검증은 Stage 2~3.
