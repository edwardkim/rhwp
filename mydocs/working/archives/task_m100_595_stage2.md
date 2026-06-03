# Task #595 Stage 2 완료 보고서

**Issue**: #595 — exam_math.hwp 2페이지부터 수식 더블클릭 hitTest 오동작
**브랜치**: `local/task595`
**Stage**: 2 — 본질 정정 + 회귀 검증
**날짜**: 2026-05-07

---

## 1. 정정 영역

**파일**: `src/document_core/queries/cursor_rect.rs`
**함수**: `hit_test_header_footer_native`

**옵션 A 적용** — Header/Footer 노드의 bbox (자식 노드 까지 확장) 가 아닌, layout 의 정확한 `header_area` / `footer_area` 로 hit 판정.

### 정정 전 (Stage 1 진단 결과)

```rust
let tree = self.build_page_tree(page_num)?;
for child in &tree.root.children {
    let is_header = matches!(child.node_type, RenderNodeType::Header);
    let is_footer = matches!(child.node_type, RenderNodeType::Footer);
    if !is_header && !is_footer { continue; }
    if x >= child.bbox.x && x <= child.bbox.x + child.bbox.width
        && y >= child.bbox.y && y <= child.bbox.y + child.bbox.height
    {
        // ... hit ...
    }
}
```

`child.bbox` 가 `expand_bbox_to_children` 으로 자식 (단 구분선 line) 까지 확장되어 본문 영역 침범.

### 정정 후

```rust
let (page_content, _, _) = self.find_page(page_num)?;
let layout = &page_content.layout;

// 머리말 영역 hit 판정 (layout.header_area — 정확한 머리말 범위)
let h = &layout.header_area;
if x >= h.x && x <= h.x + h.width && y >= h.y && y <= h.y + h.height {
    // ... hit ...
}

// 꼬리말 영역 hit 판정 (layout.footer_area)
let f = &layout.footer_area;
if x >= f.x && x <= f.x + f.width && y >= f.y && y <= f.y + f.height {
    // ... hit ...
}
```

`layout.header_area` / `layout.footer_area` 는 PageDef 의 margin 정보로 계산된 정확한 영역. bbox 확장과 무관.

**부수 효과**: `build_page_tree` 호출 제거 → 효율 ↑ (mousedown / dblclick 마다 호출되던 비싼 트리 빌드 제거).

## 2. 검증 결과

### 2.1 재현 단위 테스트 (`tests/issue_595.rs`)

```
running 5 tests
test issue_595_page1_header_area_still_hits ... ok                    ← 정상 가드
test issue_595_page1_body_center_not_header ... ok                    ← Stage 1 fail → 정정 후 pass
test issue_595_page0_body_coord_not_header ... ok                     ← 정상 baseline
test issue_595_page1_body_coord_not_header_regression_guard ... ok   ← Stage 1 fail → 정정 후 pass
test issue_595_page1_equation_coord_not_header ... ok                 ← Stage 1 fail → 정정 후 pass

test result: ok. 5 passed; 0 failed
```

### 2.2 페이지별 hit y 범위 정상화 (`inspect_595.rs`)

| 페이지 | 정정 전 | 정정 후 |
|--------|---------|---------|
| page 0 (1p) | 60.0 ~ 145.0 | **60.0 ~ 145.0** (정상 보존) |
| page 1 (2p) | **60.0 ~ 1355.0** | **60.0 ~ 145.0** (정상화) |
| page 2 (3p) | 60.0 ~ 1355.0 | 60.0 ~ 145.0 (정상화) |
| page 3 (4p) | 60.0 ~ 1355.0 | 60.0 ~ 145.0 (정상화) |

### 2.3 광범위 sweep (`samples/` 전체 164 fixture / 1684 페이지)

| 항목 | 정정 전 | 정정 후 |
|------|---------|---------|
| 머리말 hit 본문 침범 fixture | 2 / 164 (1.2%) | **0 / 164 (0.0%)** |
| 머리말 hit 본문 침범 페이지 | 32 / 1684 (1.9%) | **0 / 1684 (0.0%)** |

### 2.4 회귀 sweep

| 검증 | 결과 |
|------|------|
| `cargo test --lib --release` | **1140 passed** (회귀 0) |
| `cargo clippy --release` (lib) | warning/error 0 (회귀 0) |
| `cargo build --release` | clean |
| `cargo test --release --test issue_516/530/546/554/595` | **30 passed** (회귀 0) |

### 2.5 광범위 머리말/꼬리말/본문 영역 hit sweep (`inspect_595_regression`)

164 fixture / 1684 페이지에서 정정 전 vs 정정 후 비교 (작업지시자 요청 — "비슷한 상황에서 기존 잘 되는 동작이 안 되는 것까지 신경 써달라"):

| 항목 | 정정 전 | 정정 후 | 변화 |
|------|---------|---------|------|
| **머리말 영역 중앙 hit:true** (margin_header > 0) | 1329 pass / 27 fail | **1356 pass / 0 fail** | **+27 부수 개선** |
| **꼬리말 영역 중앙 hit:true** (margin_footer > 0) | 1383 pass / 16 fail | 1383 pass / 16 fail | **동일 (회귀 0)** |
| **본문 중앙 hit:false** (Issue #595 결함 영역) | 1652 pass / 32 fail | **1684 pass / 0 fail** | **+32 본질 정정** |

**해석**:

- **회귀 0** — 정정 후 fail 케이스 (꼬리말 16개) 는 정정 전에도 동일 fail. 본 정정과 무관.
- **본질 정정** — 본문 영역 false-positive 32 페이지 (exam_math.hwp / exam_math_no.hwp) 완전 제거 → Issue #595 완전 해결.
- **부수 개선** — 머리말 영역 hit 정확화 27 페이지. 정정 전에는 expand_bbox_to_children 결과가 자식 노드에 종속되어 머리말 영역 자체도 일부 hit 안 됐는데, 정정 후 layout.header_area 직접 사용으로 정확한 영역만 hit.
- **별도 영역** — 꼬리말 16 페이지 fail (`hwpctl_Action_Table__v1.1.hwp` 한 fixture). 정정 전후 동일 → 본 task 무관, 별도 task 영역 후보.

**검증 도구 `inspect_595_regression.rs`** — 영구 보존. 향후 hit_test_header_footer 영역 회귀 차단 가드로 재사용 가능.

## 3. 정정 영향 영역

| 영역 | 영향 |
|------|------|
| `hit_test_header_footer_native` 자체 | 본질 정정 — bbox 사용 → layout.area 사용 |
| `build_page_tree` 호출 | **제거** — 효율 ↑ |
| `expand_bbox_to_children` (`build_header`) | **무수정** — 렌더링 동작 보존 |
| 머리말 표 셀 내 Shape 클리핑 | **무영향** — `expand_bbox_to_children` 의 의도 (클리핑 방지) 보존 |
| 다른 hit_test 함수 (`hit_test_in_header_footer_native` 등) | **무영향** — 본 정정 함수 한정 |

## 4. dblclick 흐름 정합 확인

정정 후 `onDblClick` 흐름 ([rhwp-studio/src/engine/input-handler-mouse.ts:769-825](../../rhwp-studio/src/engine/input-handler-mouse.ts#L769)):

1. `wasm.hitTestHeaderFooter(pageIdx, pageX, pageY)` — page 1+ 본문 좌표에서 `hit:false` 정상 반환
2. 머리말 분기 미진입 → `if (this.cursor.isInPictureObjectSelection())` 분기 도달
3. `ref.type === 'equation'` 확인 → `equation-edit-request` 이벤트 emit
4. **수식 편집기 정상 진입** ✓

## 5. 별도 발견 — 본 task 범위 밖

Stage 1 e2e 진단에서 추가 발견 — **그리드 모드 (zoom ≤ 0.5) 좌표 결함**:

`rhwp-studio/src/engine/input-handler-mouse.ts` 의 mouse hit 좌표 계산이 단일 컬럼 가정 (`pageLeft = (sc.clientWidth - pageDisplayWidth) / 2`). 그리드 모드에서 페이지가 다중 열 배치될 때 hit 좌표 계산 어긋남. 본 이슈 (#595) 와 별개 영역 — **별도 issue/task 분리 등록 권장** (Stage 3 정리 단계에서 처리).

## 6. 다음 단계 (Stage 3)

1. ~~e2e 정정 후 재검증~~ — WASM 재빌드 (Docker) 가 별도 단계라 본 stage 종료 후 작업지시자 직접 검증 영역 (또는 별도 단계)
2. **이슈 #595 본문 정오표 코멘트 등록** — Stage 1 진단 + Stage 2 정정 + 검증 데이터 첨부
3. **Stage 3 최종 보고서 + 오늘할일 갱신**
4. `local/devel` merge → `devel` push 영역은 작업지시자 결정

## 7. Stage 2 산출물

- `src/document_core/queries/cursor_rect.rs::hit_test_header_footer_native` 정정 (단일 함수, +20/-13 LOC)
- `mydocs/working/task_m100_595_stage2.md` (본 보고서)

## 8. 정합 영역

- **HWP IR 표준 직접 사용** — `layout.header_area` / `layout.footer_area` 는 PageDef margin 으로 계산된 정확한 머리말/꼬리말 영역. 휴리스틱 미도입 (`feedback_rule_not_heuristic` 정합).
- **회귀 위험 영역 좁힘** — bbox 사용 → layout area 사용 으로 단일 함수만 정정. 렌더링 동작 무영향. `expand_bbox_to_children` 의 의도 (셀 내 Shape 클리핑 방지) 보존.
- **본질 정정** — 우회/패치 아닌 정확한 영역 사용 (`feedback_root_cause_only` 정합).

---

**Stage 2 완료 — Stage 3 진입 (이슈 정오표 등록 + 최종 보고서) 승인 요청**
