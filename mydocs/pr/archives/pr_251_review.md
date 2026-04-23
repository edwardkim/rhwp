# PR #251 검토 문서

## PR 정보

| 항목 | 내용 |
|------|------|
| PR 번호 | [#251](https://github.com/edwardkim/rhwp/pull/251) |
| 작성자 | @seanshin (FIRST_TIME_CONTRIBUTOR) |
| 제목 | fix: Visual Diff 기반 렌더링 호환성 개선 — PUA 심볼 + 문단 테두리 + 표 외곽 테두리 (#249) |
| base ← head | **devel ← feature/pua-symbol-border-margin** ✅ |
| 관련 이슈 | [#249](https://github.com/edwardkim/rhwp/issues/249) |
| 변경 | +510 / -8 (13 파일, 5 커밋) |
| mergeable | MERGEABLE · BEHIND (devel `f3591b9`, base `3932499`) |
| Reviews/Comments | 0 |

## 배경 — PR #234 리젝트 후 재제출

어제 동일 기여자의 [PR #234](https://github.com/edwardkim/rhwp/pull/234) 는 다음 사유로 closed 되었음 (`mydocs/pr/archives/pr_234_review.md` 기록):

1. Base 가 `main → main` (devel 타깃 아님)
2. 메타 변경(Cargo.toml 0.7.2→0.7.3, CHANGELOG, README) 혼입 — 릴리즈 주체와 충돌
3. Right Tab 정렬 수정 혼입 후 자체 롤백 (범위 혼재)
4. `mydocs/eng/plans/` 위치 (기존 구조는 `mydocs/plans/`)
5. 단계별 보고서 없음

## PR #251 에서 충족한 재제출 요건

| 리젝트 사유 | PR #234 | PR #251 |
|---|---|---|
| Base 타깃 | main | **devel** ✅ |
| 메타 변경 혼입 | 있음 (Cargo/CHANGELOG/README) | **제외** (릴리즈 시점에 처리한다고 body 명시) ✅ |
| Right Tab 혼입 | 혼입 후 자체 롤백 | **별도 이슈 #250 + PR 로 분리** ✅ |
| 문서 위치 | `mydocs/eng/plans/` | **`mydocs/plans/` + `mydocs/working/` + `mydocs/report/`** ✅ |
| 단계별 보고 | 없음 | **stage 1~3 + 최종 보고서** ✅ |
| 관련 이슈 | 없음 | **#249 명시 (PR body)** ✅ |
| 규모 | +810/-14, 12파일 | **+510/-8, 13파일** ✅ |

기여자가 어제 리젝트 피드백을 정확하게 반영하여 재제출함. FIRST_TIME_CONTRIBUTOR 치고 하이퍼-워터폴 구조 학습 속도가 빠름.

## 변경 범위

### 코드 (4 파일, +108/-8)

| 파일 | 변경 | 목적 |
|------|------|------|
| `src/renderer/layout.rs` | +41/-7 | `map_pua_bullet_char()` 공용 유틸 + 문단 테두리 margin 반영 |
| `src/renderer/layout/paragraph_layout.rs` | +3/-1 | border_fill rect 에 margin_left/right 적용 |
| `src/renderer/layout/table_layout.rs` | +61/-0 | 표 외곽 테두리 fallback + 셀 미커버 영역 한정 |
| `src/renderer/svg.rs` | +5/-0 | PUA 변환 호출 |
| `src/renderer/web_canvas.rs` | +5/-0 | PUA 변환 호출 |
| `src/renderer/html.rs` | +5/-0 | PUA 변환 호출 |

### 문서 (7 파일, +390)

- `mydocs/plans/task_m100_249.md` — 수행계획서
- `mydocs/plans/task_m100_249_impl.md` — 구현계획서
- `mydocs/plans/task_m100_250.md` — #250 (Right Tab) 수행계획서
- `mydocs/working/task_m100_249_stage{1,2,3}.md` — 단계별 완료보고서
- `mydocs/report/task_m100_249_report.md` — 최종 결과보고서

## 사전 검증 결과 (로컬)

`local/pr251` 로 fetch + 실험 브랜치 `pr251-merge-test` 에서 `git merge origin/devel --no-commit --no-ff` 수행:

| 항목 | 결과 |
|------|------|
| 자동 머지 충돌 | **0건** (자동 머지 클린) |
| `cargo build --lib` | 성공 |
| `cargo test --lib` | **941 passed**, 0 failed, 1 ignored |
| `cargo clippy -- -D warnings` | **0 warning** |
| `cargo test --doc` | 0 passed / 0 failed (컴파일 에러 없음) |

→ 현재 상태로 머지 가능. 충돌 해결 불필요.

## 기능 상세

### 1. PUA 심볼 문자 렌더링 (#249-1)

- Wingdings 등 심볼 폰트의 PUA 영역(U+F000~F0FF) 문자가 □(두부 문자) 로 표시되는 문제
- `map_pua_bullet_char()` 유틸을 SVG/Canvas/HTML 세 렌더러에 **일관** 적용
- 유니코드 표준 글리프(화살표/도형/체크마크 등) 로 변환

### 2. 문단 border_fill margin 반영 (#249-2)

- `border_fill` rect 계산 시 문단의 `margin_left`, `margin_right` 미반영으로 테두리 박스가 텍스트 영역보다 넓게 그려지던 문제
- `paragraph_layout.rs` 에서 rect 계산에 margin 적용

### 3. 표 외곽 테두리 fallback + clip_rect (#249-3)

- `table.border_fill_id` 설정된 경우 외곽 테두리 미출력 문제
- `table_layout.rs` 에 fallback 로직 추가:
  - 셀이 커버하는 영역은 셀 경계가 그리도록 제외
  - 셀 미커버 영역(여백 등) 만 fallback 으로 그림
- `clip_rect` 를 콘텐츠 레이아웃 후 확정하여 표 외곽 테두리 클리핑 방지

## 위험 요소

| 위험 | 수준 | 완화 |
|------|------|------|
| 충돌 | 없음 (시뮬레이션 확인) | - |
| 회귀 | 낮음 (941 테스트 pass, clippy 0) | 한컴 샘플 실사용 수동 검증 권장 |
| 표 외곽 테두리 로직 복잡도 | 중간 | `table_layout.rs` +61 라인, 셀 커버 영역 계산 로직 review 필요 |
| Visual Diff 근거 확인 | 중간 | Visual Diff 파이프라인은 PR 에 포함 안 됨 → 작성자 자체 검증 신뢰 |

## 처리 방식 권고

**옵션 A — Squash merge 권장**: 기능 3건 + 문서 7 파일이 논리적으로 묶여있고 5 커밋이 깔끔. Merge commit 으로 진행.

**옵션 B — Admin merge**: 프로젝트 기존 패턴 (PR #215, #221, #224 등과 동일). BEHIND 상태라 GitHub UI 기본 Merge 버튼은 못 쓸 수 있어 admin 플래그 필요.

권장은 **B (admin merge)**. `mergeable: MERGEABLE` 이고 충돌 0 이라 BEHIND 만으로 차단될 이유 없음.

## 브랜치

- `local/pr251` (PR head fetch 완료)
- 실험 `pr251-merge-test` 는 검증 후 삭제 완료

## 구현 계획서

→ `mydocs/pr/pr_251_review_impl.md`
