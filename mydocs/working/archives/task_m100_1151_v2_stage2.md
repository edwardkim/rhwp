# Task #1151 v2 Stage 2 완료 보고서 — 한컴 산출물 SVG 좌표 검증 + 양방향 정합 통합 테스트 4 PASS

수행계획서: [task_m100_1151_v2.md](../plans/task_m100_1151_v2.md) · 구현계획서: [task_m100_1151_v2_impl.md](../plans/task_m100_1151_v2_impl.md) · Stage 1 보고서: [task_m100_1151_v2_stage1.md](task_m100_1151_v2_stage1.md)

## 1. Stage 2-a — 한컴 산출물 SVG 좌표 변화 비교

`samples/tac-verify/scenario-{a,b,c,d}-{before,after}.hwp` 8개를 `rhwp export-svg` 로 SVG 변환 후 picture / table rect 좌표 변화 추출.

### Picture (`<image>`) 좌표 변화

| Scenario | before image (x, y) | after image (x, y) | image size (px) |
|----------|---------------------|---------------------|------------------|
| A — 1×1 작은 picture | (146.44, 180.00) | **(113.39, 132.27)** | 79.7 × 71.1 |
| B — 1×1 큰 picture | (106.59, 129.28) | **(113.39, 132.27)** | 234.6 × 213.8 |
| C — 3×3 중앙 (1,1) | (352.03, 292.31) | **(113.39, 132.27)** | 72.5 × 64.6 |
| D — 본문 floating | (179.04, 180.91) | **(113.39, 132.27)** | 278.2 × 253.6 |

**관찰**: after 의 picture 가 4 시나리오 모두 (113.39, 132.27) = body 좌상단으로 통일 (inline 위치 = paragraph 첫 글리프 자리). before 의 picture 는 한컴이 만든 floating Paper-relative 좌표. picture 의 width/height 는 토글 전후 동일.

### Paragraph 다음 line rect 변화 (Scenario A 예시)

| 상태 | rect (x, y, w, h) | 의미 |
|------|---------------------|------|
| before | (113.39, **314.91**, 566.93, 21.33) | 표 + 빈 paragraph 0.1 의 line. 표 line_height=1000 HU |
| after | (113.39, **219.35**, 566.93, 21.33) | 표 + 빈 paragraph 0.1 의 line. 표 line_height=5331 HU = picture height |

표 rect 자체 위치는 before/after 동일 (paragraph 의 control 로 anchor 점 고정). paragraph 의 layout 흐름이 picture height 만큼 자라면서 후속 paragraph 의 위치가 변동.

## 2. Stage 2-b — 양방향 정합 통합 테스트 4 PASS

`#[cfg(test)] mod issue_1151_v2_tac_toggle_tests` 에 통합 검증 추가 (object_ops.rs 6000-6090 근방).

### 검증 흐름

```text
1. samples/tac-verify/scenario-X-before.hwp 파싱 → Document
2. DocumentCore::set_document
3. paragraph 0.0 의 Picture control 찾기
4. set_picture_properties_native(0, 0, ctrl_idx, r#"{"treatAsChar":true}"#)
5. samples/tac-verify/scenario-X-after.hwp 파싱 → 한컴 정답 Document
6. (4) 의 결과 picture / line_segs[0] / paragraph 수 / picture 위치 / paragraph.text
   를 (5) 와 단언 일치
```

### 단언 항목

| 필드 | 단언 |
|------|------|
| `picture.common.treat_as_char` | rhwp 토글 결과 == 한컴 산출물 |
| `picture.common.horizontal_offset` | == |
| `picture.common.vertical_offset` | == |
| `picture.common.horz_rel_to` | == |
| `picture.common.vert_rel_to` | == |
| `paragraph.line_segs[0].line_height` | == |
| `paragraph.line_segs[0].text_height` | == |
| `paragraph.line_segs[0].baseline_distance` | == (`round(line_height × 0.85)` 정합) |
| paragraph 수 | == |
| Picture control_idx | == |
| `paragraph.text` | == |

### 통합 테스트 결과

| 테스트 | 결과 |
|--------|------|
| `integration_tac_toggle_matches_hancom_scenario_a` | PASS ✓ |
| `integration_tac_toggle_matches_hancom_scenario_b` | PASS ✓ |
| `integration_tac_toggle_matches_hancom_scenario_c` | PASS ✓ |
| `integration_tac_toggle_matches_hancom_scenario_d` | PASS ✓ |

**v2 fix 가 만든 model = 한컴이 만든 model** — 4 시나리오 모두 model 수준 dump 동치.

## 3. 자동 검증 결과 (Stage 2 합산)

| 항목 | 결과 |
|------|------|
| `cargo test --lib issue_1151_v2_tac_toggle` | **11 passed, 0 failed** (6 단위 + 4 통합 + 1 LineSeg::default) ✓ |
| `cargo test --lib` 전수 | **1436 passed, 0 failed, 6 ignored** (회귀 0) ✓ |
| `cargo clippy --lib -- -D warnings` | clean ✓ |
| `cargo fmt --all -- --check` | clean ✓ |

Stage 1 (1432 + clippy + fmt) 대비 Stage 2 에서 +4 통합 테스트가 추가되어 총 **+4 (1436 합)**. 회귀 없음.

## 4. WASM 빌드 + 브라우저 시각 검증 — 선택 사항

Stage 2-b 의 자동 통합 검증으로 model 수준의 양방향 정합이 확정되었으므로 WASM 빌드 + 브라우저 직접 시연은 **선택 사항**. 본 task 가 UI 변경 없음 (Rust API 만 + dialog 주석 1-2 줄 갱신은 Stage 3 에 묶음) 이므로 dev server 시각 확인은 PR 단계 또는 사용자 직접 시연으로 가능.

진행 옵션 (사용자 결정):
- (a) Stage 2 종료 → Stage 3 (PR + dialog 주석 갱신 + 최종 보고서) 진입
- (b) WASM 빌드 + rhwp-studio dev server 시작 + 작은/큰 picture 토글 직접 시연 후 Stage 3

## 5. Stage 3 진입 조건

- 단위/통합 테스트 11/11 PASS ✓
- 전수 회귀 1436 PASS ✓
- 양방향 정합 (rhwp ↔ 한컴) model 수준 확정 ✓
- 한컴 산출물 SVG 좌표 변화로 시각 layout 정합 확인 ✓

→ Stage 3 (picture-props-dialog.ts 주석 갱신 + 신규 PR 발행 closes #1151 + 최종 보고서) 진행 가능.
