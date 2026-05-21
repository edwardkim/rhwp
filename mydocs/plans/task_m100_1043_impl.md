# 구현계획서 — Task M100 #1043 (버그 수정)

## 이슈
- edwardkim/rhwp#1043 — 중첩 표 외곽선 미표시 (1×1 wrapper 외곽 테두리 lookup off-by-one)
- 브랜치: `local/task1043` (기준 `stream/devel` @ ac6aeed4)

## 설계 근거
`src/renderer/layout/table_layout.rs::layout_table` 의 1×1 wrapper 분기에서 외곽 테두리 borderFill 을
조회하는 부분(현재 라인 243)이 `cell.border_fill_id` 를 `border_styles` 인덱스로 **그대로** 사용한다:

```rust
styles.border_styles.get(cell.border_fill_id as usize)   // -1 누락
```

`borderFillIDRef` 는 1-based, `border_styles` 는 0-based Vec 이다. 같은 파일의 다른 모든 lookup —
일반 셀(~1679 `idx = border_fill_id.saturating_sub(1)`), 표(~523), zone(~543/642) — 은 전부
`.saturating_sub(1)` 로 변환한다. 이 분기만 누락되어 `get(4)` 가 HWPX id=5(NONE×4)를 반환,
`any_border=false` → 외곽선 미표시.

### 사전 실증 (조사 단계 완료)
- `border_styles[4]` = `[None×4]`(=id5), `border_styles[3]` = `[Solid×4]`(=id4).
- `.saturating_sub(1)` 적용 시:
  - 인공지능(HWPX) 8p 조직도: 외곽 전폭 연속선 생성 (PDF p8 정합).
  - **기존 회귀 테스트 `tests/issue_nested_table_border.rs`(exam_social.hwp, HWP5): 수정 후에도 통과** —
    exam_social 의 의도 borderFill 도 borders 보유하므로 안전. 오히려 의도한 id 로 정확해짐.

## 단계별 계획 (3단계)

### Stage 1 — 코드 수정 (`src/renderer/layout/table_layout.rs`)
- 1×1 wrapper 외곽 테두리 lookup 한 줄을 다른 경로와 동일하게:
  ```rust
  styles.border_styles.get((cell.border_fill_id as usize).saturating_sub(1))
  ```
- 다른 변경 없음(좌표/렌더 로직 동일).
- 커밋: `Task #1043: 중첩 표 외곽선 lookup off-by-one 정정`
- 보고서: `mydocs/working/task_m100_1043_stage1.md`

### Stage 2 — 회귀 테스트 (`tests/issue_nested_table_border.rs`)
- HWPX 케이스 회귀 테스트 추가: `samples/2. 인공지능(AI) … 제안요청서.hwpx` 8페이지(index 7)
  렌더 SVG 에서 조직도 외곽 박스의 **전폭 연속 가로 외곽선**(x 좌측~우측 전구간, stroke=#000000)이
  존재하는지 검증. 좌표 hardcode 회피 — "전폭(>500px) 연속 가로선 ≥ 1" 형태로 가드.
- 기존 exam_social 테스트는 그대로 유지(수정 후 통과 확인됨).
- 커밋: `Task #1043: HWPX 중첩 표 외곽선 회귀 가드`
- 보고서: `mydocs/working/task_m100_1043_stage2.md`

### Stage 3 — 검증 및 최종 정리
1. `cargo test` 전체 통과(특히 `issue_nested_table_border` 2건).
2. `export-svg -p 7` → 조직도 외곽 박스 시각 확인(PDF p8 정합).
3. exam_social 등 기존 중첩 표 무영향 재확인.
4. 수정 파일 한정 fmt.
- 최종 보고서: `mydocs/report/task_m100_1043_report.md`. orders 미편집(작업지시자 관할).
- 커밋: `Task #1043: 검증 + 최종 보고서 (closes #1043)` — closes 번호 push 전 재확인.

## 영향 범위 / 리스크
- 수정: `src/renderer/layout/table_layout.rs` 단일 lookup(1줄).
- 본 수정으로 모든 borderFill lookup 이 `-1` 로 일관됨.
- 리스크: `border_fill_id==0` 은 `saturating_sub(1)=0` 이나, 본 분기는 `has_outer_padding && any_border`
  가드 안 → 무테두리 wrapper 에는 영향 없음.
- 기존 exam_social 회귀: 수정 후 통과 실증 완료.
