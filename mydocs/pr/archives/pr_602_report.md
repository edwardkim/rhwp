# PR #602 처리 결과 보고서

**PR**: [#602 feat: rhwpDev 디버깅 툴킷 — showAllIds / search / findNearest (closes #449)](https://github.com/edwardkim/rhwp/pull/602)
**작성자**: @oksure (Hyunwoo Park, oksure@gmail.com) — 7번째 사이클 PR
**처리 결정**: ✅ **2 commits 보존 cherry-pick 머지 + close** + **Issue #449 reopen** (게이트웨이 검증에서 본질 결함 두 건 식별 → 후속 PR 권유)
**처리일**: 2026-05-07
**devel merge**: `9e97072`

## 1. 처리 결과 요약

| 영역 | 결과 |
|------|------|
| cherry-pick 충돌 | 0 |
| author 보존 | ✅ Hyunwoo Park 2 commits |
| `cargo test --lib` | 1141 passed (회귀 0) |
| Docker WASM 재빌드 | ✅ v0.7.10 (`pkg/rhwp_bg.wasm` 4.4M) |
| rhwp-studio dev 서버 (vite v8.0.10) | ✅ 7700 포트 활성 |
| **메인테이너 게이트웨이 검증** | ⚠️ **본질 결함 두 건 식별** → Issue #449 reopen |
| Copilot 자체 검토 응답 | 100% 정합 (마지막 commit `284abc7`) |

## 2. cherry-pick 영역

### 보존 cherry-pick (2 commits)

| commit | author | 영역 |
|--------|--------|------|
| `398385b` (orig `1cbe866`) | Hyunwoo Park | feat: rhwpDev 디버깅 툴킷 구현 (closes #449) |
| `284abc7` (orig `07139be`) | Hyunwoo Park | address review: API 계약 정정 (pageCount getter, searchText, getPageTextLayout) |

본질 변경: `rhwp-studio/src/core/rhwp-dev.ts` 신규 (+111) + `rhwp-studio/src/main.ts` (+3, `initRhwpDev(wasm)` 호출).

## 3. 메인테이너 판정 게이트웨이 검증

### 권위 자료: `samples/aift.hwp` (Issue #652 권위 자료, M100 v1.0.0 영역)

**본 환경 dump 결과** (`./target/release/rhwp dump samples/aift.hwp -s 0 -p 0`):
```
--- 문단 0.0 --- cc=56, text_len=23, controls=4 [구역나누기]
  텍스트: "  * 사업계획서 제출 시 상기 문구 삭제"
  controls=4: [구역나누기, 단정의, 표 1행×1열, 쪽번호]
    └─ 표 셀[0] paragraph 0: "※ 동 사업(AI 응용제품 신속 상용화 지원사업)은 복..."
```

### web 환경 콘솔 시각 판정 (작업지시자 직접)

```javascript
rhwpDev.search('사업계획')
// → { found: true, wrapped: false, sec: 0, para: 0, charOffset: 4, length: 4 }
//   ✅ 본문 paragraph 0 ("  * 사업계획서 제출 시...") 의 char offset 4 ("사업계획") 가리킴 (정합)

rhwpDev.showAllIds(0)
// → console.table:
//   page=0, secIdx=0, paraIdx=0, charStart=0, x=82.4, y=86.2, text='※ '
//   ⚠️ '※ ' 은 표 셀 내부 paragraph 의 첫 텍스트 — 본문 paragraph 0 의 "  * 사업계획서..." 가 아님
```

### 결함 식별

작업지시자 두 가지 본질 결함 식별:

#### 결함 #1: paragraph 식별 부정확 (본문 vs 표/글상자 내부)

**본질**: `getPageTextLayout` 의 `runs` 배열은 본문 + 표/글상자 내부 paragraph 를 모두 평탄화. `secIdx`/`paraIdx` 키만 사용 시 본문 paragraph 0 과 셀 내부 paragraph 0 가 같은 키 (`0:0`) 로 충돌.

**Issue #449 본질 영역 충돌**: AI activeId 가 본문 paragraph 를 가리킬 때 셀 내부 paragraph 와 구분 안 되면 매핑 실패 진단 자체가 오도. Issue #449 의 "표/글상자 내부 등 DOM 깊은 곳" 영역과 정확히 충돌.

**개선 영역 제안**:
- `getPageTextLayout` 의 `runs` 에 `cellContext` 또는 `isCellInner: boolean` 필드 추가 (본 환경 영역)
- `showAllIds` 에서 본문 + 셀 내부 paragraph 명확 구분 표시
- 또는 본문 우선 + 셀 내부 옵션 (`showAllIds(page, { includeCellInner: true })`)

#### 결함 #2: 다중 매치 미처리

**본질**: `wasm.searchText` 는 첫 매치만 반환. PR 코드도 `wasm.searchText(text, 0, 0, 0, true, false)` 로 단일 매치만 사용.

**Issue #449 본질 영역 충돌**: AI 환각/매핑 오류 시 키워드 일치 후보가 여럿일 때 메인테이너/외부 개발자가 모든 후보를 보고 정확한 ID 식별 (또는 "AI 가 잘못된 후보를 골랐는지" 판정) 필요. Issue #449 의 "AI 환각/매핑 오류 시 Fallback 용도" 영역과 직접 연관.

**개선 영역 제안**:
- `search` 가 모든 매치 위치 array 반환 (searchText loop 호출 + wrapped 종료)
- 또는 본 환경 wasm-bridge 에 `searchAllText(query, caseSensitive)` API 추가 (`document_core/queries/search_query.rs:73` 의 `search_all` 노출)
- 반환 형식: `Array<{ sec, para, charOffset, length, cellContext? }>`

## 4. 처리 결정

### PR #602 close 유지

본 PR 의 회귀 0 영역 + 정합 영역 (search 단일 매치 + findNearest distance 추천) 은 cherry-pick 머지 유지 (devel `9e97072`).

이유:
- 본 PR 의 search/findNearest 단일 매치 영역은 정합 (Issue #449 본질의 ~70%)
- showAllIds 의 본문 vs 셀 내부 구분 결함은 후속 영역 분리 가능
- `feedback_no_pr_accumulation` 정합 — closed PR 잔존 영역 cherry-pick 은 새 PR 로 등록

### Issue #449 reopen

본 두 결함 영역이 Issue #449 의 본질 영역 (AI activeId 매핑 실패 + DOM 깊은 곳 + AI 환각 Fallback) 을 정확히 가리키므로 본 Issue 에서 후속 처리.

### 컨트리뷰터에게 후속 PR 권유

PR #602 댓글 [#issuecomment-4395273056](https://github.com/edwardkim/rhwp/pull/602#issuecomment-4395273056) 에 메인테이너 판정 게이트웨이 방식 + 결함 두 건 + 후속 PR 자체 게이트웨이 검증 영역 안내.

**자체 게이트웨이 검증 권유**:
1. `samples/aift.hwp` 로드
2. `rhwpDev.search('사업계획')` 결과의 sec/para 가 본문 paragraph 0 인지 확인
3. `rhwpDev.showAllIds(0)` 결과가 본문 + 셀 내부 paragraph 명확 구분 표시
4. 다중 매치 케이스 (예: "사업" 키워드) 에서 모든 후보 위치 array 반환

`feedback_per_task_pr_branch` 정합 — 후속 PR 은 별도 fork branch (예: `oksure:contrib/rhwpdev-toolkit-v2`) 권장.

## 5. 회귀 차단 검증

### cargo test --lib (1141 passed)
회귀 0 입증.

### 코드 영역 분석
- 변경 영역: `rhwp-studio/src/core/rhwp-dev.ts` 신규 + `rhwp-studio/src/main.ts` (+3 라인)
- 무영향 영역: 코어 IR (`src/`), 렌더, 직렬화, WASM API
- 디버깅 도구 영역만 — 코어 무영향

## 6. Copilot 자체 검토 응답 정합 (100%)

마지막 commit `284abc7` "address review: API 계약 정정" 에서 Copilot 7 코멘트 모두 반영:

| Copilot 코멘트 | 응답 |
|---------------|------|
| `wasm.getPageCount()` 부재 (showAllIds) | ✅ `wasm.pageCount` getter |
| `data-page` 속성 부재 (canvas 렌더러) | ✅ DOM 오버레이 제거 → console.table |
| `getPageTextLayout` 필드 (`section`/`para` → `secIdx`/`paraIdx`/`charStart`) | ✅ 정정 |
| `searchText` 반환 (`null` 아님 → `{ found: false }`) | ✅ `result.found` 체크 |
| `searchText` 결과 shape (sec/para/charOffset/length) | ✅ 그대로 사용 |
| `findNearest` getPageCount 결함 | ✅ pageCount getter |
| `findNearest` paraIdx 필드 | ✅ 정정 |

자체 검토 응답 패턴 정합 — `user_work_style` (자체 검토 응답 직접 결정 영역) 정합.

## 7. 처리 절차

1. ✅ PR 정보 확인 (mergeable=MERGEABLE, BEHIND 148 commits)
2. ✅ 검토 보고서 작성 + 작업지시자 승인
3. ✅ cherry-pick 2 commits 보존 (충돌 0)
4. ✅ 결정적 검증 (cargo test 1141 passed)
5. ✅ Docker WASM 재빌드 (v0.6.0 → v0.7.10)
6. ✅ rhwp-studio dev 서버 (7700 포트) 재시작
7. ✅ 작업지시자 web 환경 시각 판정 (search/showAllIds 콘솔 출력 확인)
8. ⚠️ **게이트웨이 검증에서 결함 두 건 식별** (작업지시자 지적)
9. ✅ devel merge (`9e97072`) + push
10. ✅ PR #602 close (한글 댓글 + 메인테이너 판정 게이트웨이 방식 + 결함 두 건 + 후속 PR 권유)
11. ✅ **Issue #449 reopen** (본질 영역 미커버 → 후속 PR 영역 분리)

## 8. 메모리 정합 영역

- `feedback_visual_judgment_authority` — 결정적 검증 + 작업지시자 권위 자료 (samples/aift.hwp) 시각 판정에서 결함 식별
- `feedback_pr_comment_tone` — close 댓글 차분/사실 중심 + 컨트리뷰터의 정합 영역 인정 + 후속 권유 톤
- `feedback_assign_issue_before_work` — Issue #449 메인테이너 직접 assignee 지정 (@oksure)
- `feedback_no_pr_accumulation` — closed PR 잔존 영역 cherry-pick 은 새 PR 로 등록 (후속 영역 분리)
- `feedback_per_task_pr_branch` — 후속 PR 별도 fork branch 권장
- `user_work_style` — 외부 PR 옵션 분류 + 자체 검토 응답 정합 + **게이트웨이 검증에서 본질 결함 식별 본질**
- `user_role_identity` — 메인테이너 판정 게이트웨이 방식 (권위 자료 + 시각 판정) 의 권위 사례

## 9. 본 환경 산출물

- `mydocs/pr/archives/pr_602_review.md` — 검토 보고서
- `mydocs/pr/archives/pr_602_report.md` — 본 처리 보고서
- `pkg/rhwp_bg.wasm` — Docker WASM v0.7.10 재빌드 영역

## 10. 게이트웨이 검증 본질의 권위 케이스

본 처리는 **메인테이너 판정 게이트웨이 방식**의 권위 사례:

1. **결정적 검증** (cargo test/clippy/build) 만으로는 본질 결함 식별 부족 — 회귀 0 입증 영역
2. **권위 자료 시각 판정** (samples/aift.hwp 의 paragraph 0 본문 + 셀 내부 paragraph 영역) 에서 본질 결함 식별
3. **메인테이너 권위 영역** (`feedback_visual_judgment_authority`) 의 결정적 게이트웨이 가치 — 외부 PR 흡수 후에도 게이트웨이 검증으로 본질 영역 점검

본 권위 케이스를 후속 컨트리뷰터에게 안내하여 자체 게이트웨이 검증 패턴 정합 권유.
