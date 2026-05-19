# 구현계획서 v3 — 분할 셀 줄 범위: 끝 페이지 패스 유도 방식

- 타스크: 로컬 task991
- 브랜치: `local/task991`
- v1/v2 폐기 사유: `task_m100_991_stage2.md` (1차 수정이 회귀 유발)
- 작성일: 2026-05-19

## 핵심 설계

`compute_cell_line_ranges` 가 분할 끝 페이지와 분할 시작/중간 페이지에서 **줄 분할을 독립 재계산**하지 않도록 한다. 모든 컷을 단일 "prefix 패스"(예산 budget_px 안에 들어가는 문단별 줄 수 계산)에서 유도한다.

- 끝 페이지(offset=0, limit=L): `result[pi] = (0, prefix(L)[pi])`
- 시작 페이지(offset=O, limit=0): `result[pi] = (prefix(O)[pi], line_count[pi])`
- 중간 페이지(offset=O, limit=L): `result[pi] = (prefix(O)[pi], prefix(O+L)[pi])`

`prefix(budget)` 는 끝 페이지 로직(현행 `!has_offset` 경로)을 그대로 사용한다. 끝 페이지와 시작 페이지가 **같은 코드 경로·같은 budget 기준**으로 컷을 계산하므로, `limit_reached` 전파·vpos 리셋(#697)·vpos 동기화(#700)가 양쪽에서 동일하게 작동 → 중복·누락이 정의상 불가능.

## 구현

### `compute_cell_line_ranges` 분기 추가

`has_offset` 일 때 새 경로:

```rust
if has_offset {
    let skip = self.cell_line_prefix_counts(cell, composed_paras, content_offset, styles);
    let keep = if has_limit {
        self.cell_line_prefix_counts(cell, composed_paras, content_offset + content_limit, styles)
    } else {
        composed_paras.iter().map(|c| c.lines.len()).collect()
    };
    return skip.iter().zip(keep.iter())
        .map(|(&s, &e)| (s, e.max(s)))
        .collect();
}
// 이하 기존 !has_offset(끝 페이지) 로직 — 무수정
```

### 신규 헬퍼 `cell_line_prefix_counts`

예산 안에 들어가는 문단별 prefix 줄 수를 반환. 끝 페이지 패스(`compute_cell_line_ranges` 를 `offset=0, limit=budget` 로 호출)의 결과에서 추출:

```rust
fn cell_line_prefix_counts(&self, cell, composed_paras, budget, styles) -> Vec<usize> {
    let ranges = self.compute_cell_line_ranges(cell, composed_paras, 0.0, budget, styles);
    ranges.iter().map(|&(s, e)| if s == 0 { e } else { 0 }).collect()
}
```

- `offset=0` 호출 → `has_offset=false` → 재귀 없이 기존 경로 → 무한재귀 없음.
- 끝 페이지 결과에서 `s==0` 이면 `e` 가 prefix 가시 줄 수, `s!=0`(한도 초과 스킵 마커)이면 prefix 0.

## 회귀 안전성

- 끝 페이지 경로(`!has_offset`)는 한 줄도 수정하지 않음 → 끝 페이지 렌더링 불변.
- 시작/중간 페이지만 새 경로. 기존 회귀(96쪽 누락)·중복(7쪽)이 동시 해소될 것으로 예상.
- 원자 문단(중첩 표 셀)·0줄 문단 경계 케이스는 검증 단계에서 골든 테스트로 확인.

## 검증 (3단계)

- `cargo test` 전체 + `cargo clippy` 무경고.
- 골든 SVG 테스트(분할 표 케이스 포함) 회귀 없음.
- 비공개 샘플 180쪽 전수: 수정 전후 텍스트 비교 → 변경된 쪽 각각을 한컴 PDF 와 대조해 정정/회귀 판정. 특히 7·96·127·168쪽.
- 분할 셀 줄의 중복·누락이 0건인지 확인.
- WASM 영향 시 Docker 재빌드.

## 단계

- 2단계(v3): 위 구현 + 단위 검증 → `task_m100_991_stage2_v2.md` + 커밋.
- 3단계: 전체 회귀 검증 + 최종 보고서.

## 범위 / 비공개 문서

기존과 동일 — `compute_cell_line_ranges` 한정, 페이지네이션·파서·HWP3 제외. 비공개 HWPX/PDF 미커밋, 문서에 식별정보 미기재.
