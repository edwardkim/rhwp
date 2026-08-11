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

## 과제판 — pack 10개 · 과제 91건 · 만점 194

능력 영역을 **pack** 으로 나눈다. 점수는 pack 별로 보존되며 총점은 편의값이다 —
어느 능력이 모자란지는 pack 별 점수가 말한다.

| pack | 이름 | 능력 축 | 과제 | 만점 |
|---|---|---|---|---|
| `core-cli` | 코어 CLI | 조사·추출·편집·검증 (운동장 최소 코어) | 14 | 32 |
| `automation` | 자동화·검증 사다리 | 자동화 (계획·캡슐·서명·앵커·정산·감사) | 13 | 35 |
| `corpus-diagnostics` | 코퍼스·진단 | 진단 (폴더 스윕·쪽 덤프·비교 판정) | 7 | 14 |
| `layout-rendering` | 조판·렌더링 | 검증 (조판 판정·렌더 산출) | 8 | 15 |
| `objects-media` | 개체·미디어 | 발견 (필드·개체·렌더 산출물) | 7 | 15 |
| `security` | 보안 스윕 | 보안 (은닉·주입·유니코드·PII) | 9 | 18 |
| `self-description` | 자기서술 표면 | 자기서술 (도구가 스스로를 설명하는 계약) | 7 | 12 |
| `serialization` | 저장·변환 | 변환 (형식 왕복·IR 대조) | 8 | 19 |
| `table-editing` | 표 편집 | 편집 (표 좌표 지정) | 8 | 16 |
| `text-editing` | 본문 편집 | 편집 (탐색→치환→재검증) | 10 | 18 |

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
| `starter` (입문) | `core-cli`, `self-description` |
| `editor` (편집자) | `core-cli`, `text-editing`, `table-editing`, `objects-media` |
| `publisher` (배포자) | `serialization`, `layout-rendering`, `security` |
| `maintainer` (메인테이너) | `automation`, `core-cli`, `corpus-diagnostics`, `layout-rendering`, `objects-media`, `security`, `self-description`, `serialization`, `table-editing`, `text-editing` |

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

**4부 — 대확장(#4653)**: 운동장이 pack 으로 쪼개졌다. core-cli 1개였던 판이 **8개 pack · 과제 65건 · 만점 140** 이 됐고, 판정 논리는 `gym/core/`(runner·schema·check registry)로 모여 pack 이 늘어도 판정 어휘는 한 곳에서만 자란다. 신규 과제 전건이 기준 풀이 왕복으로 실측 등재됐다.

## 설계 원칙 (채점기가 지키는 것)

- 표준 라이브러리 전용, Windows/리눅스 경로 안전.
- 오라클 부패 없음 — 기대값은 항상 라이브 재계산.
- 부정 판정 없음 — 채점기는 제출물이 "무엇을 했는지"만 본다. 어떻게 했는지
  (몇 번 실패했는지, 어떤 경로로 왔는지)는 기록하지 않는다. 운동장은 감시가
  아니라 놀이다.
