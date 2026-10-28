---
kind: snapshot
status: active
canonical: mydocs/plans/task_m100_6634.md
issue: 6634
last_verified: 2026-09-05
---

# #6634 Stage 5-A 완료보고 — 로컬 배포 산출물 검증

## 검증 경계

Stage 5-A는 `task_m100_6634@f8f1a784fbd8af354f7ffa3c3f61bd043e3f3dda`의 workflow와
package build 경로를 로컬에서 검증했다. 외부 registry·Marketplace·Open VSX에 게시하지 않았고 GitHub
workflow dispatch, push, Release 변경도 수행하지 않았다.

Rust 제품 source는 바뀌지 않았으므로 renderer·시각 회귀는 반복하지 않았다. workflow 실행계약의 정적
검사와 실제 배포 산출물인 최적화 WASM, npm package 2종, VSIX를 검증 범위로 삼았다.

## 실행 환경

| 항목 | 값 |
| --- | --- |
| 관찰 시각 | 2026-09-05 23:37 KST |
| Docker | client/server 29.7.2 |
| Node.js | v24.15.0 |
| npm | 11.12.1 |
| actionlint | v1.7.7, linux/amd64 |
| 격리 | detached 임시 worktree; 검증 뒤 worktree와 임시 파일 제거 |

WASM은 개발환경 가이드의 표준 Docker `wasm` 서비스를 사용했다. `--no-opt` 진단 경로가 아니라
`scripts/wasm-pack-locked.sh --target web`의 release build와 `wasm-opt -O`까지 완료했으며 총 소요 시간은
6분 53초였다.

첫 시도에서는 `.env.docker.example`의 UID/GID 1000을 그대로 사용해, 현재 WSL 사용자의 UID/GID 1002와
불일치한 `pkg/`가 생성됐다. WASM build 자체는 성공했지만 `prepare-npm.sh`가 쓰기 권한 오류로 중단됐다.
불완전 임시 worktree와 그 전용 Compose volume/network를 제거한 뒤 실제 UID/GID를 명시해 전체 경로를
처음부터 다시 실행했다. 제품·workflow 결함이 아니라 검증 격리 환경 설정 오류였으며 두 번째 실행에서는
소유권 오류가 재현되지 않았다.

## 정적 계약 검증

| 검증 | 결과 |
| --- | --- |
| `python3 -m unittest discover -s scripts/tests -p 'test_*workflow.py'` | 159건 PASS |
| #6634 release·channel·promotion 핵심 모듈 | 62건 PASS |
| actionlint 1.7.7, 두 변경 workflow | PASS, 진단 없음 |
| `git diff --check` | PASS |

존재하지 않는 개별 파일명으로 release guard/evidence test 두 모듈을 한 번 잘못 지정해 import error가
발생했다. 두 구현의 test는 별도 파일이 아니라 `test_release_publish_orchestration.py`와
`test_release_channel_status.py`에 들어 있다. 올바른 네 모듈을 지정해 62건을 다시 통과시켰으며, 전체
workflow discovery 159건도 이미 성공한 상태다.

## 실제 package 결과

| 산출물 | 크기(byte) | SHA-256 |
| --- | ---: | --- |
| `pkg/rhwp_bg.wasm` | 10,237,907 | `554d190e40a386314b268b1e03c3f6f9c5f6aa8ff70cc0bcc4a39bc18586d28a` |
| `pkg/rhwp.js` | 434,369 | `5f85f63c23f6bf19688d31ac432080eb75f4c77c48c344a05d46820cddb57e7e` |
| `rhwp-core-0.8.6.tgz` | 3,895,281 | `65957b58111eb247703add9b1778e869b5f7fb889ebf3fd840c4d6ab0b98d7e9` |
| `rhwp-editor-0.8.6.tgz` | 23,580 | `c0b11162c7849152c0f20d13698dbb081177fb386c9527ace17f4d8f6198743c` |
| `rhwp-vscode-0.8.6.vsix` | 19,454,925 | `1f2a7c11004af9ab6204c61d27d4685f9956b3fe7e3ac944ada3698b5eedf4de` |

- `@rhwp/core@0.8.6`: 7개 파일, unpacked 10,919,494 byte
- `@rhwp/editor@0.8.6`: 6개 파일, unpacked 80,030 byte; package test 32건 PASS
- VSIX: 37개 파일, 18.55 MB; TypeScript 두 config와 Webpack 두 bundle PASS
- VSIX 필수 항목 `dist/extension.js`, `media/rhwp_bg.wasm`, `media/rhwp.js`, `media/icon.png`,
  `package.json` 포함
- 세 package에서 `.env*`, `node_modules/`, `target/`, token 이름의 경로가 포함되지 않음을 확인

해시는 이 후보의 로컬 산출물 식별값이다. VSIX archive metadata처럼 실행마다 달라질 수 있는 값의 재현성
계약으로 사용하지 않고, Stage 5-A에서 실제로 검사한 파일을 식별하는 증적으로만 사용한다.

## npm audit 관찰

`rhwp-vscode npm ci`는 기존 lockfile에서 high 2건을 경고했다. read-only audit으로 분리 확인한 결과
production dependency는 취약점 0건이고, `browserslist`, `fast-uri` 두 건은 모두 간접 dev dependency다.
이번 변경은 dependency나 lockfile을 바꾸지 않으므로 자동 `npm audit fix`나 범위 확장은 하지 않았다.
VSIX에는 `node_modules/`가 포함되지 않고 Webpack bundle과 라이선스 파일만 포함된다.

## 판정과 다음 게이트

Stage 5-A 판정은 **PASS**다. 로컬에서 검증 가능한 workflow 구문·정책·package build 계약은 충족했고,
검증용 worktree와 임시 파일은 제거했으며 기존 maintainer review worktree는 건드리지 않았다.

남은 Stage 5-B는 GitHub 전용 경계를 검증한다. 별도 승인을 받은 뒤 후보 branch를 push하고 exact remote
head에서 Release Binary `tag=test`와 Publish All Packages `publish=false` verify-only run을 실행해 다음을
확정한다.

1. Release Binary 5개 matrix build와 reusable package 호출의 실제 job 이름
2. Release와 외부 publish 4개 job의 skip
3. `wasm-pkg`, `vscode-vsix`, `release-publish-evidence` 및 CLI archive 5개의 실제 artifact
4. evidence의 `verdict=completed`와 실행 전후 외부 채널 무변경

push와 두 workflow dispatch는 아직 수행하지 않았다.
