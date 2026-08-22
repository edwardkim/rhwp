---
kind: pr-review
status: review-complete-pending-trailing-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-22
---

# PR #5857 self-review — #4962 W3·W4 renderer coverage와 조판 위험 순위

## 접수 메타데이터

| 항목 | 작성 시점 참고값 |
| --- | --- |
| PR / 작성자 | [#5857](https://github.com/edwardkim/rhwp/pull/5857) / `edwardkim` self-review |
| 관련 issue | [#4962](https://github.com/edwardkim/rhwp/issues/4962), 부모 [#4960](https://github.com/edwardkim/rhwp/issues/4960), 후속 [#4963](https://github.com/edwardkim/rhwp/issues/4963) |
| base / head | `devel` / `task_m100_4962` |
| code candidate | `683fc2c73206cef22fc2c8246e998e46d4d9e3eb` |
| 상태 | Open, non-draft, `MERGEABLE`, `CLEAN` |
| 변경 규모 | 58 files, +48,281 / -14, 40 commits |
| 라우팅 | `collaborator_self_merge` + `intake_and_review` + `local_validation` + 대형 PR 예외 |

PR 본문은 `Closes #4962`를 포함한다. reviewer는 지정하지 않았고 작성자
self-review로 독립 검토했다. 상태값은 이 문서 작성 시점의 참고값이며 merge 전에 최신
trailing head로 다시 조회해야 한다.

## 변경 범위와 대형 PR 판정

- HWP·HWPX 문서의 편집 속성과 실제 renderer 문자 폭 결정을 읽기 전용으로 조인해
  `measured-overlay`, `identity-alias-hit`, `metric-surrogate`, `exact-hit`, `char-miss`,
  `face-miss`, `heuristic` 일곱 배타 category로 계측한다.
- 문서별 worker 격리, 자원 상한, private manifest, checkpoint·resume·finalizer와 결정적
  hash chain을 추가했다. private 10k 코퍼스의 원문·경로·파일명·문서 hash는 공개하지
  않는다.
- W4는 351개 문서 face의 실증 위험을 A 6, B 11, C 32, D 302로 나누고, A+B
  17개를 #4963 controlled-ladder 후보로 공개한다. identity·successor·missing-font 관계를
  추측하지 않았다.
- 48,281줄 증가 중 33,113줄은 재생성을 가진 892,640-byte 공개 ranking JSON이다.
  나머지는 stage별 계약·테스트·보고서이며 각 단계를 별도 commit과 승인 gate로 고정했다.
- 제품 renderer 출력, metric DB, fallback target, font asset, paint·layout 동작은 바꾸지
  않았다. 시각 sweep은 적용 대상이 아니다.

## 코드·보안·불변식 검토

수집기, worker, manifest, checkpoint, finalizer, ranker와 공개 projection을 수동 교차 검토했다.

- Rust 수집기는 unknown·모순된 width source를 fail-closed하고 category 합, layout 분모,
  source join과 usage row 문자 수를 서로 대사한다. 문자 수를 잘라 성공 처리하지 않고 작업·row·
  출력·deadline·중첩 상한 초과를 명시적 실패로 반환한다.
- supervisor는 문서별 Linux process group에 CPU·address space·wall time·stdout 상한을
  적용하고 timeout 시 자식 process group 전체를 종료한다. stderr·signal·원문 error·입력
  경로는 결과 envelope에 남기지 않는다.
- manifest는 symlink·special file·인벤토리 drift·hash 중 입력 변경을 거부한다. checkpoint는
  journal을 fsync한 뒤 state를 atomic rename하고, 재개 시 source head·policy·manifest·worker·runner·
  분석 option·격리 상한의 identity drift를 거부한다.
- W4 ranker는 문서 face identity와 metric request cluster를 합치지 않고, stored LineSeg를
  유효성·버전·가중치로 쓰지 않으며 존재 lane으로만 보존한다. historical supply는 현재
  가용성이나 metric identity의 증거로 승격하지 않았다.
- 공개 JSON은 privacy validator를 통과했다. `path`, `source`, `filename`, `documentHash`,
  `blake3`, raw row·trace, home 경로, token·stack 발견은 0이고 identity guess·cross-band promotion·
  queue 범위 이탈도 0이다.

차단 소스 결함은 발견하지 못했다. 제출 문서 규칙을 교차 검사하며 최종 보고서
본문의 명시적 `Issue: #4962` 표기 누락 1건을 발견했고, 이 trailing 문서 commit에서
정정했다. 코드 보정, conflict 해결 또는 다중 선택 절차가 없으므로 별도
`pr_5857_review_impl.md`는 필요하지 않다.

## 검증 증적

code candidate `683fc2c73`에서 다음 로컬 검증을 완료했다.

- integration manifest 863 sources / 4,081 static test attrs / 32 suites + 9 exceptions가 대사됐고,
  unit-tier policy는 4,225 tests / 299 modules로 통과했다.
- W3·W4 Node 45 pass / 1 의도적 skip, #4962 Rust 8/8, #4961 보호 회귀 4/4가 통과했다.
  self-review에서 관련 Node 계약 46건을 추가로 재실행해 46/46 통과했다.
- release build, release lib 4,075 pass / 13 ignore, nextest 8,129/8,129, Native Skia lib
  4,132 pass / 13 ignore와 focused 2/2·4/4가 통과했다.
- clippy `-D warnings`, Rust doc test 8 pass / 3 ignore, `cargo fmt --all -- --check`,
  `git diff --check`, 변경 Markdown 24개 링크와 표준 Docker WASM build가 통과했다.
- 로컬 `cargo-nextest` 0.9.137이 권장 0.9.140보다 낮다는 경고를 냈지만 전체 실행과
  결과 수집은 정상 완료했다.

GitHub Actions는 작성 시점 code candidate에서 CI·CodeQL·Canvas visual diff·Native Skia·
archive shard·Proptest·adapter inter-diff를 포함한 모든 필요 check가 종료·성공했고,
적용 대상이 아닌 preflight 직후 job만 skip됐다. review thread·review·comment는 0건이었다.

## 위험·후속 범위

- 이 PR은 계측·순위 계약이지 제품 fallback 개선이 아니다. 17개 queue의 exact bytes,
  한컴 설치·미설치 PDF, glyph outline, `hmtx` advance와 첫 조판 divergence는 #4963에서
  별도 계획·승인 후 다룬다.
- 10k aggregate 재생은 실행 시간과 private 입력 보유가 필요하다. 이 PR은 동일 입력·
  실행 계보에서 r2/r3 byte exact와 checkpoint chain을 고정했으며, 추가 전수 재계측은
  변경 없는 재검증으로 보아 생략했다.
- 방대한 공개 JSON은 필요한 기계 정본이지만 후속에서 수치를 재산출할 때 계약·
  입력 hash가 달라지면 fail-closed하므로 임의 수정하지 않아야 한다.

## 최종 권고

**병합 후보로 수용 권고한다.** code candidate의 로컬 검증과 GitHub Actions가 통과했고,
제출 문서 누락 1건은 이 trailing commit에서 정정했다. 최종 merge 조건은 self-review·오늘할일
문서만 포함한 trailing commit을 별도 승인 후 push하고, 최신 PR head의 GitHub Actions 통과,
`MERGEABLE` / `CLEAN` 재확인과 메인테이너 merge 승인을 받는 것이다.
