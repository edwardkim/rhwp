# task #1022 다음 세션 재개 노트

- 브랜치: `local/task1022` (HEAD `0acd1861`)
- 명시 범위: HeightMeasurer ↔ cell_units 정합 — **Stage 3 완료 (42→38 events)**
- 사용자 결정 확장 범위: VPOS_CORR 정합 → **Stage 5 진행 중**

## 진행 상태

| 단계 | 산출물 | 상태 |
|------|--------|------|
| Stage 1 | `working/task_m100_1022_stage1.md` (8개 차이 식별) | ✅ |
| Stage 2 | `working/task_m100_1022_stage2.md` (방향 (a) C→A 채택) | ✅ |
| Stage 3 | `working/task_m100_1022_stage3.md` + 코드 변경 (`906b95b3`) | ✅ |
| Stage 4 | `working/task_m100_1022_stage4_progress.md` (VPOS_CORR 원인 발견 `0e475a9a`) | ✅ |
| Stage 5-1 | `working/task_m100_1022_stage5_1.md` (VPOS_CORR 감사) | ✅ |
| Stage 5-3 시도 | `working/task_m100_1022_stage5_3_attempt.md` (방향 3 실패) | ✅ |
| Stage 5 종합 | `working/task_m100_1022_stage5_final.md` (방향 1 분석 + 권고) | ✅ |

## 다음 세션 진행 시 필요한 컨텍스트

### 1. VPOS_CORR 발동 패턴 (페이지 22 사례)

`RHWP_VPOS_DEBUG=1` 로그 (이미 측정):
- pi=223 fresh-compute → +13.87
- pi=225 fresh-recompute (Table 후 reset) → +13.87
- pi=226 cached → 0

### 2. 핵심 산식

```
end_y = col_y + (vpos_end - lazy_base) * scale - curr_sb

lazy_base (fresh) = prev_vpos_end - y_delta_hu
y_delta_hu = (y_offset - col_y) / dpi * 7200 + trailing_ls_hu(prev)
```

fresh-compute 시 `+trailing_ls_prev` 가 effectively delta 로 들어감. cached 시 0.

### 3. 미러링 시 필요한 paginator 상태

- `vpos_lazy_base: Option<i32>` (set/reset lifecycle 추적)
- `prev_pi: Option<usize>`
- `last_item_is_partial_table_or_shape: bool` (VPOS_CORR skip gate)
- paragraph_layout 의 trailing_ls 정책 매트릭스 (`is_cell_last_line` × `cell_ctx` × `skip_advance_empty_wrap`)

### 4. 미러링 작업 항목

1. `layout.rs:2152~2625` 의 모든 set/reset 경로 추적 (line 2186, 2387, 2625 등).
2. `paragraph_layout` 의 모든 advance 경로 정책 추출.
3. paginator `TypesetState` 확장.
4. paginator `current_height` 누적 시점에 VPOS_CORR 효과 가산.
5. 광범위 골든·테스트 회귀 점검.

### 5. 회귀 위험 — 단순 미러 실패 사례

- 방향 3 (vpos_gap only): `test_task76_multi_001_group_images` 회귀, 페이지 +6.
- 일률 trailing_ls 가산 (5 transition × 13.87): 과대.
- 정밀 fresh/cached 구분 + 게이트 미러 필수.

### 6. 빌드/테스트 기준선

- `cargo test --release`: 1302 passed, 0 failed, 6 ignored (HEAD `0acd1861`).
- `cargo clippy --release`: 무경고.
- `LAYOUT_OVERFLOW` (비공개): **38건**.
- 페이지 22 18.3px 잔여 (목표 해소).

## 재개 절차

1. 브랜치 확인: `git checkout local/task1022`.
2. `working/task_m100_1022_stage5_*.md` 들 읽고 컨텍스트 복원.
3. Stage 5-2 정합 설계 시작 (방향 1 의 정밀 미러링 설계).
4. 단계적 구현·테스트.
