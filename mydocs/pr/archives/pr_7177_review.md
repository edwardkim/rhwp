---
kind: snapshot
status: active
canonical: mydocs/pr/archives/pr_7177_review.md
last_verified: 2026-09-15
---

# PR #7177 검토

## 판정과 범위

로컬 검토 승인. [PR #7177](https://github.com/edwardkim/rhwp/pull/7177)은
[PR #7171](https://github.com/edwardkim/rhwp/pull/7171)에서 확인한 지침·검증 명령의 부족을 보완한다.
GitHub 최신 head CI·merge는 별도 확인 단계이며 이 기록은 merge 완료를 뜻하지 않는다.

- 작성자: jangster77. `collaborator_self_merge` 경로, `intake_and_review`, `local_validation`,
  `review_only_fast_pass` 적용. 본인 PR이므로 reviewer 요청·self-approve는 하지 않는다.
- base: `d5fbe8b5d8ad15dc39f1b20be3bd97796f958c44`.
- 변경 후보: `b2fa2e30034f117fd5205d874951f159f1f01c9d`.
- branch: `codex/pr7171-guidance-followup-20260915`.
- 대상: AGENTS·CLAUDE·CONTRIBUTING·PR 템플릿·local_validation·visual_fixture_evidence 6개 문서.

## 원인과 변경

#7165/#7168의 원 head에도 독립 근거·합성 계약 구분 원칙은 존재했다. 실제 사용 에이전트와
지침 로드 여부는 확인하지 못했으므로 작성 도구를 원인으로 단정하지 않는다. 입력에 수동 기록한
줄·폭을 정상으로 취급한 가정과 독립 한컴 검증 시점의 문제를, 구현 전 유효성 확인과 제출·review
증거 연결로 보완했다. 저장 정보 재사용과 편집 후 재조판을 각각 확인한다.

#7171 최초 CI 실패는 메인터너가 추가한 source-side 테스트의 base 비교 누락이었다.
현재 정책 정합성 검사와 PR base 대비 증분 검사가 달라 `--check` 단독 성공만으로 CI 성공을
예측할 수 없었다. 실행 예제에 고정 base SHA를 전달하고 CONTRIBUTING의 기존 준비 변수와
연결했다. local_validation을 base 준비 정본으로 두고 AGENTS와 CLAUDE의 참조를 연결했다.

번역 통합 테스트·Visual Sweep 도구 결함은 #7171에서 이미 수정됐으며 이번에는 중복 수정하지
않는다. 지침 문구로 모든 메인터너 보정을 없앨 수 있다고 주장하지 않는다. 공통 실행 wrapper나
workflow 변경은 이번 범위에 포함하지 않는다.

## 검증 결과

| 검증 | 실제 결과 |
| --- | --- |
| 변경 6개 문서 링크 및 공백 | 통과 |
| 변경 manual metadata | 통과 |
| 기존 manifest/unit-tier Node 계약 테스트 | 35 passed, 0 failed |
| 정상 source/base `d5fbe8b5d`의 manifest prepare/check | 1337 sources / 5770 static test attrs / 48 integration targets, 성공 |
| 정상 source/base `d5fbe8b5d`의 unit-tier base 비교 | 4205 tests / 298 modules, 성공 |
| 최초 후보 `95d631f0a`의 기존 `--check` | exit 0, 4206 tests |
| 같은 후보의 당시 base `da7ec5c09` 비교 | 예상한 exit 1, 4206 > 4205 및 cursor 3 > 2 검출 |

정상 후보의 검사 스크립트·제품 source는 문서 변경 후보와 동일하다. 별도 sparse review worktree에서
manifest 파생 상태만 준비했으며 생성 파일은 제출하지 않았다. 통제 실패는 아래 기존 CI 누락을
재현한 의도된 결과이며, 수정한 지침의 실패가 아니다.

~~~text
[RustUnitTier] 오류: PR base 대비 source-side test 총량 증가 금지: 4206 > 4205
PR base 대비 source-side 기준선 상향 금지: 4206 > 4205
PR base 대비 source unit test 증가 금지: src/renderer/composer/line_breaking.rs::fill_cursor_tests#1 (3 > 2)
PR base 대비 module 기준선 상향 금지: src/renderer/composer/line_breaking.rs::fill_cursor_tests#1 (3 > 2)
~~~

실행 명령:

~~~bash
node --test scripts/tests/rust-unit-test-tiers.test.mjs scripts/tests/rust-test-suite-manifest.test.mjs
# 정상 후보 전용 worktree
node scripts/rust-test-suite-manifest.mjs --prepare
node scripts/rust-test-suite-manifest.mjs --check --base-ref d5fbe8b5d8ad15dc39f1b20be3bd97796f958c44
node scripts/rust-unit-test-tiers.mjs --check --base-ref d5fbe8b5d8ad15dc39f1b20be3bd97796f958c44
# 최초 후보 전용 worktree: 첫 명령 성공, 두 번째 명령이 예상 실패
node scripts/rust-unit-test-tiers.mjs --check
node scripts/rust-unit-test-tiers.mjs --check --base-ref da7ec5c0906b38b7a1cfa15cf70ca5534b7d7ebb
~~~

제품 Rust·WASM·Studio·fixture·workflow 변경이 없어 빌드·전체 회귀를 수행하지 않았다.
검사 자체의 코드·정책·baseline을 수정하지 않았다.

## 공통 검토와 검증 입력

조판 동작·분할·측정·배치 변경은 비해당이다. 독립 근거와 검증 절차를 보강하는 문서 변경이며,
HWP/HWPX/PDF를 실행 검증에 사용하지 않았다. 신규 입력·PDF·시각 asset 커밋 대상이 없다.

정상 검증용 `/private/tmp/rhwp-pr7171-guidance-policy-check`와 통제용
`/private/tmp/rhwp-pr7171-guidance-policy-control`은 검사 완료 후 제거했다. 기본 checkout과
다른 작업 target은 보존한다. review·오늘할일은 같은 PR에 포함하며 후속 문서 PR을 만들지 않는다.

## 후속 처리

최종 head CI·mergeability 확인과 merge는 후속 단계다. merge 이후 이 PR의 실제 SHA, 문서 범위,
preflight/CI 결과, devel 동기화와 이번 local/upstream branch 정리 결과를 PR comment에 기록한다.
원 PR #7171 및 기존 contributor PR·이슈의 종료 처리는 반복하지 않는다.
