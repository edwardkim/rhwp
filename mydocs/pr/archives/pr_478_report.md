# PR #478 처리 보고서

**제목**: Task #476 + #479 + #480 + #483 + #488 + #489 + #490 + #492 + #495 합본
**작성자**: planet6897 (Jaeuk Ryu)
**처리 결과**: cherry-pick 머지 (**7 Task / 10 commits** — 4차/5차 후속 머지 포함)

## 처리 본질

본 PR 은 작성자 devel 분기에 누적된 9 Task 합본 (97 commits). 본 사이클의 정정 영역 (특히 Task #501 의 cell.padding 비정상 IR 정정) 과 충돌 위험이 높은 페이지 레이아웃 영향 Task 4건 (#476, #479, #480, #492) 은 제외하고, **본질이 회귀 영역과 무관한 5 Task** 만 분리 cherry-pick.

작업지시자 옵션 결정: A (페이지 레이아웃 무관 안전 후보 #488 + #490) → B (#489 추가) → 시각 판정 통과 → C (#483 추가) → D (#495 추가, 부분 정정)

## 흡수된 Task (7 건 / 10 commits)

| Task | Commits | 영역 | 머지 단계 | 결과 |
|------|---------|------|----------|------|
| **#488** | 0d4ca48 + 78a0b84 + c6d2b24 (3) | 수식 토크나이저 폰트 스타일 키워드 prefix 분리 + svg/canvas 렌더러 italic 파라미터 honor | 1차 | 단위 테스트 14건 추가 |
| **#490** | 85f6779 (1) | 빈 텍스트 + TAC 수식 셀 alignment 적용 (`paragraph_layout.rs`) | 1차 | exam_science p1 셀 7/11 28/36 수식 중앙 정렬 |
| **#483** | 3e04734 + 97e64b5 (2) | 각주 multi-paragraph 처리 line_spacing 정합 + 마지막 paragraph trailing line_spacing | 1차 | 2010_01_06 트러블슈팅 2건 |
| **#489** | 79ab6da (1) | Picture+Square wrap 호스트 paragraph 텍스트 LINE_SEG cs/sw 적용 | 2차 | exam_science p1 pi=21 5번 그림 영역 침범 정정 |
| **#495** | 7bd933a (1) | 셀 paragraph 인라인 Shape 분기 `tac_pos` ls[0] 가드 (text_before 중복 차단) | 3차 (부분 정정) | exam_science p2 박스 baseline 19→9 (중복 10 제거) |
| **#480** | 39a6def (1) | wrap=Square 표 paragraph margin x 좌표 반영 | 4차 | 표 위치 정합 (시각 판정 통과) |
| **#476** | 95d6dee (1) | PartialParagraph 인라인 Shape 페이지 라우팅 | 5차 | +881/-4, 시각 판정 통과 |

## 제외 Task (2 건)

| Task | 영역 | 제외 사유 |
|------|------|----------|
| **#479** ★ | paragraph trailing line_spacing (HWP vpos) | **회귀쪽 실패** — 한컴 2020 정답지 시각 판정 필수, 별도 task → [이슈 #503](https://github.com/edwardkim/rhwp/issues/503) 분리 등록 |
| #492 | orders 갱신만 (자연 해소) | 본 처리 사이클의 orders 충돌 |

## cherry-pick 정합 (5 단계 분리)

### 1차 (#488 + #490 + #483)
- 머지 commit: `79ab6da`
- 페이지 레이아웃 무관 안전 영역
- 작업지시자 시각 판정 통과

### 2차 (#489)
- 머지 commit: `869540e`
- Picture+Square wrap (간접 영향) — 시각 판정 후 흡수
- 작업지시자 시각 판정 통과

### 3차 (#495)
- 머지 commit: `dbb1a93`
- 셀 paragraph 인라인 Shape 분기 가드
- **부분 정정** — 시각 판정 결과 잔존 결함 발견 (이슈 #502 분리)
- 작업지시자 결정: 옵션 B (개선 효과 흡수 + 잔존 결함 별도 처리)

### 4차 (#480)
- 머지 commit: `82fd66e`
- wrap=Square 표 paragraph margin x 좌표 반영
- 작업지시자 시각 판정 통과

### 5차 (#476)
- 머지 commit: `5955b10`
- PartialParagraph 인라인 Shape 페이지 라우팅 (+881/-4)
- 작업지시자 시각 판정 통과

## 검증 게이트

| 검증 | 결과 |
|------|------|
| cargo test --lib | 1086 → **1102 passed** ✓ |
| cargo test --test svg_snapshot | **6/6** ✓ |
| cargo test --test issue_418 | **1/1** ✓ |
| cargo test --test issue_501 | **PASS** ✓ (본 사이클 정정 회귀 0) |
| cargo clippy --lib -- -D warnings | **0건** ✓ |
| WASM 빌드 | **4,202,430 bytes** ✓ (5차 후) |
| **작업지시자 시각 판정** | 1차/2차/4차/5차 통과 ✓, 3차 부분 정정 (이슈 #502 분리) |

## 잔존 결함 + 후속 처리

### 이슈 #502 — 문단 내 글상자 TextRun 처리

Task #495 의 가드 (`tac_pos` 가 ls[0] char 범위 안일 때만 text_before 발행) 가 부분 정정. 잔존 결함:
- exam_science p2 7번 박스 본문 \"분자당 구\" 4글자 누락
- 사각형 위치 결함

작업지시자 통찰: \"문단 내 글상자의 TextRun 처리 문제\" — paragraph 안 어느 char 위치에 글상자 도형이 있는가의 정합 영역.

→ [이슈 #502](https://github.com/edwardkim/rhwp/issues/502) 분리 등록 (assignee: edwardkim).

### 이슈 #503 — Task #479 본질 정정 흡수 (한컴 2020 시각 판정 필수)

**작업지시자 통찰**: \"#479 는 회귀쪽에서 실패가 납니다. 이건 따로 분리해서 한컴 2020 버전으로 시각판정을 해야 합니다.\"

본 PR 의 미흡수 1 Task #479 (paragraph trailing line_spacing / HWP vpos) — 본질 정정이지만 typeset.rs core 변경 + 광범위 paragraph 누적 영향. 한컴 2020 정답지 기준 광범위 시각 판정 필수.

→ [이슈 #503](https://github.com/edwardkim/rhwp/issues/503) 분리 등록 (assignee: edwardkim).

## 작성자 본 PR 의 본질 영역 (제외 영역)

본 PR cherry-pick 에서 제외한 4 Task 는 본 사이클 (Task #501) 의 정정 영역 (cell.padding 한컴 방어 로직) 과 충돌 위험이 큰 페이지 레이아웃 영역. 본 사이클의 회귀 정정이 안정화 된 후, **본 작업이 다음 PR 사이클** 에서 재처리 가능 영역. 작성자 본인의 광범위 검증 (★ Task ─ #479 / #488 / #489 / #490) 정합 확인 후 분리 cherry-pick.

## 작성자 영역 보존

- 모든 cherry-pick 에서 author 보존 (Jaeook Ryu)
- 단계 commit 분리로 추적 용이 (수행 계획서 + 단계 보고서 + 최종 보고서 + 트러블슈팅 모두 보존)

## 다음 단계

- PR #478 댓글 + close
- 이슈 #502 (문단 내 글상자 TextRun 처리) 별도 처리 (작업지시자 우선순위 결정)
- 제외 Task (#476, #479, #480) 본 사이클 안정화 후 별도 PR 가능성

## 메모리 룰 정합

- `feedback_pr_comment_tone` — 차분한 사실 중심
- `feedback_v076_regression_origin` — 작업지시자 직접 시각 검증 게이트 (1차/2차/3차 모두 통과)
- `feedback_assign_issue_before_work` — 잔존 결함 #502 즉시 assignee 지정
