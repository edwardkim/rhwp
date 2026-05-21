# PR #1044 최종 보고서 — 중첩 표 1×1 wrapper 외곽 테두리 lookup off-by-one 정정

- PR: [#1044](https://github.com/edwardkim/rhwp/pull/1044)
- 제목: Task #1043: 중첩 표 1×1 wrapper 외곽 테두리 lookup off-by-one 정정
- 작성자: planet6897 (Jaeuk Ryu) — 누적 컨트리뷰터 (PR #221, #587, #1039 등)
- base ← head: `devel` ← `planet6897:task1043`
- 결정: **merge (수용)** — 정량 + 시각 게이트 모두 통과
- 일자: 2026-05-21

## 1. 결정

**merge 수용.** off-by-one 단순 정정 (1 라인, 다른 lookup 들과 일관성
확보) + 본 환경 재현 가능 회귀 가드 + 작업지시자 시각 판정 통과로
모든 게이트 충족.

이슈 #1043 은 OPEN, planet6897 본인 작성, assignee 비어있음. PR merge
처리 후 명시 close 처리 (`feedback_close_issue_verify_merged` 정합).

## 2. 검증 결과

| 게이트 | 결과 |
|--------|------|
| CI: Build & Test | ✅ pass |
| CI: Analyze rust/js/py | ✅ pass |
| CI: Canvas visual diff | ✅ pass |
| CI: CodeQL | ✅ pass |
| 본 환경 cargo build | ✅ |
| 본 환경 cargo fmt --check | ✅ exit 0 |
| **본 환경 PR 회귀 테스트 실증** | ✅ `nested_table_border_kwater_rfp_p19_outer_outline_present` **passed** |
| 본 환경 비회귀 (`exam_social` 기존 테스트) | ✅ passed |
| 본 환경 cargo test --release --lib | ✅ 1323 passed, 0 failed |
| WASM Docker 빌드 (release + wasm-opt) | ✅ `pkg/rhwp_bg.wasm` 4.7M (1m 38s) |
| **작업지시자 시각 판정** (k-water-rfp.hwp p19 wrapper 외곽 박스 4변 실선) | ✅ **통과** |

### 정량 게이트 + 시각 게이트 동시 충족

PR #1044 는 PR #1039 보다 한 단계 강한 검증:
- 결정적 측정: **bisect 양방향** (정정 → 2건 / 버그 임시 복원 → 0건)
- 회귀 가드 단위 테스트: **본 환경 재현 가능** (공개 `k-water-rfp.hwp`)
- 단일 책임 scope: 1 분기 1 라인 정정

여기에 작업지시자 시각 판정까지 통과하여 모든 검증 게이트 충족.

## 3. 변경 내용

### 3.1 소스 정정 (`src/renderer/layout/table_layout.rs:239`, 1 라인)

```rust
// before
styles.border_styles.get(cell.border_fill_id as usize)
// after
styles.border_styles.get((cell.border_fill_id as usize).saturating_sub(1))
```

+ 주석 추가 ("border_fill_id 는 1-based(borderFillIDRef), border_styles
는 0-based Vec 이므로 -1 변환한다. 일반 셀/표/zone lookup 과 동일").

본 수정으로 모든 borderFill lookup 이 `-1` 변환 일관. 좌표/렌더 로직
변경 없음.

### 3.2 신규 회귀 테스트 (`tests/issue_nested_table_border.rs` +65)

`nested_table_border_kwater_rfp_p19_outer_outline_present`:
- 본 환경 보유 `samples/k-water-rfp.hwp` p19 사용 (재현 가능)
- 좌표 hardcode 없이 구조 관계 ("내부 표 점선 외곽과 같은 y 의 실선") 판정
- 페이지네이션 시프트 / 무관한 다른 표 무영향
- 결정적 판정: 버그 → 0건 / 정정 → 2건 (상·하)

기존 `nested_table_border_exam_social_p1_q4_outline_present` 테스트
무영향 유지 (비회귀 보장).

## 4. Root cause + 설계 평가

### 4.1 Root cause

HWPX `borderFillIDRef` 는 1-based, `border_styles` Vec 은 0-based.
같은 파일 다른 모든 lookup (일반 셀/표/zone) 은 `.saturating_sub(1)` 로
변환하는데 **1×1 wrapper 외곽선 분기만 누락** → 한 칸 어긋난 borderFill
(테두리 NONE) 읽어 외곽 실선 통째 미표시.

### 4.2 메모리 룰 정합

- **`feedback_hancom_compat_specific_over_general`**: off-by-one 단순
  정정, 다른 lookup 과 일관성 확보. 측정 의존 분기 없음.
- **`feedback_small_batch_release_strategy`**: 단일 commit + 1 라인 실 변경.
- **`feedback_visual_judgment_authority` 정합** (PR #1039 면제 패턴
  확장): 본 PR 도 정량 게이트 + 회귀 가드 충족, 작업지시자 시각 판정도
  통과 → 검증 충실성 더 강함.

## 5. cherry-pick 처리

PR 본질 commit:
- `0dfd3f43` Task #1043: 중첩 표 1×1 wrapper 외곽 테두리 lookup off-by-one 정정

처리: 단일 commit author (planet6897 / Jaeuk Ryu) 보존 cherry-pick.
clean-up 후속 commit 없음 (코드 품질 지적 사항 없음).

## 6. 잔존 / 후속

- **이슈 #1043 assignee 누락** — PR #1031/#950/#1039 와 동일 패턴 (본인
  작성 + 본인 PR). 메모리 룰 `feedback_assign_issue_before_work` 안내
  후보, merge blocker 아님.
- **라벨 "enhancement" vs 실제 bug fix** — 마이너 불일치, 처리 후 라벨
  수정 가능 (선택).
- PR 본문 1차 증상 샘플은 컨트리뷰터 로컬 비공개 (`samples/2. 인공지능
  (AI) ...`), 회귀 테스트는 공개 `k-water-rfp.hwp` 로 작성됨 — 검증
  회복 충실.

### 분리 보존 — 본 PR scope 외

- 다른 OPEN PR (#1051, #1048, #1045 planet6897, #1019 postmelee 등) —
  본 PR 처리와 독립

## 7. 산출물

- `mydocs/pr/pr_1044_review.md` (검토 문서)
- 본 보고서
- 소스: PR `table_layout.rs:239` 1 라인 정정 + 신규 회귀 테스트 65 라인

## 8. 메모리 룰 갱신 검토

- `project_external_contributors`: planet6897 = 등재된 누적 기여자.
  갱신 불요.
- **본 PR 이 PR #1039 의 "정량 게이트 충족 시 시각 판정 면제" 메모리
  룰 후보 강화** — 본 PR 은 회귀 테스트가 본 환경 재현 가능 + 시각
  판정도 통과로 더 강한 권위 사례. 별도 정리 task 에서 두 사례
  (PR #1039 + PR #1044) 합쳐 권위 사례로 정리 권장.
