# PR #405 검토 — `Paragraph::control_text_positions` 추가 (옵션 A, #390)

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#405](https://github.com/edwardkim/rhwp/pull/405) |
| 작성자 | [@DanMeon](https://github.com/DanMeon) — 이슈 #390 자체 보고자 (이슈 assignee) |
| base / head | `devel` ← `DanMeon:feature/expose-control-text-positions` |
| state | OPEN |
| mergeable | MERGEABLE |
| mergeStateStatus | BEHIND |
| 변경 통계 | +364 / -57, 6 files |
| **CI** | **statusCheckRollup 비어있음** — 첫 PR 정황 |
| 이슈 | [#390](https://github.com/edwardkim/rhwp/issues/390) |
| 정황 | **작업지시자가 사전 PR 권유한 사안** — 이슈 작성자가 자체 정정 PR 제출 |

## 작성자 정황

@DanMeon — 신규 컨트리뷰터 (첫 PR). 다만:
- 이슈 #390 의 자체 보고자 (외부 binding `rhwp-python` 작업 중 발견)
- **작업지시자가 사전 PR 권유** 한 사안이라 의향 일치
- 이슈 본문에 **옵션 A / 옵션 B 둘 다 제시** + Claude 조사 결과 포함 — 문서화 / 분석 능력 양호
- PR 본문에 **수행/구현/최종 보고서 모두 포함** — CLAUDE.md 절차 준수 (이슈 작성자라 절차 인지)

## 변경 내용

### 본질 — 위치 이동 리팩토링 (알고리즘 변경 없음)

| 변경 유형 | 위치 |
|----------|------|
| 알고리즘 본체 (~60 줄) | `document_core::helpers::find_control_text_positions` → **`model::Paragraph::control_text_positions`** (인스턴스 메서드) |
| `helpers::find_control_text_positions` | thin wrapper (`para.control_text_positions()`) 로 교체 + `pub(crate)` 가시성 유지 |
| 26 caller (cursor / nav / 렌더러 / 책갈피 / 명령 / WASM) | **변경 없음** (wrapper 가 그대로 동작) |

### 옵션 A 선택 사유 (작성자 분석)

작성자가 본문에 명시:
> 외부 API surface 를 좁게 유지 — `Paragraph` 메서드 한 개만 노출. helpers 모듈은 내부 구현으로 자유롭게 진화. 의미 응집 — paragraph 자체에 컨트롤 위치 질의 메서드. 옵션 B 는 helpers 의 다른 비공개 함수까지 함께 노출되어 향후 SemVer 부담.

→ **합리적 분석**. 외부 binding (PyO3 / napi / JNI / `rhwp-python`) 의 long-term API 안정성 측면에서 옵션 A 우수.

### 의존성 방향 정합성

`model` ← `parser` ← `document_core` ← `renderer` ← `wasm_api` 단방향 의존 구조 보존:
- 알고리즘 본체는 model 레이어 (`Paragraph` 자체 데이터만 사용 — `text`, `char_offsets`, `controls`)
- helpers (document_core 레이어) 가 model 의 메서드를 호출하는 정방향 의존

작성자가 직접 짚었음 — 의존성 분석 정확.

### 단위 테스트 6 개 추가 (`src/model/paragraph/tests.rs`)

| 테스트 | 검증 |
|--------|------|
| `test_control_text_positions_empty` | controls 없음 → 빈 vec |
| `test_control_text_positions_no_offsets_inline_sequential` | char_offsets 없음 + Table 2개 → [0, 1] |
| `test_control_text_positions_gap_between_chars` | "AB" + char_offsets [0, 9] + Table 1개 → [1] |
| `test_control_text_positions_gap_before` | "A" + char_offsets [8] + Table 1개 → [0] |
| `test_control_text_positions_surrogate_pair_char_width` | "🎉A" surrogate pair (UTF-16 width=2) 처리 검증 |
| `test_control_text_positions_no_offsets_non_inline_skipped` | Bookmark (비인라인) 은 pos 증가 안 함 |

### 자기검증 정황 점검

**PR #400 와 다름**:
- PR #400: 새 직렬화 코드 + 자기 IR 로 자체 직렬화 검증 → 0=0 정황 (메모리 위반)
- **PR #405**: **기존 검증된 알고리즘** (v0.5.0 부터 prod 사용) **위치만 이동** + 알고리즘 단위 테스트 (특정 입력 → 특정 출력 명확)
- 알고리즘 자체가 변경 안 됐으므로, 단위 테스트가 알고리즘 정확성을 검증한다는 의미가 명확

→ **자기검증 정황 우려 없음**.

## 검증

### 본 검토에서 dry-run merge 결과

devel 위에 자동 머지 성공.

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | ✅ **1037 passed** (1031 → +6 신규) |
| `cargo test --test svg_snapshot` | ✅ 6/6 passed |
| `cargo test --test issue_418` | ✅ 1/1 (Task #418 보존) |
| `cargo clippy --lib -- -D warnings` | ✅ warning 0건 |
| `cargo test --lib model::paragraph` | ✅ 39 passed (PR #405 의 신규 6 포함) |

## 평가

### 강점

1. **사전 PR 권유 사안** — 작업지시자가 직접 제안한 방향
2. **이슈 작성자 자체 정정** — 외부 binding 사용처 (`rhwp-python`) 의 실제 필요 정황 명확
3. **옵션 A 선택 사유 정확** — long-term API 안정성 측면
4. **알고리즘 변경 없음** — 위치 이동 리팩토링이라 회귀 위험 최소
5. **의존성 정합** — model 레이어로 이동, 단방향 의존 구조 보존
6. **26 caller 영향 없음** — wrapper 가 그대로 동작
7. **CLAUDE.md 절차 준수** — 수행/구현/최종 보고서 모두 포함
8. **단위 테스트 6 개** — 알고리즘 정확성 검증 (자기검증 정황 우려 없음)
9. **dry-run merge** — 자동 성공 + 1037 passed
10. **clippy 통과**

### 약점 / 점검 필요

#### 1. CI 실행 안 됨

`statusCheckRollup` 비어있음 (PR #397, #400 와 같은 정황). 첫 PR 정황으로 추정. devel rebase + push 후 자동 트리거 예상.

#### 2. devel BEHIND

PR #395, #396 머지 전 base. devel rebase 필요.

#### 3. 시각 검증 자료 미제공

PR 본문: "API 노출, 시각적 변경 없음". 정확함 — 알고리즘 변경 없으므로 시각 검증 불필요. 대신 **단위 테스트 + cargo test 1037 passed** 가 검증 게이트.

## 메인테이너 작업과의 관계

### 충돌 가능성

본 PR 의 영향 파일 (6 files) 중 PR #395, #396, #401 (오늘 머지) 와의 영향:
- `src/model/paragraph.rs` — 다른 PR 미변경 ✅
- `src/document_core/helpers.rs` — 다른 PR 미변경 ✅
- `src/model/paragraph/tests.rs` — 미변경 ✅
- `mydocs/plans/`, `mydocs/report/` — 신규 ✅

dry-run merge 자동 성공 확인.

## 처리 방향 — cherry-pick 머지 권장 (옵션 머지)

본 PR 은 **cherry-pick 머지 권장**:

1. 사전 PR 권유 + 이슈 작성자 자체 정정 정황 — 의향 일치
2. 알고리즘 변경 없음 → 회귀 위험 최소
3. 단위 테스트 + cargo test 1037 passed 검증 통과
4. 시각 검증 불필요 (API 노출, 동작 변경 없음)
5. CLAUDE.md 절차 준수

### 옵션 후보

| 옵션 | 내용 |
|------|------|
| **A** | cherry-pick 머지 (작성자 attribution 보존) |
| B | 작성자에게 rebase 요청 후 재제출 (PR #397 와 같은 패턴) — 본 PR 은 상대적으로 절차 부담 작아 직접 머지가 합리 |
| C | 거절 / close — 추천 안 함 |

### 권장: 옵션 A (cherry-pick 머지)

이유:
- **알고리즘 변경 없음** → 회귀 위험 최소
- 사전 PR 권유 사안 → 의향 일치
- CLAUDE.md 절차 준수 → rebase 없이도 자동 머지 + 검증 통과
- CI 미실행은 본 검토에서 cargo test --lib 1037 passed 로 보완
- PR #401 같은 회귀 위험이 본 PR 에는 없음 (알고리즘 동일, API 노출만)

## 검토 항목 (Claude 점검 완료)

- [x] 작성자 신뢰도 — 이슈 작성자, 작업지시자 사전 권유 ✅
- [x] 코드 품질 — 위치 이동 리팩토링, 알고리즘 변경 없음 ✅
- [x] 단위 테스트 — 6 개, 자기검증 정황 우려 없음 ✅
- [x] dry-run merge — 자동 성공 ✅
- [x] cargo test --lib — 1037 passed ✅
- [x] cargo clippy — warning 0 ✅
- [x] 의존성 방향 — model 레이어 정합 ✅
- [ ] CI 실행 — 비어있음 ⚠️ (본 검토에서 cargo test 통과로 보완)

## 다음 단계 — 작업지시자 결정

A / B / C 중 결정 부탁드립니다.

권장: **A** — cherry-pick 머지 (작성자 attribution 보존).

## 참고

- PR: [#405](https://github.com/edwardkim/rhwp/pull/405)
- 이슈: [#390](https://github.com/edwardkim/rhwp/issues/390) (assignee @DanMeon, 본 PR 작성자)
- 외부 binding 사용처: `rhwp-python` (작성자가 작업 중)
