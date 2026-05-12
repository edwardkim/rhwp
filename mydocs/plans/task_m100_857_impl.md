# Task m100 #857 구현 계획서

> Issue: [#857 — table-vpos-01.hwp p.5 중첩 11×3 표 c=2 column 셀 클릭 misroute](https://github.com/edwardkim/rhwp/issues/857)
> 수행 계획서: [task_m100_857.md](task_m100_857.md)
> 브랜치: `local/task857`
> 작성일: 2026-05-12

## 1. 수정 범위

**파일**: [src/document_core/queries/cursor_rect.rs](../../src/document_core/queries/cursor_rect.rs)
**라인**: L643-666 (1차 bbox 매칭 분기)
**다른 파일은 건드리지 않음** — 본 버그는 hit-test 선택 정책 단일 결함.

## 2. 핵심 변경 내용

### 2.1 현재 코드 (L643-666)

```rust
// 1. 정확한 bbox 히트 검사
let mut hit_body: Option<(usize, usize)> = None;
let mut hit_cell: Option<(usize, usize)> = None;
for (i, run) in runs.iter().enumerate() {
    if x >= run.bbox_x && x <= run.bbox_x + run.bbox_w
        && y >= run.bbox_y && y <= run.bbox_y + run.bbox_h
    {
        let local_x = x - run.bbox_x;
        let char_offset = find_char_at_x(&run.char_positions, local_x);
        if run.cell_context.is_some() {
            if hit_cell.is_none() {                 // ← 첫 매칭 wins
                hit_cell = Some((i, run.char_start + char_offset));
            }
        } else if hit_body.is_none() {
            hit_body = Some((i, run.char_start + char_offset));
        }
    }
}
// 셀/글상자 히트가 있으면 우선
if let Some((idx, offset)) = hit_cell.or(hit_body) {
    return Ok(format_hit(&runs[idx], offset, page_num));
}
```

### 2.2 변경 후 코드

```rust
// 1. 정확한 bbox 히트 검사
// 셀/글상자 TextRun을 본문 TextRun보다 우선한다.
// 셀 후보가 여럿이면 bbox 면적이 가장 작은 것 = 가장 specific 한 셀 선택
// (Task #717 의 cell_bboxes selection L671-675 와 동일 best-match 패턴 — closes #857).
// 중첩 표에서 외곽 셀의 빈 placeholder TextRun (bbox 가 paragraph 영역 전체) 이
// inner cell 의 실제 TextRun (작은 bbox) 보다 트리 순서상 먼저 매칭되어
// 외곽이 선점되던 결함 정정.
let mut hit_body: Option<(usize, usize)> = None;
let mut hit_cell: Option<(usize, usize)> = None;
let mut hit_cell_area: Option<i64> = None;
for (i, run) in runs.iter().enumerate() {
    if x >= run.bbox_x && x <= run.bbox_x + run.bbox_w
        && y >= run.bbox_y && y <= run.bbox_y + run.bbox_h
    {
        let local_x = x - run.bbox_x;
        let char_offset = find_char_at_x(&run.char_positions, local_x);
        if run.cell_context.is_some() {
            let area = (run.bbox_w.max(0.0) * run.bbox_h.max(0.0) * 1000.0) as i64;
            if hit_cell_area.map_or(true, |best_area| area < best_area) {
                hit_cell = Some((i, run.char_start + char_offset));
                hit_cell_area = Some(area);
            }
        } else if hit_body.is_none() {
            hit_body = Some((i, run.char_start + char_offset));
        }
    }
}
// 셀/글상자 히트가 있으면 우선
if let Some((idx, offset)) = hit_cell.or(hit_body) {
    return Ok(format_hit(&runs[idx], offset, page_num));
}
```

### 2.3 변경 영향

- **셀 vs 본문 우선순위**: 변경 없음 (셀 우선 유지)
- **셀 후보 단일**: 동작 동일 (`hit_cell_area.map_or(true, ...)` 가 첫 후보 무조건 채택)
- **셀 후보 복수 (중첩 표)**: bbox 면적 최소 선택 → 본 버그 해결
- **본문 TextRun**: 변경 없음 (first-match 유지)
- **코드 일관성**: 같은 함수 L671-675 (Task #717) / L587-588 (안내문) / L680 (cell 안 run 거리) 모두 `min_by_key` best-match → 본 분기만 유일했던 first-match 제거로 정책 통일

## 3. Stage 분할

### Stage 1 — 진단 산출물 commit (RED 캡처)

**목표**: 현재 RED 상태(5 FAIL / 8 PASS)를 git history 에 남기고 진단 노트 보존.

**작업**:
1. 다음 untracked 파일을 add:
   - `tests/issue_table_vpos_01_page5_cell_hit_test.rs`
   - `mydocs/troubleshootings/table_vpos_01_page5_cell_hit_test.md`
   - `mydocs/plans/task_m100_857.md`
   - `mydocs/plans/task_m100_857_impl.md` (본 파일)
2. 커밋: `Task #857 Stage 1 (RED): 진단 노트 + 회귀 테스트 + 계획서 추가`
3. **이 단계에서 소스 코드 수정 없음**.

**완료 조건**:
- `cargo test --test issue_table_vpos_01_page5_cell_hit_test` → 5 FAIL / 8 PASS (현 상태 유지)
- git status clean

**보고서**: `mydocs/working/task_m100_857_stage1.md`

### Stage 2 — GREEN: cursor_rect.rs 수정

**목표**: cell-hit 선택 로직을 first-match → depth+area priority 로 변경하여 RED 테스트 PASS.

**작업**:
1. [src/document_core/queries/cursor_rect.rs:643-666](../../src/document_core/queries/cursor_rect.rs#L643-L666) 를 §2.2 코드로 교체
2. 커밋: `Task #857 Stage 2 (GREEN): cell-hit 선택 priority (depth, neg_area) 로 변경 — closes #857`

**완료 조건**:
- `cargo test --test issue_table_vpos_01_page5_cell_hit_test` → **13 PASS** (5 FAIL 모두 해결)
- 변경 코드 빌드 PASS (clippy warning 없음)
- 사람이 읽는 주석에 closes #857 명시

**보고서**: `mydocs/working/task_m100_857_stage2.md`

### Stage 3 — 회귀 sweep + 시각 회귀 + 최종 보고서

**목표**: 기존 회귀 테스트 모두 PASS 유지 확인 + 시각 회귀 없음 + 최종 보고서.

**작업**:
1. **단위·통합 테스트**:
   ```
   cargo test --release 2>&1 | tee /tmp/task857_full_test.log
   ```
   - 핵심 관련 테스트가 PASS 인지 개별 확인:
     - `tests/issue_717_table_cell_hit_test.rs` (3 cases)
     - `tests/issue_595_*` 또는 관련
     - `tests/issue_628.rs`, `tests/issue_630.rs`
     - `tests/issue_nested_table_border.rs`
     - `tests/issue_table_vpos_01_page5_cell_hit_test.rs` (13 cases, 모두 PASS)
2. **clippy**: `cargo clippy --tests --release -- -D warnings` PASS
3. **시각 회귀 (SVG diff)**:
   ```
   cargo run -- export-svg samples/table-vpos-01.hwp -p 4 --debug-overlay -o /tmp/svg-after/
   diff /tmp/tvp01-p5/ /tmp/svg-after/ || true
   ```
   - 렌더링 자체에는 변경이 없어야 하므로 SVG 산출물 동일 예상.
4. **수동 E2E (rhwp-studio)**:
   - WASM 빌드: `docker compose --env-file .env.docker run --rm wasm`
   - studio 실행 후 `samples/table-vpos-01.hwp` 5쪽으로 이동
   - c=2 column 본문 셀("포용과 균형의 기본사회 구현", ④⑤⑥⑦⑧⑨ 등) 클릭 → 셀 안 커서 진입 확인
   - 글자 입력 → 해당 inner 셀에 텍스트 정상 추가 확인
   - c=0 column 라벨 셀("1 참여소통" 등) 도 회귀 없음 재확인
   - 다른 페이지(page 4 의 pi=28/29 inline 표 등) 회귀 없음 확인
5. **최종 보고서** 작성: `mydocs/report/task_m100_857_report.md`
   - 진단 요약, fix 내용, 회귀 검증 결과, 잔존 위험 등 기록
6. **`mydocs/orders/20260512.md`** 에 task #857 완료 표시 갱신
7. 커밋: `Task #857 Stage 3 (회귀 sweep + 최종 보고서)`

**완료 조건**:
- 전체 cargo test PASS
- clippy clean
- 시각 회귀 없음
- 수동 E2E 정상
- 보고서 + orders 갱신

**보고서**: `mydocs/working/task_m100_857_stage3.md` + 최종 `mydocs/report/task_m100_857_report.md`

## 4. 종료 후 처리

1. 작업지시자 승인 후 `local/task857` → `local/devel` merge (`--no-ff`)
2. `local/devel` → `devel` merge + push (origin 원격 갱신)
3. Issue #857 close (커밋 메시지의 `closes #857` 으로 자동 close 예상)

## 5. 잔존 위험·미해결 사항

- **HWPX 변환**: 본 fix 는 HWP5 hit-test 만 다룸. 동일 문서 `samples/table-vpos-01.hwpx` 동작 미확인. HWPX 도 영향 가능성 있으나 본 Task 범위 외 (필요 시 별도 task).
- **#850**: 별개 이슈이므로 본 fix 가 #850 에 영향 줄지는 미지수. Stage 3 회귀 sweep 에서 #850 재현 case 도 함께 돌려보면 좋겠지만 우선순위 낮음.
- **다른 first-match 분기**: 같은 함수의 다른 분기 (L592-641 인라인 Shape, L671-762 셀 bbox 매칭) 의 selection 정책은 본 fix 대상 아님. 별도 회귀가 있다면 별 issue.

## 6. 단계별 진행 승인 프로토콜

- 각 Stage 완료 시 보고서 작성 → 작업지시자 승인 요청
- 승인 없이 다음 Stage 진행 금지
- 회귀 발견 시 즉시 보고 + 재계획

본 구현 계획서 승인 후 Stage 1 진행 가능.
