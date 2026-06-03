# Task #517 최종 보고서 — Layout 리팩터링 Phase 1

**이슈**: #517
**브랜치**: `local/task517`
**Phase**: 1 (디버그 인프라 + 회귀 검증 도구)

## 1. 작업 내용

Phase 2~4 (본질 정정) 의 선결 인프라 추가.

## 2. 변경 파일

| 파일 | 변경 |
|------|------|
| `src/renderer/layout/paragraph_layout.rs` | `layout_debug_enabled()` + `layout_inline_table_paragraph` 진단 로깅 (env-var-checked) |
| `scripts/svg_regression_diff.sh` | 신규 (130 LOC) — build/diff 두 모드 |
| `mydocs/manual/svg_regression_diff.md` | 사용 매뉴얼 |
| `mydocs/plans/task_m100_517.md` | 수행계획서 |
| `mydocs/working/task_m100_517_stage{1,2}.md` | 단계별 보고서 |
| 본 보고서 | |

## 3. 검증

### 3-1. 기능 검증

- Stage 1 logging: exam_science p2 pi=61 (#496 재현) 에서 `ls_count=3 tables=1 rows=2` 등 핵심 정보 출력 확인
- Stage 2 diff: 기존 /tmp/task500_{before,after} 비교에서 의도된 1페이지 정정 정확 식별

### 3-2. 회귀 검증

- `cargo test --release`: **1103 passed; 0 failed; 1 ignored**
- `scripts/svg_regression_diff.sh build devel HEAD`: **170/170 byte 동일, diff=0**

env-var-checked 로깅 + 별도 script 만 추가 → 기본 동작 변경 없음.

## 4. 영향 범위

| 케이스 | 영향 |
|--------|------|
| 기본 export-svg 동작 | 변화 없음 (env var 미지정 시 logging 미동작) |
| `RHWP_LAYOUT_DEBUG=1` 지정 시 | layout_inline_table_paragraph 진입 시 진단 로깅 출력 |
| 단위/통합 테스트 | 영향 없음 |

## 5. 활용

### `RHWP_LAYOUT_DEBUG=1`

```bash
RHWP_LAYOUT_DEBUG=1 ./target/release/rhwp export-svg samples/exam_science.hwp -p 1 2>&1 | grep "LAYOUT_"
```

### `scripts/svg_regression_diff.sh`

```bash
# 두 commit 비교
./scripts/svg_regression_diff.sh build devel local/task<N>

# 두 디렉토리 비교
./scripts/svg_regression_diff.sh diff /tmp/before /tmp/after
```

## 6. 후속

Phase 2 (line_break_char_idx 다중화) 진행 시 본 인프라 사용:
- 변경 전후 `RHWP_LAYOUT_DEBUG=1` 로 결함 케이스 baseline 측정 비교
- `svg_regression_diff.sh build devel local/taskN` 으로 170 페이지 광범위 회귀 검증

## 7. 요약

- Layout 디버깅 인프라 (`RHWP_LAYOUT_DEBUG=1`) ✓
- 회귀 검증 도구 (`scripts/svg_regression_diff.sh`) ✓
- 회귀 0건 (170/170 byte 동일, 1103 단위 테스트 통과) ✓
- Phase 2~4 진행을 위한 도구 기반 마련 ✓
