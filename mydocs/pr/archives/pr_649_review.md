---
PR: #649
제목: fix: Layout 리팩터링 Phase 1 디버그 인프라 누락 회귀 정정 (Task #517 재적용, closes #647)
컨트리뷰터: @planet6897 (Jaeuk Ryu) — 13번째 사이클 PR
base: devel (BEHIND)
mergeable: MERGEABLE
CI: ALL SUCCESS
변경: +528/-0, 7 files (1 commit)
---

# PR #649 1차 검토 보고서

## 1. 메타 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | #649 |
| 제목 | fix: Layout 리팩터링 Phase 1 디버그 인프라 누락 회귀 정정 (Task #517 재적용) |
| 컨트리뷰터 | @planet6897 (Jaeuk Ryu) — 13번째 사이클 PR |
| base / head | devel / pr-task647 |
| mergeStateStatus | BEHIND |
| mergeable | MERGEABLE |
| CI | Build & Test / CodeQL / Canvas visual diff — ALL SUCCESS |
| 변경 규모 | +528 / -0, 7 files |
| 커밋 수 | 1 (`ffb32ff7`, 원 commit `9c16a1b4` 의 cherry-pick) |
| closes | #647 |

## 2. Issue #647 본질 — 누락 회귀

Task #517 (Layout 리팩터링 Phase 1 — 디버그 인프라 + 회귀 검증 도구) 의 원 commit `9c16a1b4` 가 묶음 머지 `a7e43f9 (Task #517/#518/#519/#520/#521/#523/#528)` 영역으로만 `local/devel` 에 머지되고 `stream/devel` (= `devel`) 으로 승격 안 됨.

본 환경 직접 확증:
```
git merge-base --is-ancestor 9c16a1b4 devel  # NOT in devel
git merge-base --is-ancestor a7e43f9 devel   # NOT in devel
```

**누락 항목** (devel 기준):
- `src/renderer/layout/paragraph_layout.rs` 의 `layout_debug_enabled()` 헬퍼 부재
- `layout_inline_table_paragraph` 의 `RHWP_LAYOUT_DEBUG` 진단 로깅 부재
- `scripts/svg_regression_diff.sh` 파일 부재
- `mydocs/manual/svg_regression_diff.md` 부재

**선례**: PR #620 (Task #519, Issue #618) 가 동일 패턴 — Picture flip/rotation 누락 회귀 정정 (`c80d2272` merge).

## 3. PR 의 정정

### 단일 commit 패치
`ffb32ff7` — 원 commit `9c16a1b4` 의 `git cherry-pick -x` (`(cherry picked from commit 9c16a1b4)` 명시).

### 본질 변경
| 영역 | 경로 | 추가 |
|------|------|------|
| 디버그 로깅 | `src/renderer/layout/paragraph_layout.rs` | +32 (env-var-checked) |
| 회귀 비교 도구 | `scripts/svg_regression_diff.sh` | +154 |
| 매뉴얼 | `mydocs/manual/svg_regression_diff.md` | +85 |
| 거버넌스 | `mydocs/plans/`, `mydocs/working/`, `mydocs/report/` | +257 |

총 +528 / -0 LOC, 7 files.

### 핵심 본질 — env-var 가드
```rust
#[inline]
pub(crate) fn layout_debug_enabled() -> bool {
    std::env::var("RHWP_LAYOUT_DEBUG").map(|v| v == "1").unwrap_or(false)
}

if layout_debug_enabled() {
    eprintln!("LAYOUT_INLINE_TABLE_PARA: pi={} ...");
    // line_segs / inline_tables 상세
}
```

→ env-var 미설정 시 동작 변경 0 (기본 동작 무영향).

## 4. 본 환경 cherry-pick simulation

### 4.1 깨끗한 적용
- `local/pr649-sim` 브랜치, 1 commit cherry-pick
- **충돌 0건**

### 4.2 결정적 검증
- `cargo test --release --lib` → **1165 passed** (회귀 0)
- `cargo test --release` → ALL PASS (failed 0)
- `cargo clippy --release` → clean

### 4.3 env-var 동작 영역 정합 확인

본 환경 직접 측정:
```
$ cargo run --release --bin rhwp -- export-svg samples/exam_science.hwp -p 1 -o /tmp/pr649-default
  → /tmp/pr649-default/exam_science_002.svg

$ RHWP_LAYOUT_DEBUG=1 cargo run --release --bin rhwp -- export-svg samples/exam_science.hwp -p 1 -o /tmp/pr649-debug
  → /tmp/pr649-debug/exam_science_002.svg

$ md5sum /tmp/pr649-{default,debug}/exam_science_002.svg
842c9513bbbb833c5ba1ad27bac52694  /tmp/pr649-default/exam_science_002.svg
842c9513bbbb833c5ba1ad27bac52694  /tmp/pr649-debug/exam_science_002.svg
```

→ env-var 미설정 / 설정 영역의 **SVG byte 동일 (md5 일치)**. 본 PR 의 핵심 본질 (기본 동작 무영향) 확증.

### 4.4 본 PR commit ↔ 원 commit 동일성 확증

본 PR commit `ffb32ff7` 의 footer:
```
(cherry picked from commit 9c16a1b45b5b6d0b63cda07dad89e4665db7dd9a)
```

원 commit `9c16a1b4` 와 본 PR commit `ffb32ff7` 의 동일 변경 영역 (cherry-pick 영역의 정합).

## 5. 검토 관점

### 5.1 Co-Authored-By 영역
원 commit `9c16a1b4` 의 author 는 `Jaeook Ryu <jaeook.ryu@gmail.com>` — 본 PR commit `ffb32ff7` 도 동일 author. cherry-pick author 정합.

### 5.2 후속 PR 영역
PR 본문 명시:
> 본 PR 머지 후 후속 PR 로 #648 (Task #518 Layout Phase 2 본질 정정 — `line_break_char_idx` 다중화) 적용 예정. #648 의 `LAYOUT_BREAK_INDICES` 디버그 로깅이 본 PR 의 `layout_debug_enabled()` 헬퍼에 의존하므로 본 PR 선행 머지 필요.

→ 본 PR 은 Phase 1 (인프라). Phase 2 (본질 정정) 가 의존하므로 **선행 머지 필수**.

### 5.3 회귀 위험성
- env-var-checked → 기본 동작 무영향 확증
- 단일 함수 헬퍼 + `eprintln!` 만 추가 (렌더링 / 파싱 / 직렬화 무영향)
- `scripts/svg_regression_diff.sh` 는 도구 영역 (런타임 무영향)
- 매뉴얼 / 거버넌스 영역 (런타임 무영향)

→ **회귀 위험 0**.

### 5.4 묶음 머지 누락 패턴 (재발)

`a7e43f9 (Task #517/#518/#519/#520/#521/#523/#528)` 영역의 누락 영역:
- ✅ Task #519 → PR #620 으로 정정 완료 (`c80d2272`)
- 🔄 Task #517 → 본 PR #649 (현재 처리 중)
- 🔄 Task #518 → PR #648 (후속 예정)
- ❓ Task #520, #521, #523, #528 → 별도 점검 필요?

→ **본 PR 의 후속 영역으로 잔여 task 누락 영역 점검 권장**.

## 6. 메모리 룰 관점

### `feedback_close_issue_verify_merged`
> 이슈 close 시 정정 commit devel 머지 검증 필수. Task #376 정정 commit 이 임시 브랜치에만 있고 devel 미머지 → 동일 결함 재발 (Task #418).

→ **본 결함의 본질 영역**. Task #517 close 시 `9c16a1b4` 의 devel 머지 검증 누락 영역. 본 PR 이 정정.

### `feedback_visual_regression_grows`
→ env-var 미설정 SVG byte 동일 (md5 일치) 영역의 결정적 검증으로 충분. 시각 판정 영역의 추가 게이트는 **불필요** (런타임 동작 무영향).

### `feedback_v076_regression_origin`
→ env-var-checked 본질 영역 — 컨트리뷰터 환경과 작업지시자 환경의 차이 영역의 회귀 가능성 0.

## 7. 결정 옵션

| 옵션 | 내용 | 비고 |
|------|------|------|
| **A** | 1 commit FF/no-ff merge (단순 cherry-pick 영역) | 본질 정합 |
| **B** | 1 commit no-ff merge + 후속 #648 영역 안내 (Phase 2 의존성) | PR 본문 영역 명시된 후속 영역 정합 |
| **C** | merge 보류 — 묶음 머지 잔여 task 영역 (#520/#521/#523/#528) 사전 점검 | 본 PR 영역 외 |

## 8. 잠정 결정

**옵션 B (1 commit no-ff merge + 후속 #648 영역 안내)** 권장.

이유:
1. 결정적 검증 (1165 lib + clippy clean) ALL PASS
2. SVG md5 영역 동일 — 기본 동작 무영향 확증
3. cherry-pick 영역 충돌 0건
4. `(cherry picked from commit 9c16a1b4)` 영역의 author 정합
5. PR #620 (Task #519 재적용) 영역의 동일 패턴 선례 정합
6. PR #648 (Phase 2 본질 정정) 영역의 의존성 영역 — 본 PR 선행 머지 필수
7. **시각 판정 영역 불필요** — env-var-checked 영역의 런타임 무영향 확증

**시각 판정 게이트 면제 영역의 합리화**:
- env-var 미설정 / 설정 영역의 SVG md5 동일성 영역 직접 확증
- `layout_inline_table_paragraph` 영역의 신규 함수 영역 부재 (기존 함수 내부 영역의 logging 추가만)
- 렌더링 / 파싱 / 직렬화 영역의 변경 부재
- 작업지시자 시각 게이트 영역은 런타임 동작 영역의 변경 영역에서 의무화 영역이며, 본 PR 영역은 미해당

## 9. 작업지시자 결정 요청

1. **옵션 선택**: A / B / C 중?
2. **시각 판정 영역 면제 영역**: env-var-checked 본질 영역 + SVG md5 동일성 영역 정합으로 시각 판정 영역의 면제 영역 정합?
3. **묶음 머지 잔여 task 영역 점검**: Task #520 / #521 / #523 / #528 영역의 누락 영역 점검 영역 — 본 PR 후속 영역에서 진행?
4. **PR #648 (Phase 2 본질 정정) 영역**: 본 PR 머지 후 즉시 처리 영역?

---

작성: 2026-05-08
