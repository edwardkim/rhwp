---
kind: working
status: active
canonical: mydocs/working/task_m100_6717_stage1.md
issue: 6717
last_verified: 2026-09-04
---

# #6717 Stage 1 — Frontend package gates 설치 지연 정상화

## 1. 판정

**O3 workflow 구현과 로컬 검증은 완료됐다. 원격 push·PR·실제 Actions 효과 검증은 아직 수행하지
않았다.**

- 기준 branch는 `upstream/devel@009e30fe1f6812b046862589783c68f890b4d363`이며, 구현 완료 뒤
  다시 fetch한 결과 작업 branch와 divergence는 `0/0`이다.
- `frontend-package-gates`의 ID·표시 이름·권한·aggregate 배선은 유지했다.
- job 상한만 20분에서 30분으로 바꾸고 네 직렬 install에
  `--no-audit --prefer-offline`을 적용했다.
- package manifest·lockfile·제품 source는 변경하지 않았다.

## 2. 원인 경계와 변경 의미

로컬 npm 11.12.1의 기본 설정은 `audit=true`, `prefer-offline=false`였다. 따라서 변경 전 네
`npm ci`는 install마다 advisory 조회가 활성화되고 cache보다 최신 network data를 우선할 수 있었다.
Actions 실패 로그만으로 두 옵션 중 어느 하나를 단독 원인으로 확정하지는 않는다. 확인된 직접 원인은
네 직렬 install의 network 응답 변동이 20분 job 예산을 잠식했고, 후속 회귀 gate가 실행될 시간을
보장하지 못했다는 것이다.

`--no-audit`은 현재 차단 조건이 아닌 install 중 자동 advisory 조회를 제거하고,
`--prefer-offline`은 setup-node npm cache를 우선하되 cache miss의 registry 접근을 허용한다.
`--offline`과 `--ignore-scripts`는 사용하지 않아 cache miss와 lifecycle script를 숨기지 않는다.
30분은 성공 시간 목표가 아니라 지연과 hang을 구분하는 유한 상한이다.

## 3. 계약 테스트

`scripts/tests/test_ci_impact_workflow.py`는 다음을 고정한다.

1. package job 상한이 30분이다.
2. install step의 npm 명령 집합과 순서가 Studio, Chrome, Firefox, VS Code의 정확히 네 줄이다.
3. 네 명령 모두 `ci --no-audit --prefer-offline`을 사용한다.
4. 완전 offline과 lifecycle script 생략을 사용하지 않는다.

기존 계약 테스트가 package/full lane 판정, `Build & Test` aggregate 연결과 실패 전파를 계속
검증하므로 required check 이름을 바꾸지 않았다.

## 4. 로컬 검증 결과

### 4.1 install 명령

CI와 같은 순서로 실행한 결과는 다음과 같다.

| package | elapsed | 결과 |
| --- | ---: | --- |
| `rhwp-studio` | 6.05초 | 성공 |
| `rhwp-chrome` | 2.16초 | 성공 |
| `rhwp-firefox` | 0.95초 | 성공 |
| `rhwp-vscode` | 2.71초 | 성공 |

합계는 11.87초이며 lockfile 변경은 0건이다. 이는 명령의 유효성과 local cache 경로를 확인하는
자료이지 GitHub-hosted runner의 network 성능 보장은 아니다.

### 4.2 WASM과 frontend gate

새 worktree에는 `pkg/rhwp.d.ts`가 없어 첫 WASM binding 계약 검사가 `ENOENT`로 중단됐다. 이는
CI에서 선행하는 fresh WASM build를 로컬에서 아직 하지 않은 사전조건 실패였다. 기존 checkout의
`.env.docker` 값을 복제·출력하지 않고 작업 중에만 링크하여 다음 표준 build를 실행했으며, 종료
trap으로 링크를 제거했다.

```bash
docker compose --env-file .env.docker run --rm wasm
```

Docker WASM build는 컨테이너 내부 6분 55초, 전체 453.59초에 성공했다. 그 뒤 다음 묶음을 처음부터
재실행해 모두 통과했다.

- WASM declaration·editor embed 계약
- `@rhwp/editor` package test
- shared·Chrome·Firefox service worker test와 Chrome options test
- Studio unit test와 production build
- Chrome·Firefox extension build와 distribution 계약
- VS Code extension compile·outline keyboard 계약
- font asset 계약

후속 frontend 묶음의 전체 wall time은 22.21초였다. 변경하지 않은 실제 브라우저 E2E와 Actions
runner의 30분 상한 효과는 최신 PR head의 원격 full lane에서 최종 확인한다.

### 4.3 workflow·문서

```text
PyYAML workflow parse: OK
CiImpactWorkflowTests: 34 tests, OK
test_*workflow.py discovery: 152 tests, OK
Markdown links: 611 docs, changed 4, OK
git diff --check: OK
```

호스트에 `actionlint`와 Ruby가 없어 PyYAML parse와 저장소 workflow 계약 suite를 사용했다.
metadata 검사는 기존 `mydocs/tech/**` 네 파일의 16건을 다시 보고했지만 네 파일은
`upstream/devel`과 byte diff가 없고 이번 문서에서는 신규 오류가 발생하지 않았다.

## 5. 남은 게이트

1. 변경 문서 추가 뒤 link·metadata·diff 검사를 한 번 더 실행한다.
2. 로컬 변경을 commit하고, 별도 승인 뒤 원격 push와 PR을 생성한다.
3. 최신 exact head의 `Frontend package gates`, `Build & Test`, `CI Impact Policy` 성공과 실제 job
   소요 시간을 확인한다.
4. #6717이 `devel`에 반영된 뒤 PR #6715를 최신 base와 정합시켜 같은 full lane을 재검증한다.
