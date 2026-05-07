# PR #602 검토 보고서

**PR**: [#602 feat: rhwpDev 디버깅 툴킷 — showAllIds / search / findNearest (closes #449)](https://github.com/edwardkim/rhwp/pull/602)
**작성자**: @oksure (Hyunwoo Park, oksure@gmail.com) — 7번째 사이클 PR (PR #581/#582/#583/#600/#601/#659 + 본 PR)
**상태**: OPEN, **mergeable=MERGEABLE**, **mergeStateStatus=BEHIND** (PR base `cebe047` = 4/29 시점, 148 commits 뒤)
**관련**: closes #449 (메인테이너 직접 등록 + 외부 컨트리뷰터 assignee 지정 + 선구현 패턴)
**처리 결정**: ⏳ **검토 중**
**검토 시작일**: 2026-05-07

## 1. 검토 핵심 질문

1. **본질 영역 정합성** — Issue #449 의 3가지 요청 영역 (showAllIds 시각 오버레이 / 스마트 에러 로깅 / search 헬퍼) 을 본 PR 이 정확히 커버하는가?
2. **Copilot 리뷰 응답 정합성** — Copilot 7 코멘트의 본질 결함 (API 계약 불일치 + 줌 스케일링 + DOM 선택자 결함) 을 마지막 commit `07139be` 에서 모두 반영했는가?
3. **본 환경 API 정합성** — `wasm.pageCount` (getter), `wasm.searchText`, `wasm.doc.getPageTextLayout` 영역이 본 환경 wasm-bridge 와 정합한가?
4. **시각 오버레이 → console.table 전환 정합성** — Copilot #2 응답으로 DOM 오버레이 → console.table 으로 변경 — Issue #449 의 "시각적 ID 오버레이" 요청과 본 PR 의 "console.table 출력" 영역 차이 식별
5. **PR base skew (148 commits 뒤)** — 본 환경 cherry-pick 충돌 0?

## 2. PR 정보

| 항목 | 값 | 평가 |
|------|-----|------|
| 제목 | feat: rhwpDev 디버깅 툴킷 — showAllIds / search / findNearest (closes #449) | 정합 (Issue #449 본질 그대로) |
| author | @oksure (Hyunwoo Park) — Issue #449 의 명시 assignee + 7번째 사이클 PR | ✅ |
| changedFiles | **2** / +114 / -0 | 작은 규모 (디버깅 도구 영역, 신규 파일) |
| 본질 변경 | `rhwp-studio/src/core/rhwp-dev.ts` 신규 (+111) + `rhwp-studio/src/main.ts` (+3) | rhwp-studio (TypeScript) 영역만 |
| **mergeable** | MERGEABLE (UI), **mergeStateStatus=BEHIND** (148 commits 뒤) | 본 환경 cherry-pick 충돌 0 확인 필요 |
| Issue | closes #449 (메인테이너 직접 등록, **assignee @oksure**, M100 v1.0.0) | ✅ `feedback_assign_issue_before_work` 정합 |
| commits | **2** (`1cbe866` 초안 + `07139be` Copilot 리뷰 응답) | 단계 commit 패턴 정합 |

## 3. PR 의 2 commits 분석

### Commit 1: `1cbe866` "feat: rhwpDev 디버깅 툴킷 구현 (closes #449)"

**핵심 추가**:
- `initRhwpDev(wasm: WasmBridge)` — `window.rhwpDev` 전역 객체 등록
- `showAllIds(pageNum?)` — DOM 오버레이 (data-page 선택자 + getPageTextLayout)
- `search(text)` — `wasm.searchText` 호출 + `result.section` / `result.paragraph` 사용
- `findNearest(targetId, pageNum?)` — `wasm.getPageCount()` + `run.para` 사용
- `help()` — 사용법 console 출력
- `main.ts` `initRhwpDev(wasm)` 호출 추가 (DEV 모드 외 전체 모드)

### Commit 2: `07139be` "address review: API 계약 정정 (pageCount getter, searchText, getPageTextLayout)"

**Copilot 7 코멘트 응답** (모두 본 환경 wasm-bridge API 와 정합 정정):
- ✅ `wasm.getPageCount()` → `wasm.pageCount` (getter, 본 환경 정합)
- ✅ `searchText` 반환 형식: `{ found: boolean, sec, para, charOffset, length }` (본 환경 정합)
- ✅ `getPageTextLayout` 필드: `secIdx`/`paraIdx`/`charStart` (`section`/`para` 가 아님)
- ✅ **showAllIds 전략 변경**: DOM 오버레이 → `console.table` (canvas 렌더러에 `data-page` 속성 부재 + 줌 스케일링 회피)
- ✅ `findNearest`: `paraIdx` 필드 사용

## 4. 영역 평가

### 4.1 Issue #449 의 요청 영역 정합성

**Issue #449 요청 영역 (메인테이너 직접 등록, M100 v1.0.0)**:

| 요청 | 본 PR 의 처리 | 평가 |
|------|--------------|------|
| 1. **`rhwpDev.showAllIds()` 시각적 ID 오버레이** | ⚠️ DOM 오버레이 → `console.table` 변경 | Copilot #2 응답 (canvas 렌더러 data-page 부재 + 줌 스케일링 결함) 으로 시각 오버레이 → console.table 으로 전환. 콘솔 출력은 시각 오버레이 본질과 차이 — **후속 영역** (정식 구현은 canvas 위 overlay div 또는 Canvas API 직접 그리기 필요) |
| 2. **스마트 에러 로깅 (가장 가까운 ID 추천)** | ✅ `findNearest(targetId, pageNum?)` 영역 정합 | distance 기반 가장 가까운 paraIdx 반환 |
| 3. **`rhwpDev.search(text)` 헬퍼** | ✅ 텍스트 → `{ sec, para, charOffset, length }` 정확 매핑 | 본 환경 `searchText` 결과 직접 반환 |

**커버리지**: ~85% (showAllIds 정식 시각 오버레이 미커버, console.table 응답 — 후속 영역 분리 권장)

### 4.2 Copilot 리뷰 응답 정합성

Copilot 7 코멘트 (4 결함 영역):

| Copilot 코멘트 | PR 응답 |
|---------------|---------|
| #1 `wasm.getPageCount()` 부재 (showAllIds) | ✅ `wasm.pageCount` getter 사용 |
| #2 `data-page` 속성 부재 (canvas 렌더러) | ✅ DOM 오버레이 제거 → console.table |
| #3 `getPageTextLayout` 필드 (secIdx/paraIdx 가 아닌 section/para 사용) | ✅ `secIdx`/`paraIdx`/`charStart` 정정 |
| #4 `searchText` 반환 (`null` 가 아닌 `{ found: false }`) | ✅ `result.found` 체크로 정정 |
| #5 `searchText` 결과 shape (sec/para/charOffset/length) | ✅ 그대로 사용 |
| #6 `findNearest` getPageCount 결함 | ✅ `pageCount` getter 사용 |
| #7 `findNearest` `paraIdx` 필드 | ✅ 정정 |
| 줌 스케일링 결함 | ✅ DOM 오버레이 제거로 회피 |

**자체 검토 응답 정합도**: 100% — 외부 컨트리뷰터의 자체 검토 응답 패턴 정합 (`user_work_style` 정합).

### 4.3 본 환경 API 정합성

| 본 PR 사용 API | 본 환경 위치 | 정합 |
|---------------|------------|------|
| `wasm.pageCount` (getter) | `wasm-bridge.ts:164` | ✅ |
| `wasm.searchText(query, fromSec, fromPara, fromChar, forward, caseSensitive)` | `wasm-bridge.ts:1323` | ✅ |
| `wasm.doc.getPageTextLayout(page)` | `src/wasm_api.rs:420` | ✅ |
| `result.found` / `result.sec` / `result.para` / `result.charOffset` / `result.length` | `searchText` JSON 응답 | ✅ |
| `run.secIdx` / `run.paraIdx` / `run.charStart` / `run.x` / `run.y` / `run.text` | `getPageTextLayout` JSON | 검증 필요 (실행 시) |

**경계 영역**: PR 코드는 `(wasm as any).doc.getPageTextLayout(p)` 로 wasm-bridge 우회 직접 접근 — 디버깅 도구 영역의 허용 패턴이나, wasm-bridge 메서드 추가 권장 가능 (후속 영역).

### 4.4 회귀 위험 영역 분석

**범위**: `rhwp-studio/src/core/rhwp-dev.ts` 신규 + `rhwp-studio/src/main.ts` (+3 라인).
**무영향 영역**: 코어 IR (`src/`), 렌더 (`src/renderer/`), 직렬화 (`src/serializer/`), 본 환경 cargo build/test, WASM API.
**영향 영역**: rhwp-studio 의 main.ts 시작 시점에 `initRhwpDev(wasm)` 호출 추가 — 모든 모드에서 활성, `window.rhwpDev` 전역 노출.

**회귀 위험**: 매우 낮음 (TypeScript 디버깅 도구 영역, 코어 무영향).

### 4.5 PR base skew

**fork base**: `cebe047` (4/29 v0.7.7 직후)
**devel ahead**: 148 commits (4/29 ~ 5/7 본 사이클 처리분 누적 — v0.7.8/0.7.9/0.7.10 + 본 사이클 PR 처리)
**충돌 위험**: rhwp-studio 영역만 변경 — main.ts 의 `initRhwpDev(wasm)` 추가 위치 (`if (import.meta.env.DEV)` 다음) 가 본 환경과 정합. PR #642 (5/7 처리, rhwp-studio TypeScript 4 파일 변경) 와 main.ts 영역 충돌 가능성 점검 — 본 환경 main.ts 의 30~36 라인 영역과 PR 의 추가 위치가 같으므로 **cherry-pick 충돌 0 예상**, 검증 단계에서 확정.

## 5. 옵션 평가

### 옵션 A: 2 commits 보존 cherry-pick (권장)

**범위**: `1cbe866` + `07139be` 모두 cherry-pick (squash 또는 보존).

**장점**:
- Issue #449 본질 영역 ~85% 커버 (showAllIds 의 시각 오버레이는 후속 영역)
- Copilot 7 코멘트 자체 검토 응답 정합 100%
- 본 환경 API 정합 (pageCount getter, searchText, getPageTextLayout)
- 회귀 위험 매우 낮음 (TypeScript 디버깅 도구, 코어 무영향)
- 활발한 컨트리뷰터 7번째 PR + Issue assignee 지정 영역 정합

**단점**: showAllIds 시각 오버레이 → console.table 으로 본질 변경 — 후속 PR 권유 또는 별도 사이클로 정식 시각 오버레이 구현

**처리**:
- 2 commits 보존 cherry-pick (author @oksure 보존)
- Issue #449 의 미커버 영역 (정식 시각 오버레이) 후속 PR 권유 댓글 등록
- Issue #449 assignee 영역 `feedback_assign_issue_before_work` 정합 인정 댓글

### 옵션 B: squash cherry-pick

2 commits 을 1 commit 으로 squash. Copilot 리뷰 응답 commit 분리 의도 손실 — **비권장**.

### 옵션 C: close + 후속 권유

본 PR 이 Issue #449 본질 ~85% 커버하므로 close 비합리적. **비채택**.

## 6. 처리 권장

**옵션 A (2 commits 보존 cherry-pick)** 권장.

**이유**:
1. Issue #449 본질 ~85% 커버 (search / findNearest 100% 정합)
2. Copilot 7 코멘트 자체 검토 응답 100% 정합
3. 본 환경 API 와 정합 (pageCount getter, searchText, getPageTextLayout)
4. 회귀 위험 매우 낮음 (rhwp-studio TypeScript 디버깅 도구, 코어 무영향)
5. Issue #449 assignee 지정 영역 정합 (메인테이너 → 컨트리뷰터 직접 위임)
6. 활발한 컨트리뷰터 (7번째 PR) 사이클 흡수

**후속 권유**: 정식 시각 오버레이 구현 (canvas 위 overlay div 또는 Canvas API 직접 그리기) — 별도 PR 또는 후속 task.

## 7. 본 환경 검증 계획 (구현계획서 분리 불필요 영역)

PR 이 rhwp-studio 디버깅 도구 영역만 변경 + 회귀 위험 매우 낮음 → 구현계획서 단계 생략, 검증 단계만 진행:

1. cherry-pick 2 commits → 본 환경 충돌 0 확인
2. cargo test --lib 1141 passed 유지 (rhwp-studio 무영향 검증)
3. **`rhwp-studio npm run build`** TypeScript 타입 체크 통과 + dist 빌드
4. **결정적 검증**: rhwp-studio dev 서버 실행 → 콘솔 `rhwpDev.help()` / `rhwpDev.showAllIds(0)` / `rhwpDev.search("text")` / `rhwpDev.findNearest(11, 0)` 동작 확인
5. WASM 빌드 영향 0 (rhwp-studio 영역, WASM 변경 무관)
6. 작업지시자 시각 판정 (web 환경 콘솔 출력 검증)
7. devel merge + push
8. PR #602 close + Issue #449 close (assignee 지정 + 본인 처리 영역)

**시각 판정 영역**: web 환경 콘솔 출력 (`console.table` showAllIds + search/findNearest console.log).

## 8. 메모리 정합 영역

- `feedback_first_pr_courtesy` — @oksure 활발한 컨트리뷰터 (7번째), 첫 PR 표현 부적용
- `feedback_visual_judgment_authority` — 결정적 검증 (cargo test/npm build) + 작업지시자 web 환경 시각 판정
- `feedback_pr_comment_tone` — close 댓글 차분/사실 중심 + 컨트리뷰터의 정합 영역 인정 + 후속 권유 톤
- `feedback_assign_issue_before_work` — Issue #449 메인테이너 직접 assignee 지정 (@oksure) → 컨트리뷰터의 일차 방어선 정합
- `user_work_style` — 외부 PR 옵션 분류 (옵션 A 보존 cherry-pick) + 자체 검토 응답 정합
- `project_external_contributors` — @oksure 누적 사이클 (7번째 PR)
