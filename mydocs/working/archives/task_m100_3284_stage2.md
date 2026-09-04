# Task #3284 Stage 2 — 구현계획서 (갱신)

## 방침 확정 (작업지시자 결정 반영)

- **단일 러너로 구성** — 멀티 러너(안 1/2)는 후속으로 남긴다.
- **fallback 미구현** — preflight 감지·runner-fallback-action 도입 안 함. needs 그래프
  무변경.
- **timeout-minutes 는 넣는다** — 단일 러너 다운 시 24시간 큐잉을 짧은 타임아웃으로 대체.
- 이번 구현 = **runs-on 13곳 전환 + 대상 job 에 timeout-minutes 명시**.

## 인지된 제약 (구현엔 반영 안 하되 기록)

- 러너 인스턴스 1개 = job 직렬. test-shard 8 은 순차 실행되어 호스티드 병렬보다 벽시계는
  느릴 수 있다. 이번엔 감수하고, 실측 후 멀티 러너를 별도 이슈로 판단.
- 단일 러너 장애 시 CI 전면 중단 — timeout 으로 무한 대기만 방지, 폴백은 없음. 발생 시 대응.

## 치환 1 — runs-on (13 job)

전부 `runs-on: ubuntu-latest` → `runs-on: [self-hosted, Linux, X64]`.

| 파일 | job |
|---|---|
| `ci.yml` | preflight, lint, test-shard, build-and-test, native-skia-tests, frontend-package-gates, build-test-archive, wasm-build (8개, 파일 내 전역) |
| `codeql.yml` | preflight, analyze (2) |
| `render-diff.yml` | preflight, canvas-visual-diff (2) |
| `full-renderer-sweep.yml` | full-renderer-sweep (1) |

## 치환 2 — timeout-minutes (job 성격별)

각 job 의 `runs-on` 바로 아래 줄에 추가. 값은 job 성격 + 단일 러너 직렬 여유를 감안:

| job | timeout | 근거 |
|---|---|---|
| preflight (ci/codeql/render-diff) | 10 | 경량 감지 job(API·diff 판정) |
| lint | 20 | fmt+clippy+wasm check |
| frontend-package-gates | 20 | tsc + npm test |
| build-test-archive | 30 | 빌드 + 아티팩트 |
| native-skia-tests | 30 | skia 빌드+테스트 |
| test-shard | 30 | shard 1개(직렬이라 넉넉히) |
| build-and-test | 45 | 전체 빌드+테스트 |
| wasm-build | 30 | wasm 빌드 |
| codeql analyze | 45 | 정적분석 무거움 |
| render-diff canvas-visual-diff | 45 | 렌더 diff |
| full-renderer-sweep | 60 | 전수 sweep |

(값은 로컬 실측 대비 2~3배 여유. 러너 hang 을 실패로 드러내는 상한이지 성능 목표 아님.)

## 제외 (무수정)

- `deploy-pages.yml` / `npm-publish.yml` / `close-issues-on-devel-push.yml`.
- `release-binary.yml` — matrix.runner(ubuntu-latest)는 macos-14/windows 와 같은 매트릭스,
  139행 릴리스 게시 job 도 호스티드 유지. **건드리지 않는다.**

## 검증

1. **actionlint** — 전 워크플로 문법·스키마(runs-on 배열·timeout-minutes 정합).
2. **YAML 파싱** — python yaml.safe_load 로 4개 파일 로드 가능.
3. **diff 정확성** — 대상 4파일만 변경, release-binary/deploy/npm/close 무변경 확인.
4. **실 검증** — 이 전환 PR 의 CI 가 self-hosted 러너에서 실제로 도는 것으로 확인
   (PR 브랜치 워크플로가 즉시 적용됨).

## PR

- 브랜치 `task/3284-ci-self-hosted`, `Closes #3284`.
- 본문: 범위(13 job + timeout, 배포/릴리스 제외), 인지된 제약(단일 러너 직렬·폴백 없음 —
  발생 시 대응), 러너 사양 실증, 멀티 러너·폴백은 후속 이슈 후보로 명시.
- assignee=edwardkim / milestone=v1.0.0.

## 정정 (구현 후 — CI 실패 대응)

첫 CI 실행에서 두 스텝이 self-hosted 러너의 제한된 sudo(apt 한정)로 실패했다. 조사
(GitHub 공식 문서)로 양쪽 호환 패턴을 확정해 정정한다:

1. **`install-wasm-pack` 액션** (`sudo mv → /usr/local/bin` 실패):
   → `~/.cargo/bin` 에 sudo 없이 배치 + `$GITHUB_PATH` 추가. 호스티드·self-hosted 양쪽
   동일 동작(sudo 의존 제거가 가장 견고 — 분기 불필요). **이 액션은 deploy-pages·
   npm-publish(호스티드 유지 대상)도 쓰지만, sudo 제거는 그쪽에도 안전한 무해 개선.**

2. **ci.yml 디스크 정리 3곳** (`sudo rm android/dotnet` — 호스티드 이미지 전용):
   → `if: runner.environment == 'github-hosted'` 로 감싸 self-hosted 에서 skip.
   호스티드 동작 100% 보존(되돌려도 회귀 없음). `runner.environment` 는 공식 컨텍스트
   (값 github-hosted/self-hosted), 러너 에이전트가 노출.

원칙: **sudo/시스템경로 의존은 제거(설치 스텝), 호스티드 이미지 구조 전용 스텝은
조건 분기로 skip(작업지시자 권고).** 두 기법의 역할 분리가 portable 패턴.
