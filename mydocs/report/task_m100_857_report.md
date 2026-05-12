# Task m100 #857 최종 결과 보고서

> Issue: [#857 — table-vpos-01.hwp p.5 중첩 11×3 표 c=2 column 셀 클릭 misroute](https://github.com/edwardkim/rhwp/issues/857)
> 브랜치: `local/task857`
> 완료일: 2026-05-12

## 1. 본질

`samples/table-vpos-01.hwp` 5쪽의 중첩 11×3 표 안 **c=2 column 본문 셀들** 클릭 시 커서가 inner 셀로 진입하지 않고 외곽 1×1 wrapper 셀의 paragraph 1 영역으로 misroute 되던 결함. 글자 입력 시 inner 셀이 아닌 외곽 paragraph 의 text 영역(시각적으로 "4 공공 AX" 행 부근) 에 silent 삽입.

같은 inner 11×3 의 c=0 라벨 셀("1 참여소통" 등) 은 정상 동작 — c=2 column 만 영향.

## 2. Root cause

[src/document_core/queries/cursor_rect.rs:648-666](../../src/document_core/queries/cursor_rect.rs#L648-L666) 의 1차 bbox 매칭이 cell-context TextRun 후보 중 **첫 매칭** 만 선택 (`if hit_cell.is_none()`).

`collect_runs` 의 depth-first 트리 순회 순서상 **외곽 셀의 빈 placeholder TextRun** (char_count=0, bbox 712.5 px 의 outer paragraph 1 영역 전체) 이 inner 11×3 의 실제 텍스트 TextRun (cellPath 길이 2, 작은 bbox) 보다 **먼저 매칭** → 외곽 placeholder 선점, inner run 무시.

c=0 column row 0/3/6 라벨 셀이 정상이었던 이유: click x 좌표(≈128) 가 외곽 placeholder x_range `[396.9, 1022.1]` **밖** 이라 placeholder 매칭 자체가 안 됨. c=2 column 의 click x(≈442.5) 만 placeholder 안에 들어가서 본 버그 발현.

## 3. Pre-Task #717 검증 — v0.5.0 부터 잠재

`git log -L 648,666:src/document_core/queries/cursor_rect.rs` 결과 본 first-match 로직은 **v0.5.0 초기 커밋 `f0f7f1a4` 이후 변경 없음**. commit `1c783a89` (Task #717 parent) 에서 동일 RED 테스트 실행 → c=2 column 4개 케이스 모두 동일 FAIL → **본 버그는 v0.5.0 이후 잠재**. Task #717 의 부수 효과가 아닌 독립 결함.

부가 발견: pre-#717 에서는 c=0 column row 0/3/6 셀들도 잘못된 pi=30 으로 misroute 됐는데, Task #717 의 cell_bboxes 보완 패스 변경으로 fix 됨. c=2 column 의 first-match 결함은 #717 도 손대지 못함.

## 4. Issue #850 과의 관계 — 별개

[Issue #850](https://github.com/edwardkim/rhwp/issues/850) 도 cellPath 길이 1 증상을 공유하지만 본 이슈와 **메커니즘·발생 시점이 다른 별개 이슈**:

| | #850 | 본 이슈 #857 |
|---|---|---|
| 발생 시기 | v0.7.11 회귀 (Task #717 직접 원인) | v0.5.0 부터 잠재 |
| 회귀 라인 | L391-403 (Task #717 변경) | L648-666 (v0.5.0 이후 불변) |
| 메커니즘 | inner cell_context layout 단계 결함 | hit-test first-match 정책 |
| 증상 | 클릭 됨 + 입력 시 API 에러 `"컨트롤 인덱스 0 범위 초과"` | 클릭이 외곽 셀로 misroute (silent, 콘솔 에러 없음) |

본 fix 가 #850 도 함께 해결하는지는 미지수 — 별도 진단·fix 필요.

## 5. Fix 내용

**위치**: [src/document_core/queries/cursor_rect.rs:648-666](../../src/document_core/queries/cursor_rect.rs#L648-L666)

**변경**: cell-context TextRun 매칭 시 `first-match` → `min area best-match` 로 selection 정책 변경. Task #717 의 cell_bboxes selection (L671-675) 과 동일 패턴.

```rust
// Before
if run.cell_context.is_some() {
    if hit_cell.is_none() {
        hit_cell = Some((i, run.char_start + char_offset));
    }
}

// After
if run.cell_context.is_some() {
    let area = (run.bbox_w.max(0.0) * run.bbox_h.max(0.0) * 1000.0) as i64;
    if hit_cell_area.map_or(true, |best_area| area < best_area) {
        hit_cell = Some((i, run.char_start + char_offset));
        hit_cell_area = Some(area);
    }
}
```

### 코드 일관성

cursor_rect.rs 내 selection 패턴 통일:
- L587-588 (안내문 → 가장 가까운 본문): `min_by_key` best-match
- L671-675 (cell_bboxes 셀 선택, Task #717): `min_by_key(area)` best-match
- L680 (cell 안 거리): `min_by_key` best-match
- **L648-666 (본 fix 전): `is_none()` first-match → 유일한 first-match 였음**
- L648-666 (본 fix 후): **min area best-match — 정책 통일**

## 6. 검증 결과

### 6.1 자동 회귀 (cargo test debug)
- **전체 1232 unit tests + 35 integration test suites 모두 PASS**
- 핵심 개별 확인: Task #717 (3 PASS), `issue_630` (1), `issue_nested_table_border` (1)
- 본 RED 테스트 [tests/issue_table_vpos_01_page5_cell_hit_test.rs](../../tests/issue_table_vpos_01_page5_cell_hit_test.rs): Stage 1 5 FAIL / 8 PASS → Stage 2 후 **13 PASS**

### 6.2 clippy
- 본 변경 관련 새 warning 없음

### 6.3 시각 회귀
- page 5 SVG `diff` byte-identical — 렌더링 영향 없음 입증

### 6.4 수동 E2E (rhwp-studio)
- WASM 재빌드 후 작업지시자 직접 시연 확인 → c=2 본문 셀 클릭 정상 진입, 글자 입력 정상

## 7. 산출물

| 파일 | 목적 |
|---|---|
| [src/document_core/queries/cursor_rect.rs](../../src/document_core/queries/cursor_rect.rs) (L643-666 수정) | Fix |
| [tests/issue_table_vpos_01_page5_cell_hit_test.rs](../../tests/issue_table_vpos_01_page5_cell_hit_test.rs) | RED → GREEN 회귀 테스트 (13 cases) |
| [mydocs/troubleshootings/table_vpos_01_page5_cell_hit_test.md](../troubleshootings/table_vpos_01_page5_cell_hit_test.md) | 진단 노트 |
| [mydocs/plans/task_m100_857.md](../plans/task_m100_857.md) | 수행 계획서 |
| [mydocs/plans/task_m100_857_impl.md](../plans/task_m100_857_impl.md) | 구현 계획서 |
| [mydocs/working/task_m100_857_stage1.md](../working/task_m100_857_stage1.md) | Stage 1 보고서 (RED) |
| [mydocs/working/task_m100_857_stage2.md](../working/task_m100_857_stage2.md) | Stage 2 보고서 (GREEN) |
| [mydocs/working/task_m100_857_stage3.md](../working/task_m100_857_stage3.md) | Stage 3 보고서 (회귀 sweep) |

## 8. 커밋 (local/task857)

```
b10a83f0 Task #857 Stage 2 보고서
1135c028 Task #857 Stage 2 (GREEN): cell-hit selection first-match → min area best-match (closes #857)
37e7b7b0 Task #857 Stage 1 보고서
07168934 Task #857 Stage 1 (RED): 진단 노트 + 회귀 테스트 + 계획서 추가
```

Stage 3 보고서·최종 보고서·orders 갱신 commit 추가 예정.

## 9. 잔존 위험·미해결 사항

- **HWPX 변환** (`samples/table-vpos-01.hwpx`): 미확인. 같은 fix 가 hit-test 단을 변경하므로 HWPX 도 같이 해결될 가능성 높으나 미검증 — 필요 시 별도 task.
- **#850**: 본 fix 와 별개. 별도 진단·fix 필요.
- 같은 셀 안 여러 줄 TextRun 이 동시 매칭되는 case 에서 짧은 줄 wins (기존: 첫 줄). click 좌표가 두 line bbox 모두에 들어가는 케이스 자체가 드물고 전체 cargo test PASS 로 회귀 미발견.

## 10. 종료 후 처리 (작업지시자 승인 후)

1. **본 보고서 + Stage 3 보고서 + orders 갱신 commit**
2. `local/task857` → `local/devel` merge (`--no-ff`)
3. `local/devel` → `devel` merge + push (`origin/devel`)
4. Issue #857 자동 close (Stage 2 commit 의 `closes #857`)
