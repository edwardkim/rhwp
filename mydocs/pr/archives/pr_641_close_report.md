# PR #641 close 영역 안내 보고서

## 1. 처리 결과

| 항목 | 값 |
|------|-----|
| PR | #641 — Task #639 한컴 호환 cover-style 페이지 자동 쪽번호 미표시 |
| 컨트리뷰터 | @planet6897 (Jaeuk Ryu / Jaeook Ryu) — 11번째 사이클 PR |
| 연결 이슈 | #639 (CLOSED) + Issue #637 (CLOSED) |
| 처리 결정 | **CLOSE** (devel 반영 부재) |
| close 시점 | 2026-05-08 02:06 (컨트리뷰터 영역 close) |
| supersede PR | **#711 (Task #705)** — 본질 정정 영역 |
| 처리 일자 | 2026-05-08 |

## 2. close 결정 사유

### 2.1 본 PR 의 본질 영역

본 PR 영역은 cover-style 휴리스틱 룰 영역 (items=1 + Table + tac=false) 영역으로 aift.hwp page 2/3 자동 쪽번호 미표시 영역. 174 샘플 영역 중 영향 페이지 2 (전체의 0.06%) 영역만 영역의 좁은 영역 룰 영역.

### 2.2 메인테이너 검토 영역의 본질 결함 발견

PR #638 (Task #634) close 시 메인테이너 (@edwardkim) 영역이 발견한 본질 결함 영역:

```
section 0 / paragraph 1 / Table[0] / 셀[167] / paragraph[3]
text: "       년        월        일"
ctrl[0] = PageHide(header=true, footer=true, master=true,
                   border=true, fill=true, page_num=true)
```

→ aift.hwp page 2 의 셀 안 영역에 PageHide 영역 정확히 인코딩 영역. 본 환경 영역의 결함 3건:
1. `pagination/engine.rs:516-531` — `page_hides` 수집이 본문 paragraph 만 대상, 셀 안 PageHide 무시
2. `layout.rs:404-407` — `hide_border` + `hide_fill` 가드 부재
3. `main.rs:1897-1920` — dump 셀 안 PageHide 분기 부재

### 2.3 본 PR 의 우회 (workaround) 평가

본 PR 영역의 cover-style 룰 영역은 **본 환경 결함 영역의 우회 (workaround)** 영역으로 분석 영역:
- HWP 표준 영역의 PageHide 인코딩 영역을 무시 영역하고 휴리스틱 룰 영역으로 검출 영역
- `feedback_rule_not_heuristic` 영역의 본질 영역과 영역 — 휴리스틱 룰 영역
- 174 샘플 중 2 페이지 영역만 영역 검출 (PR #711 영역의 6 샘플 영역 영역 영역)

### 2.4 작업지시자 결정

**"이번 PR은 #711 에서 다시 코드 리뷰하겠으며, CLOSE 합니다. devel 반영은 하지 않겠습니다."**

→ 본 PR 영역의 우회 접근 영역은 폐기 영역 + PR #711 영역에서 본질 정정 영역으로 재검토 영역.

## 3. supersede PR #711 (Task #705) 영역의 본질 정정 영역

### 3.1 PR #638 메인테이너 안내 영역 ↔ PR #711 적용 영역

| # | 메인테이너 권고 | PR #711 적용 | 정합 |
|---|---------------|-------------|------|
| 1 | `pagination/engine.rs` 셀 안 PageHide 수집 추가 | `engine.rs:519-544` + `typeset.rs:2120` 두 경로 모두 정정 | ✅ |
| 2 | `layout.rs:404-407` `hide_border` + `hide_fill` 가드 추가 | `layout.rs:411-422` 가드 추가 | ✅ |
| 3 | `main.rs:1897-1920` dump PageHide 분기 추가 | `main.rs:1665-1670` PageHide 분기 추가 | ✅ |

### 3.2 PR #711 의 추가 가치 영역

- **Stage 0 영역의 198 샘플 sweep 영역** — 셀 안 PageHide 13건 / 6 샘플 발견 영역 (PR #640 H2 가설 기각 영역의 잘못된 측정 결과 영역 정량 확인)
- **두 페이지네이션 경로 양분 발견** — PR #641 description 영역의 경로 양분 진단 영역 정합 영역 정합
- **영향 샘플 6 영역** (PR #641 의 174 샘플 중 2 페이지 영역 vs PR #711 의 6 샘플 영역):
  - aift.hwp (page 2/3)
  - 2022 국립국어원 업무계획.hwp (목차 페이지)
  - KTX.hwp (TOC 페이지)
  - kps-ai.hwp
  - tac-img-02.hwp/.hwpx
- **KTX TOC 영역의 부당한 page number footer 영역 정합 정정** (골든 SVG 갱신 영역 정합)

## 4. 본 PR 의 가치 영역 보존

본 PR 영역의 11 commits 영역의 작업 영역 — devel 반영 영역 부재 영역이지만 영역 가치 영역 보존 영역:

### 4.1 분석 영역 (Task #637 5 commits)
- 174 샘플 영역 전수 조사 영역
- 5가지 가설 영역 체계적 검증 영역
- cover-style 룰 영역의 정확한 측정 영역

### 4.2 학습 영역
- H2 가설 (셀 안 PageHide) 기각 영역의 본질 영역 — 본 환경 결함 영역으로 인한 잘못된 측정 결과 영역
- PR #711 Stage 0 영역에서 정량 확인 영역 — 198 샘플 영역의 셀 안 PageHide 13건 정합 영역
- TDD 5 단계 영역의 권위 패턴 영역 — 가설 시행 착오 영역 학습 영역

### 4.3 거버넌스 산출물 영역
- 12 파일 영역의 plans + working + report
- examples/inspect_637.rs 영역의 분석 도구

→ PR #711 의 본질 정정 영역의 기반 영역 정합. 컨트리뷰터 영역의 학습 영역 + 본질 정정 영역 재제출 영역 인정 영역.

## 5. 처리 결과

### 5.1 한글 댓글 등록
PR #641 영역에 한글 댓글 등록 ([#issuecomment-4403285953](https://github.com/edwardkim/rhwp/pull/641#issuecomment-4403285953)) — supersede 안내 + 가치 영역 보존 영역.

### 5.2 devel 반영 부재
본 PR 영역의 cherry-pick 영역 부재 영역. 본 환경 영역에 cover-style 룰 영역 부재 영역 — 정합.

### 5.3 본 PR 상태
- 상태: CLOSED (5/8 02:06 영역, 컨트리뷰터 영역 영역 close 영역)
- merged: false
- devel 영향: 0

## 6. 후속 영역

### 6.1 PR #711 처리 영역
- 본질 정정 영역의 정합 영역 정합 영역 — 본 환경 영역의 cherry-pick 영역 영역 진행 영역 진행 가능 영역
- 옵션 A 권장 — 7 commits 단계별 보존 영역 (TDD 5 단계 + 골든 SVG 갱신)
- 시각 판정 영역 — 6 샘플 영역의 영향 영역 (KTX TOC 골든 SVG 갱신 영역 정합 영역 정합)

### 6.2 Issue #639 영역
- 이미 CLOSED 영역
- PR #711 머지 영역 시 closes #705 영역으로 정합 영역 — Task #705 영역의 본질 정정 영역으로 영역 영역 영역

## 7. 메모리 룰 적용

- **`feedback_rule_not_heuristic`** — 본 PR 영역의 cover-style 휴리스틱 영역 → PR #711 영역의 본질 정정 영역 (HWP 표준 PageHide 인코딩 영역 그대로 사용)
- **`feedback_visual_judgment_authority`** — PR #638 메인테이너 검토 영역에서 본질 결함 발견 영역의 권위 사례 영역 강화 (PR #602 + 본 PR 영역 동일 패턴)
- `feedback_close_issue_verify_merged` — 본 PR close 영역 영역 영역 devel 반영 부재 영역 → PR #711 영역에서 본질 정정 영역으로 재검토 영역
- `feedback_assign_issue_before_work` — Issue #639 + #705 assignee 미지정 영역

## 8. 본 사이클 (5/7~5/8) PR 처리 누적 — **17건** (close 영역 포함 영역)

| # | PR | Task / Issue | 결과 |
|---|-----|--------------|------|
| 1~14 | (이전 PR 영역) | (이전 영역) | merged |
| 15 | PR #684 | Issue #449 후속 | merged + Issue #692 신규 |
| 16 | PR #638 | Task #634 | **close** (본질 결함 발견 → 후속 PR 재제출 권고) |
| 17 | **PR #641** | **Task #639** | **close** (본 PR, cover-style 우회 → PR #711 본질 정정으로 supersede) |

### 본 사이클의 close 패턴 영역 누적 영역
- PR #638 (Task #634) — 본질 결함 발견 영역의 close
- PR #641 (Task #639) — 우회 접근 영역의 close (PR #711 supersede 영역)

→ 본 사이클 영역의 **메인테이너 게이트웨이 방식의 권위 사례 영역 강화 누적 영역** — 결정적 검증만으로 부족, 본질 결함 영역 발견 영역으로 우회 접근 영역의 close 영역 + 본질 정정 영역 재제출 영역 권고 영역.

본 PR 의 **분석 영역 (Task #637) + cover-style 룰 (Task #639) + 컨트리뷰터 학습 영역 + PR #711 본질 정정 영역 재제출 영역 + 메인테이너 게이트웨이 방식의 권위 사례 영역 강화 패턴 모두 정합**.
