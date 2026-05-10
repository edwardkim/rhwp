---
PR: #743
제목: Task #721 — 중첩 표 내 필드 값 설정 (nested_path 전체 탐색으로 field_range 인덱스 초과 정정)
컨트리뷰터: @oksure (Hyunwoo Park) — 20+ 사이클 핵심 컨트리뷰터 (5/10 사이클 12번째 PR)
base / head: devel / contrib/field-nested-table-fix
mergeStateStatus: BEHIND
mergeable: MERGEABLE — `git merge-tree` 충돌 0건
CI: ALL SUCCESS
변경 규모: +106 / -52, 1 file
검토일: 2026-05-10
---

# PR #743 검토

## 1. 메타

| 항목 | 값 |
|------|-----|
| PR 번호 | #743 |
| 제목 | Task #721 — 중첩 표 내 필드 값 설정 |
| 컨트리뷰터 | @oksure — 20+ 사이클 (5/10 사이클 12번째 PR) |
| base / head | devel / contrib/field-nested-table-fix |
| mergeStateStatus | BEHIND, mergeable: MERGEABLE — 충돌 0건 |
| CI | ALL SUCCESS |
| 변경 규모 | +106 / -52, 1 file |
| 커밋 수 | 2 (Task + Copilot 리뷰) |
| closes | #721 |

## 2. 컨트리뷰터 제기 문제 — 본 환경 검증 ✅ 타당

### 2.1 제기 본질
`setFieldValueByName` / `setFieldValue` 호출 시 **중첩 표** (표 안의 표) 내부 ClickHere 필드에서 `field_range 인덱스 초과` 에러 발생. `getFieldList()` 는 정상 인식하지만 set 호출 시 모든 필드가 실패한다.

### 2.2 본 환경 소스 검증

**`src/document_core/queries/field_query.rs`** 점검 결과:

#### A. `collect_fields_from_paragraph` — 재귀 정합 ✅

```rust
fn collect_fields_from_paragraph(para, base_location, result) {
    // ... 본 문단 field_ranges 수집 ...
    for (ci, ctrl) in para.controls.iter().enumerate() {
        match ctrl {
            Control::Table(table) => {
                for (cell_i, cell) in table.cells.iter().enumerate() {
                    for (pi, cell_para) in cell.paragraphs.iter().enumerate() {
                        let mut loc = base_location.clone();
                        loc.nested_path.push(NestedEntry::TableCell { ... });
                        collect_fields_from_paragraph(cell_para, &loc, result);  // 재귀
                    }
                }
            }
            Control::Shape(shape) => { ... }  // 동일 재귀
        }
    }
}
```

→ 임의 깊이 중첩에서 `nested_path` 가 누적된다.

#### B. `get_para_mut_at_location` (line 242-285) — **결함 확인** ❌

```rust
// 1단계 중첩만 처리                          ← 명시적 한정
let entry = &location.nested_path[0];        ← nested_path[0] 만
match entry {
    NestedEntry::TableCell { ... } => { ... }
    NestedEntry::TextBox { ... } => { ... }
}
```

→ `nested_path` 길이 ≥ 2 (중첩 표의 표) 인 경우 외부 표의 셀 문단을 반환 → `field_range` 부재 → 인덱스 초과.

#### C. `set_cell_field_text` (line ~600) — **동일 결함 확인** ❌

`nested_path[0]` 만 처리 — 동일 패턴.

#### D. `set_field_text_at` (line 167-170) — `get_para_mut_at_location` 호출 → 자동으로 정합 정정 대상

### 2.3 검증 결론

**컨트리뷰터 제기 타당성 ✅ 확정**:
- `collect_fields_from_paragraph` (재귀) 와 `get_para_mut_at_location` / `set_cell_field_text` (1단계 만) 사이의 **명백한 비대칭 결함**
- 한국 정부 공식 양식에서 빈번히 사용되는 중첩 표 (표 안 표) 의 ClickHere 필드 모두 실패
- `getFieldList()` 는 정상 인식하지만 set 호출 시 nested_path[0] 외 정보 손실

## 3. PR 의 정정 — `src/document_core/queries/field_query.rs` (+106/-52)

### 3.1 `get_para_mut_at_location` — for 루프 전체 순회

```rust
let mut para = sec.paragraphs.get_mut(location.para_index)?;

for (i, entry) in location.nested_path.iter().enumerate() {
    para = match entry {
        NestedEntry::TableCell { control_index, cell_index, para_index } => {
            let ctrl = para.controls.get_mut(*control_index)
                .ok_or_else(|| HwpError::InvalidField(
                    format!("경로[{}]: 컨트롤 인덱스 {} 초과", i, control_index)))?;
            if let Control::Table(ref mut table) = ctrl {
                let cell = table.cells.get_mut(*cell_index)?;
                cell.paragraphs.get_mut(*para_index)?
            } else {
                return Err(HwpError::InvalidField(
                    format!("경로[{}]: controls[{}]가 Table이 아님", i, control_index)));
            }
        }
        NestedEntry::TextBox { ... } => { ... 동일 패턴 ... }
    };
}

Ok(para)
```

→ `collect_fields_from_paragraph` 재귀와 대칭 정합. 임의 깊이 중첩 지원.

### 3.2 `set_cell_field_text` — 동일 패턴

마지막 항목 직전까지 중첩 탐색하고, 마지막 항목에서 셀의 첫 문단 텍스트를 교체.

### 3.3 에러 메시지에 경로 인덱스 명시
`format!("경로[{}]: 컨트롤 인덱스 {} 초과", i, control_index)` — 디버깅 편의성 향상.

### 3.4 Copilot 리뷰 반영 (commit `c2f35ec1`)
- `set_cell_field_text` 마지막 항목의 에러 메시지에 경로 인덱스 포함
- 셀 텍스트 교체 후 `char_offsets` 를 naive (0..n) 대신 `rebuild_char_offsets()` 로 재생성 (UTF-16 서로게이트 페어 반영)

## 4. 영향 범위

### 4.1 변경 영역
- `setFieldValueByName` / `setFieldValue` 가 중첩 표 (≥ 2단계) 내 ClickHere 필드에서 정상 동작
- `set_cell_field_text` 동일하게 정합

### 4.2 무변경 영역
- 1단계 중첩 (이미 정합)
- 본문 직접 필드 (`nested_path.is_empty()`)
- `collect_fields_from_paragraph` (이미 재귀로 정합)
- 다른 layout/render 경로

### 4.3 위험 영역
- 한국 정부 공식 양식에서 빈번히 사용되는 패턴 — **회귀 정정의 가치가 큼**
- 단일 파일 변경 — 격리된 수정

## 5. 본 환경 점검

- merge-base: `30351cdf` (5/9 시점에 가까움)
- merge-tree 충돌: **0건** ✓
- 변경 격리: `field_query.rs` 단일 파일 — 다른 layout/render/parser 경로와 무관

## 6. 충돌 / mergeable

- `git merge-tree --write-tree`: **CONFLICT 0건** ✓
- BEHIND — devel 5/10 사이클 진전, 본 PR 단일 파일 변경으로 충돌 부재

## 7. 처리 옵션

### 옵션 A — 2 commits cherry-pick + no-ff merge (추천)

```bash
git checkout -b local/task721 ee9b3a63
git cherry-pick fa0b5e36 c2f35ec1
git checkout local/devel
git merge --no-ff local/task721
```

→ **옵션 A 추천**.

## 8. 검증 게이트 (구현 단계 DoD)

### 자기 검증
- [ ] `cargo build --release` 통과
- [ ] `cargo test --release` — 1067 lib + 9 integration ALL GREEN (PR 본문 명시)
- [ ] `cargo clippy --release --all-targets -- -D warnings` clean
- [ ] 광범위 sweep — 7 fixture / 170 페이지 회귀 0 (field 만 변경되어 SVG 영향 없음 보장)

### 시각 판정 게이트 — **WASM 빌드 + 작업지시자 인터랙션 검증 권장**

본 PR 의 본질은 **WASM API 인터랙션** (setFieldValue):
- WASM 빌드 후 dev server 에서 한국 정부 양식 (중첩 표 ClickHere 필드) 에 대해 setFieldValue 정합 점검
- E2E 자동 테스트 신규 부재 → 작업지시자 직접 인터랙션 검증 권장

## 9. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @oksure 20+ 사이클 (5/10 사이클 12번째 PR) |
| `feedback_image_renderer_paths_separate` | field_query.rs 격리 — 다른 layout/render 경로 무영향 |
| `feedback_process_must_follow` | 컨트리뷰터 제기 본질이 정확하고, 비대칭 결함 (collect 재귀 ↔ get/set 1단계 만) 을 정확히 정정 |
| `feedback_visual_judgment_authority` | WASM API 인터랙션은 작업지시자 인터랙션 검증 권장 |

## 10. 처리 순서 (승인 후)

1. `local/devel` 에서 2 commits cherry-pick (옵션 A)
2. 자기 검증 (cargo test/build/clippy + 광범위 sweep)
3. WASM 빌드 + 작업지시자 인터랙션 검증 (한국 정부 양식의 setFieldValue)
4. 인터랙션 검증 통과 → no-ff merge + push + archives 이동 + 5/10 orders 갱신
5. PR #743 close (closes #721 자동 정합)

---

작성: 2026-05-10
