---
PR: #956
제목: fix — 쪽 테두리 paper-based outline 강제 (#920 비트 해석 회귀 정정, closes #952)
컨트리뷰터: @jangster77 (Taesup Jang) — 24+ 사이클 핵심 컨트리뷰터 (HWP3/WMF/page border 깊은 작업)
base / head: devel / local/task952
mergeStateStatus: CLEAN
mergeable: MERGEABLE
CI: ✅ Build & Test + CodeQL (js-ts/python/rust) + Canvas visual diff
변경 규모: +251 / -2, 17 files (코드 1 / 문서 4 / fixture 8 / PDF 4)
커밋: 1
검토일: 2026-05-17
---

# PR #956 검토

## 1. 메타

| 항목 | 값 |
|------|-----|
| PR 번호 | #956 |
| 제목 | fix: 쪽 테두리 paper-based outline 강제 (#920 비트 해석 회귀 정정, closes #952) |
| 컨트리뷰터 | @jangster77 (Taesup Jang) — **24+ 사이클** (HWP3/WMF/page border 핵심) |
| base / head | devel / local/task952 |
| mergeable | MERGEABLE (CLEAN — 동기화된 devel `016e694c` 기준 충돌 부재) |
| CI | ✅ 전 항목 통과 |
| 변경 규모 | +251 / -2, 17 files (코드 1 / 문서 4 / fixture 8 / PDF 4) |
| 커밋 수 | 1 (`3dc65e37`) |
| closes | #952 (Issue 1 만, Issue 2/3 별도 task 분리) |
| 연속 5 PR | **#956 (1번째)** → #958 → #961 → #963 → #964 (동일 컨트리뷰터, 순차 처리) |

## 2. 본질 (Issue #952 Issue 1)

`samples/hwp3-sample16.hwp` + `3-11월_실전_통합_2022.hwp` 등 다수 sample 영역 영역
페이지 외곽선 (page border) 이 paper-based 가 아닌 body-based 로 잘못 렌더 → 본문 돌출.

### 회귀 source 확정 (Issue #952 bisect)
| Commit | 외곽선 |
|--------|--------|
| local/task877 tip (`8514e68a`) | paper-based ✓ (정상 baseline) |
| `4bb11289` (#920) | body-based ⚠️ (회귀 도입) |
| 현재 devel | body-based ⚠️ (회귀 잔존 — 본 환경 layout.rs:764 확인) |

`4bb11289 fix: 쪽테두리 종이기준/본문기준 bit 해석 반전 정정 (closes #920)` 영역 영역
`paper_based = (attr & 0x01) == 0` 비트 반전 → 회귀.

### 진단 — bit 0 은 outline 위치 결정 비트 아님 (5+ samples 한컴 viewer 실측)

| Sample | attr (HWP5) | textBorder (HWPX) | fillArea (HWPX) | 한컴 시각 |
|--------|-------------|---------------------|-----------------|-----------|
| sample16 (HWP3→HWP5) | 0x01 | PAPER | PAPER | paper |
| 3-09월/3-11월 시험지 | 0x00 | CONTENT | PAPER | **paper** |
| biz_plan / 국립국어원 / text-align-2 / pua-test | 0x01 | - | - | paper |

→ attr bit 0 = 0/1 양쪽 다 paper-based, HWPX textBorder=PAPER/CONTENT 양쪽 다 paper-based.
즉 bit 0 은 outline 위치 결정 비트가 아닌 별도 의미 (text wrap interaction 등).

### 회귀 history (PR 본문 명시)
- task877 baseline: `(attr & 0x01) != 0` — sample16 (attr=1) 정합, 시험지 (attr=0) 회귀
- #920 (`4bb11289`): `(attr & 0x01) == 0` — 시험지 정합, sample16 회귀
- **본 PR (#952)**: `true` — 모든 sample 한컴 정합

## 3. 정정 본질 — `src/renderer/layout.rs` +16/-2 (1 곳)

```rust
// [Issue #952] 한컴 viewer 실측 결과 — 외곽선 위치는 항상 paper-spacing 기준.
// (회귀 history 코멘트 — task877 != 0 / #920 == 0 / #952 true)
let paper_based = true;
if std::env::var("RHWP_DEBUG_PAGE_BORDER").is_ok() {
    eprintln!("PAGE_BORDER: attr=0x{:08x} bit0={} paper_based={} bfid={} spacing(...)", ...);
}
```

- `paper_based = true` 강제 — `(pbf.attr & 0x01) == 0` 회귀 코드 대체
- `RHWP_DEBUG_PAGE_BORDER` 환경변수 진단 영구화 — attr 비트 + paper_based 결정 추적
- 회귀 history 코멘트 영구 보존 — 차후 재회귀 방지

## 4. fixture / PDF 추가 (13 files)

| 분류 | 파일 |
|------|------|
| 시험지 fixture | `3-09월_교육_통합_2022.{hwp,hwpx}`, `3-09월_교육_통합_2023.{hwp,hwpx}`, `3-10월_교육_통합_2022.{hwp,hwpx}`, `3-11월_실전_통합_2022.{hwp,hwpx}` (8) |
| 한컴 2022 권위 PDF | `pdf/3-09월_교육_통합_2022.pdf` 외 3 (4) |

→ `reference_authoritative_hancom` 정합 — 한컴 2022 PDF 권위 자료 영구 보존 (`pdf/`).
시험지 회귀 fixture 영역 영역 차후 회귀 가드.

## 5. 본 PR 범위 외 (#952 잔존, 별도 task 분리 — PR 본문 명시)

| Issue | 본질 | 분리 사유 |
|-------|------|----------|
| Issue 2 | sample16 page 18 본문 다음 페이지 밀림 | typeset.rs multi-TAC-shape cursor over-advance ~430px (장기 결함, typeset 2700+ 줄 multi-state 복잡도) |
| Issue 3 | 시험지 page 1 문9 vertical 처짐 | HWP5 column layout, 다른 root cause |

→ **부분 해결 + 명확한 분리** — PR 본문 영역 영역 archive/task936 ("9회 시도 + 5회 revert")
대조 교훈 명시. `feedback_diagnosis_layer_attribution` + `feedback_hancom_compat_specific_over_general` 정합.

## 6. 본 환경 점검

### 6.1 회귀 source 확인
현재 devel `layout.rs:764` 영역 영역 `paper_based = (pbf.attr & 0x01) == 0` (#920 회귀 코드)
그대로 존재 — 본 PR 영역 영역 정확히 이 라인 정정.

### 6.2 변경 격리
- 코드 변경 1곳 (`layout.rs` page border 분기) — 다른 렌더 경로 무관
- `paper_based = true` 강제 영역 영역 모든 page border 영역 영역 paper-based (한컴 실측 정합)
- 회귀 위험 — body-based outline 영역 영역 의도적으로 사용하는 sample 영역 영역 있으면 회귀 가능.
  그러나 PR 본문 + Issue #952 영역 영역 5+ sample 실측 모두 paper-based → 위험 낮음.

### 6.3 CI 통과
- ✅ Build & Test + CodeQL (js-ts/python/rust) + Canvas visual diff

### 6.4 검증 (PR 본문)
- cargo test --release --lib: 1288 passed, 0 failed
- sample16 HWP3/HWP5/HWPX page 17 외곽선: paper-based (x=18.93~774.77) ✓ task877 baseline 정합
- 시험지 (3-11월) HWP5/HWPX page 1 외곽선: paper-based (x=26.45~767.25) ✓ 한컴 정합

## 7. 처리 옵션

### 옵션 A (권장) — 1 commit cherry-pick + no-ff merge + 본 환경 검증

```bash
git checkout local/devel
git cherry-pick 3dc65e37
git checkout devel
git merge local/devel --no-ff
```

- cargo test --release + 광범위 sweep (page border 영역 영역 layout 변경 → sweep 필수)
- WASM 재빌드 (layout.rs 변경 → WASM 영향)
- 작업지시자 시각 검증 (sample16 + 시험지 page border paper-based)

### 옵션 B — squash cherry-pick (단일 commit, 이미 1 commit 영역 영역 옵션 A 정합)

## 8. 검증 게이트

### 8.1 자기 검증
- [ ] cherry-pick 1 commit (CLEAN — 충돌 부재 예상)
- [ ] cargo test --release ALL GREEN (PR 본문 1288 passed)
- [ ] cargo clippy --release -- -D warnings
- [ ] **광범위 sweep 7 fixture / 170 페이지** — page border 영역 영역 layout 변경 영역 영역 회귀 점검 필수
- [ ] WASM 재빌드 (layout.rs 변경)

### 8.2 시각 판정 게이트 — **작업지시자 시각 검증 권장**
- sample16 (HWP3/HWP5/HWPX) page 17 외곽선 — paper-based 정합 (task877 baseline)
- 시험지 (3-09월/3-11월) HWP5/HWPX page 1 외곽선 — paper-based 정합 (한컴 2022 PDF 권위)
- 다른 sample page border 회귀 부재 (sweep + 시각 점검)
- Issue 2/3 (sample16 page 18 밀림, 시험지 문9 처짐) — 본 PR 범위 외, 잔존 확인

## 9. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @jangster77 **24+ 사이클** (HWP3/WMF/page border 핵심) — 연속 5 PR 1번째 |
| `feedback_image_renderer_paths_separate` | layout.rs page border 분기 단일 — 다른 렌더 경로 무관 |
| `feedback_hancom_compat_specific_over_general` | bit 0 추측 해석 (task877/#920) → 5+ sample 한컴 실측 영역 영역 `true` 강제 — 추측보다 실측 |
| `feedback_diagnosis_layer_attribution` | 회귀 source `4bb11289` (#920) bisect 정확 + Issue 1/2/3 분리 진단 |
| `feedback_pr_supersede_chain` | task877 → #920 (`4bb11289` 회귀 도입) → **#956** (회귀 정정) — (c) 패턴 |
| `reference_authoritative_hancom` | 시험지 한컴 2022 PDF (`pdf/`) 권위 자료 영구 보존 + 회귀 fixture |
| `feedback_v076_regression_origin` | #920 영역 영역 추측 bit 해석 → 작업지시자 환경 회귀 — 한컴 실측 영역 영역 정정 |

## 10. 처리 순서 (승인 후)

1. `local/devel` 영역 cherry-pick `3dc65e37` (CLEAN, 충돌 부재 예상)
2. 자기 검증 — cargo test + clippy + 광범위 sweep + WASM 재빌드
3. 작업지시자 시각 검증 (sample16 + 시험지 page border paper-based, 한컴 2022 PDF 권위)
4. 검증 통과 → no-ff merge + push + archives + 5/17 orders
5. Issue #952 영역 영역 Issue 1 만 해결 — Issue 2/3 별도 task 분리 명시 후 close 점검
6. PR #956 close + 연속 PR #958 진행

---

작성: 2026-05-17
