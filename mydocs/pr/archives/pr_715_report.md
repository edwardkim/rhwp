---
PR: #715
제목: Task #713 — 분할 표 orphan sliver(<25px) 행 단위 push
컨트리뷰터: @planet6897 (Jaeuk Ryu) — 30+ 사이클 핵심 컨트리뷰터
처리: 옵션 A — 5 commits 단계별 보존 cherry-pick + no-ff merge
처리일: 2026-05-09
머지 commit: d4efc29a
---

# PR #715 처리 보고서

## 1. 처리 결과

✅ **머지 완료** — 옵션 A (5 commits 단계별 보존 cherry-pick + no-ff merge `d4efc29a`)

| 항목 | 값 |
|------|-----|
| 머지 commit | `d4efc29a` (--no-ff merge) |
| Issue #713 | close 자동 정합 (closes #713) |
| 시각 판정 | ★ **통과 (작업지시자 직접, 웹 에디터)** |
| 자기 검증 | lib **1173** + 통합 ALL GREEN + issue_713 1/1 + clippy clean |
| 광범위 sweep | 7 fixture / **170 페이지 / 회귀 0** |
| WASM 빌드 | 4,607,734 bytes (md5 `38a91f98...`) |

## 2. 정정 본질 — RowBreak 표 orphan sliver 행 단위 push

### 2.1 결함 메커니즘

`samples/2022년 국립국어원 업무계획.hwp` 12×5 일정 표 (pi=586, RowBreak) row 8 영역 의 **17.6 px 인트라-로우 분할** 결함. 한컴은 페이지 끝 영역 의 작은 sliver 영역 부재 영역 의 행 단위 push 가 한컴 권위.

### 2.2 본 환경 직접 재현 ✅

| 영역 | BEFORE | AFTER |
|------|--------|-------|
| page_index=30 (page 31) | rows=0..**9** split_end=**17.6 px** (orphan sliver) | rows=0..**8** split_end=0 px |
| page_index=31 (page 32) | rows=8..12 cont=true split_start=17.6 | rows=**8**..12 cont=true split_start=0 |

→ row 8 전체 영역 다음 페이지로 push 정합.

### 2.3 정정 (`src/renderer/typeset.rs:1931+`, +5 LOC + 주석)

```rust
const MIN_TOP_KEEP_PX: f64 = 25.0;
if avail_content_for_r >= MIN_SPLIT_CONTENT_PX
    && avail_content_for_r >= min_first_line
    && avail_content_for_r >= MIN_TOP_KEEP_PX  // [Task #713] orphan 가드
    && remaining_content >= MIN_SPLIT_CONTENT_PX
{
    end_row = r + 1;
}
```

### 2.4 임계값 25 px 결정 근거

| 케이스 | avail_content_for_r | 가드 적용 후 |
|--------|--------------------|----|
| 본 결함 (row 8 sliver) | **17.6 px** | < 25 → 차단 ✓ |
| synam-001 p23 정합 | **27.3 px** | ≥ 25 → 변경 없음 ✓ |
| 기타 정합 분할 | 93/437/510 px | ≥ 25 → 변경 없음 ✓ |

→ 17.6 ↔ 27.3 px 사이 영역 의 25 px 영역 임계값 (`feedback_hancom_compat_specific_over_general` 정합).

### 2.5 활성 경로 메모

`src/document_core/queries/rendering.rs:1041-1042` — `RHWP_USE_PAGINATOR=1` 미설정 시 `typeset.rs::typeset_section` 영역 활성, `pagination/engine.rs::paginate_with_measured_opts` 영역 fallback. 본 정정은 활성 경로 (typeset.rs) 영역 만 적용 — fallback 경로 영역 부재 영역. RHWP_USE_PAGINATOR=1 영역 미설정 영역 의 fallback 영역 영역 위험 매우 낮음.

## 3. 본 환경 cherry-pick + 검증

### 3.1 cherry-pick (5 commits)
```
ca280499 Task #713 Stage 0: 수행 + 구현 계획서
e2fb974b Task #713 Stage 1 (RED): RowBreak 표 인트라-로우 분할 회귀 테스트 — FAIL 확인
69600723 Task #713 Stage 2: 가설 H1/H3 폐기 + enum 매핑 결함 발견
3bb87191 Task #713 Stage 3 (GREEN): orphan 임계값 가드 — avail_content_for_r >= 25px
c7018587 Task #713 Stage 4-5-6: 회귀 + 광범위 + 최종 보고서 (closes #713)
```
충돌 0건 (auto-merging typeset.rs).

### 3.2 결정적 검증

| 검증 | 결과 |
|------|------|
| `cargo build --release` | ✅ 통과 (29.20s) |
| `cargo test --release --test issue_713` | ✅ **1/1 PASS** (회귀 가드, 동적 페이지 sweep) |
| `cargo test --release --test issue_712` | ✅ PASS (PR #714 영역 보존) |
| `cargo test --release --test svg_snapshot` | ✅ 8/8 (form-002 PR #706 영역 보존) |
| `cargo test --release` | ✅ lib **1173** + 통합 ALL GREEN, failed 0 |
| `cargo clippy --release --all-targets` | ✅ 신규 경고 0 |
| 광범위 sweep | 7 fixture / **170 페이지 / 회귀 0** ✅ |
| WASM 빌드 (Docker) | ✅ 4,607,734 bytes |

### 3.3 시각 판정 ★ 통과

작업지시자 직접 시각 판정 (2026-05-09, 웹 에디터):
- BEFORE / AFTER SVG 비교 — page 31 row 8 sliver (17.6 px) → row 8 전체 page 32 push 정합
- 한컴 PDF 권위본 정합

### 3.4 머지 commit

`d4efc29a` — `git merge --no-ff local/task715` 단일 머지 commit. PR #694/#693/#695/#699/#706/#707/#710/#711/#714 패턴 일관.

## 4. 영향 범위

### 4.1 무변경 영역
- avail_content_for_r ≥ 25 px 영역 의 분할 영역 (synam-001 등 정합 영역)
- 비-RowBreak 표 / 비-Partial 분할 영역
- fallback 경로 (pagination/engine.rs) 영역 (위험 매우 낮음)
- 광범위 sweep 7 fixture / 170 페이지 / 회귀 0

### 4.2 변경 영역 (영향 좁힘)
- avail_content_for_r < 25 px 영역 의 orphan sliver 분할 차단
- 행 전체 영역 다음 페이지 push

→ **위험 매우 낮음**. 5 라인 본질 변경 + 임계값 결정 근거 명확.

## 5. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @planet6897 30+ 사이클 핵심 컨트리뷰터 (누적 23 머지) |
| `feedback_hancom_compat_specific_over_general` | 임계값 25 px 영역 의 영향 좁힘 — 17.6 ↔ 27.3 px 사이 |
| `feedback_image_renderer_paths_separate` | ⚠️ 활성 경로 (typeset.rs) 만 정정 — fallback 경로 영역 부재 (위험 매우 낮음) |
| `feedback_process_must_follow` | TDD Stage 0 → 1 RED → 2 가설 → 3 GREEN → 4-5-6 검증/보고서 절차 정합 |
| `feedback_visual_judgment_authority` | 결정적 검증 (CI ALL SUCCESS + 회귀 가드 + 광범위 sweep) + 작업지시자 시각 판정 ★ 통과 (웹 에디터) |
| `feedback_visual_regression_grows` | 회귀 가드 영역 의 동적 페이지 sweep 영역 (PR #714 패턴 정합) |

## 6. 잔존 후속

- 본 PR 본질 정정 영역 의 잔존 결함 부재
- fallback 경로 (pagination/engine.rs) 영역 의 동일 가드 영역 미적용 — 본 PR 범위 외 (RHWP_USE_PAGINATOR=1 영역 미설정 영역 의 fallback 영역 영역 위험 매우 낮음). 향후 audit 가능.

---

작성: 2026-05-09
