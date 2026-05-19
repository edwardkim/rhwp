# PR #995 최종 보고서 — HWP5 multi-TAC paragraph z-order 정합

- PR: [#995](https://github.com/edwardkim/rhwp/pull/995)
- 제목: fix: HWP5 multi-TAC paragraph z-order 정합 — composer marker synthesis (closes #991)
- 작성자: jangster77 (Taesup Jang) — 기존 컨트리뷰터 (sample16 계열 #989/#969 연속)
- base ← head: `devel` ← `jangster77:local/task991-fix`
- 결정: **merge (수용) + 코드 clean-up 추가 commit**
- 일자: 2026-05-19

## 1. 결정

**merge 수용.** PR 의 F2-narrow 설계 (composer-only marker synthesis +
3중 좁힘 가드) 는 메모리 룰 정합이며, 모든 검증 게이트 통과.
검토에서 지적한 코드 품질 2건 (§3.3 a/b) 은 본 처리에서 직접 정정 후
별도 commit 으로 동반 merge.

이슈 #991 은 이미 CLOSED 상태로 확정 (작업지시자 결정) — PR body 의
`closes #991` 은 merge 시 no-op. reopen 하지 않음.

## 2. 검증 결과

| 게이트 | 결과 | 비고 |
|--------|------|------|
| CI: Build & Test | ✅ pass | |
| CI: Analyze rust/js/py | ✅ pass | |
| CI: Canvas visual diff | ✅ pass | 페이지 수 회귀 교차 확인 |
| CI: CodeQL | ✅ pass | 보안 경고 0 |
| cargo build (네이티브) | ✅ | clean-up 적용 후 |
| cargo test --release --lib | ✅ **1301 passed, 0 failed** | PR 보고 1297 + devel merge 신규 4, 회귀 0 |
| cargo fmt --check | ✅ 통과 | |
| 240 sample 페이지 수 회귀 | ✅ 0 | PR 검증 + CI Canvas diff 교차 |
| **작업지시자 시각 판정** | ✅ **통과** | `hwp3-sample16-hwp5.hwp` p18 z-order 정합 (가. 위, 다이어그램, 나. 아래) |

> 시각 판정은 작업지시자가 직접 수행 (메모리 룰
> `feedback_pdf_not_authoritative` / `feedback_visual_judgment_authority`
> 정합 — PR 의 "PDF 정합" 자가검증이 아닌 작업지시자 직접 판정으로
> 게이트 충족). WASM 빌드 (Docker, release + wasm-opt) 제공 → 판정 수행.

## 3. 코드 clean-up (검토 §3.3 처리)

검토 문서 §3.3 의 지적 2건을 본 처리에서 직접 정정.
**동작 완전 불변** (좁힘 3조건·합성 로직 그대로), 중복/죽은 코드만 제거.

`src/renderer/composer.rs` (6 insertions, 17 deletions):

**(a) `first_off`/`n_leading` 중복 계산 제거.**
좁힘 가드(이전 L157-158)와 합성 본문(이전 L184-185)이 동일
`first_off`/`n_leading` 을 2회 계산(shadowing)하던 것을, `offsets`
바인딩을 가드 앞으로 이동하여 1회 계산 → 본문 재사용으로 정리.

**(b) 빈 paragraph 분기 (도달 불가 죽은 코드) 제거.**
`if offsets.is_empty() && chars.is_empty()` 분기는 좁힘 가드
`n_leading < 2` (빈 offsets ⇒ `n_leading=0` ⇒ 항상 early-return)
때문에 현재 좁힘 조건 하 도달 불가. 분기 제거 + 근거 주석 명시.

검증: `cargo fmt --check` ✅ / `cargo test --release --lib` 1301/0 ✅
(clean-up 전후 테스트 수·통과 동일 → 동작 불변 확인).

## 4. 설계 평가

- **메모리 룰 `feedback_hancom_compat_specific_over_general` 정합**:
  3중 좁힘 가드(`inline_ctrl_count>=3` + `n_leading>=2` +
  `existing_markers<inline_ctrl_count`)로 pi=394 패턴만 catch.
  일반화(F1/F2-wide: cargo test 5~9 fail) 대비 F2-narrow 0 fail —
  케이스별 구조 가드의 타당성 입증.
- **composer-only 격리**: parser/IR 원본 미변경 → editor pipeline
  (insert_text/save/cursor) 무영향. HWP3/HWPX path 미오염. 회귀
  표면 최소.
- **scope 정직**: HWPX 변종(#942/#988)·sample16 p22(#994)을 잔존
  영역으로 명시 분리. 과대 주장 없음.

## 5. cherry-pick 처리

PR 고유 commit (devel merge 제외):
- `ad1f20da` Task #991: F2 composer-only marker synthesis (소스)
- `09bfd102` Task #991: 보완 docs (구현계획서 + Stage 2/3/4)

처리: PR 2 commit author(jangster77) 보존 cherry-pick + 코드
clean-up 정정 commit 별도 분리 (메모리 룰 `feedback_*` cherry-pick
패턴 — author 보존, 본 환경 정합 commit 분리).

## 6. 잔존 / 후속

- HWPX 변종 `hwp3-sample16-hwp5.hwpx` p19 z-order: parser path 상이,
  본 fix 미적용 — #942/#988 close 영역 (별도, 본 PR scope 외).
- HWP5 sample16 p22 paragraph overlap: 별도 root cause(line_segs
  누락) → [#994](https://github.com/edwardkim/rhwp/issues/994) (OPEN) 추적.
- 좁힘 조건 완화(1-2 TAC 동일 패턴 catch) 또는 HWP5 parser spec
  정합(모든 downstream 영향)은 별도 task 후보 — 본 PR 범위 외.

## 7. 산출물

- `mydocs/pr/pr_995_review.md` (검토 문서)
- 본 보고서
- 소스: PR `composer.rs` (synthesize_marker_paragraph) + 본 처리
  clean-up (6+/17-)

## 8. 메모리 룰 갱신 검토

- `project_external_contributors`: jangster77 = 이미 등재된 누적
  기여자. 갱신 불요.
- 신규 룰 후보 없음 — 본 PR 처리는 기존 룰(좁힘 가드 우선, PDF 비정답지,
  cherry-pick author 보존, close 검증) 적용 사례. 메모리 추가 불요.
