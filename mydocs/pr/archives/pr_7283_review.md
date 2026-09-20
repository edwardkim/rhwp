# PR #7283 self-review — Chrome 확장 E2E와 영향 기반 CI

## 검토 경로와 metadata

- base route: `collaborator_self_merge.md`
- modifiers: `intake_and_review.md`, `local_validation.md`, `review_only_fast_pass.md`, `rework_and_exceptions.md` (1,000줄 초과)
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`, 위 기본·보조 문서, `github_operations.md`, `docs_and_git_workflow.md`, `dev_environment_guide.md`
- 작성자 self-review이며 별도 reviewer는 지정하지 않았다. 코드·테스트·운영 정책을 로컬 검증과 원격 실행으로 나누어 검토했다.

| 항목 | 작성 시점 참고값 |
| --- | --- |
| PR | [#7283](https://github.com/edwardkim/rhwp/pull/7283), Open, postmelee |
| base | `devel`, `517df04ba110dc108fbbb203e72229c51819b591` |
| 원격 code candidate | `8d1eb08a4f87cb54bccb6897c84f9d877bfe8fa5` (중앙 배선·Linux 실행 환경 보완) |
| 규모 | 최초 게시 32파일, +1,660 / -94. source/test/workflow와 설명·검증 기록 포함 |
| mergeability | `MERGEABLE`, `BLOCKED`. 최신 required check와 merge 조건은 병합 직전 재조회 필요 |
| 이슈 | Epic #3512, #3513 → #3515. 기존 #3514/#5912와 #7279 회귀 검사 재사용 |

## 변경 범위와 호출 경로 검토

- 실제 production dist에서 options UI 저장·재진입, worker 종료·재기동, 같은 격리 profile 재시작,
  HWP/HWPX 다운로드 완료·바이트와 viewer 0/1개, 과거 다운로드 기록 보존을 검사한다.
  `monitorTabs`는 생성 이력을 보존하여 잠깐 열렸다 닫힌 두 번째 탭도 검출한다.
- CI는 trusted base의 Chrome 영향 분류 → frontend package 승격 → fresh WASM/dist artifact ID →
  Chrome job → `Build & Test` 집계로 연결한다. trusted policy도 같은 영향 함수와 예상 job 결과를
  소비한다. 분류 누락·잘린 목록·불확실성은 실행으로 처리하고 fast-pass는 Chrome skip을 요구한다.
- `run-ci.mjs`는 세 suite 전체를 한 번씩 실행하며 case/repeat 환경변수로 검사 축소를 허용하지 않는다.
  220초 전체 예산, 단계별 timeout, 실패 전파와 소유 profile 정리를 확인했다.
- PR browser cache는 exact lock key 복원 전용이다. 신규 기본 브랜치 수동 workflow의 cache 저장은
  별도 실행 경계이며 최초 승격은 CI 설치 실행과 cache 분기 계약의 contracts-only adapter를 쓴다.
- promotion collector가 하나의 CI run을 두 workflow의 증거로 연결할 때 기존 단일 키 덮어쓰기를
  수정했다. 각 subject의 hash/mode를 보존하고 offline verifier의 실패 조건은 유지했다.

제품 source, 확장 권한·CSP, Rust source/test, 조판 baseline은 바뀌지 않았다. 조판 원칙·Visual Sweep은
**비해당**이다. 이번 주장은 설정 유지·탭 개수와 CI 실행 계약이며 PDF 조판 일치가 아니다.

## 검증 입력 커밋 확인

판정 **충족**. 기존 tracked fixture를 재사용했고, 아래 실제 실행 파일과 candidate `git show` 내용의
바이트·SHA-256이 같음을 확인했다. 앞 두 입력은 실제 HWP/HWPX 다운로드 및 viewer smoke, 마지막은
기존 #7279 초기 다운로드 상태 경합 검사에 사용했다. 새 fixture·기준 PDF는 추가하지 않았다.

| 저장소 경로 | SHA-256 |
| --- | --- |
| `samples/hwp3-pagedef-1915.hwp` | `b272fdd218b4e91355167e63438a1605ef6902d75970c4ff5b8bae67087122d0` |
| `samples/hwpx_sample2.hwpx` | `188bdfe21f89e117d8897f4102aa6f741962b23a3019ad2aaf7bdc222d90fdb2` |
| `samples/re-font-dotum-empty-hancom.hwp` | `1ee8871f37bec2e97d0928709dc411c0eacad656f35aa3d92c5cf89f61c5761b` |

확인 commit은 `8d1eb08a4f87cb54bccb6897c84f9d877bfe8fa5`이다. 저장소에 있던 입력을 다운로드 fixture로
재사용한 것이며 한컴 기준 출력과의 시각 일치를 주장하지 않는다.

## 완료한 로컬 검증

[정식 보고서](../../report/task_m100_3512_report.md)와 [회차별 검증 기록](../../report/3512-extension-e2e/validation.json)에
명령·source SHA·시간·mutation·최초 실패 기록을 연결했다.

- 최신 기준의 fresh WASM 및 production Chrome dist 빌드가 통과했다.
- 전체 smoke/download/lifecycle 10회 연속 통과, retry 0. 평균 46.874초, 범위 46.126~48.315초였다.
- 기존 확장 Node 174개, CI·harness Node 174개, Python workflow/promotion 계약 91개가 통과했다.
- freshness 제거는 Node 상태 계약에서 3건 실패, 중복 방어 제거는 실제 Chrome에서 viewer 2개를 검출했다.
  과거 다운로드 E2E 자체가 freshness 제거를 검출한다고 해석하지 않았다.
- 실패를 유도한 smoke에서 exit 1, 최소 JSON/PNG 생성과 원본 복원을 확인했다.
- 변경 workflow의 Actions 구문을 통과했다. ShellCheck SC2016 info 1건은 기존 base에서도 재현됐으며
  새 오류와 구분했다. 새 Rust 변경이 없어 전체 Rust lint를 반복하지 않았다.
- 최종 browser source는 `8d1eb08a4f87cb54bccb6897c84f9d877bfe8fa5`이다. console 출처 구분과 fixture HTML 보완 이후 전체 로컬 10회와 Linux 3회를 다시 검증했다. production dist와 tracked fixture 바이트는 동일하다.

동작 기반 회귀 검증은 **충족**이다. 실제 options·download 진입점과 worker 재기동 경계, 불확실한
분류 fallback, 0/1/초과 탭 결과, 정상 대조군 및 방어 제거 음성 대조를 위 증거에서 확인했다.

## 원격 검증과 남은 조건

[첫 CI](https://github.com/edwardkim/rhwp/actions/runs/35490767626)의 중앙 배선 검사에서 새 Chrome 계약 검사 등록 누락을 검출했다. lint job 연결과 검사 목록을 보완했고 관련 94개 및 전체 workflow 계약 248개가 통과했다. 전체 checkout의 98초 준비 시간을 확인해 Chrome 소비 job과 cache 준비 job에 필요한 파일만 checkout하도록 좁혔다. 첫 run은 수정 push 뒤 stale 취소됐으며 성공 회차에 포함하지 않는다.

두 번째 CI에서는 runner 단위 테스트의 stdout/stderr 도착 순서 가정을 검출했다. 같은 pipe에 payload와
정리 marker를 기록하도록 `3e948d8de`에서 보완했고 guard 8개가 통과했다. 해당 실행의 배포 artifact를
macOS Chrome에서 추가 실행해 전체 suite 49.407초 통과 및 원래 dist 복원을 확인했다.

세 번째 CI의 Chrome launch는 Ubuntu sandbox 제한으로 실패했다. `17886c336`에서 기존 root 소유
sandbox helper를 연결했고 관련 계약 78개가 통과했다. 실제 실패 artifact의 JSON/LOG 3개·2,322바이트와
API digest를 확인했다. 상세 원인과 Chromium 근거는 정식 보고서에 연결했다.

기존 sandbox helper의 사전 조건이 충족되지 않아, 최종 후보는 확인된 CfT 실행 파일 하나에만
AppArmor user namespace 권한을 적용하고 종료 시 profile을 제거한다. 실행 경로 조회는 비동기 API를
기다리며 로컬 명령 실행으로 확인했다. GitHub token 권한·제품 코드·browser assertion은 유지했다.

`fcb630d5e`의 Linux 실행은 sandbox 설정·해제, smoke/download와 lifecycle 9개가 통과했다.
과거 다운로드는 UI·탭 검사를 마친 뒤 Chrome 내부 화면의 console error로 실패했다. `cb2b4eea0`에서
source URL이 명시적인 `chrome://` 오류만 로그에 보존하고 확장·fixture·불명확한 출처 오류는 실패로
유지했다. 실제 확장 오류 주입 시 exit 1, 원복 시 exit 0을 확인했다. 화면 표시와 0/1 탭 assertion은 동일하다.

[최종 후보 CI](https://github.com/edwardkim/rhwp/actions/runs/35496249526)에서 Chrome job을 3회 연속 통과했다.
첫 full CI 성공 뒤 동일 artifact를 소비하는 성공 job을 두 번 반복했으며 실패 재시도는 없다. 각각의
job ID·원격 시간·실제 checkout SHA는 [정식 보고서](../../report/task_m100_3512_report.md#최종-linux-연속-검증)에
기록했다. cold job 실측과 미검증인 shared cache seed/hit을 구분한다.
초기 전체 로컬 반복의 9회차 다운로드 취소는 실패로 보존했고 원인은 미확정이다.

운영 등급 O3. rollback은 CI/harness 변경을 되돌리는 devel 대상 PR이다. required context 이름과
branch push 실행 정책은 유지했다. Firefox는 별도 설치·worker 구현이 필요해 Chrome Phase 1 통합과 기본 브랜치 운영 적용 뒤로
보류하며 부모 이슈에 근거를 남겼다. `확장 E2E 증적 보고서 연동`은 Chrome Phase 1 통합 완료 이후의 후속 이슈다.


이전 후보의 [첫 Chrome 성공 job](https://github.com/edwardkim/rhwp/actions/runs/35495528465/job/106038714425)은
실제 source가 `chrome://fileicon/`인 두 오류를 기록하면서 전체 suite를 51.192초에 통과했다.
해당 URL은 격리된 테스트 다운로드 파일의 Chrome 내부 아이콘 요청이다. 실제 제품 오류를 무시한
결과가 아니며, 원래 오류 출처가 없던 실패 로그의 원인을 직접 확정한 증거와도 구분한다.

CodeQL의 [경고 #206](https://github.com/edwardkim/rhwp/security/code-scanning/206)은 fixture 페이지가
등록 여부를 검사한 query 값을 HTML에 직접 넣는 경로를 지적했다. `8d1eb08a4`에서 서버가 소유한
등록 키를 찾아 URI encoding 후 출력하고, 미등록 요청은 404로 반환한다. 제품 source는 동일하다.
실제 과거 다운로드 대조 검사를 통과했으며, 이 최종 후보에서 로컬 10회와 Linux 3회 연속 검증을 다시 통과했다. CodeQL 재검사도 통과했다.
이전 `cb2b4eea0`의 전체 CI/Chrome 성공은 참고 기록이며 최종 후보의 통과 횟수에 포함하지 않는다.

## 최종 판정

기능·회귀·Linux 반복 실행 증거는 **충족**이다. 제품 source·조판 변경은 **비해당**이며,
shared cache seed/hit은 기본 브랜치 workflow 등록 이후의 운영 확인으로 **미검증**이다.

**머지 보류** — code candidate의 구현·필수 원격 검증은 완료했다. 이 기록을 포함한 trailing head의
merge tree·링크·오늘할일 보존과 최신 required checks를 확인하고 작업지시자 승인을 받아야 한다.
최종 trailing head의 SHA·merge tree 검증·CI 결과는 PR 본문에 연결한다. 실제 merge·이슈 close 및
Chrome Phase 1 이후 증적 보고서 후속 이슈는 승인된 통합 절차에서 수행한다.
