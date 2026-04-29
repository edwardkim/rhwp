# PR #446 처리 보고서 — set_field 후 저장/재오픈 시 필드 값 유실 정정 (#270)

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#446](https://github.com/edwardkim/rhwp/pull/446) |
| 작성자 | [@oksure](https://github.com/oksure) (Hyunwoo Park) |
| 이슈 | [#270](https://github.com/edwardkim/rhwp/issues/270) (closes) |
| 처리 결정 | **cherry-pick 머지** + 메인테이너 후속 e2e 추가 |
| 처리 일자 | 2026-04-29 |

## 처리 절차

### Stage 0: 검토 (옵션 변천)

작업지시자 직접 결정 — 정황 변화에 따라 옵션 3회 변경:

1. **옵션 C** (메인테이너 통합 테스트 직접 추가) — "두번 일할 필요 없음, 작업지시자가 반드시 검증해야 하는 영역" 정황
2. **옵션 A** (작성자 보강 요청) — 한컴 편집기 검증 실패 정황 (누름틀 제거 / 빈 값) 발견 후
3. **cherry-pick 머지** — 한컴 2010 + 한컴 2020 양 환경 검증 성공 (작업지시자 1페이지 vs 2페이지 혼동 정정 후)

### Stage 1: cherry-pick

`local/pr446` 브랜치 (`local/devel` 분기) 에서 단일 commit cherry-pick — 작성자 attribution 보존:

| commit (cherry-pick) | 작성자 | 내용 |
|---------------------|--------|------|
| `1698a9c` (← `2929f30`) | @oksure | fix: set_field 후 저장/재오픈 시 필드 값 유실 수정 (#270) |

cherry-pick 결과: 충돌 없이 자동 적용.

### Stage 2: 메인테이너 후속 정정 (옵션 C 잔재 — e2e 추가)

옵션 C 결정 시 추가한 e2e 가 회귀 게이트로 가치 있어 함께 commit:

- `rhwp-studio/e2e/issue-270-set-field-persist.test.mjs` (155 lines) — 메인테이너 e2e 검증 표준 패턴
- `rhwp-studio/public/samples/field-01.hwp` — Vite `/samples/` 라우팅 정합

### Stage 3: 검증 게이트

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | ✅ **1069 passed** (회귀 0건) |
| `cargo test --test svg_snapshot` | ✅ 6/6 passed |
| `cargo test --test issue_418` | ✅ 1/1 (Task #418 보존) |
| `cargo clippy --lib -- -D warnings` | ✅ warning 0건 |
| 작성자 회귀 테스트 3건 (`field_query::tests`) | ✅ 모두 통과 |
| WASM 빌드 (Docker) | ✅ 1m 19s |
| **메인테이너 e2e** (`issue-270-set-field-persist`) | ✅ 모두 통과 (12 assertions) |

### Stage 4: 한컴 편집기 검증 (작업지시자 직접)

**검증 자료**: e2e 산출물 `output/hwp/issue_270_persist_test.hwp` (473,600 bytes)

| 한컴 환경 | "회사명" 필드 | 결과 |
|----------|--------------|------|
| **한컴 2020** | "PERSIST_TEST" 정상 표시 | ✅ **성공** |
| **한컴 2010** | "PERSIST_TEST" 정상 표시 | ✅ **성공** |
| set_field 안 한 다른 필드 | placeholder 정상 표시 | ✅ 회귀 없음 |

→ 메모리 `feedback_self_verification_not_hancom` 의 핵심 게이트 (한컴 편집기 직접 검증) 양 환경 통과.

## 변경 요약

### 본질 — `field_query.rs::rebuild_char_offsets` 정정 (107 라인)

`field_begin_at` 배열 추가 + FIELD_BEGIN(0x03) 위치에 8바이트 갭 생성. 이중 계산 방지 가드 (`control_idx >= ctrls_before_text && start_char_idx > 0`).

### 메인테이너 후속 정정

- e2e 테스트 (`rhwp-studio/e2e/issue-270-set-field-persist.test.mjs`) — 회귀 게이트 + 컨트리뷰터 e2e 학습 자료
- `field-01.hwp` 를 `rhwp-studio/public/samples/` 에 추가 (Vite 라우팅 정합)

## 본 검증 사이클의 의의

본 PR 의 검증 흐름 자체가 **메모리 `feedback_self_verification_not_hancom` 원칙의 실증**:

1. **자기 라운드트립** (e2e: rhwp parse → set_field → exportHwp → re-parse → get_field) — 통과
2. **한컴 편집기 직접 검증** (한컴 2010 + 2020) — 양 환경 통과

두 게이트가 분리되어야 하는 이유 (PR #428 / 본 PR 검증 사이클의 1차/2차 결정 변천 포함) 가 본 사이클에서 명확히 드러났습니다.

또한 본 e2e 패턴은 향후 컨트리뷰터들에게 **저장 경로 PR 의 표준 검증 절차** 로 활용 가능 — `rhwp-studio/e2e/` + Chrome CDP `localhost:19222` 환경에서 실제 사용자 시나리오 + 한컴 편집기 검증용 산출물 자동 생성.

## 메모리 원칙 부합

| 메모리 | 부합 여부 |
|--------|----------|
| 외부 PR 머지 전 검증 게이트 | ✅ cargo test 1069 + svg_snapshot 6/6 + clippy 0 + WASM + e2e |
| **자기 라운드트립 ≠ 한컴 호환** | ✅ 두 게이트 분리 검증 (e2e + 한컴 2010/2020 직접) |
| 한컴 호환은 일반화보다 케이스별 명시 가드 | ✅ FIELD_BEGIN/END 비대칭 정확 식별 + 이중 계산 방지 가드 |
| output 폴더 가이드라인 | ✅ `output/hwp/issue_270_persist_test.hwp` |
| 보고서는 타스크 브랜치에서 커밋 | ✅ `local/pr446` 에서 커밋 |
| 이슈 close 시 commit devel 머지 검증 | ✅ 머지 + push 후 close |
| 작은 단위 PATCH 회전 | ✅ 단일 파일 정정 + 메인테이너 후속 e2e 만 |

## 다음 단계

1. 본 보고서 + e2e + field-01.hwp + 오늘할일 갱신 commit
2. `local/pr446` → `local/devel` → `devel` 머지 + push
3. PR #446 close + 작성자 댓글 (이슈 #270 자동 close)

## 참고

- 검토 문서: `mydocs/pr/pr_446_review.md`
- PR: [#446](https://github.com/edwardkim/rhwp/pull/446)
- 이슈: [#270](https://github.com/edwardkim/rhwp/issues/270)
- e2e 산출물: `output/hwp/issue_270_persist_test.hwp`
- e2e 보고서: `output/e2e/issue-270-set-field-persist-report.html`
- 같은 작성자 머지 PR (본 사이클): [#395](https://github.com/edwardkim/rhwp/pull/395), [#396](https://github.com/edwardkim/rhwp/pull/396), [#427](https://github.com/edwardkim/rhwp/pull/427), [#444](https://github.com/edwardkim/rhwp/pull/444)
