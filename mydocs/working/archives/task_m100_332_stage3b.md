# Task #332 Stage 3b — vpos correction 양방향 + collapse 가드 — 완료보고서

- **계획서**: `mydocs/plans/task_m100_332_impl.md`
- **브랜치**: `task332`
- **작성일**: 2026-04-25

---

## 변경 사항

### 코드 (`src/renderer/layout.rs:1392-1402`)

```diff
-                                // 자가 검증: 보정값이 컬럼 영역 내에 있고
-                                // 현재 y_offset보다 뒤로 가지 않아야 유효
-                                if end_y >= col_area.y && end_y <= col_area.y + col_area.height
-                                    && end_y >= y_offset - 1.0
-                                {
-                                    y_offset = end_y;
-                                }
+                                // 자가 검증: 보정값이 컬럼 영역 내에 있어야 유효.
+                                // Task #332: 양방향 보정 + collapse 가드 —
+                                // 기존 단방향(`end_y >= y_offset - 1.0`) 은 layout 이 vpos 보다 앞설
+                                // 때 보정 미적용 → drift 누적. 양방향으로 풀되, 비정상적인 큰 후퇴
+                                // (이전 문단과 같은 y 로 collapse) 만 가드하기 위해 backward 한도를
+                                // line_height 의 3 배로 제한한다 (소량 drift 보정은 허용).
+                                let max_backward_px = hwpunit_to_px(seg.line_height as i32, self.dpi) * 3.0;
+                                if end_y >= col_area.y && end_y <= col_area.y + col_area.height
+                                    && end_y >= y_offset - max_backward_px
+                                {
+                                    y_offset = end_y;
+                                }
```

`max_backward_px = line_height * 3.0` 가드: 3 line 이상의 후퇴는 비정상으로 간주. layout drift 보정용 소량 후퇴(수~수십 px)는 허용.

## 검증 결과

### 자동 테스트

```
cargo test --lib                  → 992 passed
cargo test --test '*' (기타)      → 모두 passed (회귀 없음)
cargo test --test svg_snapshot    → 4 passed, 2 FAILED (Stage 2 와 동일 baseline 차이, 추가 변동 없음)
```

### 수동 회귀 (21_언어 page 0)

| 단계 | LAYOUT_OVERFLOW |
|------|-----------------|
| Stage 3a | col=0 pi=10 partial (9.5px) — 1 건 |
| Stage 3b | col=0 pi=10 partial (9.5px) — **1 건 (변동 없음)** |

본 케이스는 vpos correction 자체가 trigger 되지 않는 경우(prev_layout_para 의 segment_width 가 col_width 와 일치하지 않거나 다른 조건 미만족) 라 양방향 가드 변경 영향 없음. **Stage 4 의 clamp pile 제거가 진짜 해결책.**

### 다른 샘플 회귀 비교

`hwp-multi-002.hwp`:
- 변경 전 (Stage 3a): `LAYOUT_OVERFLOW: page=2, col=0, para=67, type=Table, y=1078.2, bottom=1046.9, overflow=31.3px`
- 변경 후 (Stage 3b): 동일

Pre-existing 표 overflow 로 본 변경의 회귀 아님.

기타 회귀 샘플: `form-01`, `multi-table-001`, `lseg-06-multisize` — 모두 OVERFLOW 0 건, 정상.

## 다음 단계

Stage 4: `paragraph_layout.rs:807-816, 2529-2542` 의 clamp pile 분기 제거. overflow 시 line 을 그리지 않고 warn 로그만 출력. typeset 측 fit 정합으로 trigger 빈도 0 을 목표.
