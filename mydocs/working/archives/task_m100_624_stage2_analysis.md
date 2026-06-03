# Task #624 Stage 2 분석 보고서 (코드 수정 미진입)

> 사용자 지시: "분석만 계속 진행" — Stage 2 정정 적용 보류, 영향 범위·edge case·후속 진단 분석으로 전환.

---

## 1. 광범위 sweep 결과 (정정 시뮬레이션)

이전 stash/pop 기반 sweep (정정 미커밋 상태에서 빌드 → SVG 생성 → revert) 의 실측 데이터:

| 샘플 | 페이지 | 변경 페이지 | 비고 |
|---|---|---|---|
| `exam_science.hwp` | 4 | **1** (page 2) | ㉠ 사각형 + ㉠ 텍스트 y +21.47 px |
| `exam_kor.hwp` | 20 | 0 | (multi-line + tac rect 있으나 변경 없음) |
| `exam_math.hwp` | 20 | 0 | |
| `exam_eng.hwp` | 8 | 0 | |
| `synam-001.hwp` | 35 | 0 | (multi-line + tac rect 3 개 있으나 변경 없음) |
| `21_언어_기출_편집가능본.hwp` | 15 | 0 | |
| **합계** | **102** | **1** | 0.98% — 매우 좁은 회귀 |

### 1.1 exam_science_002.svg 의 정확한 변경

```diff
-<rect x="117.066" y="213.946" width="62.986" height="22.880" fill="#ffffff" stroke="#000000" stroke-width="0.5"/>
-<text x="141.56"  y="229.986" ...>㉠</text>
+<rect x="117.066" y="235.413" width="62.986" height="22.880" fill="#ffffff" stroke="#000000" stroke-width="0.5"/>
+<text x="141.56"  y="251.453" ...>㉠</text>
```

- `<rect>` y: 213.95 → 235.41, Δ = +21.47 px
- `<text>㉠` y: 229.99 → 251.45, Δ = +21.47 px
- Δ 21.47 px = (3220 - 1610) HU / 75 = exact ls[1].vpos − ls[0].vpos.

→ 정정 효과는 ㉠ 사각형 + 그 안 ㉠ 텍스트가 line 2 영역으로 정확히 이동.

## 2. 회귀 발생 조건 (3 항 모두 충족 시)

`table_layout.rs` 의 `Control::Shape(shape) if shape.common().treat_as_char` 분기 (line 1671~1820) 에서:

| 조건 | exam_science p[1] | exam_kor p[0] | synam-001 p[6/0/8] | 21_언어 p[0] | k-water p[0] |
|---|---|---|---|---|---|
| (a) cell 안 paragraph | ✓ | ✓ | ✓ | ✓ | ✓ |
| (b) `line_segs.len() ≥ 2` (multi-line) | ✓ (2) | ✓ (4) | ✓ (2/3/3) | ✓ (10) | ✓ (2) |
| (c) `target_line > 0` (rect on ls[1]+) | ✓ | ? | ? | ? | ? |
| (d) `first_vpos > 0` (paragraph[i>0]) | **✓ (1610)** | ✗ (0) | ✗/✗/✗ | ✗ (0) | ✗ (0) |
| **회귀 가시화** | **YES** | NO | NO | NO | NO |

→ 본 회귀는 **(c) AND (d)** 두 조건이 동시 성립해야 가시화.
→ exam_science p[1] 만이 유일한 가시 회귀 사례 — `first_vpos = 1610 HU` 로 paragraph[1]+ 위치이며 rect 가 ls[1] 에 있음.

### 2.1 다른 multi-line + tac rect 케이스 무회귀 사유

- `exam_kor` p[0] (paragraph 0, first_vpos=0): rect 의 target_line 무관, first_vpos=0 으로 산식 영향 없음. shape_area.y 차이는 ls[0] 와 ls[1] 모두 같은 para_y_before_compose 일 때만 동일 (즉 target_line=0 이면 tac_img_y == para_y_before_compose, target_line=1 이어도 ls[0].vpos=0 인 경우 첫 line 시작 = paragraph 시작이라 일치).
- `synam-001` p[6/p[0]/p[8]: wrap=InFrontOfText / Square — 본 분기는 `treat_as_char=true` 만 처리, wrap 모드 무관 (단지 IR 표기). 실제 표시 위치는 inline 룰 적용. 그러나 first_vpos=0 케이스로 산식 차이 무관.
- `21_언어` p[0] / `k-water` p[0]: 동일 — first_vpos=0.

## 3. 정정의 안전성 (edge case)

### 3.1 첫 paragraph (`first_vpos = 0`) 무회귀

```
seg.vpos - first_vpos = seg.vpos - 0 = seg.vpos
```
→ 산식 변경 후도 동일 동작. cell 첫 paragraph 의 모든 케이스에서 무회귀.

### 3.2 첫 줄 (`target_line = 0`) 무회귀

```
target_line > current_tac_line 가드 (line 1693)
```
→ target_line == 0 (current_tac_line 도 0) 이면 if 블록 진입 안 함, tac_img_y 갱신 없음, 초기값 = para_y_before_compose 유지. 즉 shape_area.y = tac_img_y == para_y_before_compose 로 정정 후도 동일 동작.

### 3.3 단일 줄 paragraph (`line_segs.len() == 1`) 무회귀

target_line 산출 시 `composed.lines` 가 1 줄이면 `target_line = 0`. 동일 가드로 무회귀.

### 3.4 wrap=Square / InFrontOfText / TopAndBottom

본 분기는 `treat_as_char=true` 만 처리. wrap 모드는 IR 보존만 하고 inline 배치 룰 자체에는 영향 없음 (다른 분기가 wrap 모드 사용). 즉 본 정정의 wrap 모드별 영향 차이 없음.

## 4. Picture 분기 산식 회귀의 별도 영향

### 4.1 패턴 검색 결과

`Picture(pic) if pic.common.treat_as_char` 분기 안 paragraph[i>0] (`first_vpos > 0`) + multi-line + ls[1]+ 위치 picture:

```
scan: 0 hits (158 sample × 평균 dump LOC)
```

→ 검토 가능한 sample 풀에서 가시 회귀 0 건. Picture 산식 회귀는 **이론적 결함**으로만 존재 (defensive consistency 차원).

### 4.2 정정 권고 (Picture 산식)

Shape 분기와의 **일관성** 차원에서 함께 복원 권고. Task #520 의 본질이 "Picture/Shape 양 분기 동일 산식" 임을 commit message 에서 명시 (`313e65d`).

## 5. 회귀 출처의 본질 — PR cherry-pick 의 함정

`git diff 9dc40dd 3de0505 -- src/renderer/layout/table_layout.rs`:

원저자 `9dc40dd` (2026-05-04) 는 Task #544 v2 Stage 3 에서 Phase C (#548) 만 추가하면서 Task #520 의 산식 복원을 **유지**. 그러나 PR #561 cherry-pick 의 메인테이너 정리 단계에서 conflict resolution 또는 cherry-pick base 차이로 인해 Task #520 의 일부가 누락되어 `3de0505` 로 commit.

### 5.1 정정 경로의 분기 상태

| 브랜치 | shape_area.y | Picture tac_img_y | Shape tac_img_y |
|---|---|---|---|
| `local/devel` (e9f3562) | `tac_img_y` ✓ | `seg.vpos - first_vpos` ✓ | `seg.vpos - first_vpos` ✓ |
| `pr-task618` (7ac8777) | `para_y_before_compose` ✗ | `seg.vpos` ✗ | `seg.vpos - first_vpos` ✓ |
| `devel` (9b49063) | `para_y_before_compose` ✗ | `seg.vpos` ✗ | `seg.vpos - first_vpos` ✓ |

→ `local/devel` 은 회귀 없음. `devel`/`pr-task618` 은 회귀.
→ `local/devel..devel` 사이 173 commit 분기 — `local/devel` 이 먼저 나뉘어 origin/devel 로 진행, 이후 cherry-pick 들이 `devel` 에 누적되며 회귀 유입.

### 5.2 후속 권고

1. **PR cherry-pick 시 base diff 자동 점검**: 동일 함수 영역의 산식이 base 와 cherry-pick 후 차이 발생 시 경고. 본 결함은 `9dc40dd → 3de0505` 비교만으로 즉시 식별 가능.
2. **Task #520 회귀 방지 회귀 테스트**: 본 task 의 `test_624_textbox_inline_shape_y_on_line2_p2_q7` 가 RED → GREEN 가드 역할. 추가 cherry-pick 시 본 테스트 RED 발생하면 즉시 차단.
3. **`local/devel` ↔ `devel` 동기화 점검**: 173 commit 차이의 다른 영역에도 유사 회귀 가능. `local/devel` ahead 인 영역의 fix 들 (Task #520 외) 도 `devel` 에 정확히 반영되어 있는지 검증.

## 6. 정정 적용 보류 결정 (분석 모드)

본 보고서는 분석에 집중하며 Stage 2 코드 정정은 **보류**. 작업지시자 승인 후 진행.

### 6.1 정정 시 체크리스트 (참고)

- [ ] `src/renderer/layout/table_layout.rs:1606` Picture tac_img_y 산식 (3 line 추가)
- [ ] `src/renderer/layout/table_layout.rs:1816` Shape `shape_area.y` (1 line)
- [ ] `src/renderer/layout/table_layout.rs:1820` Shape `layout_cell_shape` para_y arg (1 line)
- [ ] `cargo test --lib test_624_textbox_inline_shape_y_on_line2_p2_q7` GREEN 전환
- [ ] `cargo test --lib` 1135 passed (회귀 0)
- [ ] `cargo test --test svg_snapshot` 6/6 GREEN
- [ ] `cargo clippy --lib -- -D warnings` clean
- [ ] 광범위 sweep 102 페이지 (현재 데이터) — 1/102 변경 (의도된 정정), 회귀 0
- [ ] 추가 sweep 권고: 158 sample 전체 fixture 회귀 0 확인

### 6.2 정정 미진행 사유

작업지시자 명시 — "분석만 계속 진행". 정정 적용은 후속 명시 승인 후.

---

## 7. 핵심 결론

1. **본질**: PR #561 cherry-pick (`3de0505`) 시 Task #520 의 일부 정정 누락 — 3 라인 회귀.
2. **가시 영향**: `samples/exam_science.hwp` page 2 7번 글상자 ㉠ 사각형 1 건 (102 sweep 페이지 중 1).
3. **정정 안전성**: `first_vpos = 0` AND `target_line = 0` 케이스 모두 동일 동작 보장 — edge case 회귀 0.
4. **Picture 산식 회귀**: 가시 영향 0 건이나 Shape 와의 산식 일관성 차원에서 복원 권고.
5. **방지책**: cherry-pick base diff 자동 점검 + 본 task 의 회귀 테스트 가드 도입.

분석 완료. 후속 명시 승인 시 Stage 2 코드 정정 진행 가능.
