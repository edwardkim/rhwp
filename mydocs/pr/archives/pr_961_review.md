---
PR: #961
제목: fix — horz_rel_to=Column picture column 외부 emit 시 cursor advance skip (시험지 page 1 문9 정합, closes #959)
컨트리뷰터: @jangster77 (Taesup Jang) — 24+ 사이클 핵심 컨트리뷰터 (연속 5 PR 3번째)
base / head: devel / local/task959
mergeStateStatus: DIRTY
mergeable: CONFLICTING
CI: ✅ Build & Test + CodeQL (js-ts/python/rust) + Canvas visual diff
변경 규모: +679 / -0, 21 files (코드 1 / 문서 8 / fixture 8 / PDF 4)
커밋: 1
검토일: 2026-05-18
---

# PR #961 검토

## 1. 메타

| 항목 | 값 |
|------|-----|
| PR 번호 | #961 |
| 제목 | fix: horz_rel_to=Column picture column 외부 emit 시 cursor advance skip (시험지 page 1 문9 정합) |
| 컨트리뷰터 | @jangster77 — **24+ 사이클** (연속 5 PR **3번째**, #956/#958 직후) |
| base / head | devel / local/task959 |
| mergeable | CONFLICTING (DIRTY) |
| CI | ✅ 전 항목 통과 |
| 변경 규모 | +679 / -0, 21 files |
| 커밋 수 | 1 (`47699977`) |
| closes | #959 (Issue #952 영역 영역 **Issue 3** 분리 task) |
| 연속 5 PR | #956 ✅ → #958 ✅ → **#961 (3번째)** → #963 → #964 |

## 2. 본질 (Issue #959 = Issue #952 Issue 3)

`samples/3-11월_실전_통합_2022.hwp` page 1 우측 단 문9 가 한컴 viewer 보다
~250px 아래 처짐.

### Root cause (RHWP_DEBUG_TAC_CURSOR 추적)
```
Shape pi=69 ci=0 y_in=709.4 y_out=983.4 dy=274.0 ⚠️
FullPara pi=73 y_in=1043.5 ... (문9)
```

pi=69 picture:
- horz_rel_to=Column, h_offset=79.5mm, 정렬=Center
- compute_object_position: pic_emit_x = col.x + (col.w - pic.w)/2 + h_offset = 399 + 68 + 300 = 767
- col_area right = 759.7 → picture **좌측 edge 가 column 외부**
- 한컴 PDF: 우측 단에 picture 표시 안 됨 (column flow 에서 제외)

→ picture 가 column 외부 emit 됨에도 cursor 274px advance → 문9 처짐.

## 3. 정정 본질 — `src/renderer/layout.rs:3500-3556`

```rust
let saved_y_offset = y_offset;
result_y = self.layout_body_picture(...);
// horz_rel_to=Column picture 의 pic_emit_x 가 col_area 우측 초과 시
// result_y = saved_y_offset (advance skip)
```

- horz_rel_to=Column picture 의 `pic_emit_x` 가 `col_area` 우측 초과 시 `result_y = saved_y_offset` (advance skip)
- `RHWP_DEBUG_TAC_CURSOR` 진단 영구화 (PR #958 머지 영역 영역 이미 devel 존재 — **충돌 영역**)

## 4. 영역 좁힘 (PR 본문 명시)

| 영역 | 영향 |
|------|------|
| horz_rel_to=Column + col 외부 emit picture | advance 제거 (회귀 fix) |
| horz_rel_to=Column + col 내부 emit picture | 영향 없음 |
| horz_rel_to=Paper/Page picture | 영향 없음 (is_paper_based 분기) |
| horz_rel_to=Para picture | 영향 없음 |
| TAC picture | 영향 없음 |

→ horz_rel_to=Column + col 외부 한정 — `feedback_hancom_compat_specific_over_general` 정합.

## 5. ⚠️ 본 환경 충돌 분석 (핵심)

### 5.1 layout.rs — PR #958 머지 영역 영역 중복 hunk

PR #961 base = devel 동기화 전 시점 → PR #958 의 `RHWP_DEBUG_TAC_CURSOR` 진단
(layout.rs hunk 2005/2014) 영역 영역 **포함**. 그러나 devel 영역 영역 PR #958 (`0b630773`)
이미 머지 → `RHWP_DEBUG_TAC_CURSOR` (:2022) + `caption_is_empty` (:3491) 존재.

| 영역 | devel HEAD (PR #958 머지) | PR #961 incoming | 정합 |
|------|---------------------------|-------------------|------|
| RHWP_DEBUG_TAC_CURSOR (:2022) | 존재 (PR #958) | 동일 추가 | **중복 충돌 — devel 측 보존** |
| caption_is_empty (:3491) | 존재 (PR #958) | 미변경 | devel 측 유지 |
| saved_y_offset + col 외부 advance skip (:3499) | 부재 | 신규 | **PR #961 측 적용 (핵심 fix)** |

→ cherry-pick 충돌 — 진단 hunk 중복 (devel 측 보존) + 핵심 fix (saved_y_offset) PR 측 적용.

### 5.2 fixture / PDF — PR #956 머지 영역 영역 중복

PR #956 (`b31e38ff`) 영역 영역 시험지 fixture 8 + 권위 PDF 4 이미 머지.
PR #961 영역 영역 동일 fixture/PDF 포함 — PR 본문 명시 ("먼저 머지되는 PR 가 가져감").
→ cherry-pick 시 동일 content → skip 또는 already exists (충돌 없음 예상).

### 5.3 orders/20260517.md
본 환경 PR #956/#958 처리 섹션 + PR #961 Task #959 작업 일지 — 충돌, 양측 보존.

## 6. 본 환경 점검

### 6.1 PR #956/#958/#961 양립
- PR #956: layout.rs:770 `paper_based = true`
- PR #958: layout.rs:3491 `caption_is_empty`
- PR #961: layout.rs:3499 `saved_y_offset` (col 외부 advance skip)
→ 3 정정 영역 영역 다른 영역, 양립. 충돌은 진단 hunk 중복만 (devel 측 보존으로 해결).

### 6.2 CI 통과
- ✅ Build & Test + CodeQL (js-ts/python/rust) + Canvas visual diff

### 6.3 검증 (PR 본문)
- cargo test --release --lib: 1288 passed, 0 failed
- 시험지 (3-11월) page 1 문9: y=1061 → y=805 ✓ 한컴 PDF 정합
- pi=69 ci=0 dy: 274 → 18 ✓
- 시험지 4종 page 1 정상, exam_kor page 18 (Square wrap, Task #722) 정상, 그 외 회귀 없음

### 6.4 잔존 (별도 issue #960)
page 2 multi-line equation (cases) off-by-one line 매핑 결함 — pre-existing,
본 PR fix 와 무관. Issue #960 등록 (연속 5 PR #963/#964 영역 영역 후속 가능).

## 7. 처리 옵션

### 옵션 A (권장) — 1 commit cherry-pick + 충돌 수동 해결 (진단 hunk devel 보존 + saved_y_offset PR 적용) + 자기 검증 + WASM 재빌드

```bash
git checkout local/devel
git cherry-pick 47699977
# 충돌 수동 해결:
#   - layout.rs: RHWP_DEBUG_TAC_CURSOR 진단 hunk → devel 측 (PR #958) 보존
#                saved_y_offset + col 외부 advance skip → PR #961 측 적용
#   - orders/20260517.md: 양측 보존 (PR #956/#958 처리 + Task #959 일지)
#   - fixture/PDF: devel 측 (PR #956) 보존 — 동일 content
# cargo test + 광범위 sweep (column picture layout 변경 → sweep 필수)
# WASM 재빌드
git checkout devel
git merge local/devel --no-ff
```

### 옵션 B — squash (이미 1 commit, A 정합)

## 8. 검증 게이트

### 8.1 자기 검증
- [ ] cherry-pick 1 commit + layout.rs/orders/fixture 충돌 수동 해결
- [ ] PR #956/#958/#961 layout.rs 3 정정 양립 확인 (:770 + :3491 + :3499)
- [ ] cargo test --release --lib ALL GREEN (PR 본문 1288 passed)
- [ ] cargo clippy --release -- -D warnings
- [ ] **광범위 sweep 7 fixture / 169 페이지** — column picture advance 변경 영역 영역 회귀 점검 필수
- [ ] WASM 재빌드 (layout.rs 변경)

### 8.2 시각 판정 게이트 — **작업지시자 시각 검증 권장**
- 시험지 (3-11월) page 1 문9 — y=805 정합 (한컴 PDF `pdf/3-11월_실전_통합_2022.pdf` 권위, 이전 ~250px 처짐 해소)
- 시험지 4종 (3-09월/3-09월2023/3-10월/3-11월) page 1 정상
- exam_kor page 18 (Square wrap, Task #722) 정상
- PR #956 page border + PR #958 sample16 page 18 회귀 부재 (양립 확인)
- 잔존: page 2 multi-line equation (Issue #960) — 본 PR 범위 외

## 9. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @jangster77 **24+ 사이클** (연속 5 PR 3번째) |
| `feedback_image_renderer_paths_separate` | layout.rs column picture advance 단일 — 다른 렌더 경로 무관 |
| `feedback_hancom_compat_specific_over_general` | horz_rel_to=Column + col 외부 한정 가드 — 케이스별 명시 (일반화 없음) |
| `feedback_diagnosis_layer_attribution` 권위 사례 강화 | RHWP_DEBUG_TAC_CURSOR 추적 영역 영역 pi=69 ci=0 dy=274 정확 진단 + Issue #952 → Issue 1/2/3 + #960 분리 |
| `feedback_pr_supersede_chain` | Issue #952 통합 → #956 (Issue 1) → #958 (Issue 2) → **#961 (Issue 3)** + #960 (신규 분리) — Issue 분리 정합 |
| `reference_authoritative_hancom` | 시험지 한컴 PDF (`pdf/3-11월`) 권위 — page 1 문9 y=805 정합 기준 |

## 10. 처리 순서 (승인 후)

1. `local/devel` 영역 cherry-pick `47699977` + 충돌 수동 해결:
   - layout.rs: 진단 hunk devel(PR #958) 보존 + saved_y_offset PR #961 적용
   - orders: 양측 보존
   - fixture/PDF: devel(PR #956) 보존 (동일 content)
2. 자기 검증 — PR #956/#958/#961 양립 확인 + cargo test + clippy + 광범위 sweep + WASM 재빌드
3. 작업지시자 시각 검증 (시험지 page 1 문9 y=805 정합, 한컴 PDF 권위 + 회귀 부재)
4. 검증 통과 → no-ff merge + push + archives + 5/17 orders
5. Issue #959 close + Issue #952 영역 영역 Issue 3 해결 → **Issue #952 close 가능 점검** (Issue 1/2/3 모두 해결)
6. PR #961 close + 연속 PR #963 진행 (Issue #960 = page 2 equation 가능성)

---

작성: 2026-05-18
