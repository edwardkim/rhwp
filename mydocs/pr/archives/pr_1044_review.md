# PR #1044 검토 — 중첩 표 1×1 wrapper 외곽 테두리 lookup off-by-one 정정 (closes #1043)

## 1. PR 정보

| 항목 | 값 |
|------|-----|
| 번호 | #1044 |
| 제목 | Task #1043: 중첩 표 1×1 wrapper 외곽 테두리 lookup off-by-one 정정 |
| 작성자 | planet6897 (Jaeuk Ryu) — 누적 컨트리뷰터 (PR #221, #587, #1039 등) |
| base ← head | `devel` ← `planet6897:task1043` |
| 라벨 | enhancement (실제 bug fix — 라벨 마이너 불일치, merge blocker 아님) |
| 연결 이슈 | `closes #1043` (OPEN, planet6897 본인 작성, assignee 없음) |
| mergeable | MERGEABLE / BEHIND (rebase 필요) |
| CI | Build & Test ✅ / Analyze rust·js·py ✅ / Canvas visual diff ✅ / CodeQL ✅ / WASM skip |
| 변경 | 7 파일 +321 / -2 — 소스 1 (`table_layout.rs` +5/-2, 실 변경 1 라인), 회귀 테스트 1 (+65), 문서 5 |
| 본질 commit | **단일 commit `0dfd3f43`** |
| 생성 | 2026-05-20 18:36 |

## 2. 배경 (이슈 #1043)

`samples/2. 인공지능(AI) 기반 재정통합시스템 구축 용역 제안요청서.hwpx`
8페이지 "추진조직 구성" 조직도에서 **외곽 1×1 wrapper 표의 테두리가
미표시**. 내부 9×8 표는 정상. 한컴 2022 PDF p8 에는 외곽 박스 존재.

### Root cause

`src/renderer/layout/table_layout.rs::layout_table` 의 1×1 wrapper 분기
(L239) 가 외곽 테두리 borderFill 조회 시 `cell.border_fill_id` (1-based
`borderFillIDRef`) 를 0-based `border_styles` Vec 인덱스로 **그대로** 사용:

```rust
styles.border_styles.get(cell.border_fill_id as usize)  // -1 누락
```

- HWPX `borderFillIDRef` = 1-based
- `border_styles` Vec = 0-based
- 같은 파일의 다른 모든 lookup (일반 셀/표/zone) 은 `.saturating_sub(1)`
  로 변환, **이 분기만 누락** → 한 칸 어긋난 borderFill (테두리 NONE) 읽어
  외곽 실선 누락

## 3. 변경 내용

### 3.1 소스 정정 (`table_layout.rs:239`, 1 라인)

```rust
// before
styles.border_styles.get(cell.border_fill_id as usize)
// after
styles.border_styles.get((cell.border_fill_id as usize).saturating_sub(1))
```

+ 주석 추가 ("border_fill_id 는 1-based, border_styles 는 0-based ...
일반 셀/표/zone lookup 과 동일").

본 수정으로 **모든 borderFill lookup 이 -1 변환 일관**. 좌표/렌더 로직
변경 없음.

### 3.2 신규 회귀 테스트 (`tests/issue_nested_table_border.rs` +65)

`nested_table_border_kwater_rfp_p19_outer_outline_present`:
- **본 환경 보유 샘플** `samples/k-water-rfp.hwp` p19 사용 (재현 가능)
- "외곽 1×1 wrapper 표 안에 내부 표가 든 자료 박스" 구조
- 내부 표 외곽 = 점선 (`stroke-dasharray`), wrapper 외곽 = 점선과 겹치는 실선
- **좌표 hardcode 없음** — 구조 관계 ("점선 외곽과 같은 y 의 실선") 로 판정
- 페이지네이션 시프트 / 무관한 다른 표 무영향
- 결정적 판정: 버그 → 0건 / 정정 → 2건 (상·하)

기존 `nested_table_border_exam_social_p1_q4_outline_present` 테스트
무영향 유지 (비회귀 보장).

### 3.3 PR 본문 검증 표 (bisect 양방향)

| 코드 상태 | 결과 |
|----|----|
| 정정 적용 | 통과 (겹치는 전폭 실선 2건) |
| 버그 임시 복원 | 실패 (0건) |

→ 회귀 가드 단방향 (정정만 통과) 이 아니라 **bisect 양방향** 입증.
본 환경 재현 가능.

## 4. 검토 항목

### 4.1 설계 적합성 — 메모리 룰 정합 ✅

- **`feedback_hancom_compat_specific_over_general`**: off-by-one 단순
  정정, 다른 lookup 과 일관성 확보. 측정 의존 분기 없음.
- **`feedback_small_batch_release_strategy`**: 단일 commit + 1 라인
  실 변경. 소형. 파서/렌더러 무관 영향.
- **scope 정직**: PR 본문 "좌표/렌더 로직 변경 없음" 명시. 실제 변경
  파일 검증 — `table_layout.rs` 단일 분기 정정.
- **회귀 가드 동봉 + 본 환경 재현 가능**: PR #1039 와 차별점. 본
  PR 본문이 가리키는 1차 증상 샘플은 컨트리뷰터 로컬 비공개이나
  회귀 테스트는 본 환경 보유 `k-water-rfp.hwp` 로 작성됨 — `feedback_
  external_docs_self_censor` 영역 의식이 충분.

### 4.2 코드 품질 ✅

- **주석 명료**: 정정 위치에 1-based vs 0-based 명시 + "일반 셀/표/
  zone lookup 과 동일" 일관성 근거 명시
- **회귀 테스트 설계 우수**:
  - `parse_lines` 헬퍼 (SVG `<line>` 좌표 + dashed 추출)
  - 구조 관계 판정 (좌표 hardcode 회피)
  - 무관한 다른 표 영향 차단
- 큰 지적 사항 없음

### 4.3 검증 충실성 ✅

PR body 검증 결과:
- cargo test 전체 0 failed ✅
- cargo fmt --check (수정 파일) clean ✅
- bisect 양방향 (정정/버그 임시 복원) 검증 ✅
- "k-water-rfp p19 wrapper 외곽 박스 4변 실선 복원 시각 확인" 주장

PR #1039 (parser-only) 와 동일한 "정량 게이트 충족 시 시각 판정 면제"
조건 만족 + **본 환경 회귀 테스트 재현 가능** 추가 강점.

### 4.4 잔존 / scope 외

- 라벨 "enhancement" vs 실제 bug fix — 마이너 불일치, merge blocker 아님
- 이슈 #1043 assignee 누락 — PR #1031/#950/#1039 와 동일 패턴 (본인
  작성 + 본인 PR), 메모리 룰 `feedback_assign_issue_before_work`
  안내 후보, merge blocker 아님
- PR 본문 1차 증상 샘플은 컨트리뷰터 로컬 비공개 — 그러나 회귀 테스트
  는 공개 fixture (`k-water-rfp.hwp`) 로 작성 → 검증 회복 가능

## 5. 처리 절차 (간소화 4단계)

1. ✅ PR 정보 확인 (본 문서 §1~2)
2. → 본 검토 문서 작성 + 작업지시자 승인 요청 (현 단계)
3. (불요 예상) 코드 품질 양호, 본 PR 수정요청 항목 없음
4. 검증 (로컬 빌드/테스트 + 회귀 테스트 실증) → `pr_1044_report.md`

## 6. 1차 판단 (작업지시자 승인 전 잠정)

| 영역 | 평가 |
|------|------|
| 설계 방향 | ✅ 적합 — off-by-one 단순 정정, lookup 일관성 확보 |
| CI / 결정적 검증 | ✅ 통과 (CI 전부 + 신규 회귀 테스트 bisect 양방향) |
| 코드 품질 | ✅ 양호 — 주석/회귀 테스트 설계 우수, 지적 사항 없음 |
| scope | ✅ 단일 파일 단일 분기, 1 라인 실 변경 |
| 회귀 가드 | ✅ **본 환경 재현 가능** (`k-water-rfp.hwp` p19) + 기존 테스트 비회귀 보장 |
| 시각 검증 | ⚠️ PR #1039 패턴 — 정량 게이트 충족 시 면제 가능 (작업지시자 결정) |
| 이슈 연결 | #1043 assignee 누락 (안내 후보, merge blocker 아님) |

**잠정 결론**: 코드·설계·검증 모두 양호. PR #1039 보다 강한 검증
(본 환경 재현 가능한 회귀 테스트 + bisect 양방향). **머지 전 1개 게이트**:
시각 판정 — 어제 PR #1039 처리에서 면제 결정한 동일 패턴 적용 가능.
본 환경에서 `cargo test --test issue_nested_table_border` 로 회귀 가드
실증 보조 가능.

> 본 문서는 검토 계획 + 항목 통합. 작업지시자 승인/피드백 후
> 검증 단계 → `pr_1044_report.md` 로 최종 판단 기록.
