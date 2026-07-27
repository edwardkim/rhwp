# rhwp autofix bot — 설치 절차

`kevin9327/rhwp` 포크에서 30분마다 돌면서 **증명 가능한 버그 1건**을 찾아
`edwardkim/rhwp` 의 `devel` 로 PR 을 제안한다. PC 를 켜둘 필요 없다.

## 설계 한 줄 요약

> PR 을 열지 말지는 **모델이 아니라 `verify-gate.sh` 의 종료코드**가 정한다.

```
탐색 → red 증명 → green 증명 → 회귀 0 → CI 3종 → 이슈 등록 → PR
        └─ 여기서 실패하면 upstream 에는 아무것도 남지 않는다
```

`red 증명`이 오탐 차단의 전부다. 수정분만 `git stash` 로 원복한 뒤 신규 테스트를 돌려
**단정 실패(assertion failure)로 떨어지는지** 확인한다.

- 원복해도 통과 → 애초에 버그가 없었음 → **폐기**
- 원복하니 컴파일 에러 → 재현 증명이 아님 → **폐기**
- 원복하니 단정 실패 → 진짜 결함 → 통과

모델이 "버그를 찾았다"고 주장해도 이 게이트를 못 넘으면 upstream 에 도달하지 못한다.
`CONTRIBUTING.md` 의 "red→green 회귀 테스트 동봉" 규정을 그대로 기계화한 것이다.

---

## 1. 배치 위치

파일은 **포크의 기본 브랜치(`main`)** 에 올려야 한다. GitHub 의 `schedule` 트리거는
기본 브랜치의 워크플로만 실행한다. `devel` 에 올리면 영원히 안 돈다.

```
kevin9327/rhwp@main
├── .github/workflows/rhwp-autofix.yml
└── .github/autofix/
    ├── hunt-prompt.md
    └── verify-gate.sh
```

작업 대상 코드는 런타임에 `edwardkim/rhwp@devel` 을 따로 체크아웃하므로,
포크의 `main` 이 upstream 보다 뒤처져 있어도 상관없다.

## 2. 시크릿 2개

포크 → Settings → Secrets and variables → Actions → New repository secret

| 이름 | 값 | 비고 |
| --- | --- | --- |
| `ANTHROPIC_API_KEY` | console.anthropic.com 에서 발급 | **예산 상한을 반드시 걸 것** |
| `RHWP_BOT_PAT` | classic PAT, `public_repo` 스코프 | fine-grained 토큰은 크로스 레포 PR 에서 자주 막힌다 |

`GITHUB_TOKEN` 으로는 안 된다 — 포크 컨텍스트에서 read-only 라 upstream 에 PR 을 못 연다.
그래서 PAT 가 필요하다.

## 3. Actions 활성화

포크는 Actions 가 기본으로 꺼져 있다. 포크의 **Actions 탭** → 초록 버튼 클릭 →
스케줄 워크플로도 별도로 활성화. 이 단계를 빼먹으면 아무 일도 안 일어난다.

## 4. 첫 실행은 반드시 dry run

Actions 탭 → `rhwp autofix bot` → Run workflow → `dry_run: true` (기본값)

게이트까지만 돌고 이슈·PR 은 만들지 않는다. 로그 아티팩트(`autofix-log-N`)에서
탐색이 무엇을 봤고 왜 폐기했는지 확인할 수 있다. **1~2주는 dry run 으로만 돌려보고**
게이트가 실제로 오탐을 걸러내는지 눈으로 확인한 다음 스케줄을 켜라.

---

## 운영상 반드시 알아야 할 것

### 30분 주기는 "30분마다 완주"가 아니다

upstream CI 가 job 당 20~45분을 잡는다. 전체 회귀(`--tests`, 3,400+)를 포함한 한 사이클은
30분 안에 안 끝난다. 그래서 `concurrency: rhwp-autofix / cancel-in-progress: false` 로
직렬화했다 — 실행 중이면 다음 run 은 대기하고, GitHub 이 대기열을 1건만 유지한다.
**결과적으로 "30분마다 확인, 끝나는 대로 다음 사이클"** 로 동작한다. 이게 의도한 동작이다.

### 비용은 Actions 가 아니라 API 에서 난다

퍼블릭 저장소라 러너 분(minutes)은 무료다. 돈은 Anthropic API 에서 나간다.
`--max-turns 60` 짜리 세션이 하루 최대 48회다.

**첫 주는 `cron: "0 */3 * * *"` (3시간마다, 하루 8회)로 시작해서 콘솔에서 실제 비용을
측정한 뒤 `*/30` 으로 조이는 걸 권한다.** 파일에는 요청대로 `*/30` 으로 넣어뒀다.
바꾸려면 `rhwp-autofix.yml` 의 `cron:` 한 줄만 고치면 된다.

### PR 큐 자동 상한

`MAX_OPEN_PRS: "4"` — 내 이름으로 upstream 에 열린 PR 이 4건이면 그 사이클은
Claude 를 호출조차 하지 않고 종료한다. 기존 PR 이 머지되면 자동 재개된다.
지금 열린 PR 이 정확히 4건이라 **설치 직후에는 아무것도 안 돌 것이다** — 정상이다.
당장 시험해보려면 이 값을 임시로 올려라.

### 60일 무활동 자동 정지

포크에 60일간 커밋이 없으면 GitHub 이 스케줄을 꺼버린다. 봇이 PR 을 만들면 그게
활동으로 잡히지만, 계속 NO-FINDING 이면 정지될 수 있다. Actions 탭에서 재활성화하면 된다.

### 정지 스위치

포크 Settings → Actions → "Disable actions". 즉시 멈춘다.

---

## 탐색 대상 선정 규칙

| 조건 | 모드 | 대상 |
| --- | --- | --- |
| 최근 12시간 `devel` 커밋 있음 | `diff` | 그 커밋 범위 — **새로 유입된 회귀** (가치 최상) |
| 없음 | `module` | `src/parser/{hwp,hwpx,hwp3}`, `renderer`, `layout`, `editor`, `document` 를 run 번호로 순회 |

내가 이미 열어둔 PR·이슈 제목은 `HUNT_EXCLUDE` 로 넘겨 중복 제출을 막는다.
**페이지네이션·페이지 분할 영역은 프롬프트에서 명시적으로 제외**했다 —
`CONTRIBUTING.md` 가 이 영역은 한컴 환경 의존성 때문에 메인테이너 재검증이 필요하다고
못박고 있어서, 무인 자동화 대상이 아니다.

## 저장소 규약 준수

| 규약 | 적용 위치 |
| --- | --- |
| 이슈등록 → 분석 → 코드변경 → 처리결과문서 → PR | 워크플로 스텝 순서 그대로 |
| `tests/issue_{번호}_{설명}.rs` | `issue_TBD_*` 로 작업 → 게이트 통과 후 실제 번호로 rename → 재검증 |
| PR 대상은 `devel` (main 아님) | `gh pr create --base devel` |
| `fmt` / `release-test --tests` / `clippy -D warnings` | 게이트 3-1~3-3 |
| 브랜치는 최신 `upstream/devel` 기준 | 매 사이클 devel 을 새로 체크아웃 |
| 처리결과 문서 | `mydocs/report/task_m100_{번호}_report.md` |
| 한글 PR | 프롬프트가 한글 산출물을 강제 |

## 아직 검증 못 한 것

- **로컬 문법 검증을 못 돌렸다.** YAML/bash 파싱 확인을 시도했지만 이 PC 의 권한
  설정에 막혔다. GitHub 이 push 시 YAML 을 검증하고, `dry_run` 실행에서 bash 가
  실제로 돌면서 드러난다 — 그래서 4번 단계(dry run)를 건너뛰지 마라.
- **PR 본문 전/후 스크린샷**은 자동화하지 않았다. 렌더링 버그 PR 이 나오면 이건
  수동으로 붙여야 한다. 비-시각 버그에는 red→green 표가 그 역할을 대신한다.
