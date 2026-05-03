# Task #544 v3 최종 보고서

**제목**: 박스 안 sequential paragraph 사이 line-spacing 좁음 정정 (5+ 케이스, Task #539 그룹 A 영역)

**브랜치**: `local/task544_v3` (`local/task544_v2` 위에 분기)
**작업 기간**: 2026-05-03 (Stage 1~4 단일 세션)
**이슈 등록**: 작업지시자 결정으로 미등록 (v3 suffix)

## 요약

Task #544 v2 시각 판정 중 작업지시자 발견된 회귀 5 케이스. **Task #544 v2 와 무관한 사전 회귀** (byte-identical baseline 검증). PR #538 의 Task #539 가 그룹 A 로 분류 보류한 영역 ("지문 시작 [X~Y] 직후 9곳 — IR 정확. 한컴 환경 검증 후 별도 issue 등록 권고") 일치.

**본질**: 박스 안 sequential paragraph (prev/next 둘 다 같은 박스 outline) 끝에서 Task #479 trailing-ls 제외 메커니즘이 작동. Task #552 의 `next_para_starts_visible_border` 가드는 "border 시작" 만 처리, 박스 안 sequential 영역 미처리 → 시각적 line-spacing 누락.

**fix**: 새 Cell `next_para_continues_visible_border` 도입 (방법 1). Task #552 의 의미 보존하며 trailing-ls 보존 영역 확장.

## 변경 본문

### 1. layout.rs

**LayoutEngine 필드 추가**:
```rust
/// [Task #544 v3] 다음 paragraph 가 prev paragraph 와 같은 visible border
/// 진행 중이면 true (박스 안 sequential paragraph). Task #552 (`_starts_`)
/// 와 의미 분리 — 본 Cell 은 박스 outline 이 prev 와 동일 (같은 박스 안)
/// 인 케이스. paragraph_layout 의 trailing ls 제외 분기에서 본 플래그가
/// true 면 ls 보존 (박스 안 sequential paragraph 사이 line spacing 정합).
next_para_continues_visible_border: std::cell::Cell<bool>,
```

**helper 함수 추가**:
```rust
pub(crate) fn next_paragraph_continues_visible_border(
    &self,
    curr_pi: usize,
    paragraphs: &[Paragraph],
    styles: &ResolvedStyleSet,
) -> bool {
    // curr/next 둘 다 visible border + 같은 border_fill_id 이면 true
}
```

**3 caller 위치에 set/reset 추가** (`next_para_starts_visible_border` set 와 동등 위치, 의미 분리):
- `PageItem::FullParagraph` 분기 (`comp` Some + `is_wrap_host` 우회)
- `PageItem::FullParagraph` 분기 (`final_comp` 본 호출)
- `PageItem::PartialParagraph` 분기 (full_end 검사)

### 2. paragraph_layout.rs

**is_full_paragraph_end 분기 가드 확장**:
```rust
let next_starts_border = self.next_para_starts_visible_border.get();
let next_continues_border = self.next_para_continues_visible_border.get();
if is_cell_last_line && cell_ctx.is_some() {
    y += line_height;
} else if is_full_paragraph_end && cell_ctx.is_none()
    && !next_starts_border && !next_continues_border {
    y += line_height;  // Task #479: trailing-ls 제외
} else {
    y += line_height + line_spacing_px;  // Task #552/#544 v3: 보존
}
```

### 3. integration_tests.rs

3 GREEN 테스트 추가:
- `test_544_v3_passage_inner_lspacing_p2_4_6` — pi=46→47 gap = 24.21 ±2 px
- `test_544_v3_passage_inner_lspacing_p10_19_21` — pi=208→209
- `test_544_v3_passage_inner_lspacing_p11_22_24` — pi=234→235

## 코드 영향 (누계)

| 파일 | 변경 LOC |
|------|----------|
| `src/renderer/layout.rs` | +37 (Cell + helper + 3 caller) |
| `src/renderer/layout/paragraph_layout.rs` | +6 / -1 (가드 추가) |
| `src/renderer/layout/integration_tests.rs` | +170 (3 GREEN 테스트) |
| `mydocs/plans/task_m100_544_v3.md` | +110 (수행계획서) |
| `mydocs/plans/task_m100_544_v3_impl.md` | +200 (구현계획서) |
| `mydocs/working/task_m100_544_v3_stage1.md` | +98 |
| `mydocs/working/task_m100_544_v3_stage3.md` | +94 |

## 측정값 (PDF 한컴 2010 정합, 24.21 px = 1816 HU = 1 line spacing)

| 케이스 | 수정 전 gap | 수정 전 drift | 수정 후 gap | 회복 |
|--------|------------|--------------|------------|------|
| p2 [4~6] pi=46→47 | 18.35 | -5.86 | **24.21** | +5.86 |
| p5 [10~12] pi=112→113 | 19.10 | -5.11 | **24.21** | +5.11 |
| p10 [19~21] pi=208→209 | 14.67 | -9.55 | **24.21** | +9.54 |
| p11 [22~24] pi=234→235 | 14.66 | -9.55 | **24.21** | +9.55 |
| p13 [25~27] pi=266→267 | 18.34 | -5.87 | **24.21** | +5.87 |

→ **5 케이스 모두 PDF 정합**. 부분 제외 (-5~6) 와 완전 제외 (-9.55) 모두 같은 fix 로 해결.

## 검증

### 단위 테스트

| 단계 | passed | ignored | 비고 |
|------|--------|---------|------|
| Pre-Stage 1 | 1122 | 2 | baseline (Task #544 v2 후) |
| Stage 1 (RED 추가) | 1122 | 5 | +3 ignore |
| Stage 2 (Phase A) | 1125 | 2 | +3 GREEN, -3 ignored |
| Stage 3 (회귀 검증) | 1125 | 2 | 변동 없음 |
| **최종** | **1125** | **2** | **회귀 0건** |

### 회귀 가드 (31/31 GREEN)

| Suite | 결과 |
|-------|------|
| issue_301 | 1 GREEN |
| issue_418 | 1 GREEN |
| issue_501 | 1 GREEN |
| issue_505 | 9 GREEN |
| issue_514 | 3 GREEN |
| issue_516 | 8 GREEN |
| issue_530 | 1 GREEN |
| issue_546 | 1 GREEN |
| svg_snapshot | 6 GREEN |

### Task #552 무회귀 (양립 검증)

`test_552_passage_box_top_gap_p2_4_6` (`--ignored`) **GREEN 유지**. Task #552 의 `next_para_starts_visible_border` 와 본 task 의 `next_para_continues_visible_border` 가 의미 분리 (mutually exclusive) 로 양립.

### Task #544 v2 무회귀

`test_544 / test_547 / test_548` 모두 **GREEN 유지** (Task #544 v2 의 paragraph border 좌표/inset 정정 보존).

### 빌드

- `cargo build --release`: 47.43s, 0 error
- `cargo clippy --lib`: 본 task 신규 결함 **0건**
  - 기존 잔존 결함 2건 (`table_ops.rs:1007`, `object_ops.rs:298`) — orders 메모에 이미 기록

## Commit 이력

| Stage | Commit | 메시지 |
|-------|--------|--------|
| 0 | `6b45671e` | Task #544 v3 수행계획서 |
| 0 | `5ebf434c` | Task #544 v3 구현계획서: 4 단계 분할 + Phase A 두 방법 안 |
| 1 | `584e0644` | Task #544 v3 Stage 1: TDD RED 3건 + 5 케이스 측정 + Phase A 방법 결정 |
| 2 | `84d1d4b2` | Task #544 v3 Stage 2: Phase A 적용 (박스 안 sequential paragraph trailing-ls 보존) |
| 3 | `e3f455e0` | Task #544 v3 Stage 3: 광범위 회귀 검증 + Phase B 불필요 확정 |

## 검증 게이트 (작업지시자)

1. **시각 판정 1차 (SVG)**: 21_언어_기출 페이지 2 [4~6] / 페이지 5 [10~12] / 페이지 10 [19~21] / 페이지 11 [22~24] / 페이지 13 [25~27] 의 박스 안 paragraph 사이 줄간격 PDF 한컴 2010 정합
2. **시각 판정 2차 (rhwp-studio web Canvas)**: 동일 페이지 web 렌더링 정합
3. **다른 박스 무회귀**: 21_언어_기출 [1~3] / [7~9] / [13~15] / [16~18] / [28~30] 줄간격 무변화

## 잔존 / 후속

1. **다른 fixture 광범위 시각 회귀**: 본 task 가 모든 박스 안 sequential paragraph 의 line-spacing 을 +5~9.55 px 늘림. 21_언어_기출 외 fixture (exam_kor / exam_eng / exam_math / exam_science / exam_social / aift / treatise / hwpspec) 의 시각 판정 권고. svg_snapshot 6/6 으로 fixture 무회귀 확인했으나 다른 페이지 영향 가능.
2. **Clippy 기존 결함**: `table_ops.rs:1007`, `object_ops.rs:298` — orders 메모에 이미 기록. 별도 task 후보.

## 머지 절차 (작업지시자 시각 판정 후)

본 task 는 `local/task544_v2` 위에 분기되어 있음:

```bash
# Task #544 v2 + v3 합산 머지 (둘 다 시각 판정 통과 후)
git checkout local/devel
git merge local/task544_v3 --no-ff -m \
  "Merge local/task544_v2 + v3: Task #544 v2 (paragraph border 좌표) + v3 (박스 안 sequential paragraph line-spacing) 정정"

git checkout devel
git merge local/devel --no-ff
git push origin devel
```

## 참조

- Task #479 (본문 paragraph 마지막 줄 trailing ls 제외) — 본 task 의 영향 영역, 본질 보존
- Task #552 (border 시작 직전 trailing ls 보존) — 본 task 의 양립 가드
- Task #539 그룹 A (지문 시작 [X~Y] 직후 9곳 보류) — 본 task 의 영역
- Task #544 v2 (paragraph border 좌표/inset 산식 정정) — 본 task 의 sibling
- 코드:
  - `src/renderer/layout.rs:247, 281~308, 311~340, 364, 2028~2049, 2088~2109, 2204~2225`
  - `src/renderer/layout/paragraph_layout.rs:2649~2666`
- 샘플: `samples/21_언어_기출_편집가능본.hwp`, PDF 한컴 2010
