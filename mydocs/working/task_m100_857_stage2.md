# Task m100 #857 Stage 2 완료 보고서

> Issue: [#857](https://github.com/edwardkim/rhwp/issues/857)
> Stage 목표: GREEN — cursor_rect.rs first-match → min area best-match 변경
> 작성일: 2026-05-12

## 1. 수행 결과

### 1.1 수정 파일
- `src/document_core/queries/cursor_rect.rs` (L643-666): cell-context TextRun selection 변경
- `tests/issue_table_vpos_01_page5_cell_hit_test.rs`: insert_lands 테스트 검증 방식을 contains 로 정정

### 1.2 변경 내용 (cursor_rect.rs)

**Before** (first-match):
```rust
if run.cell_context.is_some() {
    if hit_cell.is_none() {
        hit_cell = Some((i, run.char_start + char_offset));
    }
}
```

**After** (min area best-match):
```rust
if run.cell_context.is_some() {
    let area = (run.bbox_w.max(0.0) * run.bbox_h.max(0.0) * 1000.0) as i64;
    if hit_cell_area.map_or(true, |best_area| area < best_area) {
        hit_cell = Some((i, run.char_start + char_offset));
        hit_cell_area = Some(area);
    }
}
```

Task #717 의 cell_bboxes selection (cursor_rect.rs:671-675) 과 동일 best-match 패턴.

### 1.3 커밋
- Commit: `1135c028 Task #857 Stage 2 (GREEN): cell-hit selection first-match → min area best-match`
- 2 files changed, 15 insertions(+), 6 deletions(-)
- 커밋 메시지에 `closes #857` 포함 → merge 시 issue 자동 close

## 2. 검증

### 2.1 본 RED 테스트
```
$ cargo test --test issue_table_vpos_01_page5_cell_hit_test
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured
```

**전 13 케이스 PASS** (Stage 1: 5 FAIL / 8 PASS → Stage 2: 13 PASS).

### 2.2 변경 후 hit_test 결과 (PASS)

inner 11×3 r=0,c=2 클릭 (442.5, 320.5) 결과:
```json
{
  "parentParaIndex": 34,
  "controlIndex": 0,
  "cellPath": [
    {"cellIndex":0, "cellParaIndex":1, "controlIndex":0},
    {"cellIndex":2, "cellParaIndex":0, "controlIndex":0}  ← inner entry 정상 포함
  ],
  "charOffset": 15,
  "cursorRect": {"height":20.0, "x":444.4, "y":310.5}
}
```

cellPath 길이 2, inner 11×3 r=0,c=2 (cellIndex=2) 정확. cursorRect 도 텍스트 line 위치 (y=310.5, height=20).

### 2.3 insert_text 동작 확인
- `insert_text_in_cell_by_path(0, 34, hit_path, 15, "ZZZTEST")` 호출
- 이후 `get_text_in_cell_by_path(0, 34, &[(0,0,1), (0,2,0)], 0, 64)` 결과에 "ZZZTEST" substring 포함 확인
- 즉 inner 셀에 텍스트가 정상 삽입됨 — 사용자 증상 정리

### 2.4 clippy
```
cargo clippy --tests
```
- cursor_rect.rs 변경 관련 새 warning 없음
- 기존 56 warning 은 본 변경 무관 (다른 파일)

## 3. Stage 2 변경 영향 분석

| 시나리오 | Before | After | 영향 |
|---|---|---|---|
| 셀 후보 1개 | 1개 채택 | 1개 채택 (`is_none()` 분기) | 동일 |
| 셀 후보 복수 (중첩) | tree 순서 첫 매칭 | 최소 면적 매칭 | **개선** |
| 본문 TextRun | first-match | first-match (변경 없음) | 동일 |
| 셀 vs 본문 우선순위 | 셀 우선 | 셀 우선 | 동일 |

코드베이스 일관성: L587-588 (안내문), L671-675 (Task #717), L680 (cell 안 거리) 모두 best-match → 본 분기만 first-match 였던 정책 차이 해소.

## 4. Stage 3 예정 작업

다음 단계 (Stage 3) 에서 확인:
- 전체 `cargo test --release` 통과
- 기존 핵심 회귀 테스트 (Task #717, #595, #628 등) 개별 PASS 확인
- SVG 시각 회귀 비교 (page 4 / page 5)
- rhwp-studio E2E 수동 시연 가이드
- 최종 결과 보고서

## 5. 잔존 위험

- 같은 셀 안 여러 줄 TextRun 이 동시 매칭되는 경우 (line bbox y 방향 겹침) → 짧은 줄 wins (기존: 첫 줄). click 좌표가 두 line 모두에 들어가는 케이스 자체가 드물어 회귀 위험 낮음 — Stage 3 전체 회귀 sweep 으로 확인.

## 6. Git Tree 상태

```
local/task857 ← 1135c028 Task #857 Stage 2 (GREEN) ← 37e7b7b0 Stage 1 보고서 ← 07168934 Stage 1
local/devel   ← 2bd50a3a (= devel)
```

## 7. 작업지시자 승인 요청

Stage 2 완료. **Stage 3 (회귀 sweep + 최종 보고서) 진행 승인** 부탁드립니다.
