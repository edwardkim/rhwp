---
kind: guide
status: active
canonical: gym/README.md
last_verified: 2026-08-10
---

# rhwp 에이전트 짐(gym) — 운동장

**에이전트야, 여기서 놀아라.** 이 폴더는 rhwp 위에서 에이전트(모델·사람·스크립트
무엇이든)가 실제 한국 문서로 실제 작업을 수행하고, 기계 채점으로 실력을 기록으로
남기는 운동장이다. 문서를 읽는 곳이 아니라 뛰는 곳이다 — 이 README 하나만 읽고
스스로 수행→제출→자가 채점이 되도록 만들어져 있다.

> 🎡 **놀이공원처럼 둘러보려면** → [PARK.md](PARK.md) (테마파크 지도) ·
> [tutorial/](tutorial/README.md) (☕ 휴게실, 첫 방문 5분) ·
> [INVITE.md](INVITE.md) (💌 친구·부모님 초대). 처음이면 입문존
> `--profile family`, 담력이 붙으면 보스존 `--profile boss`.

## 30초 입장

```bash
cargo build --bin rhwp                 # 1) 운동화 (바이너리)
cat gym/tasks/T01.json                 # 2) 과제 읽기 (instructions 필드가 일감)
mkdir -p gym/submissions/<너의이름>/T01  # 3) 과제별 폴더에 제출물 넣기
python gym/score.py --agent <너의이름>   # 4) 자가 채점 — 스코어카드 발급
```

## 규칙 — 세 줄

1. **과제 파일이 유일한 지시서다.** `tasks/T*.json` 의 `instructions` 를 읽고
   `input` 문서에 대해 수행하라. 힌트는 있지만 경로 탐색(어느 명령을 어떻게
   조합할지)은 네 몫이다 — 그것이 측정 대상이다.
2. **제출은 파일이다.** 과제의 `submit` 이 요구하는 것(answer.json, 산출물,
   또는 산출물 쌍)을 `submissions/<이름>/<과제ID>/` 에 놓아라.
3. **채점은 라이브다.** 정답은 골든 파일로 박제돼 있지 않다 — `score.py` 가
   채점 시점에 rhwp 로 기대값을 재계산하고, 산출물은 rhwp 로 재검증한다
   (검색·재조회·해시). 픽스처가 진화하면 정답도 따라 진화한다.

## 과제판 — pack 12개 · 과제 100건 · 만점 221

능력 영역을 **pack** 으로 나눈다. 점수는 pack 별로 보존되며 총점은 편의값이다 —
어느 능력이 모자란지는 pack 별 점수가 말한다. 🎡 [테마파크 지도](PARK.md)는
같은 pack 들을 놀이공원 존으로 안내한다.

| pack | 이름 | 능력 축 | 과제 | 만점 |
|---|---|---|---|---|
| `casual-rides` | 🎠 입문 놀이기구 | 입문 (읽고 세기 — 누구나·부모님도) | 4 | 4 |
| `core-cli` | 코어 CLI | 조사·추출·편집·검증 (운동장 최소 코어) | 14 | 32 |
| `automation` | 자동화·검증 사다리 | 자동화 (계획·캡슐·서명·앵커·정산·감사) | 13 | 35 |
| `corpus-diagnostics` | 코퍼스·진단 | 진단 (폴더 스윕·쪽 덤프·비교 판정) | 7 | 14 |
| `expert-challenges` | 🐉 보스 어트랙션 | 자동화 (사다리 완주 — tier 4~5 고난도) | 5 | 23 |
| `layout-rendering` | 조판·렌더링 | 검증 (조판 판정·렌더 산출) | 8 | 15 |
| `objects-media` | 개체·미디어 | 발견 (필드·개체·렌더 산출물) | 7 | 15 |
| `security` | 보안 스윕 | 보안 (은닉·주입·유니코드·PII) | 9 | 18 |
| `self-description` | 자기서술 표면 | 자기서술 (도구가 스스로를 설명하는 계약) | 7 | 12 |
| `serialization` | 저장·변환 | 변환 (형식 왕복·IR 대조) | 8 | 19 |
| `table-editing` | 표 편집 | 편집 (표 좌표 지정) | 8 | 16 |
| `text-editing` | 본문 편집 | 편집 (탐색→치환→재검증) | 10 | 18 |

난도 티어는 1~5다: **1=입문(부모님도), 2=초급, 3=중급, 4=고급, 5=보스**.
한쪽 끝(`casual-rides`)엔 키 제한 없는 회전목마를, 다른 끝(`expert-challenges`)엔
한 단만 틀려도 판정이 막히는 자이로드롭을 둔다.

각 pack 은 `packs/<id>/` 아래에 있다.

```text
packs/<id>/
├── pack.json      # manifest — id·요구 capability·기준 실행 신원
├── tasks/*.json   # 과제
├── reference/*.json  # 기준 풀이(정답 노출 — 채점 재현용, 푸는 쪽은 보지 않는다)
└── assets/        # 과제 고정 자산(정책 등)
```

### pack manifest 가 선언하는 것

```json
{
  "id": "table-editing",
  "requires": { "commands": ["export-tables", "edit", "table-to-csv"] },
  "runner": { "rhwpVersion": "…", "rhwpCommit": "…", "capabilitiesSha256": "…" }
}
```

- `requires.commands` — 이 pack 을 채점하려면 바이너리에 있어야 하는 명령.
  없으면 **0점이 아니라 `unavailable`** 로 보고한다. 부재를 실패로 위장하지
  않는 것이 이 저장소의 결이다 — 오래된 바이너리에게 "너는 0점"은 거짓말이다.
- `runner` — **기준 실행의 신원**. 점수는 바이너리마다 달라질 수 있으므로
  "이 점수가 어느 바이너리에서 났는가"를 pack 과 스코어카드 양쪽에 남긴다.

## 프로파일 — pack 을 고르는 도구

| profile | 묶음 |
|---|---|
| `family` (🎠 가족 코스) | `casual-rides` — 부모님·친구와 함께 도는 입문존만 |
| `starter` (입문) | `core-cli`, `self-description` |
| `editor` (편집자) | `core-cli`, `text-editing`, `table-editing`, `objects-media` |
| `publisher` (배포자) | `serialization`, `layout-rendering`, `security` |
| `boss` (🐉 보스 코스) | `expert-challenges` — 사다리 완주급 고난도만 |
| `maintainer` (메인테이너) | 전 12 pack 완주 코스 |

```bash
python gym/score.py --agent <이름>                 # 전 pack
python gym/score.py --agent <이름> --profile editor  # 프로파일
python gym/score.py --agent <이름> --pack security   # pack 지목
```

프로파일은 pack 을 **고르는** 도구이지 점수를 뭉치는 도구가 아니다.

## 새 과제를 등재하는 법 — 기준 풀이 왕복

과제를 손으로 늘리면 "돌아가지 않는 과제" 가 섞인다. pack 이 8개면 그 위험도
8배다. 그래서 신규 과제는 **기준 풀이 왕복을 통과해야만** 등재된다.

```bash
python gym/tools/build_baseline.py --agent <이름> --pack <id>  # 기준 풀이 실행 → 제출물 생성
python gym/score.py --agent <이름> --pack <id>                 # 즉시 채점
```

즉 **저장소에 들어간 모든 과제는 풀 수 있음이 실측된 과제**다. 기준 풀이 형식은
`gym/tools/build_baseline.py` 의 문서 문자열에 있다.

## 제출 형식

- `submit.kind: "answer"` → `answer.json` 하나 (과제가 요구한 키만).
- `submit.kind: "artifact"` → 지정된 이름의 산출 파일(예: `out.hwp`).
  **원본 픽스처를 절대 덮어쓰지 마라** — 항상 `-o` 로 새 파일을 만들어라.
- `submit.kind: "pair"` → 산출물 2개 + 그 산출에 쓴 계획서 (T10 결정론).
  채점기가 계획서를 `replay` 로 되돌려 실행해 산출물을 실제로 재현하는지 본다 —
  같은 파일 두 벌만으로는 통과하지 못한다(원본 복사 방어).

산출물(.hwp/.hwpx)은 커밋하지 않는다(`.gitignore`) — 과제 지시대로 재실행하면
누구나 재생산할 수 있고, 그 재생산 가능성 자체가 이 저장소의 검증 문화다.

## 베이스라인 — 1호 선수

`baselines/` 에 이 운동장을 처음 뛴 에이전트의 기록(answer·계획서·스코어카드·
리포트)이 있다. 네 기록을 그 옆에 놓고 싶다면: 채점 산출물을
`baselines/<너의이름>/` 로 `--out` 지정해 PR 로 제출하라.

## 2부 — 하네스 결합 (T13, 개장)

2부가 열렸다: **제출이 곧 증명이다.** T13(티어 3)은 `harness init` 로 만든
작업장에서 실편집 2건을 `harness wrap` 으로 체인 실행해 폴더째 제출한다 —
채점기는 `harness status --keyring --deep` **한 호출**로 체인 무결·서명
귀속·전수 재현을 판정한다. 운동장(과제)과 하네스(루프)가 서로를 소비하는
폐루프의 첫 실증이다.

**3부(T14)도 열렸다**: 채점기가 곧 반입 관문이다 — 과제 고정 정책(assets/T14_policy.json)에 대해 `rhwp gate --deep` 의 verdict:allow 가 통과 조건. 재계산 원칙이라 골든 부패가 없고, 떨어지면 violations 가 어느 축이 모자란지 말해준다. 남은 후속은 리더보드다.

**4부 — 대확장(#4653)**: 운동장이 pack 으로 쪼개졌다. core-cli 1개였던 판이 **10개 pack · 과제 91건 · 만점 194** 이 됐고, 판정 논리는 `gym/core/`(runner·schema·check registry)로 모여 pack 이 늘어도 판정 어휘는 한 곳에서만 자란다. 신규 과제 전건이 기준 풀이 왕복으로 실측 등재됐다.

**5부 — 테마파크(#4664)**: 운동장에 놀이공원을 입혔다. 한쪽 끝에 키 제한 없는 **입문존**(`casual-rides`, 부모님·친구도), 다른 끝에 한 단만 틀려도 판정이 막히는 **보스존**(`expert-challenges`, tier 4~5)을 열어 **12 pack · 과제 100건 · 만점 221** 이 됐다. [테마파크 지도](PARK.md)·[휴게실](tutorial/README.md)·[친구 초대장](INVITE.md)이 방문 동기를 만들고, 리더보드엔 외부 참가자를 부르는 `invite`(판 지문 확인)가 붙었다. 테마는 장식일 뿐 채점·판정 논리는 그대로다 — 보스 과제도 예외 없이 기준 풀이 왕복을 통과했다.

## 위조 불가능한 리더보드 — 점수판을 검증 사다리 위에 (#4659)

AI 벤치마크 리더보드의 병폐는 점수의 신뢰다: 수치는 자기 신고이고, 소급 수정이
가능하고, 같은 결과의 재등재를 막을 방법이 없다. 이 저장소에는 그 문제의 해답이
이미 있다 — 검증 사다리. 그래서 **운동장이 자기 사다리 위에서 돈다.**

```bash
python gym/score.py --agent <이름>                    # 채점 → scorecard + admission
python gym/tools/leaderboard.py attest --agent <이름>  # 등재 (keygen→settle→anchor)
python gym/tools/leaderboard.py verify                # 전 사슬 재검증
python gym/tools/leaderboard.py render                # 검증본에서 순위표
```

등재 사슬은 전부 기존 rhwp 명령이다(새 암호학 0줄):

| 공격 | 막는 축 | 실측 |
|---|---|---|
| 점수 위조 | 청구 capsuleSha256 고정(P1) | 스코어카드 99999 부풀림 → `verify` exit 3 (`pin.scorecard`) |
| 소급 조작 | 원장 스냅샷 봉인 + 교차 대조 | 원장 첫 줄 변조 → exit 3 (스냅샷 불일치 + `ledger.crossPin`) |
| 이중 등재 | 원장 전역 유일성(P3) | 같은 스코어카드 재등재 → 원장 거부 `duplicate: true` |
| 대리 제출 | 청구 Ed25519 서명(4년 축) | keyring 판정 `signerOk` |

`render` 는 총점 순위표 + **pack 별 능력 격자**를 낸다 — 총점이 숨기는 강약을
드러낸다(만점 칸은 굵게, 미제출 pack 은 `0/N` 으로 정직 표기). 검증 통과 항목만
순위에 올리고, 검증 불가 항목은 숨기지 않고 unverified 로 남긴다. 실측: 5선수
등재(194·185·145·133·44) → 전 사슬 verify 5/5 통과, 순위는 점수를 따른다.

**봉인 범위(정직)**: 이 사슬이 봉인하는 것은 "이 스코어카드가 이 시점에 이
신원으로 등재되어 이후 변조되지 않았다" 까지다. 채점 자체의 재현은 스코어카드에
박힌 runner 신원과 커밋된 제출물로 제3자가 수행한다.

## CI 릴리스 게이트 — 도구를 파이프라인에 물린다 (#4662)

아래 회귀 도구들이 도구로만 있으면 사람이 기억해서 돌려야 한다. 릴리스
파이프라인에 물리면 잊어도 돈다. `gym/tools/release_gate.py` 가 셋을 하나의
판정으로 묶는다:

```bash
python gym/tools/release_gate.py --old <직전 태그 바이너리> --new target/debug/rhwp
```

| 판정 | exit | 조건 |
|---|---|---|
| pass | 0 | 릴리스 차등 stable + 리더보드 체인 무결 |
| review | 2 | surface-changed — 표면 변경, 사람 판정(차단 아님) |
| block | 3 | regression 또는 리더보드 체인 파손 |

**regression 만 차단한다** — 도구는 "무엇이 바뀌었나"를 가리키지 "어느 쪽이
옳은가"를 판정하지 않으므로(#4661), 표면 변경은 리뷰 신호이지 자동 차단이 아니다.
독립 워크플로 `.github/workflows/gym-release-gate.yml`(수동 실행 + 태그 관찰)로
돌며, 릴리스 본체(`release-binary.yml`)는 건드리지 않는다. old 바이너리가 없으면
차등을 생략한다(부재≠실패).

## 판별력 감사 — 약한 오라클(false-pass)을 못 들어오게 막는다 (#4808)

2026 벤치마크의 최대 위기는 **false-pass**다: OpenAI 감사에서 SWE-Bench Verified
최난도 과제의 59.4%가 버그를 안 고쳐도 테스트가 통과했다(약한 오라클). 채점이
"일을 했나"가 아니라 "파일이 있나"만 보면, 아무것도 안 한 제출도 만점을 받는다.

`gym/tools/discriminate.py` 는 각 과제에 **음성 대조**(일 안 한 제출)를 자동
구성해 채점하고, **통과하면(=거부 실패)** 약한 오라클로 리포트한다:

```bash
python gym/tools/discriminate.py --bin target/debug/rhwp   # 전 과제 판별 감사
```

- **answer 과제** — 모든 답 키에 명백한 오답(sentinel). answer_eq 가 진값과 대조하니 거부해야 한다.
- **artifact 과제** — 입력을 산출물로 무편집 복사하는 대조와 synthetic garbage 대조를
  모두 실행한다. `differs_from_input`만이 아니라 형식·핵심값 검사도 garbage를 거부해야 한다.

음성 대조에 통과하는 과제 = 판별력 없는 약한 오라클. 거부하면 진짜 일을 요구하는
것이다. 이 감사는 릴리스 게이트(`gym-release-gate.yml`)에서 old/new 차등 **이전**에
돌며, 약한 오라클이 하나라도 있으면 릴리스를 차단한다 — 벤치마크 자체가 성립하는지
먼저 보는 무결성 전제조건이다(표면 변경과 달리 리뷰 신호가 아니라 결함이다).

## 릴리스 간 차등 회귀 — 시간축 차등 오라클 (#4661)

교차형식 차등(아래)이 형식축이라면, 이건 시간축이다. **같은 제출물을 구/신
바이너리로 채점하면 답이 같아야 한다** — 다르면 그 사이 릴리스에서 동작이
바뀐 것이다. 총점(통과/실패)이 아니라 각 검사의 **관측값(raw)** 을 대조하므로,
아무도 골든을 적어두지 않은 자리의 변화까지 잡는다.

```bash
python gym/tools/release_diff.py --old target/debug/rhwp.exe --new <신 바이너리>
```

분류(오검출 관문):

- **stable** — 관측 동일. 회귀 없음.
- **regression** — 관측이 다른데 명령 표면(capabilities digest)은 같다. 순수 동작 변화.
- **surface-changed** — digest 가 달라 표면 자체가 바뀜. 의도된 변경일 수 있어 사람 판정.

정직 조항: 이 도구는 "무엇이 바뀌었나" 를 가리키지 "어느 쪽이 옳은가" 를
판정하지 않는다(한컴 정답지 없음 — 차등 오라클과 같은 결). 자기-대조 실측:
관측 108건 전부 stable(비결정성 0), 도구 정합 확인.

## 트라젝토리 필요성 감사 — 경로의 무의미한 스텝(연극)을 막는다

2026 에이전트 평가의 합의: 종점만 채점하면 안 된다. 에이전트가 옳은 결과에
낭비·우회 경로로 도달해도 만점이면 프로덕션 실패다. 프론티어는 트라젝토리(결정
경로)를 채점하지만 대부분 **LLM-judge** 나 **골든 경로**로 — 둘 다 취약하다.

`gym/tools/trajectory.py` 는 골든도 judge 도 없이 그 사각을 잡는다: 각 다단계
과제에서 trailing answer·keyring 수집은 남기고 **마지막 외부 의미 스텝을 뺀**
부분 트라젝토리를 채점한다.

```bash
python gym/tools/trajectory.py --bin target/debug/rhwp
# → gym 트라젝토리 필요성 감사: 마지막 외부 의미 스텝이 load-bearing인지 확인
```

부분 트라젝토리가 **통과** = 마지막 외부 의미 스텝이 채점에 무의미 =
**트라젝토리 연극**. 실패(빌드 실패 포함) = 그 스텝이 load-bearing(정상). 이는
판별력 감사(종점: "산출이 입력과 다른가")를 **경로**로 민 것이다 — 모든 선언된
스텝이 결과를 바꿔야 한다. 릴리스 게이트(`gym-release-gate.yml`)에서 차등 이전에
돌며, 연극이 하나라도 있으면 릴리스를 차단한다.

## 차등 오라클 — 골든 파일 없는 회귀 사냥

채점기가 정답을 박제하지 않는다는 성질에는 아직 덜 쓴 쓸모가 있다.

> 같은 문서의 HWP 판과 HWPX 판에 **같은 관측**을 물리면 답이 같아야 한다.
> 다르면 둘 중 한 읽기 경로가 틀린 것이다.

즉 기대값을 아무도 적어두지 않은 자리까지 훑는 **차등 테스트**가 공짜로 생긴다.
저장소에 쌍둥이 픽스처가 139쌍 있으므로 관측 6종이면 즉시 834건의 판정이 된다.

```bash
python gym/tools/differential.py            # 전수 스윕
python gym/tools/differential.py --limit 20 # 표본
```

**오검출 관문 2단** — 이게 없으면 도구가 거짓말을 한다. 이름이 같다고 같은
문서라는 보장이 없기 때문이다.

1. **본문 동일성** — 공백 무시 본문이 바이트로 같아야 한다. 다르면 그냥 다른
   개정판이다(결함 아님).
2. **IR 동일성** — `ir-diff` 가 `identical: true` 를 내야 한다. rhwp 자신이
   "구조가 같다" 고 말한 뒤에도 관측이 어긋나면 **내부 모순**이고 우선순위가 높다.

첫 전수 주행 실측: 139쌍·834대조 → 이름만 같은 다른 문서 7쌍 제외, 후보 2건,
그중 IR 동일 모순 1건([#4658](https://github.com/edwardkim/rhwp/issues/4658)).

## 손상-강건성 감사 — 도구가 적대적 입력에 죽지 않는가

2026 프론티어(AgentHijack 등)는 에이전트가 **환경 손상**에 견디는지 잰다. gym 은
에이전트가 rhwp 를 몰아 능력을 낸다 — 그런데 rhwp 가 손상 문서에 **패닉**하면
에이전트가 아무리 유능해도 과제를 못 끝낸다. **도구 강건성이 능력의 천장이다.**

`gym/tools/robustness.py` 는 코퍼스를 **결정적으로 손상**시켜(무작위 없음) rhwp 가
언제나 우아하게(패닉·행 없이) 실패/부분복구하는지 인증한다:

```bash
python gym/tools/robustness.py --bin target/debug/rhwp        # 결정적 부분집합
python gym/tools/robustness.py --bin target/debug/rhwp --limit 40
```

- **패닉**(exit 101·시그널·'panicked'·스택 오버플로)·**행**(timeout) → 실패.
- 깨끗한 비-0 실패·경고 후 부분복구·정상 파싱 → 우아함(정상).

이는 종점 무결성(판별력)·경로 무결성(트라젝토리)에 이은 세 번째 기둥 — 도구
자체의 손상 강건성이다. 다른 문서 벤치마크가 안 재는 축: 벤치마크가 자기 도구가
적대적 입력에 죽지 않음을 인증한다. 첫 주행이 HWP3 파서의 실제 DoS 2건을 잡았다
(line-spacing 곱셈 i32 오버플로 패닉 — 이 PR 에서 수정 · 무한루프 1건 — 후속 이슈).

## 코퍼스 퍼징 발견 엔진 — DoS 를 근본원인별로 색출한다

`robustness.py` 가 릴리스 **게이트**(바운드된 부분집합으로 "패닉·행 0" 강제)라면,
`fuzz_corpus.py` 는 그 앞단의 **발견 엔진**이다. 전 코퍼스를 여러 명령·여러 손상으로
**exhaustive** 하게 병렬로 두들겨, 아직 안 고쳐진 DoS 를 **소스 위치(file:line)별로
클러스터링**해 "고쳐야 할 고유 버그 목록"을 낸다.

```bash
python gym/tools/fuzz_corpus.py --bin target/debug/rhwp                     # 전 코퍼스·기본 명령
python gym/tools/fuzz_corpus.py --bin <bin> --commands info,export-text --json
```

- 패닉은 `panicked at file:line` 로 클러스터(스택 오버플로·어보트 코드도 별도 버킷).
- 무한루프는 timeout → 명령·샘플별 버킷.

아무도 손으로 수백 문서를 수천 가지로 퍼징하지 않는다 — 에이전트가 이걸 돌려 rhwp 를
계속 경화한다(발견 → 수정 → `robustness.py` 게이트가 회귀를 막음). 이 캠페인의 실제
DoS(렌더러·파서 오버플로·무한루프·스택 오버플로)를 전부 이 엔진이 잡았다.

## 설계 원칙 (채점기가 지키는 것)

- 표준 라이브러리 전용, Windows/리눅스 경로 안전.
- 오라클 부패 없음 — 기대값은 항상 라이브 재계산.
- 부정 판정 없음 — 채점기는 제출물이 "무엇을 했는지"만 본다. 어떻게 했는지
  (몇 번 실패했는지, 어떤 경로로 왔는지)는 기록하지 않는다. 운동장은 감시가
  아니라 놀이다.
