---
kind: guide
status: active
canonical: mydocs/tech/agent_runtime/surface_spec.md
last_verified: 2026-08-03
---

# 진입로 판단표 — 지금 이 작업에 무엇을 쓸 것인가

rhwp를 부르는 공식 경로는 지금 다섯이다: CLI `--json` 단건 · CLI `batch` · MCP 무상태 도구 ·
MCP 세션 도구 · 계획 실행기 `run`. 별도 WASM 에이전트 표면이
설계 단계에 있다([surface_spec.md](surface_spec.md), 로드맵
[#3869](https://github.com/edwardkim/rhwp/issues/3869)).

**다섯 개가 같은 일을 다르게 한다.** 어느 것을 골라도 봉투는 같지만
([envelope_parity.md](envelope_parity.md)), 비용·상태·원자성·실패 표현이 다르다.
이 문서는 그 선택을 **대신 내려준다**. "상황에 따라 다르다"로 끝내지 않는다.

- 각 경로의 실제 비용 숫자: [cost_model.md](cost_model.md)
- 경로별 실패 증상 사전: [failure_dictionary.md](failure_dictionary.md)
- 축 전체 지도와 읽는 순서: [README.md](README.md)
- 명령·플래그의 권위: [CLI 명령어 매뉴얼](../../manual/cli_commands.md)

이 문서의 모든 성능 주장은 [cost_model.md](cost_model.md) 의 실측에 근거한다.
측정하지 않은 것은 "미측정"이라고 쓰고 추정치를 쓰지 않는다.

---

## 0. 관문 — 바이너리를 구할 수 있는가

공식 CLI와 MCP 서버(`rhwp mcp-serve`)는 `rhwp` 실행 파일을 요구한다. v0.8.4에서
공식 Python·Node 바인딩은 철회됐으므로(#4655), 아래 바인딩 비교는 철회 전 실측
이력으로만 읽는다.

```
바이너리를 놓을 수 있나?
├─ 아니오 (브라우저·샌드박스·설치 권한 없음)
│   └─ 지금은 **아무 경로도 쓸 수 없다.** 이것이 #3869 의 동기다.
│      브라우저 임베드가 목적이면 @rhwp/editor(iframe+WASM)를 보되,
│      그것은 뷰어/에디터 컴포넌트이지 에이전트 봉투 표면이 아니다.
└─ 예 → §1 로

바이너리는 있는데 기능이 없나? (예: export-png)
└─ `rhwp capabilities` 의 해당 명령 `available` 필드를 먼저 본다.
   추측하지 않는다 — 없는 feature 는 exit 2 로 끝난다.
```

관문을 통과한 뒤의 선택이 §1 부터다.

---

## 1. 다섯 갈래 결정 흐름

```
① 이 작업을 한 프로세스 안에서 스크립트로 묶을 수 있나?
│  (= 모델 턴을 매 단계 쓰지 않아도 되나)
├─ 예 ─┬─ 파일이 여럿인가?
│      │  ├─ 예 → **CLI batch**            (§3.2)
│      │  └─ 아니오 → **CLI --json 단건**   (§3.1)
└─ 아니오 (모델이 매 단계 판단해야 한다)
   │
   ② 같은 문서를 여러 번 조회하나?
   ├─ 예 → **MCP 세션** (hwp_open → … → hwp_close)  (§3.4)
   └─ 아니오 → **MCP 무상태 도구**                    (§3.3)

③ 편집이 여러 단계이고 "전부 되거나 전부 안 되거나"여야 하나?
└─ 예 → **계획 실행기 run** (다른 축과 배타가 아니다 — 편집 축의 기본값) (§3.5)
```

### 판단을 가르는 여섯 축

| 축 | 물어볼 것 | 답이 이쪽이면 | 답이 저쪽이면 |
|---|---|---|---|
| A. 반복성 | 같은 문서·같은 명령을 두 번 이상 부르나 | 반복 → batch·세션 | 1회 → 단건 CLI·무상태 MCP |
| B. 모델 턴 | 매 호출마다 모델이 판단해야 하나 | 예 → MCP | 아니오 → CLI(batch 포함) |
| C. 상태 | 문서를 파싱한 채로 들고 있어야 하나 | 예 → MCP 세션 | 아니오 → 무상태 전부 |
| D. 원자성 | 중간 실패 시 디스크가 깨끗해야 하나 | 예 → `run` | 아니오 → 개별 `edit` |
| E. 산출물 | 결과가 **파일**인가 **값**인가 | 파일 → CLI | 값 → CLI JSON·MCP |
| F. 실행 환경 | 프로세스를 띄울 수 있나 | 예 → CLI·MCP | 아니오 → 현재 없음(#3869) |

**축 A 와 B 가 사실상 두 개의 큰 갈림길이다.** 나머지는 그 안에서 갈린다.

---

## 2. 비용이 만드는 경계선 (요약)

판단표를 읽기 전에 알아야 할 숫자 넷. 조건과 원문은
[cost_model.md](cost_model.md) 에 있다 — 여기서는 결론만 쓴다.

| 사실 | 값 | 무엇을 가른다 |
|---|---|---|
| 프로세스 기동 바닥값 | **약 73.9 ms/회** (`rhwp --version` ×20) | 호출을 몇 번 할지 |
| 대형 문서 파싱 | **약 134 ms/회** (393쪽, 10.7 MB) | 재파싱을 피할 값어치 |
| 세션 재조회 | **0.68 ms/쪽** (같은 문서, 재파싱 없음) | 세션 vs 무상태 |
| 배치 이득 | 개별 20건 2,318 ms → batch **308 ms** (7.5배) | 단건 vs batch |

정리하면 이렇다.

- **호출을 2회 이상 할 것 같으면** 프로세스를 다시 띄우지 않는 경로(batch·세션·
  `run`)를 먼저 검토한다. 기동 73 ms 는 작아 보이지만 20회면 1.5초다.
- **같은 문서를 2회 이상 열 것 같으면** 세션이다. 393쪽 문서에서 20쪽을 읽을 때
  CLI 는 4,419 ms, 세션은 148 ms 다(열기+읽기+닫기 전부 포함).
- **문서가 크면 무엇을 부르는지가 컨텍스트를 결정한다.** 같은 393쪽 문서에서
  `digest` 는 1,375 B, `export-text` 는 645,108 B — **469배**다.

---

## 3. 진입로별 카드

각 카드는 **잘 맞는 경우 / 안 맞는 경우**를 구체 작업으로 적는다.
"작지 않은 문서"류의 모호한 서술을 쓰지 않는다.

### 3.1 CLI `--json` 단건

한 프로세스가 파일 하나를 열고 봉투 하나를 stdout 에 내고 끝난다.
[cli_json_pipeline_guide.md](../../manual/cli_json_pipeline_guide.md) 가 실무 사례집이다.

**잘 맞는 경우**

- 셸 파이프라인의 한 칸. `rhwp search --json 계약서.hwp "위약금" | jq '.matches[0].page'`
- CI 게이트. 종료 코드가 그대로 `&&`·`set -e` 에 물린다.
- **쪽 주소가 필요한 조회.** `export-text --json` 의 `pages[]` 는 쪽 단위 청킹을
  공짜로 준다 — batch 레코드에는 이 배열이 없다(§3.2).
- 산출물이 **파일**인 명령(`export-pdf`·`export-svg`·`extract-pages`).
  MCP도 결국 이 경로를 부른다.

**안 맞는 경우**

- **같은 문서에 명령을 3회 이상 거는 것.** 393쪽 문서에서 `info`→`export-structure`
  →`search` 를 각각 부르면 파싱만 3번 한다(약 400 ms). 세션으로 한 번 열어라.
- **파일 20개에 같은 명령.** 개별 2,318 ms vs `batch` 308 ms. 자동화 스크립트가
  `for f in *.hwp; do rhwp info --json "$f"; done` 을 쓰고 있으면 그것이 신호다.
- **대형 문서 전체 본문을 모델 컨텍스트에 넣는 것.** 645 KB 는 컨텍스트가 아니라
  파일이다. `digest` → `export-structure` → 좁힌 `-p N` 순서로 간다(§4).

### 3.2 CLI `batch`

stdin 의 파일 목록을 **한 프로세스**가 전건 처리해 NDJSON 으로 흘린다.
축은 `export-text·info·export-structure·export-tables·fields·search·convert·fill`.

**잘 맞는 경우**

- **아카이브 대장화.** `find docs/ -name '*.hwp' | rhwp batch info --json > meta.ndjson`
  후 `jq` 로 선별. 파일 수가 늘수록 이득이 커진다(실측 이득 7.3~7.6배, N=20).
- **메일머지.** `batch fill --form 서식.hwp --data rows.jsonl --out-dir out/`.
  이 축만 stdin 을 읽지 않는다 — 입력이 경로가 아니라 데이터 '행'이기 때문이다.
- 실패를 흘리지 않아야 하는 무인 운영. 건별 실패는 `error`/`exitClass` 레코드로
  격리되고 성공분은 이미 스트림에 있다.

**안 맞는 경우**

- **쪽 주소가 필요한 RAG 청킹.** `batch export-text` 레코드는 `text`(문서 전체
  한 덩어리)를 주고 `pages[]` 를 주지 않는다. 실측: 같은 27쪽 문서에서 batch
  레코드 46,137 B / 단건 봉투 46,714 B — **크기는 같은데 쪽 주소가 없다.**
  쪽 인용이 목적이면 선별 후 단건으로 내려받는다.
- **모델이 파일마다 판단해야 하는 작업.** batch 는 스크립트용이다. 파일마다
  "이건 계약서인가?"를 모델이 결정해야 하면 `batch info` 로 대장을 만든 뒤
  모델은 대장만 보고 고른다 — 그것이 batch 의 올바른 쓰임이다.
- **한 건이 극단적으로 느린 코퍼스.** 병렬 막차는 가장 느린 파일이 정한다.
  `batch info` 로 먼저 규모를 재고 큰 것을 분리한다.

### 3.3 MCP 무상태 도구

`mcp-serve` 가 도구 호출을 받아 **rhwp 자신을 서브프로세스로** 실행하고 봉투를
그대로 돌려준다. CLI 계약의 얇은 껍데기다
([mcp_integration_guide.md](../../manual/mcp_integration_guide.md)).

**잘 맞는 경우**

- **모델이 도구를 직접 고르는 대화형 작업.** 호스트 설정 한 줄로 붙고
  (`{"mcpServers":{"rhwp":{"command":"rhwp","args":["mcp-serve"]}}}`) 인자 조립을
  모델이 스키마로 한다.
- 파일이 1~2개이고 명령도 1~2개인 단발 질의. 서버 기동+핸드셰이크가
  **약 50 ms** 라 왕복 비용이 CLI 한 번과 비슷하다.
- `structuredContent` 를 소비할 수 있는 호스트. 문자열 재파싱을 아낀다.

**안 맞는 경우**

- **대형 봉투를 그대로 받는 것.** MCP 응답은 같은 내용을 `content[0].text`(이스케이프된
  JSON 문자열)와 `structuredContent`(객체)에 **두 번** 담는다. 실측 배율
  **2.03~2.46배** — 393쪽 `export_text` 는 CLI 645,107 B, MCP 응답 1,309,290 B.
  큰 것을 부를 거면 무상태 도구로 부르지 말고 `-p`·`--max-matches` 로 먼저 좁힌다.
- **도구 51개를 통째로 노출하는 것.** `tools/list` 응답이 **40,503 B** 다. 직무가
  정해진 에이전트라면 `mcp-serve --profile <직무>` 로 6~8개만 연다(§5).
- **같은 문서 반복 조회.** 무상태 도구는 호출마다 자식 프로세스를 띄우고 재파싱한다.
  서버 안에서 잰 `hwp_info`(393쪽) 왕복이 **189 ms** 인 반면 세션 `hwp_doc_text` 는
  **0.68 ms** 다.

### 3.4 MCP 세션 도구

`hwp_open` 이 파싱된 문서를 서버 프로세스 안에 잡아두고 `docId` 를 준다.
이후 조회·편집은 재파싱이 없다. 세션 도구 12종은
`agent_profiles::ALL_SESSION_TOOLS` 가 단일 출처다.

**잘 맞는 경우**

- **대형 문서 탐색.** "393쪽 편람에서 결재 절차가 어디 있는지 찾아 그 앞뒤 3쪽을
  읽어라" — `hwp_open` 1회(125 ms) + `hwp_doc_search` + `hwp_doc_text` 몇 번(쪽당
  0.68 ms). 같은 일을 CLI 로 하면 호출마다 132 ms 재파싱이다.
- **읽고 고치고 다시 읽는 편집 루프.** `hwp_doc_set_cell` → `hwp_doc_tables` 로
  확인 → 만족하면 `hwp_doc_save`. 중간 산출물이 디스크에 남지 않는다.
- 모델이 **쪽을 넘겨가며** 판단하는 검토 작업.

**안 맞는 경우**

- **한 번만 볼 문서.** `hwp_open`+`hwp_close` 가 왕복 2회를 더 쓴다. 1회 조회는
  무상태 도구가 짧다.
- **핸들을 오래 들고 있는 것.** 핸들은 서버 프로세스 수명과 같고 영속되지 않는다.
  서버가 내려가면 전부 사라진다 — 세션을 "저장소"로 쓰면 안 된다.
- **여러 문서 대량 처리.** 세션은 문서마다 메모리를 잡는다. 아카이브 스윕은
  `hwp_batch` 다.
- **원자적 다단계 편집.** 세션 편집은 단계마다 적용된다. 3단계 중 2번째가
  실패하면 1번은 이미 IR 에 반영돼 있다 — 그럴 땐 `run` 이다(§3.5).

### 3.5 계획 실행기 `run`

편집 계획(JSON)을 받아 **전 step 을 정적 선검증**하고, 통과한 것만 인메모리로
원자 실행해 단언이 통과할 때만 **한 번** 저장한다. 실패 시 디스크는 무변경이다.

**잘 맞는 경우**

- **서식 한 장을 여러 조작으로 완성.** 누름틀 채움 + 표 셀 두 칸 + 체크박스 하나를
  한 계획에 담는다. 실측: 2 step 계획 `run` **89 ms** vs `edit set-cell` 2회 체이닝
  **165 ms**(같은 문서, 5회 평균). 프로세스도 중간 파일도 하나씩 준다.
- **선검증이 필요한 곳.** 잘못된 표 번호·병합 칸·없는 필드·치환 0건을 **실행 전에**
  `invalid[]` 로 전부 모아 보고하고 exit 2 로 끝낸다. 하나 고치면 다음 위반이
  나오는 두더지잡기를 없앤다.
- **되돌릴 수 없는 작업의 사전 확인.** `--dry-run` 이 `preview[]` 로 각 step 의
  `currentText`→`newText` 를 보여준다(480 B).

**안 맞는 경우**

- **단일 조작.** step 하나면 `edit` 단건이 더 짧고 읽기 쉽다.
- **결과를 보고 다음 step 을 정해야 하는 작업.** 계획은 미리 다 적어야 한다.
  "표를 읽고 그 값에 따라 어느 칸을 채울지 정한다"면 세션에서 조회한 뒤
  계획을 만들어 `run` 에 넘긴다 — 조회와 실행을 나눈다.
- `run` 이 지원하지 않는 편집(`insert_image`·`redact`·`sanitize`). v1 의 step 은
  `fill_fields`·`replace_text`·`set_cell`·`set_checkbox` 넷이다.

### 3.6 외부 언어 래퍼

공식 Python·Node 바인딩은 v0.8.4에서 철회됐다(#4655). 코드에서 결과를 다룰 때는
CLI JSON을 직접 파싱하거나 MCP 클라이언트를 사용한다. 다운스트림 래퍼를 선택하면
버전·오류 매핑·배포와 성능 검증은 해당 프로젝트의 책임이다.

### 3.7 WASM 에이전트 표면 (아직 없음)

설계는 [surface_spec.md](surface_spec.md) 에서 확정 중이고 **구현체가 없다.**
성능은 **미측정**이며, 이 문서는 WASM 에 대해 어떤 성능 주장도 하지 않는다.
"바이너리를 놓을 수 없는 환경"의 답이 지금 비어 있다는 사실 자체가 #3869 다.

---

## 4. 컨텍스트 예산 — 무엇을 먼저 부를 것인가

큰 문서에서 진입로보다 더 큰 차이를 만드는 것은 **첫 호출의 선택**이다.
같은 393쪽 문서에서(실측, 조건은 [cost_model.md](cost_model.md) §3):

| 첫 호출 | 봉투 | 배율 |
|---|---:|---:|
| `digest --json` | 1,375 B | 1× |
| `info --json` | 636 B | 0.46× |
| `digest --json --sections` | 2,692 B | 2× |
| `export-structure --json` | 284,892 B | 207× |
| `export-text --json` | 645,108 B | **469×** |
| `export-tables --json` | 759,030 B | **552×** |

권장 순서는 하나다.

```
digest → (필요하면) export-structure 또는 digest --sections
       → search --max-matches N 으로 위치 좁히기
       → export-text -p N 으로 그 쪽만 (393쪽 문서에서 324 B)
```

**주의: `--max-chars` 는 컨텍스트 절약 수단이 아니다.** 393쪽 문서에
`--max-chars 1` 을 줘도 봉투는 **22,904 B** 다 — 쪽 객체 393개의 골격이 남기
때문이다. 절약하려면 `-p N`(324 B)이나 `digest --pages a..b`(398 B)로 **쪽을 좁혀야**
한다.

작은 문서에서는 반대다. 1쪽 문서에서 `digest` 2,003 B > `export-text` 1,913 B —
`digest` 는 메타+개요+발췌+`nextStep` 을 묶은 매크로라 **바닥값**이 있다.
1~2쪽 문서라면 `export-text` 를 바로 부르는 편이 짧다.

---

## 5. 도구 표면 줄이기 — `--profile`

MCP 로 붙일 때 51개 도구 전부를 노출할 이유가 있는 에이전트는 드물다.
`agent_profiles.rs` 가 직무별 도구 집합의 단일 출처이고,
`capabilities --mcp --profile` 과 `mcp-serve --profile` 을 같은 표가 구동한다.

| 프로필 | 무상태 도구 | 선언 크기 | 세션 |
|---|---:|---:|---|
| (없음) 전체 | 39 | 47,022 B | 전부 |
| 경영보고 | 6 | 6,378 B | 열지 않음 |
| 행정서식 | 8 | 10,237 B | 전부 |
| 데이터분석 | 6 | 6,931 B | 열지 않음 |
| 콘텐츠제작 | 6 | 5,939 B | 열지 않음 |
| 아카이브검색 | 7 | 8,257 B | **조회 전용 8종** |
| 품질검증 | 6 | 5,970 B | 열지 않음 |
| 개발통합 | 39 | 47,490 B | 전부 |

프로필은 추천 목록이 아니라 **서버가 실제로 제공하는 도구 집합의 경계**다.
`아카이브검색` 은 세션을 열되 `hwp_doc_save` 를 주지 않는다 — 읽기 전용을 표방한
프로필이 원본을 덮어쓸 수 없다는 뜻이다.

각 프로필은 `recipe`(권장 호출 순서)도 함께 선언한다. 경량 모델이 순서를 틀리지
않도록 계약으로 주는 것이다.

---

## 6. 흔한 오선택과 교정

| 관측되는 패턴 | 무엇이 잘못됐나 | 대신 |
|---|---|---|
| `for f in *.hwp; do rhwp info --json "$f"; done` | 파일당 프로세스 기동 73.9 ms 를 N배 낸다 | `... \| rhwp batch info --json` (실측 7.5배) |
| 393쪽 문서에 `export-text --json` 먼저 | 645 KB 를 컨텍스트에 넣는다 | `digest`(1,375 B) 로 시작 |
| 같은 문서에 무상태 MCP 도구 5회 | 재파싱 5회 = 660 ms | `hwp_open` 1회 후 세션 도구 |
| `edit` 3개를 `-o` 로 체이닝 | 중간 파일 2개 + 중간 실패 시 디스크 오염 | `run` 계획 하나 |
| `--json` 을 아무 명령에나 붙임 | `dump` 처럼 선언에 없는 명령은 **조용히 무시**하고 사람용 텍스트를 exit 0 으로 낸다 | `capabilities` 의 `json:true`·`flags` 를 먼저 본다 |
| `isError:false` 를 "검증 통과"로 읽음 | MCP 는 판정(차이 발견)을 오류로 만들지 않는다 | 봉투의 `identical`·`notFound` 를 읽는다 |
| 프로필 없이 MCP 서버 등록 | 40 KB 도구 목록이 매 세션 컨텍스트에 들어간다 | `mcp-serve --profile <직무>` |

---

## 7. 한 장 요약

| 하려는 일 | 진입로 | 근거 |
|---|---|---|
| 문서 하나 규모 파악 | CLI `info --json` 또는 MCP `hwp_info` | 636 B, 205 ms |
| 대형 문서 첫 접촉 | `digest --json` | 1,375 B — 본문의 1/469 |
| 아카이브 N건 스윕 | CLI `batch` (`--threads` 기본값 유지) | N=20 에서 7.5배 |
| 대형 문서 여러 쪽 읽기 | MCP 세션 | 20쪽 148 ms vs CLI 4,419 ms |
| 서식 한 장 다단계 채움 | `run` 계획 | 89 ms, 원자적, 선검증 |
| 서식 N행 메일머지 | `batch fill` | 프로세스 1개 |
| 결과를 코드로 다루기 | CLI JSON 또는 MCP | 공개 봉투 계약을 직접 사용 |
| 변환 무손실 판정 | `--verify` + `ir-diff --json` | exit 3 은 오류가 아니라 판정 |
| 브라우저에서 실행 | **현재 없음** | #3869 / [surface_spec.md](surface_spec.md) |

---

## 인접 문서

- [surface_spec.md](surface_spec.md) — WASM 에이전트 표면 설계(이 축의 canonical)
- [envelope_parity.md](envelope_parity.md) — 진입로가 달라도 봉투는 같다는 계약
- [cost_model.md](cost_model.md) — 이 문서의 모든 숫자의 출처와 재현 방법
- [failure_dictionary.md](failure_dictionary.md) — 고른 진입로가 실패했을 때
- [README.md](README.md) — 축 지도
- [cli_commands.md](../../manual/cli_commands.md) — 명령·플래그의 권위
- [cli_json_pipeline_guide.md](../../manual/cli_json_pipeline_guide.md) — CLI 실무 사례
- [mcp_integration_guide.md](../../manual/mcp_integration_guide.md) — MCP 두 경로
- [agent_surface_playbook.md](../../manual/agent_surface_playbook.md) — 표면을 **추가**할 때의 절차
- [agent_troubleshooting_guide.md](../../manual/agent_troubleshooting_guide.md) — 진입로와 무관한 공통 실패
- 이슈 [#3869](https://github.com/edwardkim/rhwp/issues/3869) — 로드맵
