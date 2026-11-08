# PR #7954 — rhwp-q 통합 self-review

## 최종 판정

**승인**. 코드 후보 `c068f0b094e2985e9e2cec83eb83b8618bf705ee`의 로컬 검증과 실제 GitHub CI가 통과했다.
local 검증 commit `9acd01745de69c09b0d173abd5aa80e1f9b3fd74` 이후 후보까지는 실측 기록만 추가했으며
Rust/Cargo/scripts/tests의 동일성을 확인했다. 남은 절차는 이 review·오늘할일 trailing head의 CI 확인과
normal merge·post_merge다. 아직 실행 전인 trailing CI와 merge를 성공으로 기록하지 않는다.

## 접수와 역할

- [PR #7954](https://github.com/edwardkim/rhwp/pull/7954), 작성자 `jangster77`의 **작성자 self-review**다. reviewer를 지정하지 않았다.
- base `devel`: `552514f423ec91f1b7171f6e6b0882a4efe034f2`.
- source branch `task_m100_7952`, same repository `edwardkim/rhwp`, Open/non-draft.
- [Issue #7952](https://github.com/edwardkim/rhwp/issues/7952)를 전체 해결하며, [#7001](https://github.com/edwardkim/rhwp/issues/7001)의 조사 기록을 참조했다.
- milestone `v1.0.0`, labels `cli`, `performance`, `documentation`, assignee `jangster77`.
- 코드 후보 기준 4 commits / 133 files / +2,948 / -596. 대부분 기존 소스 이동·조회 사용법 선언 공유와 문서다.
- 규모가 1,000줄을 넘으므로 대형 PR modifier로 별도 코드 검토·전체 회귀·merge simulation을 수행한다. normal merge 경로다.
- 작성 시 API 상태는 mergeable=true / clean이며, merge 직전에 최신 SHA와 check를 재조회한다.

base route: collaborator_self_merge.md
modifiers: intake_and_review.md, local_validation.md, rework_and_exceptions.md, review_only_fast_pass.md, post_merge.md
loaded documents: pr_review_workflow.md와 위 기본·보조 문서, docs_and_git_workflow.md, dev_environment_guide.md.

## 범위와 코드 검토

| 변경 | 실제 확인 |
| --- | --- |
| Cargo target | 기존 루트 26개·q 23개를 루트 4개·q 1개로 통합. `rhwp`, `rhwp-agent`, `font-metric-gen` 역할을 유지한다. `autobins=false`와 명시적 진입점으로 등록한다. |
| argv 전달 | dispatcher의 `fn(Vec<String>)`이 기존 handler에 남은 인자를 직접 전달한다. 하위 실행 파일 재실행이나 전역 argv 교체 경로가 없다. |
| 조회 의미 | 20개 단건과 kit 50개, more/pack 각각 51개의 기존 호출을 보존한다. more/pack 동명 volume-probe의 서로 다른 의미를 유지한다. |
| 출력·오류 | JSON의 legacy tool 식별자·기능 데이터와 정상/사용법 오류 의미를 유지한다. 사용자 요청으로 상세 도움말을 추가했고 root stdout BrokenPipe도 exit 0으로 보정했다. |
| 도움말 | parser의 USAGE와 kit 목록/상세 안내가 같은 선언을 쓴다. 누락됐던 필수 주소·검색/바이트 옵션을 공개한다. more/pack의 현재 반환 필드와 필터 제한을 명시한다. |
| 호출 연결 | 기존 22개 q 계약과 8개 Node/Python 호출자의 실행 경로·선행 selector를 이관했다. nextest archive용 runtime CARGO_BIN_EXE override를 보존했다. |
| 등록 계약 | 과거 explicit bin=2 핀 8개를 승인된 4개 이름/진입점과 자동 발견 금지의 공용 계약으로 보정했다. 추가·교체·자동 발견 활성화 반례와 설명용 주석 대조군을 검증했다. |
| profile | dev debug=1로 함수·파일·줄 정보를 남기며 release/release-test 설정은 유지한다. release 사용을 기본으로 문서화했다. |

[상세 사용법](../../manual/rhwp_query_cli.md), [구현](../../working/task_m100_7952_stage1.md),
[등록 계약 보정](../../working/task_m100_7952_stage2.md), [최종 검증](../../working/task_m100_7952_stage3.md)에 실제 근거를 연결했다.
새 코드 파일은 1,000줄 이하다. 기존 1,000줄 초과 파일은 이동·사용법 접두어/호출 인자 수준으로 한정했다.

## 검증 결과

- 최종 local 전체 nextest **10,459 PASS / 0 FAIL / 45 skipped**, focused CLI·스킬 **227/227 PASS**.
- format·native Clippy·WASM32 lib Clippy·workspace build·workspace all-target Clippy·base 기준 suite 정책 PASS.
- Node 호출 18개·Python 호출 44개·suite 도구 Node 23개 및 기존 unit tier 정책 PASS.
- release 실제 빌드·172개 개별 도움말·매뉴얼 HWP 13예제와 HWPX 조회 PASS.
- 기존 캐시 도구와 사용법 오류·JSON 32건이 일치했다. 도움말은 의도적으로 상세화했다.
- 첫 전체 실행의 8개 FAIL은 configuration 검사였고 renderer 결함이 아니었다. 보정 후 동일 8개와 반례를 포함해 최종 전체를 통과했다.
- release 15,044,584 bytes, SHA-256 `dac55f809ba5ed41490af30301b45660d8713c675dc33fccf0f84cf47d08df24`.
  dev 198,702,248 bytes. 기존 캐시와의 크기 관측은 통제된 동일 소스 A/B 또는 실제 여유 공간 증가가 아니다.
- Windows 구문을 안내했으나 실제 실행 검증은 Linux에서 했다. 외부 호출자는 기존 실행 이름을 새 selector 구문으로 이관해야 한다.
- 기존 shared target과 user dirty Studio JS를 보존했다. 사용자 JS는 PR diff에 포함하지 않았다.

## 실제 GitHub CI

아래는 모두 위 code candidate의 같은 repository·source branch에서 실행된 결과다. workflow와 GHAS를 별도로 확인했다.

| Workflow | 실제 결과 |
| --- | --- |
| [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/38935178504) | completed / success |
| [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/38935178558) | completed / success |
| [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/38935178506) | completed / success |
| [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/38935178452) | completed / success |
| [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/38935178404) | completed / success |
| [CI](https://github.com/edwardkim/rhwp/actions/runs/38935178448) | completed / success |

동일 SHA의 GitHub Advanced Security `CodeQL` check도 success이며 pending/failure check는 없었다.
CI의 Native Skia·Frontend package·모든 archive build/test worker와 최종 Build & Test가 success다.
별도 WASM Build와 frontend unit 등 영향 없는 job의 skipped는 실제 workflow 결과와 구분한다.

## 조판 원칙 준수 검토

**비해당**: 변경의 생산·소비 경로는 실행 타깃·명령 인자·도움말·호출자다.
DocumentCore, 저장 정보, renderer/typeset/layout/paint, font 공급 및 WASM 구현을 바꾸지 않았다.
새 검사는 문단/개체 배치 품질을 주장하지 않으며 기존 렌더링 px·쪽수·golden/baseline 기대값도 바꾸지 않았다.
따라서 Native/fresh WASM Visual Sweep·0.01%p 비교·완료 페이지 좌표 핀·Print PDF 판독은 이번 diff에서 비해당이다.
CI 통과를 시각 품질 증거로 대신하지 않는다.

## 입력 커밋 확인

기존 `samples/form-01.hwp`, `samples/hwpx/form-01.hwpx`, `samples/pic-crop-01.hwp`,
`samples/equation-lim.hwp`는 검증 commit의 blob과 실제 입력 바이트가 동일하다.
SHA-256은 [최종 검증](../../working/task_m100_7952_stage3.md)의 표에 기록했다. 새 문서/PDF fixture는 없다.
등록 반례는 테스트 안에서 Cargo 문자열을 합성하므로 별도 fixture 파일이 필요 없다.

## Merge 후 contributor PR comment 계획

self PR에도 실제 merge SHA와 최종 trailing head CI 결과, local 10,459/227 및 lint 통과 사실을 존댓말로 남긴다.
첫 8개 configuration 실패와 stricter 등록 계약 보정이 필요했던 이유를 사실대로 구분한다.
조판·시각 비교는 비해당이며 score나 이미지를 만들어 넣지 않는다.
최종 devel의 이 review·매뉴얼·검증 문서를 merge SHA 고정 링크로 연결한다.
[#7952](https://github.com/edwardkim/rhwp/issues/7952)의 close 상태를 확인하고 같은 merge·증거를 담은 maintainer comment를 한 번 남긴다.
원본 HWP/HWPX는 기존 저장소 경로를 유지한다. raw 로그는 로컬 영구 `pdf/pr-review-evidence/issue7952-20261017/`에 보존한다.
duration metadata 갱신 결과, exact head lease에 따른 임시 remote branch 제거, local branch/worktree 제거,
사용자 자료와 shared target 보존, `output/` 전체 부재 확인 뒤에만 후속 완료로 보고한다.
