# PR #1048 검토 — 본문 하단 overflow 정합, 측정 통일(B) (closes #1046)

## 1. PR 정보

| 항목 | 값 |
|------|-----|
| 번호 | #1048 |
| 제목 | Task #1046: 본문 하단 overflow 정합 — 측정 통일(B) (overflow 18→5) |
| 작성자 | planet6897 (Jaeuk Ryu) — 누적 컨트리뷰터 (오늘 PR #1039/#1044 머지, PR #1045 close) |
| base ← head | `devel` ← `planet6897:pr/task1046-overflow` |
| 라벨 | enhancement (실제 bug fix 성격이나 새 인프라 추가 영역도 있음) |
| **변경** | **19 파일 +1170 / -11 (대형 PR)** — 소스 4 (`typeset.rs` +167, `layout.rs` +80/-10, `paragraph_layout.rs` +7, `rendering.rs` +15), 문서 15 |
| 연결 이슈 | `closes #1046` (OPEN, planet6897 본인 작성, assignee 없음) |
| 분리 영역 | **#1049** (렌더러 줄높이 과대 계산, pi=781 4.6px) — PR 본문 자체 명시 분리 |
| mergeable | **CONFLICTING / DIRTY** — `rendering.rs` 1 파일 충돌 (PR #1036 라인 이동, 양립 가능) |
| CI | Build & Test ✅ / Analyze rust·js·py ✅ / Canvas visual diff ✅ / CodeQL ✅ / WASM skip |
| 본질 commit | 2 — `a148ed43` (본질) + `a05b284b` (fmt 정정) |
| 정량 측정 | **LAYOUT_OVERFLOW 18→5** (in-scope 14→1), cargo test 1518 passed 0 failed |
| 생성 | 2026-05-21 01:34 |

## 2. 배경 (이슈 #1046)

비공개 185p 문서에서 본문 하단 `LAYOUT_OVERFLOW` 16 건 잔여.

### 2.1 A → B 노선 전환

당초 사후 reflow(A) 시도:
- 본문 초과 항목을 다음 페이지로 이월
- **결과**: 측정 드리프트로 overflow 가 줄지 않고 이동·증식 (페이지마다 ~5px
  과소측정 → 항목 이월 시 도착 페이지 재과밀)
- 어떤 임계값도 baseline (16건) 보다 개선 불가
- → **작업지시자 결정**: 측정 통일(B) 전환 (이슈 #1046 본문)

### 2.2 근본 원인 — 두 축

**축 1: 배치 판정 overhead 누락 (표 강제배치)**
- 분할 진입부 가드의 `remaining_on_page` 가 첫 (비연속) fragment 의
  렌더러 y_start 점프 (`host_spacing.before` + TopAndBottom·vert=Para
  표의 `vertical_offset`) 를 차감하지 않음
- 안 들어가는 표를 강제 배치 → 본문 초과
- **수정**: 가드에 동일 overhead 차감 + 다행 표 비분할 첫행 조건부 이월
  (`multirow_clean_defer`). genuine page-larger·1×1 셀 (#874) 제외.

**축 2: overflow 검출의 trailing 간격 오검출**
- 표/문단 콘텐츠는 본문 안에 들어가나, 표 뒤/문단 끝의 trailing 간격
  (줄간격/spacing_after/outer_margin_bottom) 이 더해진 `y_offset` 으로
  초과 판정 (false-positive)
- **수정**: `last_item_content_bottom` (Cell) 로 콘텐츠 하단 (trailing
  가산 직전) 기록. 검출이 표/문단 항목에서 이 값으로 비교.
- **렌더링 출력 불변 — 검출만 정정** (#359/#404 trailing_ls 정책과 정합)

### 2.3 supersede chain

```
PR #1022 (closes #1022): 분할 표 cut 모델(RowCut) + LAYOUT_OVERFLOW 42→12
                        — Task #1022 의 사후 reflow(A) 시도
   ↓
이슈 #1046: A 접근 실패 → 측정 통일(B) 전환 결정
   ↓
PR #1048 (closes #1046): 측정 통일(B), LAYOUT_OVERFLOW 18→5
   ↓
이슈 #1049: 렌더러 줄높이 과대 계산 (pi=781 4.6px) 분리
```

PR 본문 자체가 supersede chain + 잔여 영역 분리 명시. 메모리 룰
`feedback_pr_supersede_chain` 정직 패턴.

## 3. 변경 내용

### 3.1 `src/renderer/typeset.rs` (+167, 페이지네이터 본질)

배치 판정 overhead 차감 + 다행 표 첫행 조건부 이월. 본 PR 의 핵심
변경 영역 (페이지네이터의 분할 진입부 가드).

### 3.2 `src/renderer/layout.rs` (+80/-10, 렌더러 본질)

overflow 검출 비교 기준 변경 — `y_offset` → `last_item_content_bottom`
(콘텐츠 하단). 검출 정정만, 렌더링 출력 불변.

### 3.3 `src/renderer/layout/paragraph_layout.rs` (+7, 최소)

본문 문단 콘텐츠 하단 기록 (셀 밖, wrap zone 호스트 paragraph만 영향).
`cell_ctx.is_none() && !skip_advance_empty_wrap` 가드로 영역 한정.

### 3.4 `src/document_core/queries/rendering.rs` (+15, **충돌 영역**)

- `paginate` → `paginate_pass(force_breaks)` 분리
- 사후 reflow(A) 접근 폐기 코멘트 + `paginate_pass` 의 `force_break_before`
  훅과 `LayoutOverflow` 의 section_index/is_first_in_column 계측은 측정
  통일 작업의 진단·후속용으로 유지
- 충돌 origin: PR #1036 (Task #1035, HWP3 vs HWP5 변환본 페이지 alignment,
  5/20 머지) 가 같은 파일 수정 → 라인 이동 충돌, **양립 가능** (PR #1045
  같은 본질 양립 불가 아님)

### 3.5 진단 인프라 — `RHWP_TABLE_DRIFT` env 게이트

env 미설정 시 동작 불변. 후속 진단 + 트러블슈팅 보존 목적.

## 4. 검토 항목

### 4.1 설계 적합성 — 메모리 룰 정합 ✅

- **`feedback_pr_supersede_chain`**: A (reflow) → B (측정 통일) 전환을
  PR 본문 + 이슈 #1046 모두 명시. PR #1022 (#1022 사후 reflow 잔여) →
  #1046 → #1049 분리 chain 정직 패턴.
- **`feedback_hancom_compat_specific_over_general`**: genuine page-larger
  · 1×1 셀 (#874) 제외 가드 + `cell_ctx.is_none()` 가드 등 케이스별
  구조 가드 적용. 일반화 회피.
- **`feedback_image_renderer_paths_separate`**: 페이지네이터 (typeset.rs)
  와 렌더러 (layout.rs) 두 path 측정 통일. 본 PR 의 핵심 본질이 두
  path 격차 해소.
- **scope 정직**: PR 본문 "렌더링 출력 불변 — 검출만 정정" 명시. 잔여
  영역 (#1049) 분리 명시. 골든 SVG 회귀 0 + CI Canvas visual diff pass
  로 주장 사실 검증.

### 4.2 코드 품질 ✅

- **주석 명료**: 정정 위치마다 [Task #1046] 태그 + 본질 + 가드 사유 명시
- **인프라 보존**: 폐기된 A 접근의 `force_break_before` 훅 등을 코멘트
  로 보존 (후속 활용 가능)
- **진단 게이트**: `RHWP_TABLE_DRIFT` env 로 동작 불변 보장하면서 진단
  인프라 보존
- 큰 지적 사항 없음

### 4.3 검증 충실성 ✅

PR body 검증 결과:
- `cargo test --release`: **1518 passed / 0 failed** ✅ (회귀 0)
- 한컴 2022 PDF 대조: 요구사항 표 (SIR-002/COR-003/TER-003) 한 페이지
  통째 배치 정합 ✅
- 대상 185p / aift.hwp 74p 페이지 수 불변 (회귀 0) ✅
- 골든 SVG 회귀 0 — "렌더링 출력 불변" 주장 사실 ✅

본 PR 의 검증 근거는:
- **결정적 측정** (LAYOUT_OVERFLOW 18→5, in-scope 14→1)
- **CI 전부 pass** (Canvas visual diff 포함 — 렌더링 출력 불변 결정적 검증)
- **페이지 수 불변** (광범위 sweep)
- **한컴 PDF 대조** (메모리 룰 `feedback_pdf_not_authoritative` 관련 —
  컨트리뷰터 자가 검증, 별도 메인테이너 hands-on 게이트 권고)

### 4.4 #1055 회귀와의 관계 — **무관 확인** ✅

본 검토 직전 등록한 회귀 이슈 #1055 (`hwp3-sample16-hwp5.hwp` p2 목차
배치 무너짐) 와 본 PR 영역 격리:

| 영역 | #1055 회귀 | PR #1048 |
|----|----|----|
| 변경 파일 | `text_measurement.rs` (PR #1026/#1047 영역) | `typeset.rs`/`layout.rs`/`paragraph_layout.rs`/`rendering.rs` |
| 본질 | 폰트 폭 정합 (WASM 측정) | 페이지네이터·렌더러 높이 측정 통일 |
| 영향 경로 | composer → text run → CharShape | 페이지 분할 + overflow 검출 |

→ **#1055 회귀 영역 무수정** = 본 PR 머지가 #1055 추가 회귀 유발하지
않음. 독립 처리 가능.

### 4.5 충돌 본질 분석 — 양립 가능 ✅

`rendering.rs:1414` 영역 충돌:
- PR #1048: `paginate` → `paginate_pass(force_breaks)` 분리 + 코멘트 +
  훅 추가
- 충돌 origin (PR #1036, Task #1035): 같은 파일 영역의 페이지 alignment
  fix

본 환경 임시 머지 시도 결과:
- `rendering.rs` 단일 파일 충돌
- `typeset.rs` auto-merge 성공 (PR #1040 Task #1052 와 다른 영역)
- 라인 이동/병합 가능 (PR #1045 같은 본질 양립 불가 아님)
- **컨트리뷰터 rebase 또는 메인테이너 충돌 해소로 통합 가능**

### 4.6 잔존 / scope 외

- 라벨 "enhancement" vs 실제 bug fix — 마이너 불일치
- 이슈 #1046 + #1049 assignee 누락 — PR #1031/#950/#1039/#1044 와 동일
  패턴 (본인 작성). 메모리 룰 `feedback_assign_issue_before_work` 안내
  후보, merge blocker 아님
- 잔여 4.6px (pi=781) — #1049 로 분리 정직
- page-larger 2건 (pi=323 단독 표, pi=567 nested) — 단일 항목이 본문보다
  큼, 범위 외 명시

## 5. 처리 절차 (간소화 4단계)

1. ✅ PR 정보 확인 (본 문서 §1~2)
2. → 본 검토 문서 작성 + 작업지시자 승인 요청 (현 단계)
3. **충돌 해소 필요** — 컨트리뷰터 rebase 요청 또는 메인테이너 충돌 해소.
   본질 양립 가능 (PR #1045 와 차별)
4. 검증 (충돌 해소 + 본 환경 빌드/테스트 + 작업지시자 시각 판정) →
   `pr_1048_report.md`

## 6. 1차 판단 (작업지시자 승인 전 잠정)

| 영역 | 평가 |
|------|------|
| 설계 방향 | ✅ 적합 — A→B 전환 + 측정 통일 + scope 정직 |
| CI / 결정적 검증 | ✅ 통과 (1518 passed, Canvas diff pass) |
| 코드 품질 | ✅ 양호 — 주석/scope/진단 인프라 모두 명료 |
| 정량 측정 | ✅ LAYOUT_OVERFLOW 18→5 (in-scope 14→1) |
| 렌더링 출력 불변 주장 | ✅ 골든 SVG 회귀 0 + Canvas visual diff pass 로 검증 |
| 충돌 본질 | ⚠️ CONFLICTING — 양립 가능 (PR #1036 라인 이동), 충돌 해소 필요 |
| **#1055 회귀와의 관계** | ✅ 무관 (독립 영역) |
| 시각 검증 | ⚠️ "측정 통일" 핵심 변경이라 메인테이너 hands-on 권고 (페이지네이션 핵심 경로) |
| 이슈 연결 | #1046 + #1049 assignee 누락 (안내 후보, merge blocker 아님) |
| 분리 영역 (#1049) | ✅ PR 본문 자체 분리 명시 정직 |

**잠정 결론**: 코드·설계·결정적 검증 모두 양호. PR #1039 의 "정량 게이트
충족 시 시각 판정 면제" 후보 패턴이나 본 PR 은 **페이지네이션 핵심 경로
대형 변경** 이라 메인테이너 hands-on 권고. **머지 전 2개 게이트**:
1. **충돌 해소** (양립 가능)
2. **메인테이너 시각 판정** (페이지네이션 핵심 경로라 PR #1039/#1044 보다
   강한 게이트 권고)

> 본 문서는 검토 계획 + 항목 통합. 작업지시자 승인/피드백 후
> 충돌 해소 → 검증 → `pr_1048_report.md` 로 최종 판단 기록.
