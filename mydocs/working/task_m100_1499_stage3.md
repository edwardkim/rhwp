# 단계별 완료보고서 — Task #1499 단계 3: 회귀 baseline 게이트

- **이슈**: #1499 · **브랜치**: `local/task1499`

## 작업 내용

`tests/visual_roundtrip_baseline.rs` 신설 — `samples/hwpx` 전수 렌더 기하 정합성 게이트.
`hwpx_roundtrip_baseline` (Task #1315) 등급제·신규 자동 포함 관례를 미러링.

- `render_geom_diff::self_roundtrip_diff()` 를 공개 헬퍼로 분리(CLI/테스트 공유).
- `visual_baseline_all_samples`: XFAIL/EXCLUDED 제외 전수 PASS 단언.
- `xfail_entries_still_pass_promote`: XFAIL 이 PASS 하면 승격 알림(실패).
- `grade_lists_are_consistent`: 목록 실재 가드.
- 임계 `DEFAULT_THRESHOLD_PX = 0.5` (PASS 샘플 변위 전부 0.0px → 여유 확보).

## 등급 분류 (측정 `output/poc/task1499/geom_inventory.tsv`)

| 등급 | 건수 | 내용 |
|------|------|------|
| PASS (baseline) | 36 | 라운드트립 렌더 불변 — 회귀 방지 고정 |
| XFAIL | 20 | IrDiff 0 인데 렌더 변동(이미지/노드 누락·변위) — 후속 수정 대상 |
| EXCLUDED | 1 | `hwpx-01.hwpx` (HWP5 매직 d0cf11e0, HWPX 아님) |

XFAIL 중 단순문서 7건(el-school-001 Image 누락 등)은 후속 우선 수정 후보,
복합 실문서(ORACLE_UNFIT) 13건은 표·그림·다단 혼합 known 변화.

## 검증

`cargo test --test visual_roundtrip_baseline` — 3 테스트 PASS (34.7s).

## 다음 단계

단계 4: 매뉴얼(`render_diff_command.md`) + CLAUDE.md 한 줄 + 최종 보고서 + 전체 cargo test.
