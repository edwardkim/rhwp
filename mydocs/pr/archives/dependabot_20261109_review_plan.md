# Dependabot PR 19건 검토 계획 — 2026-11-09

## 범위와 승인

작업지시자가 Dependabot으로 등록된 PR 처리를 지시했다. 열린 Ready PR #8811–#8829 19건의 접수·로컬 검토·적용 검증을 진행한다. 원 PR별 reviewer edwardkim을 지정하고 API로 확인했다. 원격 push·새 PR·공개 댓글·merge·close와 CI 재실행·workflow 보정 적용은 각각 기존 원격 승인 절차를 따른다.

- 기준 devel: `024d610633ede166ebcbb915f9c90e7a9d0649d8`.
- 작업 브랜치: `review/integration-20261109-dependabot-updates`.
- 원 head: `refs/rhwp-review/dependabot-20261109/pr-<번호>`에 보존.
- 역할: maintainer_general + intake_and_review·local_validation·multi_pr_update_branch·rework_and_exceptions·visual_fixture_evidence·post_merge. GitHub 운영 정본도 적용한다.

| PR | 변경 | 고정 head |
| --- | --- | --- |
| #8811 | chore(deps): bump zip from 9.1.3 to 9.4.0 | `73f6b8c1f431dc36de0d4b17ab5ce2bf423be5ca` |
| #8812 | chore(deps-dev): bump vite from 8.3.5 to 8.3.6 in /rhwp-studio in the vite-stack group | `bb3d70c892bfffb5ee59d7570328ee86186ec5e4` |
| #8813 | chore(deps): bump softprops/action-gh-release from 3.0.3 to 3.0.4 | `5a98e6d56974481e831eb232f5c20631b48da087` |
| #8814 | chore(deps): bump log from 0.4.34 to 0.4.35 | `efd1a65979cfe42ced6649f7118d1d65dbecaad6` |
| #8815 | chore(deps-dev): bump vite from 8.3.5 to 8.3.6 in /rhwp-firefox | `384d609211f9caa0408d4912c1a1e8f23c872a07` |
| #8816 | chore(deps): bump @anthropic-ai/sdk from 1.3.0 to 1.6.0 in /rhwp-studio | `5d8b73a5f75482af8f1ee770f5ca698fcca1d00b` |
| #8817 | chore(deps): bump cfb from 0.15.0 to 0.16.0 | `2fb2264a8242539611a424e53c722651166301f6` |
| #8818 | chore(deps): bump github/codeql-action/analyze from 4.39.1 to 4.39.3 | `9b4ead9100f821bbcfe2331a3bf2916b96b575fb` |
| #8819 | chore(deps): bump unicode-properties from 0.1.4 to 0.1.5 | `526ef018fa216d42808a24b6f522ac6bad4bfdbd` |
| #8820 | chore(deps): bump github/codeql-action/init from 4.39.1 to 4.39.3 | `c607ea643619fced34132144e07d43d4f9637c6e` |
| #8821 | chore(deps): bump taiki-e/install-action from 2.88.1 to 2.90.2 | `6bf1774453978c45d954c5cd68a7977ae6f293fb` |
| #8822 | chore(deps): bump ml-kem from 0.3.3 to 0.3.4 | `4153bd5a09960fa69bf45e15d092b5b6861c985d` |
| #8823 | chore(deps): bump ml-dsa from 0.1.1 to 0.1.2 | `0de0b5e3ede6df2d257aaa62308521a25ea3aced` |
| #8824 | chore(deps): bump svgtypes from 0.16.1 to 0.16.2 | `7f26c047348a920fd2397a8b20c4edfaa0c8dd46` |
| #8825 | chore(deps): bump crc32fast from 1.5.3 to 1.5.4 | `a705ee565a3985b560db58167e70b5678285ddd6` |
| #8826 | chore(deps): bump ureq from 3.4.0 to 3.5.1 | `a55a1ed94a3c26b480418d3898c8dbd20b22ee6b` |
| #8827 | chore(deps): bump encoding_rs from 0.8.42 to 0.8.44 | `bb4981791b116b393e3f432c14cdf261304f7a69` |
| #8828 | chore(deps-dev): bump webpack-cli from 7.2.3 to 7.3.1 in /rhwp-vscode | `d65c82058e7caf6bc8e330785328563d88513475` |
| #8829 | chore(deps-dev): bump vite from 8.3.5 to 8.3.6 in /rhwp-chrome | `24ed2188056425b95b7c70b255c2e7d0ee4a447b` |

## 조사와 로컬 검증

1. 최신 devel에서 기존 해결·도입 PR과 운영 계약을 추적한다. CodeQL #7851/#7854의 init/analyze 동시 갱신과 기존 Dependabot 그룹을 먼저 대조한다. 이번 19건의 누적 merge-tree는 모두 종료 코드 0이며 충돌이 없다.
2. 실패한 각 원 head CI 로그·annotation·명령을 대조한다. 제품·운영 계약 오류와 checkout/다운로드/시간 초과를 분리하고, 검사·기대값·보안 탐지를 완화하지 않는다. 실패한 run을 임의 재실행하거나 취소하지 않는다.
3. 원 commit을 보존해 오래된 순서로 로컬 merge한다. CodeQL init/analyze는 같은 action SHA로 함께 적용해 실제 버전·tag·댓글과 소비 계약을 대조한다. 새로운 충돌·보정 범위가 확인되면 정확한 방침을 제시한다.
4. Rust의 직접·간접 lock graph, checksum·MSRV·feature/API와 실제 소비 지점을 대조한다. cfb/zip은 열기·저장·재열기, encoding/Unicode/SVG는 원본과 실제 출력, 암호 crate는 기존 보호·해제 회귀를 검증한다. 실제 graph 변경 범위의 focused·전체 release-test·Native 공식 3종·적용 lint와 workspace 검증을 완료한다.
5. npm 5건은 네 manifest 디렉터리의 Vite와 Studio SDK·VS Code webpack-cli를 함께 대조한다. 실제 lockfile의 전이 변경도 확인하며 dependency-audit의 변경 전후 취약점·설치 스크립트 조사를 적용한다. Studio TypeScript·unit·실제 AI/편집 브라우저, Chrome/Firefox 확장과 VS Code build를 검증한다.
6. 같은 source의 최적화 fresh WASM을 브라우저에서 실행한다. 독립 Print PDF와 Native/fresh WASM 전쪽 TSV 및 base/후보 출력을 대조하고 변경 영역을 직접 판독한다. 조판 생산자를 직접 바꾸지 않아도 의존성의 실제 출력 후퇴가 있으면 보정·재검증한다. 편람 gate 적용 여부는 소비 경로 변화로 판정한다.
7. Action 4건은 공식 tag의 full SHA·action.yml·Node runtime·최소 권한과 실제 설치/분석/배포 소비 계약을 대조한다. 구문·기존 workflow/정책 검사와 적용 명령을 검증하고, 릴리즈 게시를 로컬 검증으로 실행하지 않는다.
8. 원 PR별 판정·검증 SHA·실행 증거를 기록한다. 검증 중인 항목을 완료로 쓰지 않는다. 새 통합 head를 제출하면 원 head의 성공 CI와 구분하며, 원격 실행안은 로컬 검증 완료 뒤 제시한다.

## 후속 기록과 종료

실행 중 Chrome 확장 smoke의 기존 대기 순서 오류를 확인했다. #8757의 메인테이너 보정
`b2b0b9f8ad3564fc2412a24ce9d7db3eaa63960c`가 추가한 글꼴 선택 처리보다 canvas 대기가
앞에 남아 있다. 기준 devel과 후보의 해당 테스트 파일은 같았고 Chrome 156·Chromium 166에서
로컬 글꼴 감지 대화창을 띄운 채 실패했다. `main.ts`의 문서 초기화는 글꼴 선택을 먼저
기다리므로, 기존 canvas 대기 한 줄을 실제 선택 뒤로 옮겨 전 과정을 재검증한다. 제품 코드·
선택 대화창·기존 assertion·시간 한도는 유지한다. 이 보정은 검증 하네스의 순서 수정이며,
이미 통과한 Rust·Native·WASM·시각 검증과 별도로 실제 수정한 smoke 경로를 검증한다.

전체 통합 diff에 CI workflow Action 변경이 포함되므로 실제 CI·self-review·merge 뒤 review·오늘할일·운영 증거는 post_merge 7.1.1의 **옵션 2: 별도 후속 기록 PR**로 반영한다. 문서 trailing push로 전체 CI를 반복하지 않는다. 원 PR의 코드·workflow를 바꾸는 추가 보정은 해당 범위의 승인과 검증을 따른다.

기본 작업공간과 다른 #8296 worktree·설치 도구·로컬 crates.io 검증 환경을 보존한다. 승인된 원격 후속 처리가 끝나면 이번 전용 branch/worktree, primary output 전체와 미사용 target/pr-review/debug를 정리한다.

## 실행 결과

[통합 PR #8840](https://github.com/edwardkim/rhwp/pull/8840)의 최신 head CI·self-review 후 원 기여를 보존하는 merge로 수용했다. [실제 원격 결과](../assets/dependabot-20261109/remote-completion.md)를 따른다.
