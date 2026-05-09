---
PR: #711
제목: Task #705 — 한컴 호환 셀 안 PageHide 컨트롤 본질 정정 (closes #705)
컨트리뷰터: @planet6897 (Jaeuk Ryu) — 30+ 사이클 핵심 컨트리뷰터
base / head: devel / pr-task705
mergeStateStatus: BEHIND
mergeable: MERGEABLE — 충돌 0건
CI: **FAILURE** ⚠️ (Build & Test — `test_634_gukrip_page3_shows_page_number` FAILED)
변경 규모: +1533 / -7, 17 files
검토일: 2026-05-09
---

# PR #711 검토

## 1. 메타

| 항목 | 값 |
|------|-----|
| PR 번호 | #711 |
| 제목 | Task #705 — 한컴 호환 셀 안 PageHide 컨트롤 본질 정정 |
| 컨트리뷰터 | @planet6897 (Jaeuk Ryu) — 30+ 사이클 핵심 컨트리뷰터 |
| base / head | devel / pr-task705 |
| mergeStateStatus | BEHIND, mergeable: MERGEABLE — `git merge-tree` 충돌 0건 |
| CI | **FAILURE** — `test_634_gukrip_page3_shows_page_number` (lib 테스트 1171 + 1 failed) |
| 변경 규모 | +1533 / -7, 17 files (소스 4 + 통합 테스트 113 LOC + examples 2 + 보고서 11) |
| 커밋 수 | 7 (Stage 0 측정 + Stage 1 RED + Stage 2/3/4 GREEN + Stage 5 sweep + 후속 골든 SVG 갱신) |
| closes | #705 |
| 선행 PR | PR #638 (Task #634, close, src 무변경) + PR #640/#641 (Task #637/#639 close, 휴리스틱 폐기) |

## 2. PR 본질 — PR #638/#641 close 영역의 본질 정정

작업지시자 (메인테이너) 가 PR #638 close 시 발견한 본 환경 결함 3건의 정확한 정정. PR #641 close 시 작업지시자 안내:

> "본 환경 결함 3 영역 정확 정정 (PR #638 메인테이너 댓글 영역의 3 권고 영역 정확 정합)"

→ PR #711 이 PR #641 의 **공식 supersede** (작업지시자 권고 영역).

## 3. 정정 영역 (3 결함)

### 3.1 결함 #1 — `page_hides` 수집이 셀 안 PageHide 무시

**두 페이지네이션 경로 양분 정정**:

`src/renderer/pagination/engine.rs:519-544`:
```rust
Control::Table(table) => {
    Self::collect_pagehide_in_table(table, pi, &mut page_hides);
}
```

`src/renderer/typeset.rs:2213` (main path):
```rust
Control::Table(table) => {
    Self::collect_pagehide_in_table(table, pi, &mut page_hides);
}
```

`collect_pagehide_in_table` 재귀 함수 — 표 셀 안 paragraph 의 PageHide 수집 (depth 2+ 중첩 표 0건 확인 영역).

→ **`feedback_image_renderer_paths_separate` 권위 룰 정합** (두 경로 동기화 정정).

### 3.2 결함 #2 — `build_page_background()` / `build_page_borders()` `hide_fill`/`hide_border` 가드 부재

`src/renderer/layout.rs:411-422`:
```rust
let hide_fill = page_content.page_hide.as_ref()
    .map(|ph| ph.hide_fill).unwrap_or(false);
if !hide_fill {
    self.build_page_background(...);
}

let hide_border = page_content.page_hide.as_ref()
    .map(|ph| ph.hide_border).unwrap_or(false);
if !hide_border {
    self.build_page_borders(...);
}
```

기존: `hide_master`, `hide_header`, `hide_footer`, `hide_page_num` 가드만 존재. PR #711이 6 필드 모두 가드 정합화.

### 3.3 결함 #3 — `main.rs:dump` 셀 안 PageHide 분기 부재

dump 도구 영역의 셀 안 controls 매칭에 PageHide 분기 추가 — 디버깅 영역 입증.

## 4. CI FAILURE 분석 ⚠️

### 4.1 실패 테스트
`renderer::layout::integration_tests::tests::test_634_gukrip_page3_shows_page_number`

```
assertion `left == right` failed: 국립국어원 페이지 3 은 쪽번호 표시되어야 함.
  left: 0
  right: 3
```

→ PR #711 적용 후 페이지 3 쪽번호 미표시 (0 회) 인데 가드는 3 회 표시 기대.

### 4.2 본질 — test_634 가드 자체가 잘못된 영역

`src/renderer/layout/integration_tests.rs:1485-1491` 마지막 주석:
> "페이지 2, 3 (사업계획서 표지/요약문) 미표시 메커니즘은 별도 issue. 한컴: PageHide 없는데도 미표시. rhwp: 표시 (현재). 메커니즘 미확인 — 후속 분석 필요."

→ test_634 가드 영역 의 의도 자체 가 **rhwp 의 한컴 부정합 영역 보존** 영역. 메커니즘 미확인 시점 영역의 임시 가드.

### 4.3 한컴 PDF 권위본 직접 측정 — PR #711 정답지 정합 입증

```
$ pdftotext -f 3 -l 3 'pdf/2022년 국립국어원 업무계획-2022.pdf' -
... 마지막 줄: "- 1 -"
```

→ 한컴 PDF page 3 마지막 줄: **`- 1 -`** (페이지 번호 1, NewNumber 영역으로 1번부터 다시 시작).

해석:
- 한컴 PDF page 3 = 본 환경 page 3 = 국립국어원 업무계획.hwp 의 3번째 페이지
- 한컴 PDF에서 표시 = `- 1 -` (페이지 번호 1)
- 본 환경 BEFORE: rhwp 가 페이지 번호 (count=3 영역) 표시 — test_634 가드 통과
- 본 환경 AFTER (PR #711 적용): 셀 안 PageHide 영역으로 hide_page_num 적용 → count=0 (미표시) — test_634 가드 실패

여기서 핵심 점:
- 한컴 PDF page 3 마지막 줄이 `- 1 -` 인데도 PR #711 적용 시 0 — **한컴 권위와 부정합** 가능성
- 또는 NewNumber 가 별도 메커니즘으로 동작해야 하는데 PR #711이 NewNumber 까지 무시 가능성

→ **메인테이너 정밀 점검 필요** — PR #706/form-002 패턴과 유사하지만 PDF 권위 영역 의 직접 측정 결과가 한컴 PDF 표시 ↔ PR #711 미표시 부정합. **단순 골든 갱신으로 처리 불가**.

## 5. PR #634 회귀 가드 영역 의 본질 분석

PR #634 (`ff47aefa`) 시점 에 본 환경 devel 에 `test_634_*` 가드 8건 추가됨. PR #711 본문 정합 영역 (한컴 정답지):

| 테스트 | rhwp BEFORE (devel) | 한컴 권위 | PR #711 적용 후 |
|--------|---------------------|-----------|-----------------|
| `test_634_gukrip_page1_pagehide_no_page_number` | count=0 (미표시, body PageHide) | 미표시 | count=0 (변화 없음, body PageHide 영역) |
| `test_634_gukrip_page3_shows_page_number` | count=3 (표시) | **`- 1 -` 표시** | count=0 (미표시) ⚠️ |

→ test_634_gukrip_page3 영역 의 가드 (count=3) 자체 가 **한컴 권위 영역과 정합도 부정합**. 한컴 PDF 마지막 줄 `- 1 -` 영역 1 회 표시 영역 인데, 본 환경 BEFORE 는 3 회 표시 영역 (header/footer/body 영역 전체).

PR #711 본문 명시:
> "aift.hwp page 3, 2022 국립국어원, KTX, kps-ai 목차 페이지: hide_page_num 적용 → page_num 미표시"

→ 작업지시자 한컴 편집기 권위 ([감추기] 다이얼로그 직접 확인) 영역 — 셀 안 PageHide 영역 의 hide_page_num 영역 적용 영역. **한컴 의도는 page_num 미표시 + 본문 표 영역의 NewNumber 영역 별도 영역**.

## 6. 처리 옵션

### 옵션 A — close + 작업지시자 시각 판정 (메인테이너 직접 권위)

PR #706/form-002 패턴 정합 — CI FAILURE 영역 의 시각 판정 권위로 정답 확정 후 처리:
1. 메인테이너 시각 판정 — 한컴 편집기 직접 영역 의 [감추기] 다이얼로그 영역 + 페이지 3 쪽번호 표시 여부 확인
2. PR #711 정답 확정 시: cherry-pick + test_634_gukrip_page3 가드 갱신 (count=3 → count=0) + no-ff merge
3. PR #711 부정확 확정 시: close + 정정 요청

### 옵션 B — 쪽번호 NewNumber 메커니즘 직접 분석

본 환경 page 3 마지막 줄 `- 1 -` 영역 의 NewNumber 별도 메커니즘 동작 점검:
- 한컴: `[감추기 page_num=true]` + `[NewNumber 1]` 동시 영역 → NewNumber 가 우선 적용
- rhwp PR #711: `hide_page_num=true` 영역 만 적용 → NewNumber 영역 무시

→ 위험 큼. PR #711 본질 정정 자체 부정확 가능성.

## 7. 충돌 / mergeable

- `mergeStateStatus: BEHIND` (PR base = `215abb52`, devel HEAD = `60aeaa8d`, 31 commits 뒤처짐)
- `git merge-tree --write-tree` 실측: **CONFLICT 0건** (auto-merging main.rs + typeset.rs)

## 8. 회귀 가드 영역 의 충돌

PR #711 본문 의 회귀 가드 (`test_705_*`) 6건 + 본 환경 devel 의 `test_634_gukrip_page3` 가드 영역 의 **충돌**:

| 가드 | 의도 |
|------|------|
| `test_634_gukrip_page3_shows_page_number` | 페이지 3 count=3 (rhwp 현재 행위 보존, PR #634 영역) |
| `test_705_kor2022_cell_pagehide_collected` | 셀 안 PageHide 매핑 ≥ 2건 (PR #711 영역) |

→ 두 가드 영역 양립 영역. `test_634_*` 영역 의 행위 영역 의 변경 영역 (count=3 → count=0) 이 한컴 권위 정합 영역으로 시각 판정 영역 시 정답 확정 영역.

## 9. 메인테이너 시각 판정 권고

PR #706/form-002 패턴 정합 — **메인테이너 직접 시각 판정 권위**:

### 9.1 시각 판정 자료
- `samples/2022년 국립국어원 업무계획.hwp` page 3 (rhwp BEFORE / PR #711 AFTER)
- `pdf/2022년 국립국어원 업무계획-2022.pdf` page 3 (한컴 권위, 마지막 줄 `- 1 -`)

### 9.2 판정 영역
- 한컴 편집기 [감추기] 다이얼로그 영역 의 page 3 셀 안 PageHide 6 필드 확인
- 한컴 편집기 직접 영역 의 page 3 쪽번호 표시 여부 확인 (page header/footer/body 영역)
- PR #711 적용 후 영역 의 표시 여부 (count=0 영역) 정합성 영역 직접 영역

### 9.3 PR #706 패턴 정합 영역 처리

PR #706 form-002 영역 처리 패턴 정합:
1. 작업지시자 시각 판정 ★ 통과 시 → cherry-pick + test_634_gukrip_page3 가드 갱신 (count=3 → count=0) + no-ff merge
2. 작업지시자 부정확 확정 시 → close + 정정 요청

## 10. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @planet6897 30+ 사이클 핵심 컨트리뷰터 |
| `feedback_pr_supersede_chain` | PR #638/#641 close → PR #711 supersede (작업지시자 권고 영역) |
| `feedback_visual_judgment_authority` | CI FAILURE + test_634 가드 영역 vs 작업지시자 시각 판정 (메인테이너 권위) — **PR #706 form-002 패턴 정합** |
| `feedback_visual_regression_grows` | test_634 가드 영역 의 본질 부정확 영역 — golden 갱신 신규 사례 |
| `feedback_image_renderer_paths_separate` | 두 페이지네이션 경로 (engine.rs + typeset.rs) 동기 정정 |
| `feedback_rule_not_heuristic` | PR #641 cover-style 휴리스틱 폐기 영역 의 본질 정정 |
| `pdf_not_authoritative` | PR 본문 명시 영역 의 IR 기반 검증 (page.page_hide + dump + SVG bg_rect) |
| `feedback_close_issue_verify_merged` | Issue #705 (현재 OPEN) 영역 close 영역 의 PR 머지 정합 |

## 11. 처리 순서 권고

### 옵션 A 추천 — 시각 판정 → 머지 + 가드 갱신

1. **메인테이너 시각 판정 영역** — 한컴 편집기 [감추기] 다이얼로그 영역 + page 3 쪽번호 표시 영역 직접 확인
2. (시각 판정 ★ 통과 시) `local/devel` 에서 7 commits cherry-pick (옵션 A)
3. `test_634_gukrip_page3_shows_page_number` 가드 갱신 (count=3 → count=0) — 메인테이너 추가 commit
4. 자기 검증 (cargo test/build/clippy + 신규 test_705 6건 + test_634 가드 갱신 후 통과)
5. (선택) WASM 빌드 + 시각 판정 보강
6. no-ff merge + push + archives 이동 + 5/9 orders 갱신
7. PR #711 close (closes #705 자동 정합)

### 옵션 B — close + 정정 요청

작업지시자 시각 판정 영역 의 PR #711 부정확 확정 시 — close + 컨트리뷰터에게 정정 후 재제출 요청.

---

작성: 2026-05-09
