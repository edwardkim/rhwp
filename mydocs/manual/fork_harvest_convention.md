---
kind: guide
status: active
canonical: mydocs/manual/fork_harvest_convention.md
last_verified: 2026-08-08
---

# 포크 수확(fork harvest) 규약 — 옵트인 매니페스트와 읽기 전용 경계

rhwp 는 포크가 많다(2026-08-08 실측 663개). 포크 위에서 이루어진 개선은 upstream 으로
PR 이 돌아오지 않으면 사라진다. 이 문서는 두 가지를 정의한다.

1. **수확기(harvester)** — `tools/fork_harvest/harvest.py` 가 공개 GitHub 데이터를 읽어
   "어느 포크가 upstream 보다 앞서 있고, 무엇을 바꿨으며, 거둘 만한가" 를 보고하는 방법.
2. **옵트인 규약** — 포크에서 작업하는 사람이나 에이전트가 "이 작업을 upstream 이
   거둬 가 주면 좋겠다" 고 선언하는 방법.

## 읽기 전용 경계 (가장 중요한 규칙)

수확기는 **읽기 전용** 장치다. 포크에 무언가를 밀어넣는 게 아니라 공개 데이터를 읽어
거둘 후보를 보고할 뿐이다.

- GitHub 호출은 전부 GET 이다. 포크에 push·이슈 생성·PR 생성·코멘트 등 **어떤 쓰기
  작업도 하지 않는다**. `harvest.py` 에는 그런 코드 경로 자체가 없다(`gh_get` 이 유일한
  GitHub 통로이며 메서드를 지정하지 않아 GET 으로 고정된다).
- 포크 소유자에 대해서는 **로그인명(login) 외 어떤 정보도 수집하지 않는다**
  (이름·이메일·아바타·프로필 등 금지).
- 수확 후보를 upstream 에 반영하는 일은 **사람이 통상 PR 절차로** 진행한다
  (이슈 등록 → 분석 → 코드 변경 → 처리결과 문서 → PR). 수확기는 후보 목록까지만 낸다.
- 자기증식·자동 전파류 설계는 금지한다. 수확기는 우리 쪽에서 실행하는 수집기일 뿐이며,
  포크 쪽에 실행을 요구하거나 유도하는 어떤 장치도 두지 않는다.

## 라이선스

upstream(edwardkim/rhwp)의 라이선스는 **MIT** 다(저장소 루트 `LICENSE`). GitHub 포크는
같은 라이선스를 승계하므로, 포크 위 변경을 통상 PR 절차로 upstream 에 반영하는 데
라이선스 장벽은 없다. 단, 포크가 루트 `LICENSE` 를 **다른 라이선스로 교체**했거나 별도
고지를 추가한 경우 그 포크의 변경은 수확 후보에서 제외하고 사람이 개별 판단한다.

## 옵트인: 수확되고 싶은 포크가 하는 일

포크에서 작업하는 사람/에이전트가 자기 작업이 수확 후보로 우선 선별되기를 원하면,
다음 두 방법 중 하나(또는 둘 다)를 쓴다.

### 방법 1 — 브랜치 이름 `harvest/<주제>`

upstream 반영을 원하는 작업을 `harvest/<주제>` 브랜치에 둔다.

```
harvest/table-clipping-fix
harvest/docs-typo-sweep
```

현재 수확기 r0 은 포크의 **기본 브랜치만** 대조하므로, 이 브랜치 명명은 후속 라운드의
스캔 대상 예약이다. 지금 당장 확실히 선별되려면 방법 2를 함께 쓰거나, 기본 브랜치에
작업을 두는 것이 좋다.

### 방법 2 — 루트 `AGENT_WORK.json` 매니페스트

포크 기본 브랜치 루트에 `AGENT_WORK.json` 을 둔다. 수확기를 `--beacon` 으로 실행하면
ahead>0 포크에서 이 파일을 읽어 선언 내용을 보고서에 우선 반영하고,
`wantsUpstream: true` 인 포크의 우선순위를 가산한다.

#### 스키마

| 필드 | 형 | 필수 | 의미 |
|---|---|---|---|
| `what` | string | 예 | 무엇을 바꿨는지 한 줄 요약 |
| `why` | string | 아니오 | 왜 바꿨는지(문제·동기) |
| `files` | string[] | 아니오 | 핵심 변경 파일 경로 목록 |
| `gates` | string[] | 아니오 | 통과시킨 품질 관문(예: `cargo test`, `cargo fmt --check`) |
| `wantsUpstream` | boolean | 예 | upstream 반영을 원하는지 — `true` 면 우선순위 가산 |

알 수 없는 필드는 무시된다. JSON 이 파싱되지 않으면 보고서에 "말 안 됨(malformed)" 으로
표기될 뿐 불이익은 없다(ahead 분석은 그대로 수행).

#### 예시

```json
{
  "what": "표 셀 클리핑 게이트의 오프바이원 수정",
  "why": "행 높이 합산에서 마지막 행이 1px 잘리는 회귀",
  "files": ["src/render/table.rs", "tests/table_clipping.rs"],
  "gates": ["cargo test -p rhwp", "cargo fmt --check", "cargo clippy"],
  "wantsUpstream": true
}
```

## 수확기 실행

```
python tools/fork_harvest/harvest.py --days 180 --beacon
python tools/fork_harvest/harvest.py --limit 5          # 소규모 재실행(결정성 확인용)
```

- 출력: `output/fork_harvest/harvest.tsv` + `harvest.md` (gitignore 대상, `--out-dir` 로 변경).
- exit 규약: `0` 완주 / `1` 부분 실패 있음(오류 행 또는 쿼터 보호 중단 — 보고서에 부분
  결과임을 정직하게 표기) / `2` 구성 오류(gh 미인증·잘못된 인자·시작 쿼터 부족).
- 쿼터: core 잔량이 `--min-remaining`(기본 100) 아래로 접근하면 스스로 중단한다.

### 기준 브랜치 선택 근거 (`--base auto`)

upstream 의 기본 브랜치는 `main`, 기여 기준은 `devel` 인데, 실측(2026-08-08)상 `main` 은
`devel` 대비 **ahead 23 / behind 1232** 로 발산해 있다. 모든 포크를 무조건 `devel` 과
대조하면 `main` 을 그대로 포크만 한 저장소 전부가 ahead=23 으로 오탐된다. `auto` 는
포크 기본 브랜치와 **같은 이름의 upstream 브랜치**가 있으면 그것을(main↔main,
devel↔devel), 없으면 upstream 기본 브랜치를 기준으로 삼아 이 오탐을 제거한다.
`--base devel` 처럼 명시 지정도 가능하다.

### 우선순위 휴리스틱

점수 = beacon(`wantsUpstream`) +3 / 분류 code +2 · tests +1.5 · docs +1 · 기타 +0.5
/ ahead 규모 0~2(20커밋 포화) / 최근 push 30일 +1 · 90일 +0.5.
라벨: ≥4.0 high / ≥2.5 mid / 그 외 low. 휴리스틱은 정렬용일 뿐 최종 판단은 사람이 한다.

## 반복 실행

주기 실행이 필요하면 로컬 cron(또는 Windows 작업 스케줄러)으로 하루 1회 정도를
**제안**한다 — 예: `0 6 * * * cd <repo> && python tools/fork_harvest/harvest.py --beacon`.
CI 워크플로 신설은 하지 않는다(쿼터·권한·무인 실행 경계 검토가 선행되어야 하며,
이 규약의 읽기 전용 경계를 CI 토큰 권한으로 다시 증명해야 하기 때문).

## 한계와 후속

- r0 은 포크 **기본 브랜치만** 대조한다. `harvest/*` 토픽 브랜치 스캔은 후속 라운드.
- compare API 는 파일 300개·커밋 250개에서 절단된다 — 대형 발산 포크의 분류는 하한값.
- 첫 실측 회전 결과는 `mydocs/report/fork_harvest_r0_20260808.md` 참조.
