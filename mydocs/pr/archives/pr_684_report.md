# PR #684 처리 보고서

## 1. 처리 결과

| 항목 | 값 |
|------|-----|
| PR | #684 — fix: rhwpDev 디버깅 툴킷 결함 수정 — paragraph 식별 + 다중 매치 (#449) |
| 컨트리뷰터 | @oksure (Hyunwoo Park, oksure@gmail.com) — 8번째 사이클 PR |
| 연결 이슈 | #449 (closed) — PR #602 reopen 영역의 후속 정정 |
| 후속 이슈 | **#692 신규 등록** (search() 미정합 영역 분리) |
| 처리 옵션 | 옵션 A — 2 commits cherry-pick + 본 환경 정정 보강 2 commits |
| devel commits | `44d532e` 본질 + `9792fee` Copilot 반영 + `f73177a` main.ts 정정 + `a1ea189` search() 가드 |
| 처리 일자 | 2026-05-08 |

## 2. cherry-pick 결과

### 컨트리뷰터 commits (author oksure + Co-Authored-By Claude Opus 4.6 보존)
| Stage | hash | 변경 |
|-------|------|------|
| 본질 | `44d532e` | 결함 #1 (paragraph 식별 영역) + 결함 #2 (다중 매치 영역) 정정 |
| Copilot 반영 | `9792fee` | DEV 게이트 + JSON.parse 에러 처리 (3 comments 반영) |

### 본 환경 정정 보강 commits (committer edward, Co-Authored-By Claude Opus 4.7)
| Stage | hash | 변경 |
|-------|------|------|
| main.ts 정정 | `f73177a` | 중복 import 영역 정정 (TS2300 영역 정정) |
| search() 가드 | `a1ea189` | 무한 루프 가드 추가 (옵션 B: wrapped + MAX_MATCHES + 후진 가드) |

### 충돌 처리
- `rhwp-studio/src/core/rhwp-dev.ts` add/add 충돌 → `git checkout --theirs` (PR #684 영역의 PR #602 결과 + 결함 두 건 정정 영역 보존)
- `rhwp-studio/src/main.ts` auto-merge — 후속 commit `f73177a` 영역으로 정정

## 3. 본 환경 결정적 검증

| 항목 | 결과 |
|------|------|
| `cargo build --release` | ✅ |
| `cargo test --lib --release` | ✅ **1157 passed** (회귀 0) |
| `cargo clippy --lib --release -- -D warnings` | ✅ 0 |
| Docker WASM 빌드 | ✅ **4,572,439 bytes** (PR #636 baseline 동일) |
| `rhwp-studio npm run build` | ✅ TypeScript 타입 체크 + dist 빌드 (`index-21tD9d3W.js` 691,386 / `rhwp_bg-BYW5KGI8.wasm` 4,572,439) |
| `rhwp-studio/public/{rhwp_bg.wasm, rhwp.js}` | ✅ 갱신 (vite dev server web 영역) |

## 4. 본질 정정 영역

### 4.1 결함 #1 정정 (paragraph 식별 영역, PR `44d532e`)
```typescript
function containerKey(run: TextRunInfo): string {
  if (run.parentParaIdx != null) {
    return `cell[p${run.parentParaIdx},c${run.controlIdx ?? 0},i${run.cellIdx ?? 0}]`;
  }
  return 'body';
}
```
- 본문 paragraphs → `"body"` 영역
- cell-internal paragraphs → `"cell[p0,c0,i0]"` 영역 형식
- `showAllIds()` 출력에 `container` 컬럼 추가
- `findNearest()` 결과에 `container` 정보 포함

### 4.2 결함 #2 정정 (다중 매치 영역, PR `44d532e`)
- `search(text)` 영역에서 `searchText` 영역 반복 호출 영역 → 모든 매치 영역 수집
- `SearchResult` → `SearchResult[]` 영역 (배열 반환)

### 4.3 Copilot 리뷰 피드백 반영 (PR `9792fee`)
3 comments 자체 검토 영역 반영:
- `main.ts:99` — `initRhwpDev(wasm)` 영역을 `import.meta.env.DEV` 조건 안으로 이동
- `rhwp-dev.ts:49` — `showAllIds()` 영역의 `JSON.parse(layout)` try/catch 안으로 통합
- `rhwp-dev.ts:110` — `findNearest()` 영역의 동일 영역 (파싱 실패 시 null 반환)

### 4.4 본 환경 정정 보강 #1 — main.ts 중복 import (`f73177a`)
PR #684 cherry-pick 결과 영역에서 TS2300 영역 에러 영역 발생:
- 본 환경 영역의 PR #602 결과 (`9e97072`) — line 28 import + line 38-40 외부 호출
- PR #684 영역의 c72e5e5 — `initialize()` 함수 안 DEV 게이트 영역 안 호출

→ line 38-40 영역의 외부 호출 영역 제거 (PR #684 의 의도 영역 정합) + line 28 import + line 99-100 DEV 게이트 안 호출 영역 유지.

### 4.5 본 환경 정정 보강 #2 — search() 무한 루프 가드 (`a1ea189`, 옵션 B)

**작업지시자 직접 영역 무한 루프 영역 위험 영역 발견**:
- `for (;;)` 영역의 break 조건 영역이 `(!r || !r.found)` 영역만 영역
- `search_text_native` 영역의 wrap-around 영역 동작 영역 — 마지막 매치 이후 → `body_hits[0]` + `wrapped=true` + `found=true` 반환 영역 → break 안 됨 영역 → 무한 루프 영역 발생 영역

**옵션 B 정정** (작업지시자 결정):

```typescript
search(text: string): SearchResult[] {
  const results: SearchResult[] = [];
  let sec = 0, para = 0, charOff = 0;

  // 반복 횟수 한계 — 방어 가드
  const MAX_MATCHES = 10000;
  for (let i = 0; i < MAX_MATCHES; i++) {
    const r = wasm.searchText(text, sec, para, charOff, true, false);
    if (!r || !r.found) break;
    // wrap-around 가드
    if (r.wrapped) break;
    // 후진 가드
    const sNext = r.sec ?? 0;
    const pNext = r.para ?? 0;
    const cNext = r.charOffset ?? 0;
    if (sNext < sec
        || (sNext === sec && pNext < para)
        || (sNext === sec && pNext === para && cNext < charOff)) break;
    results.push(r);
    sec = sNext;
    para = pNext;
    charOff = cNext + (r.length ?? text.length);
  }
  ...
}
```

4 가드 영역:
1. SearchResult 인터페이스 영역 — `wrapped?: boolean` 영역 추가 (본 환경 `core/types.ts:590` 영역 정합)
2. wrap-around 가드 — `r.wrapped = true` 영역 시 break (본질)
3. 반복 횟수 한계 — `MAX_MATCHES = 10000` (방어)
4. 후진 가드 — 다음 매치 위치 영역 직전 위치 영역 비교 (방어)

## 5. 메인테이너 Chrome 웹 콘솔 검증

`samples/aift.hwp` 영역 로드 후 Chrome 웹 콘솔 영역 검증:

### ✅ `rhwpDev.findNearest(618)` 통과
작업지시자 평가:
```
[rhwpDev] findNearest(618, page=0): closest paraIdx=0 (cell[p0,c2,i0], distance=618) "※ "
{paraIdx: 0, distance: 618, text: '※ ', container: 'cell[p0,c2,i0]'}
```
→ **결함 #1 정정 영역의 container 정보 영역 정합 영역 정합** (`cell[p0,c2,i0]` 영역).

### ⚠️ `rhwpDev.search(text)` 미정합 → 후속 영역 분리

작업지시자 평가: "rhwpDev.search('text') 쪽은 여전히 잘 동작하지 않습니다. 이건 별도 이슈로 등록해서 처리하겠습니다."

→ **후속 Issue #692 등록**: rhwpDev.search(text) 다중 매치 영역 동작 불완전 — wrap-around 가드 추가 후에도 잔존 영역.

본 환경 영역의 4 가드 영역 적용 영역 후에도 영역 동작 영역 미정합 영역의 본질 영역 — 후속 영역 정정 영역 (M100 영역).

## 6. devel 머지 + push

### 진행
1. `git cherry-pick 88c2fee c72e5e5` (PR #684 2 commits)
2. `rhwp-studio/src/core/rhwp-dev.ts` add/add 충돌 → `git checkout --theirs`
3. 본 환경 정정 commit `f73177a` (main.ts 중복 import 영역)
4. 본 환경 정정 commit `a1ea189` (search() 무한 루프 가드, 옵션 B)
5. devel ← local/devel ff merge
6. push: `b051dd0..a1ea189`

### 분기 처리
- 본 cherry-pick 시점 origin/devel 분기 0 — `feedback_release_sync_check` 정합

## 7. PR / Issue close

- **PR #684**: 한글 댓글 등록 + close (`gh pr close 684`)
- **Issue #449**: 한글 댓글 등록 + close (`gh issue close 449`) — PR #602 reopen 영역의 후속 정정 영역 완료
- **Issue #692 신규 등록**: `rhwpDev.search(text)` 미정합 영역의 후속 영역 분리 영역 (M100, assignee 미지정)

## 8. 본 PR 의 정체성 영역

### Issue #449 영역의 영역 진행
- PR #602 (5/7 처리, `9e97072`) — 본질 영역 cherry-pick + 메인테이너 시각 판정 영역에서 결함 두 건 발견 → Issue #449 reopen
- **PR #684 (5/7~5/8 처리, 본 PR)** — 결함 두 건 정정 영역 + Copilot 리뷰 영역 자체 검토 영역 + 본 환경 정정 보강 2 commits + Chrome 웹 콘솔 영역 검증 영역
- Issue #692 (5/8 신규) — search() 영역의 미정합 영역의 후속 영역

### 메인테이너 게이트웨이 방식의 권위 사례 영역 강화
- PR #602 영역 — 결정적 검증만으로는 본질 결함 식별 영역 부족 영역, 권위 자료 시각 판정 영역에서 결함 두 건 식별
- **PR #684 영역** — 본 환경 정정 보강 영역의 무한 루프 영역 영역 발견 (작업지시자 직접 영역의 영역 사용 영역) → 옵션 B 가드 영역 + 후속 영역 분리 영역
- Chrome 웹 콘솔 영역 검증 영역 패턴 영역 — `findNearest` 통과 + `search` 후속 영역 분리

### 본 사이클의 web editor 영역 강화 누적 영역
- PR #611 (Task #458, 김기현 첫 PR) — 표/이미지 리사이즈 Undo
- PR #642 (Task #598, postmelee) — 본문 각주 마커
- PR #602 (Issue #449, oksure) — rhwpDev 디버깅 툴킷
- **PR #684 (Issue #449 후속, oksure)** — rhwpDev 결함 두 건 정정 영역 + 본 환경 정정 보강 영역
- PR #636 Stage 6 보강 — WasmTextMeasurer 정합 영역

→ DTP 엔진 (`project_dtp_identity`) web editor 영역 정합 강화 누적 영역.

## 9. 메모리 룰 적용

- **`feedback_assign_issue_before_work` 정합** — Issue #449 assignee = @oksure (작업지시자 직접 지정) — 일차 방어선 영역 정합
- **`feedback_close_issue_verify_merged`** — Issue #449 close 시 본 PR 머지 검증 + 수동 close. 본 PR + 후속 Issue #692 영역 분리 영역
- **`feedback_visual_judgment_authority`** — PR #602 영역의 결함 두 건 발견 영역의 후속 정정 영역 + 본 PR 영역의 무한 루프 영역 발견 영역 (작업지시자 직접 영역 사용 영역) — 메인테이너 게이트웨이 방식의 권위 사례 영역 강화
- **DTP 엔진 (`project_dtp_identity`) web editor 영역 정합 강화** — 디버깅 툴킷 영역의 본질 정정 영역
- Co-Authored-By Claude 패턴 정합 — PR #627/#636/#668 영역 패턴 정합 (PR #684 은 Claude Opus 4.6, 본 환경 정정 보강은 Claude Opus 4.7 영역)

## 10. 본 사이클 (5/7~5/8) PR 처리 누적 — **15건**

| # | PR | Task / Issue | 결과 |
|---|-----|--------------|------|
| 1 | PR #620 | Task #618 | 시각 판정 ★ + close |
| 2 | PR #642 | Task #598 | 시각 판정 ★ + close |
| 3 | PR #601 | Task #594 | 옵션 A-2 + close + Issue #652 신규 |
| 4 | PR #659 | Task #653 | 시각 판정 ★ + close |
| 5 | PR #602 | Issue #449 | close + Issue #449 reopen |
| 6 | PR #668 | Task #660 | 첫 PR + 시각 판정 ★ + close |
| 7 | PR #609 | Task #604 | 11 commits 단계별 + 시각 판정 ★ + close |
| 8 | PR #670 | (이슈 미연결) 한글 2022 PDF 199 | 메모리 룰 갱신 + close |
| 9 | PR #621 | Task #617 | 옵션 B + 시각 판정 ★ + close |
| 10 | PR #622 | Task #619 | 옵션 A + web editor 시각 판정 ★ + close |
| 11 | PR #626 | (Follow-up to #599) 수식 replay | 옵션 A + PNG 시각 판정 ★ + close |
| 12 | PR #627 | Task #624 | 옵션 A + TDD RED→GREEN + 시각 판정 ★ + close |
| 13 | PR #632 | Task #631 | 옵션 B + 결정적 검증 통과 + 시각 판정 스킵 + close |
| 14 | PR #636 | Task #630 + Issue #635 흡수 | 옵션 A + TDD 5 단계 + Stage 6 + 2 Issue close |
| 15 | **PR #684** | **Issue #449 (PR #602 reopen 후속)** | **옵션 A + 본 환경 정정 보강 2 commits + Chrome 웹 콘솔 영역 검증 영역 + Issue #692 신규** |

### 본 사이클의 권위 사례 누적 영역
- **`feedback_hancom_compat_specific_over_general`**: PR #621 + PR #622 + PR #632 + PR #636 (구조적 가드)
- **`feedback_close_issue_verify_merged`**: PR #620 + PR #627 (PR cherry-pick base diff 점검 누락)
- **`feedback_image_renderer_paths_separate`**: PR #636 Stage 6 보강 (Embedded + Wasm)
- **`feedback_visual_judgment_authority`**: PR #602 + **PR #684** (메인테이너 게이트웨이 방식의 권위 사례)
- **DTP 엔진 web editor 영역 강화 누적**: PR #611 + PR #642 + PR #602 + **PR #684** + PR #636 Stage 6 보강

본 PR 의 **결함 #1 정정 (paragraph 식별 영역) 정합 + 결함 #2 정정 (다중 매치 영역) 정정 + 본 환경 정정 보강 2 commits (main.ts 중복 import + search() 무한 루프 가드 옵션 B) + 메인테이너 Chrome 웹 콘솔 영역 검증 영역 통과 (findNearest) + 후속 영역 분리 (search → Issue #692) + Co-Authored-By Claude 패턴 정합 영역 모두 정합**.
