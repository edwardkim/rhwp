# PR — Task #534 v2 + Task #537 합본

> **base**: `edwardkim/rhwp:devel`
> **head**: `planet6897/rhwp:devel`
> **이슈**: closes #537 (Task #534 v2 는 별도 #534 v1 의 후속)

---

## PR 제목

```
Task #534 v2 + Task #537: LINE_SEG.column_start 정합 + lazy_base trailing-ls drift 정정
```

---

## PR 본문

```markdown
## Summary

- **Task #537**: 21_언어_기출.hwp TAC `<보기>` 표 직후 첫 답안 ① ↔ ② 사이 줄간격이 IR `LINE_SEG.vpos` delta 보다 716 HU(=9.55 px) 좁게 렌더링되던 결함 정정. `layout.rs:1494-1521` lazy_base 산출 시 prev_pi 의 last seg `line_spacing` 만큼 `y_delta_hu` 보정 (+14 / -2 LOC) — paragraph_layout 의 trailing-ls 제외(Task #479) 가 lazy_base 에 drift 로 동결되는 부작용을 차단.
- **Task #534 v2**: layout_shape_item TAC Picture 의 LINE_SEG.column_start 정합 보강 (별도 PR 메시지 #534 v2).
- closes #537

## Task #537 상세

### 문제

`samples/21_언어_기출_편집가능본.hwp` 에서 작업지시자가 보고한 11곳 모두 동일 패턴: TAC `<보기>` 표 직후 첫 답안의 ① → ② gap 만 IR vpos delta 보다 716 HU(=9.55 px) 좁음.

| 페이지 | 문제 | 라인수 | 수정 전 (px) | 수정 후 (px) | IR (px) |
|--------|------|--------|-------------|-------------|---------|
| P2 | 3번 | 3 | 63.09 | **72.64** ✓ | 72.64 |
| P3 | 6번 | 3 | 63.09 | **72.64** ✓ | 72.64 |
| P5 | 9번 | 1+2 | 14.67 | **24.21** ✓ | 24.21 |
| P6/P8/P9/P12/P13/P14 | 12/15/17/18/23/24/27/29 | 2 | 38.88 | **48.43** ✓ | 48.43 |

### 근본 원인

세 메커니즘의 상호작용으로 lazy_base 에 716 HU drift 가 영구화:

1. **`prev_tac_seg_applied` 가드** (`layout.rs:1434`): TAC 표 직후 paragraph 의 vpos 보정 건너뜀.
2. **trailing line_spacing 제외** (`paragraph_layout.rs:2645-2654`, Task #479): paragraph 마지막 줄에서 `ls` 가산 생략 → sequential y_offset 이 IR vpos 보다 1 ls 부족.
3. **lazy_base drift 동결** (`layout.rs:1497-1507`): pi=39 부터 vpos 보정 재개되지만 lazy_base 를 sequential y 에서 역산하므로 drift 716 HU 가 base=716 으로 박힘.

결과: ① 만 IR 정확, ②~⑤ 는 IR_vpos − 716 HU → ①→② gap 만 좁아 보임.

### 수정 (A'안)

`src/renderer/layout.rs:1494-1521`:

```rust
// [Task #537] trailing-ls 보정:
// paragraph_layout 의 마지막 줄은 trailing line_spacing 을
// 제외하여 y 를 advance 한다 (Task #479, lh_sum + (n-1)*ls 정책).
// 그 결과 sequential y_offset 은 IR vpos 누적보다
// prev_pi 의 last seg ls 만큼 부족해진다.
// 이 부족분을 y_delta_hu 에 더해야 lazy_base 가
// IR 절대 좌표와 일치한다 (drift 가 base 에 동결되는 것을 방지).
let trailing_ls_hu = paragraphs.get(prev_pi)
    .and_then(|p| p.line_segs.last())
    .map(|s| s.line_spacing.max(0))
    .unwrap_or(0);
let y_delta_hu = ((y_offset - col_area.y) / self.dpi * 7200.0).round() as i32
    + trailing_ls_hu;
let lazy_base = prev_vpos_end - y_delta_hu;
```

### 검증

- **TDD 통합 테스트**: `test_537_first_answer_after_tac_table_line_spacing` 추가 (FAIL → PASS).
- **자동 회귀**: `cargo test --release --lib` → 1117 passed, 0 failed, 1 ignored.
- **본 task 명시 11곳**: 모두 IR vpos delta 와 정확 일치.
- **광범위 회귀** (synam-001 / 복학원서 / exam_math/kor/eng/science / 2010-01-06):
  - 대부분 paragraph 가 누적 drift 만큼 IR-정확 위치로 하향 보정 = 정합성 개선.
  - `exam_kor.hwp` pi=7 ① at y=1205.19 = IR 기대값 1205.19 직접 검증.
  - `exam_math.hwp` 의 수식(Shape) 직후 paragraph 일부에서 -14.67 px 시프트 발견 — 시각 비교 후 회귀 시 task537_v2 후속 수정 권고.

## Test plan

- [x] `cargo test --release --lib` (1117 passed, 0 failed)
- [x] 작업지시자 명시 11곳 ①→② gap 정량 측정 = IR vpos delta
- [x] 광범위 7개 샘플 SVG 비교 (drift 보정 패턴 확인)
- [x] `exam_kor.hwp` pi=7 ① 직접 IR 정합 검증
- [ ] (작업지시자) 한컴 2010 / 2020 / 한컴독스 PDF 200dpi 시각 비교
- [ ] (작업지시자) `exam_math.hwp` 수식 직후 paragraph 회귀 여부 시각 확인
```

---

## 잔존 / 후속 사항

1. **base=716 일부 잔존**: 21_언어_기출 의 pi=147 등. 수정 전에도 존재 → 회귀 아님. 별도 메커니즘 (prev_seg vs line_segs.last() 차이) → 별도 issue 검토 후보.
2. **exam_math 음의 시프트**: 시각 비교 후 회귀 확인 시 `local/task537_v2` 로 가드 조건 (prev_pi 가 FullParagraph PageItem 일 때만 보정) 추가.
3. **Clippy 기존 결함** (본 task 외): `table_ops.rs:1007`, `object_ops.rs:298` — 별도 issue 권장.

---

## 산출물 일람

| 파일 | 종류 | 변경 |
|------|-----|------|
| `src/renderer/layout.rs` | 코드 | +14 / -2 (lazy_base trailing-ls 보정) |
| `src/renderer/layout/integration_tests.rs` | 테스트 | +76 (TDD 통합 테스트) |
| `mydocs/plans/task_m100_537.md` | 수행계획서 | 신규 |
| `mydocs/plans/task_m100_537_impl.md` | 구현계획서 | 신규 |
| `mydocs/working/task_m100_537_stage{1,2,3}.md` | 단계별 보고서 | 신규 |
| `mydocs/report/task_m100_537_report.md` | 최종 보고서 | 신규 |
| `mydocs/orders/20260503.md` | 오늘 할일 | 신규 |
