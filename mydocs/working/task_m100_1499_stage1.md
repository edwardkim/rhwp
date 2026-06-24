# 단계별 완료보고서 — Task #1499 단계 1: 비교 코어 모듈

- **이슈**: #1499 · **브랜치**: `local/task1499`

## 작업 내용

`src/diagnostics/render_geom_diff.rs` 신설 — 폰트 비의존 결정론적 렌더 기하 비교 코어.

- `type_tag(&RenderNodeType) -> &'static str`: 24개 variant 판별자 → 안정 태그 (텍스트/스타일 무시).
- `flatten_page(&RenderNode, &mut Vec<FlatNode>)`: 전위순회 평탄화.
- `lcs_match(&[FlatNode], &[FlatNode])`: 태그 시퀀스 LCS(DP) → 매칭/삽입/삭제 페어.
- `diff_page`: 매칭쌍 변위 `max(|Δx|,|Δy|,|Δw|,|Δh|)` + 삽입/삭제 카운트.
- `diff_documents_geom(&HwpDocument, &HwpDocument, threshold)`: 페이지별 비교 → `DocGeomDiff`.
- `Verdict { Pass, StructMismatch, DispOver }` + 등급 함수.
- `DEFAULT_THRESHOLD_PX = 0.5`.

모듈은 `diagnostics/` 내부에만 두어 공통 모듈(`renderer/`, `document_core/`)을 건드리지 않았다.

## 검증

`cargo test --lib render_geom_diff` — 5 케이스 PASS:
1. 동일 시퀀스 → 변위 0, 삽입/삭제 0
2. 1px 평행이동 → 변위 1.0, worst 태그 식별
3. 노드 삽입 → inserted 1
4. 노드 삭제 → deleted 1
5. 중간 삽입 시 LCS 정렬 → 매칭 노드 변위 오탐 0

## 다음 단계

단계 2: `render-diff` CLI 3모드(자기 라운드트립 / 두 파일 / `--batch` geom_inventory.tsv) 배선.
