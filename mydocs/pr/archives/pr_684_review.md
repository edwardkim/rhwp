# PR #684 1차 검토 보고서

## 1. PR 메타 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | #684 |
| 제목 | fix: rhwpDev 디버깅 툴킷 결함 수정 — paragraph 식별 + 다중 매치 (#449) |
| 컨트리뷰터 | @oksure (Hyunwoo Park, oksure@gmail.com) — 8번째 사이클 PR (PR #602 후속 영역) |
| base / head | `devel` ← `oksure:contrib/rhwpdev-toolkit-v2` |
| state / mergeable | OPEN / **UNKNOWN** |
| 변경 | 2 files, +143 / -0 (`rhwp-dev.ts` +139 + `main.ts` +4) |
| commits | 2 (본질 + Copilot 리뷰 피드백 반영) |
| labels / milestone | 없음 / 미지정 |
| 연결 이슈 | **fixes #449** (PR #602 의 reopen 영역의 후속 정정) |
| 작성일 / 갱신 | 2026-05-07 14:24 / 14:54 |

### CI 상태
- statusCheckRollup 영역 부재 (CI 영역 영역 영역)

### Co-Authored-By 영역
- Co-Authored-By: Claude Opus 4.6 (1M context) 명시 (2 commits 모두) — PR #627/#636/#668 패턴 정합

### Copilot 리뷰 영역 (3 comments)
- 5/7 14:30 (Copilot): 3 comments → 컨트리뷰터 자체 검토 영역에서 모두 반영
- 5/7 14:54 (컨트리뷰터): Copilot 리뷰 피드백 반영 commit (`c72e5e5`):
  - DEV 게이트 추가 — `initRhwpDev(wasm)` 영역을 `import.meta.env.DEV` 조건 안으로 이동 (프로덕션 빌드 영역 노출 방지)
  - `showAllIds()` JSON.parse 에러 처리 영역
  - `findNearest()` JSON.parse 에러 처리 영역

---

## 2. Issue #449 권위 영역 + PR #602 후속 영역

### Issue #449 영역의 본질
서드파티 애플리케이션 영역에서 AI 모델이 제공한 `activeId` 영역의 매핑 실패 영역 → 디버깅 툴킷 영역으로 보강 영역 (`showAllIds()` / `search()` / `findNearest()`).

### PR #602 영역의 결과 + 결함 두 건 (본 환경 영역에서 발견)
PR #602 cherry-pick 후 메인테이너 시각 판정 영역에서 발견:
- **결함 #1**: paragraph 식별 부정확 (본문 vs 셀 내부 충돌) — `getPageTextLayout` 의 `runs` 영역이 본문 + 표/글상자 내부 paragraph 평탄화 → `secIdx`/`paraIdx` 영역 키 충돌 (본문 paragraph 0 "  * 사업계획서..." 가 아닌 셀 내부 paragraph "※ " 표시)
- **결함 #2**: 다중 매치 미처리 — `wasm.searchText` 단일 매치

→ Issue #449 reopen 영역 + 본 PR 의 본질 영역 정정 영역.

### Issue #449 assignee 영역
- assignee = **@oksure** — `feedback_assign_issue_before_work` 영역 정합 (작업지시자 직접 영역 지정 영역)

---

## 3. 본 환경 정합 상태 점검

### 본 환경 `rhwp-studio/src/core/rhwp-dev.ts` 영역 (PR #602 영역의 결과)
- 본 환경 영역 이미 존재 (3,888 bytes, 5/7 영역, PR #602 머지 commit `9e97072` 영역의 결과)
- 본 환경 영역의 `entries` 영역에 `container` 컬럼 부재 — 결함 #1 영역 잔존
- 본 환경 영역의 `search()` 영역 단건 영역 — 결함 #2 영역 잔존

### 본 환경 직접 비교 영역 (diff)
PR 의 영역 영역 vs 본 환경 영역 영역 차이:
- `parentParaIdx` / `controlIdx` / `cellIdx` / `cellParaIdx` 영역 추가 (TypeScript interface 영역)
- `containerKey()` 영역 신규 (cell vs body 영역 구분 영역)
- `entries` 영역에 `container` 컬럼 영역 추가
- 중복 제거 영역 키 영역에 container 영역 포함
- `search()` 영역 다중 매치 (배열 반환) 영역
- `findNearest()` 영역 결과에 container 정보 영역 포함

### 본 환경 `main.ts` 영역
- `import.meta.env.DEV` 영역 영역 정합 영역 (line 33, 189) — 본 환경 영역의 PR #602 영역 결과 영역 정합
- `initRhwpDev(wasm)` 영역 호출 영역 (line 38-39) — 이미 존재 영역. 본 PR 영역의 main.ts 영역은 `initRhwpDev` 영역을 `import.meta.env.DEV` 조건 안으로 이동 영역

### 본 사이클 영역의 Co-Authored-By Claude 패턴 영역
- PR #627 (Task #624): Claude Opus 4.7 (1M context) — 5 commits
- PR #636 (Task #630): Claude Opus 4.7 (1M context) — 8 commits
- PR #668 (Task #660): Claude Opus 4.7 — 1 commit
- **PR #684 (Issue #449 후속)**: Claude Opus 4.**6** (1M context) — 2 commits

→ 본 사이클의 Co-Authored-By Claude 패턴 정합 누적 영역.

---

## 4. PR 의 본질 정정 영역

### 4.1 결함 #1 정정 (본문 vs 셀 내부 충돌)
```typescript
function containerKey(run: TextRunInfo): string {
  if (run.parentParaIdx != null) {
    return `cell[p${run.parentParaIdx},c${run.controlIdx ?? 0},i${run.cellIdx ?? 0}]`;
  }
  return 'body';
}
```

- `containerKey()`: body paragraphs → `"body"`, cell-internal → `"cell[p0,c0,i0]"` 영역
- `showAllIds()` 출력 영역에 `container` 컬럼 추가
- 중복 제거 키 영역에 container 포함 → `0:body:0:0` ≠ `0:cell[p0,c0,i0]:0:0`
- `findNearest()` 결과 영역에 container 정보 포함

### 4.2 결함 #2 정정 (다중 매치)
```typescript
// search(text)가 searchText를 반복 호출하여 모든 매치 수집
// 반환 타입: SearchResult → SearchResult[]
// console.table로 전체 매치 목록 출력
// 매치 없을 때만 console.warn
```

### 4.3 Copilot 리뷰 피드백 반영 (Stage 2)
- `main.ts:99` — `initRhwpDev(wasm)` 영역을 `import.meta.env.DEV` 조건 안으로 이동 (프로덕션 빌드 영역의 디버그 API 노출 방지)
- `rhwp-dev.ts:49` — `showAllIds()` 영역의 `JSON.parse(layout)` 영역을 try/catch 안으로 통합 (잘못된 JSON 영역 발생 시 해당 페이지 영역 건너뛰기 + 계속 진행 영역)
- `rhwp-dev.ts:110` — `findNearest()` 영역의 동일 영역 (파싱 실패 시 null 반환)

---

## 5. 본 환경 cherry-pick simulation 결과

본 환경 임시 clone (`/tmp/pr684_test`) 에서 진행:

### cherry-pick + 충돌 처리
- 2 commits cherry-pick — `rhwp-dev.ts` 영역의 **add/add 충돌** 발생 영역
- 본 환경 영역의 PR #602 결과 영역 (`9e97072`) 과 PR #684 영역의 신규 `rhwp-dev.ts` 영역 영역 add/add 영역
- **충돌 처리**: `git checkout --theirs rhwp-studio/src/core/rhwp-dev.ts` — PR #684 영역의 정정 영역 보존 (PR #684 의 영역은 PR #602 영역의 결과 + 결함 두 건 정정 영역으로 본 환경 영역의 모든 기능 영역 보존 + 정정 영역 추가 영역)
- `main.ts` 영역 auto-merge 통과

### 결정적 검증 결과
- `cargo test --lib --release`: ✅ **1157 passed** (회귀 0)
- `rhwp-studio npm run build`: TypeScript 영역 — 임시 clone 영역의 `node_modules/@types/` 영역 부재 영역으로 영역 미실행 영역 (본 환경 영역에서 정합 영역 점검 필요)

→ 본 환경 영역에서 직접 cherry-pick + 결정적 검증 + `rhwp-studio npm run build` 영역 진행 영역 정합.

---

## 6. 옵션 분류

### 옵션 A — 전체 cherry-pick (2 commits, theirs 보존)
**진행 영역**:
```bash
git checkout local/devel
git cherry-pick 88c2fee c72e5e5
# rhwp-dev.ts add/add 충돌 → git checkout --theirs (PR #684 영역의 PR #602 + 정정 영역 보존)
```

**장점**:
- 결함 #1 + #2 영역 정정 영역
- Copilot 리뷰 피드백 반영 영역 (DEV 게이트 + JSON.parse 에러 처리)
- author oksure + Co-Authored-By Claude 2 commits 모두 보존
- 본 사이클의 Co-Authored-By Claude 패턴 정합 영역

**잠재 위험**:
- `rhwp-dev.ts` 영역의 add/add 영역 — 본 환경 영역의 PR #602 영역 결과와 PR #684 영역의 결과 영역 비교 영역 + theirs 영역 정합 영역 점검 필요
- TypeScript 빌드 영역 결정적 검증 영역 본 환경 영역에서 진행 필요

### 옵션 A-2 — squash 머지 (1 단일 commit)
- TDD 흐름 영역 (Stage 1 결함 정정 → Stage 2 Copilot 리뷰 영역 반영) 손실 영역

### 권장 영역 — 옵션 A (2 commits 단계별 보존)

**사유**:
1. **본 환경 결정적 검증 통과** — cargo test 1157 passed (회귀 0)
2. **add/add 충돌 영역의 명확한 영역 영역** — PR #684 영역 (theirs) 영역의 본질 영역이 본 환경 영역의 PR #602 영역 결과 영역의 모든 기능 영역 + 결함 두 건 정정 영역 = 본 환경 영역의 영역 모두 보존 영역
3. **Copilot 자체 리뷰 응답 영역 정합** — 본 사이클의 Copilot 리뷰 영역 자체 검토 응답 영역 패턴 정합 (PR #601 + PR #659 + 본 PR 영역)
4. **`feedback_assign_issue_before_work` 정합** — Issue #449 assignee = @oksure 영역 (작업지시자 직접 지정 영역)
5. **DTP 엔진 (`project_dtp_identity`) web editor 영역 정합 강화** — rhwp-studio 영역의 디버깅 툴킷 영역 본질 정정 영역

### 옵션 영역 요약 표

| 옵션 | 진행 가능 | Issue #449 정정 | 결정적 검증 | 권장 |
|------|----------|----------------|------------|------|
| **A** (2 commits + theirs) | ✅ 충돌 1 (theirs) | ✅ 결함 #1 + #2 정정 | ✅ cargo test 1157 | ⭐ |
| **A-2** (squash) | ✅ | ✅ 동일 | ✅ 동일 | ❌ TDD 흐름 손실 |

---

## 7. 잠정 결정

### 권장 결정
- **옵션 A 진행** — 2 commits 단계별 cherry-pick + theirs 영역 보존
- 본 환경 결정적 검증 + WASM 빌드 + `rhwp-studio npm run build` 영역 + 시각 판정 ★

### 검증 영역 (옵션 A 진행 시 본 환경 직접 점검)
1. cherry-pick (2 commits) — `rhwp-dev.ts` add/add 충돌 → theirs 보존
2. `cargo test --lib --release` 1157 passed (회귀 0)
3. `cargo test --test svg_snapshot --release` 7/7
4. `cargo test --test issue_546 / issue_554 / issue_418 / issue_501 / issue_630` 통과
5. `cargo clippy --lib -- -D warnings` 0
6. `cargo build --release`
7. Docker WASM 빌드
8. `rhwp-studio npm run build` TypeScript 타입 체크 + dist 빌드 영역 (본 환경 영역의 `@types/chrome` 영역 정합 영역 정합)
9. `rhwp-studio/public/{rhwp_bg.wasm, rhwp.js}` 영역 갱신 (web editor 영역 시각 판정 영역)
10. **시각 판정 ★** — `samples/aift.hwp` page 0 영역 + `rhwpDev.showAllIds(0)` 영역 web editor 영역 작업지시자 직접 시각 판정 (PR #602 영역의 결함 두 건 영역 정정 영역 정합 영역 점검 영역)

---

## 8. 메모리 룰 관점

본 PR 검토에 적용되는 메모리 룰:
- **`feedback_assign_issue_before_work` 정합** — Issue #449 assignee = @oksure (작업지시자 직접 지정) — 일차 방어선 영역 정합
- **`feedback_close_issue_verify_merged`** — Issue #449 close 시 본 PR 머지 검증 + Issue #449 영역의 reopen 영역의 후속 정정 영역 정합
- **`feedback_visual_judgment_authority`** — PR #602 영역의 메인테이너 시각 판정 영역에서 결함 두 건 발견 → 본 PR 정정 영역 (메인테이너 게이트웨이 방식의 권위 사례)
- **DTP 엔진 (`project_dtp_identity`)** web editor 영역 정합 강화 — 디버깅 툴킷 영역의 본질 정정 영역
- Copilot 자체 리뷰 응답 영역 정합 — PR #601/#659 영역 패턴 정합
- Co-Authored-By Claude 패턴 정합 — PR #627/#636/#668 영역 패턴 정합 (본 PR 은 Claude Opus 4.6 영역, 다른 PR 은 Claude Opus 4.7 영역)

---

## 9. 다음 단계 (CLAUDE.md PR 처리 4단계)

1. (완료) PR 정보 + Issue 연결 + base/head + mergeable + CI 상태 확인
2. (현재) `pr_684_review.md` 작성 → 승인 요청
3. (필요 시) `pr_684_review_impl.md` 작성 → 승인 요청
4. 검증 (빌드/테스트/clippy + 시각 판정 ★) + 판단 → `pr_684_report.md` 작성

### 작업지시자 결정 요청
1. **옵션 결정** — 옵션 A (2 commits, 권장) / 옵션 A-2 (squash)
2. **시각 판정 권위 영역** — `samples/aift.hwp` 영역 + `rhwpDev.showAllIds(0)` / `rhwpDev.search("사업계획")` / `rhwpDev.findNearest(...)` 영역 web editor 영역 작업지시자 직접 시각 판정 진행 가/부 (PR #602 영역의 결함 두 건 정정 영역 점검 영역)
3. **WASM 빌드 + rhwp-studio public 갱신** 가/부

결정 후 본 환경 cherry-pick + 결정적 검증 + WASM 빌드 + 시각 판정 ★ + `pr_684_report.md` 작성.
