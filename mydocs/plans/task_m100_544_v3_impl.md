# Task #544 v3 구현계획서

본 문서는 `task_m100_544_v3.md` 수행계획서의 4 단계 분할 구현 상세.

브랜치: `local/task544_v3` (`local/task544_v2` 위에 분기)
영역: `src/renderer/layout/paragraph_layout.rs` + `src/renderer/layout.rs`

## 코드 영향 영역 사전 분석

### Task #479 의 trailing-ls 제외 메커니즘 (paragraph_layout.rs:2645~2664)

```rust
let is_full_paragraph_end = line_idx + 1 >= end && end >= composed.lines.len();
let next_starts_border = self.next_para_starts_visible_border.get();
if is_cell_last_line && cell_ctx.is_some() {
    y += line_height;
} else if is_full_paragraph_end && cell_ctx.is_none() && !next_starts_border {
    // 셀 외부 paragraph 의 마지막 줄 (#479): trailing-ls 제외
    y += line_height;
} else {
    // 마지막 아닌 줄 / border-start 직전: trailing ls 보존
    y += line_height + line_spacing_px;
}
```

### Task #552 의 next_para_starts_visible_border Cell (layout.rs)

`next_para_starts_visible_border` 는 다음 paragraph 가 **새로운 visible border 를 시작** 할 때 set. 박스 안 sequential paragraph (prev/next 둘 다 같은 border_fill_id 진행 중) 는 **set 안 함** → 본 task 회귀의 본질 영역.

## Stage 1 — 정밀 진단 + TDD RED

### 작업

1. **p10/13 측정**: 페이지 10 [19~21] / 페이지 13 [25~27] 의 prev→next paragraph gap 측정. SVG 분석.

2. **p2/5 의 부분 제외 본질**:
   - p2 drift -5.86 (= 716 HU 의 ~61%)
   - p5 drift -5.11 (= 716 HU 의 ~53%)
   - p11 drift -9.55 (= 716 HU 의 100%)
   - 가설 A: `lazy_base` (Task #537 영역) 보정이 일부 케이스에서 작동
   - 가설 B: prev paragraph 의 `line_segs.last().line_spacing` 값이 다름
   - 검증: 각 paragraph 의 `[CS]` / `[PS]` / `ls[]` 출력으로 차이 식별

3. **TDD RED 3건** 추가:
   - `test_544_v3_passage_inner_lspacing_p2_4_6` — pi=46 마지막 줄 y vs pi=47 첫 줄 y delta = 24.21 ±2 px
   - `test_544_v3_passage_inner_lspacing_p5_10_12` — pi=112 → pi=113
   - `test_544_v3_passage_inner_lspacing_p11_22_24` — pi=234 → pi=235

   각 테스트 형식 (test_544 와 유사):
   - `core.render_page_svg_native(page_idx)` 로 SVG 추출
   - 두 paragraph 의 첫 글자 (특정 한글) translate(x, y) 좌표 매칭
   - delta y 검증

4. **baseline 무회귀**: cargo test --lib 1122 GREEN + ignored 3 (#544 v3) 확인

5. **Phase A 방법 결정**:
   - 방법 1: layout.rs 에 `next_para_continues_visible_border: Cell<bool>` 도입. 다음 paragraph 가 prev 와 같은 border_fill_id 진행 중이면 set
   - 방법 2: `next_starts_border` 의미 확장 — "border 시작" → "다음 paragraph 가 border 가짐 (시작 + 진행 중 모두)"
   - 비교: 방법 2 가 더 단순 (Cell 1 개) 이지만 Task #552 의 의미 변경 (test_552 영향 가능). 방법 1 이 안전 (Task #552 보존).
   - Stage 1 결과로 결정.

### 산출

- `src/renderer/layout/integration_tests.rs` (+3 RED 테스트)
- `mydocs/working/task_m100_544_v3_stage1.md` (측정 + 본질 + 방법 결정)

### 커밋

`Task #544 v3 Stage 1: TDD RED 3건 + 정밀 진단 + Phase A 방법 결정`

---

## Stage 2 — Phase A 적용

### 작업

#### 방법 1 (예상 우선)

1. `src/renderer/layout.rs` LayoutEngine struct:
   ```rust
   /// [Task #544 v3] 박스 안 sequential paragraph (prev/next 같은 border_fill_id
   /// 진행 중) 시 prev paragraph 의 trailing-ls 보존. paragraph_layout 의
   /// is_full_paragraph_end 분기에서 본 Cell 검사.
   /// next_para_starts_visible_border (Task #552) 와 다름 — 본 Cell 은 다음
   /// paragraph 가 같은 box outline 안에서 진행 중일 때 set.
   next_para_continues_visible_border: std::cell::Cell<bool>,
   ```

2. layout.rs 에서 `next_para_starts_visible_border` set 하는 caller (~3 곳) 와 동등한 위치에 `next_para_continues_visible_border` set 추가:
   - 다음 paragraph (composed.get(item_para)) 의 border_fill_id 가 **prev paragraph 의 border_fill_id 와 동일** 이고 둘 다 > 0 이면 set
   - 호출 직후 reset

3. `src/renderer/layout/paragraph_layout.rs` is_full_paragraph_end 분기:
   ```rust
   let next_starts_border = self.next_para_starts_visible_border.get();
   let next_continues_border = self.next_para_continues_visible_border.get();
   if is_cell_last_line && cell_ctx.is_some() {
       y += line_height;
   } else if is_full_paragraph_end && cell_ctx.is_none()
       && !next_starts_border && !next_continues_border {
       y += line_height;
   } else {
       y += line_height + line_spacing_px;
   }
   ```

#### 방법 2 (대안)

`next_starts_border` 의 의미 확장. layout.rs 의 set 시 "다음 paragraph 가 border 가짐 (≥ 0 무관, 단 visible stroke 있는)" 으로 변경. test_552 영향 검증 필수.

### 검증

- TDD: test_544_v3_*_p2_4_6 / _p5_10_12 / _p11_22_24 RED → GREEN
- test_552_passage_box_top_gap_p2_4_6 (`--ignored`) GREEN 유지 (Task #552 무회귀)
- test_544 (#544 v2) / test_547 / test_548 GREEN 유지
- cargo test --lib: 1125 GREEN (1122 + 3)

### 커밋

`Task #544 v3 Stage 2: Phase A 적용 (박스 안 sequential paragraph trailing-ls 보존)`

---

## Stage 3 — Phase B (조건부) + 광범위 회귀

### 작업

#### Phase B (Stage 1 의 p2/5 부분 제외 본질에 따라 조건부)

p2/5 의 -5~6 px drift 가 Phase A 적용 후에도 남으면 (예: 프리(prev) paragraph 의 마지막 줄 line_spacing 가 716 HU 가 아닌 다른 값) Phase B 진행. 본질에 맞는 추가 fix.

p11 의 -9.55 px 는 Phase A 만으로 해결 (trailing-ls 716 HU 보존 → 24.21 px).

#### 광범위 회귀 검증

- 회귀 가드 8 suite: issue_301/418/501/505/514/516/530/546
- svg_snapshot 6/6
- 21_언어_기출 다른 페이지 무회귀:
  - [1~3] / [7~9] / [13~15] / [16~18] / [28~30]
  - 페이지 12 (Task #479 본질 fix 영역) 200 px drift 보존
- exam_kor / exam_eng / exam_math / exam_science / exam_social 6 fixture
- aift / treatise sample / hwpspec / 2010-01-06

### 커밋

`Task #544 v3 Stage 3: Phase B (필요 시) + 광범위 회귀 검증`

---

## Stage 4 — 최종 보고서 + orders 갱신

### 작업

1. clippy 검증 (본 task 신규 결함 0)
2. cargo build --release 성공 확인
3. WASM build 성공 확인 (선택 — Docker 필요)
4. 시각 판정 SVG 1차 → web Canvas 2차 (작업지시자 게이트)
5. `mydocs/report/task_m100_544_v3_report.md` 작성
6. `mydocs/orders/20260503.md` 갱신

### 커밋

`Task #544 v3 Stage 4: 최종 보고서 + orders 갱신`

---

## 머지 절차

본 task 는 `local/task544_v2` 위에 분기되어 있음. 본 task #544 v2 가 먼저 머지 후 v3 머지:

```
# Task #544 v2 머지 (시각 판정 통과 후)
git checkout local/devel
git merge local/task544_v2 --no-ff

# Task #544 v3 머지 (시각 판정 통과 후)
git checkout local/devel
git merge local/task544_v3 --no-ff

# devel push
git checkout devel
git merge local/devel --no-ff
git push origin devel
```

또는 v2 와 v3 가 함께 시각 판정 통과한다면 v3 만 머지 (v2 변경 포함).

## 위험 / 잔존 가능성

1. **Task #552 와의 의미 분리**: `next_para_starts_visible_border` (Task #552) vs `next_para_continues_visible_border` (Task #544 v3) — 두 Cell 의 set 조건 정확히 분리해야. set 위치 (caller) 도 분리.
2. **광범위 회귀**: 박스 안 sequential paragraph 의 line spacing 이 +9.55 px (716 HU) 만큼 늘어남. 박스 안에 paragraph 여러 개 있는 모든 케이스 영향. 시각 판정 게이트 필수.
3. **Phase B 본질 미식별 시**: p2/5 의 -5~6 px drift 가 Phase A 만으로 해결 안 되면 추가 진단 필요. Stage 1 의 가설 A/B 검증으로 사전 식별.

## 코드 영향 요약

| 파일 | 변경 LOC (예상) | 영역 |
|------|-----------------|------|
| `src/renderer/layout.rs` | +20 / -0 | LayoutEngine Cell + caller set 3 곳 |
| `src/renderer/layout/paragraph_layout.rs` | +3 / -1 | is_full_paragraph_end 분기 가드 추가 |
| `src/renderer/layout/integration_tests.rs` | +180 / -0 | TDD 3 건 |
