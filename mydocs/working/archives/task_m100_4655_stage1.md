# Task M100 #4655 Stage 1 — 신규 배포 채널 도입 범위 감사

- 이슈: [#4655](https://github.com/edwardkim/rhwp/issues/4655)
- 기준: `upstream/devel@193e26b7ffb0`
- 비교 기준: `v0.8.2`
- 상태: 구현·로컬 검증·원격 devel 통합 완료

## 1. v0.8.2 공식 채널

`v0.8.2`의 `publish_guide.md`와 workflow tree를 교차 확인했다. 자동·수동 배포 대상은 GitHub Pages,
GitHub Release 바이너리, npm `@rhwp/core`·`@rhwp/editor`, VS Code Marketplace/Open VSX,
Chrome·Edge·Firefox 확장이었다.

## 2. v0.8.2 이후 추가된 제거 대상

| 표면 | 최초 도입 | 현재 상태 |
| --- | --- | --- |
| PyPI·`@rhwp/node` | #4337, contributor `kevin9327` | 태그 workflow가 게시 시도 |
| Scoop·Homebrew·Winget | #4339, contributor `kevin9327` | 매니페스트·갱신 도구 존재 |
| GHCR Docker CLI | #4372, contributor `kevin9327` | v0.8.3 이미지 실제 게시 |
| 설치용 GitHub Action | #4373, contributor `kevin9327` | action과 3-OS self-test 존재 |
| deb/rpm/MSI·스크립트·AUR·binstall·server.json | #4376, contributor `kevin9327` | deb/rpm 실제 첨부, MSI 실패 |
| Python·Node 개발 표면 | #3762·#3776 이후 | 전용 코드·문서·workflow가 유지보수 범위에 포함 |

원 PR들은 개별 merge 대신 collaborator `jangster77`의 일괄 통합 PR #4496에 체리픽됐다. 배포 채널
채택 의미가 대규모 통합과 공식 배포 가이드 사이에서 연결되지 않았고, 일부 workflow는 태그 push나
기존 secret/GITHUB_TOKEN을 승인으로 간주했다.

## 3. 메인테이너 결정

v0.8.4에서는 v0.8.2 공식 채널만 남긴다. 신규 채널용 개발 자산과 테스트도 함께 제거한다. 향후 신규
채널은 명시적 채택과 안전 검증 없이는 merge 또는 tag만으로 활성화되지 않아야 한다.

## 4. 통합 예외

일반 source·CI 변경은 PR이 기본이나, 작업지시자가 이번 #4655를 PR 없이 메인테이너 권한으로 처리하도록
명시했다. 작업 브랜치에서 diff와 검증 결과를 고정한 뒤 로컬 devel에 통합하며, 원격 push 전에는 실제
대상 SHA와 작업공간 상태를 다시 확인한다.

## 5. 구현 결과

- PyPI·`@rhwp/node`·GHCR CLI Docker·설치 패키지·GitHub Action·MCP registry의 게시 workflow와
  전용 개발·테스트·문서를 제거했다.
- `npm-publish.yml`, `release-binary.yml`, `deploy-pages.yml`과 브라우저·VS Code 확장 경로는 현재의
  보안·재실행 보완을 보존한 채 유지했다.
- v0.8.2 채널 자산이 사라지거나 철회 채널이 다시 추가되면 실패하는
  `test_release_channel_policy_workflow.py`를 CI 계약에 추가했다.
- 패키지와 확장 버전을 v0.8.4로 정렬하고 한국어·영어 CHANGELOG에 채널 복원 결정을 기록했다.
- Studio About, Chrome·Edge와 Firefox 옵션·개발자 도구가 표시하는 버전 원천을 v0.8.4로
  정렬하고, 루트 릴리스 버전과 표시 배선의 정합을 workflow 계약으로 고정했다.
- 빌드 과정에서 발견된 로컬 생성물은 삭제하지 않고 `/tmp/rhwp-4655-node-generated/`로 이동했다.
  저장소 추적 대상이나 배포 산출물에는 포함되지 않는다.

## 6. 로컬 검증

| 검증 | 결과 |
| --- | --- |
| workflow 계약 테스트 | 66개 통과 |
| Markdown 링크·redirect 검사 | 561개 문서, 오류 0 |
| 문서 메타데이터 검사 | 547개 문서, 오류 0 |
| 로드맵 진행률 정합성 | 통과 |
| `cargo fmt --all -- --check` | 통과 |
| `cargo clippy --all-targets -- -D warnings` | 통과 |
| schema·provenance focused Rust 테스트 | 통과 |
| 전체 Rust nextest | 5,767/5,767 통과, 36 skipped |
| Docker WASM 빌드 | 통과, `pkg` v0.8.4 및 사용자 소유권 확인 |
| `npm/editor` 테스트 | 8/8 통과 |
| `rhwp-studio` 테스트 | 샌드박스 밖 재검증 846/846 통과, 1 skipped |
| `rhwp-studio` 프로덕션 빌드 | 통과 |
| VS Code 확장 compile | 통과 |
| Chrome·Firefox 확장 build | 각각 통과 |

Studio 테스트의 최초 샌드박스 실행에서 나타난 6건은 `spawnSync`가 `EPERM`으로 차단되어 자식
프로세스 JSON이 비어 발생한 알려진 실행 환경 오탐이다. 동일 작업공간을 샌드박스 밖에서 즉시
재실행해 제품 테스트 실패가 0건임을 확인했다.

## 7. 원격 통합 결과

- 배포 채널 복원: `7342dda65`
- 사용자 표시 버전 및 정책 계약: `5dbe99f4f`
- `devel` CI: [run 31547394146](https://github.com/edwardkim/rhwp/actions/runs/31547394146) 성공
- `devel` CodeQL: [run 31547393914](https://github.com/edwardkim/rhwp/actions/runs/31547393914) 성공
- 이슈 #4655: 구현과 검증 근거를 반영한 뒤 종료

v0.8.4 릴리스는 별도 PR을 만들지 않고 메인테이너 직통 절차로 진행한다. `main` 반영과 태그·공식
채널 게시 결과는 릴리스 후속 기록에서 확정한다.
