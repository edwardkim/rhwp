---
PR: #743
제목: Task #721 — 중첩 표 내 필드 값 설정 (nested_path 전체 탐색)
컨트리뷰터: @oksure (Hyunwoo Park) — 20+ 사이클 핵심 컨트리뷰터 (5/10 사이클 12번째 PR)
처리: 옵션 A — 2 commits cherry-pick + no-ff merge
처리일: 2026-05-10
머지 commit: c4d506d4
---

# PR #743 처리 보고서

## 1. 처리 결과

✅ **머지 완료** — 옵션 A (2 commits cherry-pick + no-ff merge `c4d506d4`)

| 항목 | 값 |
|------|-----|
| 머지 commit | `c4d506d4` (--no-ff merge) |
| Cherry-pick commits | `35c22c2a` + `c187b031` (Copilot 리뷰) |
| closes | #721 |
| 시각 판정 | ✅ 작업지시자 웹 인터랙션 검증 통과 (k-water-rfp.hwp 24페이지 분리된 표 > 표) |
| 자기 검증 | cargo test ALL GREEN + sweep 170/170 same + WASM 4.65 MB |

## 2. 컨트리뷰터 제기 문제 타당성 검증 ✅

본 환경 소스 점검 결과 **컨트리뷰터 제기 정확**:

| 함수 | 동작 | 평가 |
|------|------|------|
| `collect_fields_from_paragraph` | 재귀 호출 (nested_path 누적) | ✅ 정합 |
| `get_para_mut_at_location` | `nested_path[0]` 만 (`// 1단계 중첩만 처리` 명시) | ❌ **결함** |
| `set_cell_field_text` | `nested_path[0]` 만 | ❌ **결함** |
| `set_field_text_at` | `get_para_mut_at_location` 호출 | ❌ (간접 결함) |

→ `collect` 재귀 ↔ `get/set` 1단계 만의 **명백한 비대칭 결함**.

## 3. 정정 본질 — `src/document_core/queries/field_query.rs` (+106/-52)

### 3.1 `get_para_mut_at_location` — for 루프 전체 순회

```rust
let mut para = sec.paragraphs.get_mut(location.para_index)?;

for (i, entry) in location.nested_path.iter().enumerate() {
    para = match entry {
        NestedEntry::TableCell { control_index, cell_index, para_index } => {
            // ... 중첩 셀 탐색 ...
        }
        NestedEntry::TextBox { control_index, para_index } => {
            // ... 글상자 탐색 ...
        }
    };
}

Ok(para)
```

→ `collect_fields_from_paragraph` 재귀와 대칭 정합. 임의 깊이 중첩 지원.

### 3.2 `set_cell_field_text` — 동일 패턴

마지막 항목 직전까지 중첩 탐색하고, 마지막 항목에서 셀의 첫 문단 텍스트를 교체.

### 3.3 에러 메시지에 경로 인덱스 명시
`format!("경로[{}]: 컨트롤 인덱스 {} 초과", i, control_index)` — 디버깅 편의성 향상.

### 3.4 Copilot 리뷰 반영 (commit `c187b031`)
- `set_cell_field_text` 마지막 항목의 에러 메시지에 경로 인덱스 포함
- 셀 텍스트 교체 후 `char_offsets` 를 naive (0..n) 대신 `rebuild_char_offsets()` 로 재생성 (UTF-16 서로게이트 페어 반영)

## 4. 본 환경 cherry-pick + 검증

| 검증 | 결과 |
|------|------|
| `cherry-pick` (2 commits) | ✅ 충돌 0건 (auto-merge 정합) |
| `cargo build --release` | ✅ 통과 (31.83s) |
| `cargo test --release` | ✅ ALL GREEN |
| 광범위 sweep (7 fixture / 170 페이지) | ✅ **170 same / 0 diff** (field_query 만 변경되어 SVG 무영향 입증) |
| WASM 빌드 (강제 재빌드) | ✅ 4.65 MB |

## 5. 작업지시자 웹 인터랙션 검증 ✅ 통과

`samples/k-water-rfp.hwp` 24페이지의 분리된 표 > 표 영역에서 hitTest 인덱스 초과 오류 해결 확인. 한국 정부 양식의 중첩 표 ClickHere 필드 정합 동작.

## 6. 영향 범위

### 6.1 변경 영역
- `setFieldValueByName` / `setFieldValue` 가 중첩 표 (≥ 2단계) 내 ClickHere 필드에서 정상 동작
- `set_cell_field_text` 동일하게 정합

### 6.2 무변경 영역 (sweep 170/170 same 입증)
- 1단계 중첩 (이미 정합 — 회귀 부재)
- 본문 직접 필드 (`nested_path.is_empty()`)
- `collect_fields_from_paragraph` (이미 재귀로 정합)
- 다른 layout/render 경로
- HWP3/HWPX 변환본 시각 정합

## 7. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @oksure **20+ 사이클** (5/10 사이클 영역 PR #728/#729/#730/#734/#735/#737/#738/#739/#740/#742/#743 영역 12번째 PR) |
| `feedback_image_renderer_paths_separate` | field_query.rs 격리 — 다른 layout/render 경로 무영향 (sweep 170/170 same 입증) |
| `feedback_process_must_follow` | 컨트리뷰터 제기 본질이 정확하고, 비대칭 결함을 정확히 정정 (collect 재귀 ↔ get/set 1단계 만 의 정합) |
| `feedback_visual_judgment_authority` | 작업지시자 웹 인터랙션 검증 ✅ 통과 (k-water-rfp.hwp 분리된 표 > 표 hitTest 정합) |

## 8. 잔존 후속

- 본 PR 본질 정정의 잔존 결함 부재
- 한국 정부 공식 양식의 중첩 표 ClickHere 필드 사용 영역의 큰 가치

---

작성: 2026-05-10
