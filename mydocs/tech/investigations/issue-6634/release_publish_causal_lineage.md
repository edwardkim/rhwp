---
kind: investigation
status: active
canonical: mydocs/tech/investigations/issue-6634/release_publish_causal_lineage.md
last_verified: 2026-09-05
---

# #6634 Release publish 자동 기동 원인 계보

## 조사 질문

v0.8.6에서 GitHub Release는 정상 게시됐지만 왜 `Publish All Packages` run은 생성되지 않았는가. 과거
성공 이력은 현재 `Release Binary → package publish` 순서를 보증했는가.

## 증적과 재현 방법

정규화 원문은 [`release_publish_lineage.json`](release_publish_lineage.json)에 보존했다. secret 값이나
private URL은 포함하지 않았다.

- Release: `GET /repos/edwardkim/rhwp/releases/tags/<tag>`
- workflow: `gh run view <run-id> --json event,headSha,createdAt,updatedAt,jobs`
- package 이력: `gh run list --workflow npm-publish.yml`
- 고정 action 입력: `softprops/action-gh-release@3d0d9888...`의 `action.yml`
- workflow blob: 현재 `.github/workflows/release-binary.yml`, `npm-publish.yml`

GitHub Release API의 `author`는 Release를 최초 생성한 계정이며 draft를 실제 게시한 credential을 별도
필드로 제공하지 않는다. 따라서 `author` 하나가 아니라 다음 네 증거를 결합했다.

1. Release 게시 시각이 자산 첨부 job의 앞인지 내부인지
2. package run의 event와 생성 시각
3. release asset uploader
4. action이 실제 사용한 token 기본값과 GitHub의 workflow 재귀 억제 계약

## 관찰

| 군 | tag | Release 게시와 binary release job | package 결과 |
| --- | --- | --- | --- |
| 선행 게시 | v0.8.0 | 게시 22:29:06, attachment 22:41:09 시작 | `release` run 생성 |
| 선행 게시 | v0.8.1 | 게시 12:59:16, attachment 13:10:22 시작 | `release` run 생성 |
| 선행 게시 | v0.8.2 | 게시 15:57:18, attachment 16:08:29 시작 | `release` run 생성 |
| 선행 게시 | v0.8.3 | 게시 20:36:50, attachment 20:51:05 시작 | `release` run 생성 |
| job 내부 게시 | v0.8.4 | attachment 01:33:44~55, 게시 01:33:52 | 자동 run 없음, 수동 복구 |
| job 내부 게시 | v0.8.6 | attachment 03:00:24~34, 게시 03:00:31 | 자동 run 없음, 수동 복구 |

v0.8.0~v0.8.3의 package run은 Release Binary 완료보다 5~15분 먼저 시작했다. 녹색 package 결과는
“다섯 binary가 성공한 뒤 package가 배포됐다”는 증거가 아니다. 오히려 Release 게시가 binary gate를
앞질렀음을 보여 준다.

v0.8.4와 v0.8.6에서는 `Attach to GitHub Release`가 실행되는 10여 초 사이에 Release가 게시됐고 모든
asset uploader는 `github-actions[bot]`이다. 해당 action의 `token` 기본값은 `${{ github.token }}`이며
workflow는 다른 credential을 넘기지 않는다. GitHub는 repository `GITHUB_TOKEN`이 만든 일반 event로
새 workflow run을 만들지 않는다. `release.published`는 예외 목록에 없다.

## 추가로 발견한 exact-SHA 위반

v0.8.6 tag는 `f1f9c6ae58344ee9368996d3543f76b9345cf227`이지만 수동 복구 run #33623579151은
`main@e8800c8def63449808a4092798442652ed460552`에서 실행됐다. 두 commit 사이에는 릴리스 CI hotfix가
있고 다음 두 workflow가 달라졌다.

- `.github/workflows/ci.yml`
- `.github/workflows/gym-release-gate.yml`

제품 package 입력 파일이 같았더라도 공개 provenance의 실행 commit은 승인된 tag와 달랐다. 따라서
“현재 main의 버전 문자열이 같다”는 확인으로 exact release source를 대신할 수 없다.

## 확정 원인

직접 원인은 draft 게시 주체가 메인테이너 credential에서 workflow의 `GITHUB_TOKEN`으로 바뀌었는데도
`release.published` 간접 이벤트를 유일한 package 기동 계약으로 유지한 것이다.

동시에 과거 성공 절차에는 두 번째 결함이 있었다. package publish가 Release Binary 완료를 기다리지 않아
다섯 플랫폼 gate를 우회했다. v0.8.6의 수동 복구에는 세 번째 결함인 tag SHA 이탈도 있었다.

따라서 복구 목표는 trigger 하나를 되살리는 것이 아니다. 다음 세 계약을 함께 세워야 한다.

1. binary 전건 성공 뒤 직접 package 호출
2. 호출 workflow와 checkout을 release tag exact commit에 고정
3. 채널별 idempotency와 부분 재시도

## Stage 1 판정

- 원인: **확정**
- 기존 `release.published` 단독 구조: **폐기 대상**
- 과거 성공 이력의 binary gate 증명력: **없음**
- Stage 2 진입 조건: RED 계약이 위 세 결함과 promotion policy 누락에서만 실패해야 함
