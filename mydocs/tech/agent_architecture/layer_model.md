---
kind: canonical
status: active
canonical: mydocs/tech/agent_architecture/layer_model.md
last_verified: 2026-08-03
---

# 에이전트 표면 4층 성숙도 모델

> **v0.8.4 현행성 주의:** 본문의 Python·Node 바인딩 실측은 2026-08-03 당시의
> historical evidence다. 두 공식 바인딩과 전용 테스트·배포는 #4655에서 철회됐으며,
> 현재 L3 지원 표면이나 다음 작업 지시로 해석하지 않는다.

> rhwp 의 에이전트 표면은 **바닥에서 위로** 자랐다. 명령이 필요해서 명령을 만들고,
> 가드가 필요해서 가드를 만들고, 문서가 필요해서 문서를 썼다. 그 결과 **층은 생겼는데
> 층 사이의 순서가 없다.** 이 문서는 그 순서를 확정한다.
> 로드맵 7개의 전수 지도는 [로드맵 지도](roadmap_atlas.md), 축 진입점은 [README](README.md),
> 원 제안은 [#3880](https://github.com/edwardkim/rhwp/issues/3880).

이 문서의 모든 기술 주장에는 **이슈 번호 · PR 번호 · 코드 경로 · 실측 명령 출력** 중
하나가 붙는다. 근거를 대지 못하는 항목은 **"확인되지 않음"** 으로 적었다(§9).
추측을 사실처럼 적은 아키텍처 문서는 반년 뒤 잘못된 우선순위의 근거가 된다.

**측정 기준** — 이 문서의 모든 실측은 2026-08-03, `<저장소>/target/release/rhwp.exe`
(`rhwp v0.8.2`, 로컬 릴리스 빌드)와 `upstream/devel` 기준 워크트리에서 얻었다.
바이너리 커밋과 `upstream/devel` 의 동일성은 확인하지 않았다(§9 U-1).

---

## 0. 이 문서가 하는 것과 하지 않는 것

### 하는 것

- 열린 로드맵 7개를 **네 개의 성숙도 층**으로 배치하고, 그 구분의 근거를 논증한다
- 층마다 **완성 정의(DoD)** 를 내린다 — "이 층이 끝났다"고 말할 수 있는 조건
- "아래 층이 위 층의 전제"라는 문장이 **구체적으로 무슨 뜻인지** 세 형태로 나눈다
- **L1 의 구멍이 L3 를 막는 실례 8건**을 실측으로 제시한다(§4). 이 절이 이 문서의 본체다
- 층마다 **지금 여기가 어디까지 왔는가**와 **무엇이 막고 있는가**를 실측과 함께 적는다

### 하지 않는 것

- **새 기능을 제안하지 않는다.** 이 문서에 등장하는 모든 항목은 이미 열린 이슈·PR 이거나
  실측으로 재현한 현재 동작이다
- **기존 로드맵을 대체하지 않는다.** 로드맵 본문의 권위는 각 이슈에 있다.
  이 문서는 그 사이의 **순서**만 정한다
- **구현 스택을 다시 설계하지 않는다.** [#3719](https://github.com/edwardkim/rhwp/issues/3719)
  의 6층 스택은 그대로 유효하다 — 이 문서는 **다른 축의 층**이다(§1)

---

## 1. 층 기호 충돌 — 먼저 정리해야 할 것

### 1.1 같은 기호가 다른 것을 가리킨다

[#3719](https://github.com/edwardkim/rhwp/issues/3719) 는 에이전트 표면을 **L0~L6 의 6층**
으로 서술한다. [#3880](https://github.com/edwardkim/rhwp/issues/3880) 은 열린 로드맵 7개를
**L1~L4 의 4층**으로 서술한다. **두 체계는 같은 `L1`·`L3`·`L4` 기호를 쓰면서 전혀 다른
것을 가리킨다.**

| 기호 | #3719 에서 | #3880 에서 |
|---|---|---|
| L1 | CLI `--json` 기계 계약 | **표면** — "있는가" |
| L2 | MCP 무상태 도구 | **신뢰** — "믿을 수 있는가" |
| L3 | MCP 세션 도구 | **도달** — "쓸 수 있는가" |
| L4 | 계획 실행기 `rhwp run` | **표준** — 클라우드·접근성·생태계 |

이 충돌을 방치하면 "L4 를 먼저 하자"는 문장이 **계획 실행기 v2** 를 뜻하는지
**M26~M30 표준화**를 뜻하는지 알 수 없다. 두 해석의 우선순위는 정반대다.

### 1.2 명명 규약 — 이 축이 쓰는 표기

이 문서와 [로드맵 지도](roadmap_atlas.md)는 다음 표기를 쓴다. 다른 문서를 인용할 때는
반드시 접두어를 붙인다.

| 축 | 표기 | 무엇을 재는가 | 권위 |
|---|---|---|---|
| **구현 스택** | `S0`~`S6` | 무엇이 무엇 **위에 올라가는가** | [#3719](https://github.com/edwardkim/rhwp/issues/3719) §1 |
| **성숙도 사다리** | `L1`~`L4` | 무엇이 무엇의 **전제인가** | [#3880](https://github.com/edwardkim/rhwp/issues/3880) §2, 이 문서 |

`S`(Stack)는 #3719 의 L0~L6 을 그대로 옮긴 것이고, `L`(Ladder)은 #3880 의 4층이다.

```
구현 스택 S                              성숙도 사다리 L
─────────────────────────────           ───────────────────────
S6  도메인 매크로                        L4  표준   ← S0~S6 이 굳은 뒤에 의미
S4  계획 실행기 rhwp run                 L3  도달   ← 진입로·바인딩·설치
S3  MCP 세션 도구                        L2  신뢰   ← 보안·퍼징·재작업 제거
S2  MCP 무상태 도구                      L1  표면   ← S1~S6 전체의 계약 정합
S1  CLI --json 기계 계약
S0  엔진 코어
```

**두 축의 관계** — 성숙도 `L1`("표면이 있는가")은 구현 스택 `S1`~`S6` **전체**를 포괄한다.
계획 실행기(`S4`)에 자기서술 구멍이 있으면 그것도 `L1` 문제다. 즉 `L` 은 `S` 의 부분집합이
아니라 **`S` 를 가로지르는 품질 축**이다. 이것이 §4 의 실례가 `S1`·`S2`·`S4` 에 골고루
흩어져 있는 이유다.

---

## 2. 네 층 — 정의·완성 정의·현황

### 2.1 층 배치 요약

| 층 | 질문 | 속하는 로드맵 | 오늘 상태 |
|---|---|---|---|
| **L1 표면** | 있는가 | [#3608](https://github.com/edwardkim/rhwp/issues/3608) · [#3719](https://github.com/edwardkim/rhwp/issues/3719) | 넓다. **정합이 안 맞는다** |
| **L2 신뢰** | 믿을 수 있는가 | [#3787](https://github.com/edwardkim/rhwp/issues/3787) · [#3793](https://github.com/edwardkim/rhwp/issues/3793) · [#3796](https://github.com/edwardkim/rhwp/issues/3796) · M21 | 문서·탐지는 착지. **CI 미편입** |
| **L3 도달** | 쓸 수 있는가 | [#3828](https://github.com/edwardkim/rhwp/issues/3828) · [#3869](https://github.com/edwardkim/rhwp/issues/3869) · M18~M20·M24 | **devel 에 아무것도 없다** |
| **L4 표준** | 남는가 | M26~M30 | 미착수. **지금은 옳다** |

---

### 2.2 L1 — 표면 ("있는가")

#### 무엇을 파는가

에이전트가 **부를 수 있는 동사**와, 그 동사가 돌려주는 **기계가 읽는 봉투**.
CLI `--json`, MCP 무상태·세션 도구, 계획 실행기, 그리고 그 전부를 설명하는
`capabilities` 자기서술이 여기 속한다.

#### 완성 정의 (DoD)

L1 은 "명령이 많다"로 끝나지 않는다. 다섯 조건을 **동시에** 만족해야 한다.

1. **자기서술 = 실물.** `capabilities` 가 선언한 플래그·필드·하위 명령이 실제 CLI 에서
   전부 동작하고, 실제 봉투에 실린 키가 전부 선언돼 있다
2. **봉투 어휘 단일.** 전 명령의 봉투 키가 같은 규약(camelCase)을 따르고,
   `schemaVersion`·출처 표지를 빠짐없이 싣는다
3. **exit 사전 준수.** 미지 옵션 = 2, 런타임 실패 = 1, 검증 실패 = 3 이 전 명령에서 동일
4. **실패 시 stdout 규약 단일.** `jsonContract.failure` 가 선언한 것과 실물이 같다
5. **제외의 근거가 명시적.** 기계 계약을 주지 않는 명령은 "왜 안 주는가"가 문서에 있고,
   그 명령이 `--json` 을 받으면 **침묵하지 않고 거절**한다

#### 지금 여기가 어디까지 왔나 (2026-08-03 실측)

```
$ rhwp capabilities | jq '{명령:(.commands|length),
                            json:([.commands[]|select(.json)]|length),
                            게이트:([.commands[]|select(.requiresFeature)]|length)}'
{ "명령": 61, "json": 31, "게이트": 1 }
```

| 지표 | 2026-08-01 (#3719 §2-1) | **2026-08-03 (본 문서)** | 증감 |
|---|---:|---:|---:|
| CLI 명령 총수 | 54 | **61** | +7 |
| `--json` 기계 계약 | 21 | **31** | +10 |
| MCP 무상태 도구 | 23 | **39** | +16 |
| MCP 세션 도구 | 12 | **12** | 0 |
| 서버 총 노출(개발통합) | 35 | **51** | +16 |
| 계약 테스트 | 215건 / 38파일 | **523건 / 61파일** | +308 |
| 전체 테스트 | 1,486건 / 405파일 | **1,864건 / 440파일** | +378 |

명령 카테고리: `diagnostic` 25 · `export` 18 · `query` 8 · `internal` 5 · `edit` 3 ·
`serve` 1 · `batch` 1.

세션 도구 12종(실측):
`hwp_open` · `hwp_close` · `hwp_doc_{text,info,fields,tables,search,render_page,fill_fields,replace_text,set_cell,save}`

역할 프로필 7종 노출 도구 수(실측):
경영보고 6 · 행정서식 20 · 데이터분석 6 · 콘텐츠제작 6 · 아카이브검색 15 · 품질검증 6 ·
개발통합 51.

**표면의 폭은 문제가 아니다.** 이틀 만에 `--json` 계약이 21 → 31 로 늘었다.
문제는 **정합**이다.

#### 무엇이 L1 을 막고 있는가

DoD 다섯 조건 중 **1·3·4·5 가 오늘 실측으로 깨진다**(§4 에서 전수).
요약하면 이렇다.

| DoD | 위반 실례 | §4 항목 |
|---|---|---|
| 1 자기서술 = 실물 | `inspect` 하위 명령이 선언에 없다 / `--password` 선언 7건 미배선 | B5 · B7 |
| 2 봉투 어휘 단일 | `structure.node_count` 1건 | B1 |
| 3 exit 사전 준수 | `dump --bogus-flag` → exit 1 (사전상 2) | B4 |
| 4 실패 stdout 규약 | `run` 194B · `bench --json` 407B (사전상 0B) | B2 |
| 5 제외의 명시성 | `dump`/`diag` 가 `--json` 을 침묵 무시 | B4 |

---

### 2.3 L2 — 신뢰 ("믿을 수 있는가")

#### 무엇을 파는가

**봉투를 믿어도 되는 근거.** 신뢰할 수 없는 문서가 에이전트를 조종하지 못하게 막는 계약
([#3787](https://github.com/edwardkim/rhwp/issues/3787) 구현 /
[#3793](https://github.com/edwardkim/rhwp/issues/3793) 문서), 파서가 임의 입력에서
죽지 않는다는 증거(M21 퍼징), 그리고 기여가 같은 실수를 반복하지 않게 하는 작업 순서
([#3796](https://github.com/edwardkim/rhwp/issues/3796)).

#### 완성 정의 (DoD)

1. **보장·비보장이 문서에 확정**돼 있고, 그 문서가 `mydocs/tech/` 권위 표에 등재돼 있다
2. **탐지가 봉투 필드로 보고**되고, 정상 문서 전수에서 **오탐 0**
3. **악성 코퍼스가 red→green 을 실증**한다 — 방어 전엔 통과하고 후엔 잡힌다
4. **퍼징이 CI 에서 돈다.** 인프라 존재가 아니라 **실행**이 기준이다
5. **선검사(재작업 제거)가 나머지 축의 작업 순서에 명시**돼 있다

#### 지금 여기가 어디까지 왔나 (실측)

**착지한 것** — `mydocs/tech/agent_security/` 11편 5,626줄이 워크트리에 존재한다
(PR [#3800](https://github.com/edwardkim/rhwp/pull/3800)). `mydocs/tech/README.md`
권위 표에 등재돼 있다(고아 문서 0). 탐지 명령도 실물로 동작한다.

```
$ rhwp inspect hidden-text samples/field-01.hwp --json
{"clean":true,"hiddenCharCount":0,"hiddenText":[],"includeOffPage":false,
 "schemaVersion":"1.0","source":"samples/field-01.hwp",
 "thresholdPt":1.0,"untrustedContent":false,"untrustedFields":[]}
```

`inspect injection` 은 `injectionSignals`·`highestConfidence`·`scanScopes`,
`inspect unicode` 는 `findings`·`kindCounts`·`severityCounts` 를 싣는다(실측).
출처 표지는 전 봉투에 실린다 — 오늘 확인한 8개 조회 명령 전부가
`untrustedContent`·`untrustedFields` 를 가진다.

`tools/agent_preflight.py` 와 `mydocs/manual/agent_preflight_guide.md` 도 워크트리에 있다
(PR [#3795](https://github.com/edwardkim/rhwp/pull/3795)).

**착지하지 않은 것** — 퍼징이다.

```
$ ls fuzz/fuzz_targets/
parse_hml.rs  parse_hwp.rs  parse_hwp3.rs  parse_hwpx.rs
parse_ooxml_chart.rs  parse_wmf.rs          ← 타깃 6종

$ grep -ril fuzz .github/ | wc -l
0                                            ← CI 어디에서도 안 돈다
```

**타깃 6종이 있는데 CI 가 한 번도 부르지 않는다.** L2 DoD 4 미충족이다.
(로드맵 [#3608](https://github.com/edwardkim/rhwp/issues/3608) M21 은 "타깃 4종"으로
적혀 있으나 실측은 6종이다 — 로드맵 본문이 실물보다 낡았다.)

#### 무엇이 L2 를 막고 있는가

1. **퍼징 CI 미편입** — 인프라와 실행 사이의 간극. PR [#3877](https://github.com/edwardkim/rhwp/pull/3877)
   이 운영 문서 4편을 열어 뒀으나 **문서이지 워크플로가 아니다**
2. **악성 코퍼스 회귀 스위트 미착지** — PR [#3867](https://github.com/edwardkim/rhwp/pull/3867)
   (S10) 열림. DoD 3 미충족
3. **선검사 자체의 오탐** — PR [#3872](https://github.com/edwardkim/rhwp/pull/3872) 가
   "스키마를 출력하는 명령이 자기 스키마 안의 오류 설명 문자열에 걸려 스스로를 미구현으로
   신고"하는 오탐 2건을 고친다. [#3796](https://github.com/edwardkim/rhwp/issues/3796) §7 의
   수용 기준("깨끗한 devel 에서 오탐 0")이 **오늘 기준 미충족**이다
4. **L1 의 봉투 구멍** — 보안 축의 결론은 "봉투를 읽어라"인데(consumer_guide),
   그 봉투가 부분 목록을 부분 목록이라고 말하지 않는다(§4 B3)
5. **문서의 현행성 — 하루 만에 낡았다.**
   [agent_security/README.md](../agent_security/README.md) 는
   "현재 `rhwp edit` 의 하위 명령은 `fill-fields`·`replace-text`·`set-cell` **3종뿐**
   (2026-08-02 실측). 개인정보 마스킹 명령(`edit redact`)은 **설계된 것이고 아직 없다**"
   라고 적었다. 오늘 실측은 다르다.

   ```
   $ rhwp edit
   사용법: rhwp edit <fill-fields|replace-text|set-cell|insert-image|redact|sanitize> …
                                                              ← 3종이 아니라 6종
   ```

   이 문서는 스스로 "아직 없는 필드를 있는 것처럼 쓰면 그 문서는 즉시 거짓말이 된다"고
   경고했는데, **반대 방향으로 같은 일**이 일어났다 — 있는 것을 없다고 적고 있다.
   **표면이 이틀에 절반씩 바뀌는 동안 문서의 `last_verified` 가 하루면 낡는다.**
   L2 의 산출물이 문서이므로, 이것은 문체 문제가 아니라 **L2 의 신뢰 문제**다

---

### 2.4 L3 — 도달 ("쓸 수 있는가")

#### 무엇을 파는가

**처음 오는 에이전트가 실제로 도달할 수 있는 경로.**
이름을 몰라도 찾게 하는 다리([#3828](https://github.com/edwardkim/rhwp/issues/3828)),
바이너리 없이 시작하게 하는 진입로([#3869](https://github.com/edwardkim/rhwp/issues/3869) · M24),
자기 언어에서 쓰게 하는 바인딩(M18~M20).

#### 완성 정의 (DoD)

1. **도메인 지식 0 인 에이전트가 왕복 1~2회로 부트스트랩**을 끝낸다
2. **바이너리 없이 시작**할 수 있는 경로가 최소 하나 있다
3. **모든 진입로의 봉투가 동일**하다 — 진입로마다 다른 문서를 읽어야 하면 이 층은 실패다
4. **진입로별 실패 사전과 판단표**가 있다 — 언제 어느 경로를 쓰는지
5. 각 다리가 `AGENTS.md`·`llms.txt`·에이전트 지식 지도에서 **링크로 발견**된다

#### 지금 여기가 어디까지 왔나 (실측)

**착지한 것** — 바인딩 두 종(M18·M19)이다. `bindings/python`(`.py` 36개)·
`bindings/node`(`.ts` 48개)가 워크트리에 있다(PR [#3775](https://github.com/edwardkim/rhwp/pull/3775) ·
[#3779](https://github.com/edwardkim/rhwp/pull/3779)). `bindings/Native/src/lib.rs` 376줄 ·
`bindings/csharp/RhwpNative.cs` 63줄 · `bindings/swift/Sources/` 2파일 274줄도 존재한다 —
M20 미착수라고 적혀 있으나 **디렉터리는 다른 계약이 이미 점유**하고 있다
([#3879](https://github.com/edwardkim/rhwp/pull/3879) §8).

**착지하지 않은 것 — 나머지 전부다.**

```
$ grep -c '"explain"' src/main.rs                → 0
$ grep -c 'export-agent-manifest' src/main.rs    → 0
$ ls mydocs/manual/recipes                       → 없음
$ grep -c 'capabilities --search' src/main.rs    → 0
```

[#3828](https://github.com/edwardkim/rhwp/issues/3828) 의 다리 4개(B1~B4)는
PR [#3836](https://github.com/edwardkim/rhwp/pull/3836) ·
[#3843](https://github.com/edwardkim/rhwp/pull/3843) ·
[#3835](https://github.com/edwardkim/rhwp/pull/3835) ·
[#3832](https://github.com/edwardkim/rhwp/pull/3832) 로 **네 건 전부 열려 있고 devel 에 없다.**

[#3869](https://github.com/edwardkim/rhwp/issues/3869) 의 WASM 표면(W1~W6)도 마찬가지다.
`src/wasm_api.rs` 는 7,621줄 · `wasm_bindgen` 372곳으로 크지만, PR
[#3873](https://github.com/edwardkim/rhwp/pull/3873) 의 실측 인용에 따르면
`digest`·`extract-data`·`inspect` 3종·`run`·`capabilities` 자기서술이 **WASM 표면에 없다.**
코드가 아니라 **설계 문서 2건**(PR #3873 · #3876)만 열려 있다.

#### 무엇이 L3 를 막고 있는가

1. **L1 의 봉투 구멍** — §4 전체가 이 항목이다. 특히 B1(snake_case)·B2(실패 stdout 예외)·
   B5(하위 명령 미선언)·B6(`-o` 조합에서 봉투 소실)은 **L3 의 DoD 3(봉투 동일성)을
   직접 무효화**한다
2. **머지 큐** — L3 조각이 전부 열린 PR 이다. 2026-08-03 실측 열린 PR **22건**으로,
   [#3719](https://github.com/edwardkim/rhwp/issues/3719) §7 과
   [#3796](https://github.com/edwardkim/rhwp/issues/3796) §6 이 못박은 "10건 내외"를 초과한다
3. **축의 중복** — 같은 WASM 주제로 두 문서 디렉터리가 동시에 열려 있다
   (§[로드맵 지도 D2](roadmap_atlas.md#32-d2--wasm-축이-두-곳에서-자란다))

---

### 2.5 L4 — 표준 ("남는가")

#### 무엇을 파는가

[#3608](https://github.com/edwardkim/rhwp/issues/3608) §8 의 M26~M30 —
클라우드 변환 청사진 · 접근성·국제화 · 생태계 협업 규약 · 신뢰 공개 지표 · 표준화 제안.

#### 완성 정의 (DoD)

이 층의 DoD 를 지금 쓰는 것은 이르다. **착수 판단 기준**만 적는다.

L4 는 **L3 가 굳은 뒤**에 의미가 있다. 구체적으로:

- 외부 소비자가 실제로 붙어 있어야 "생태계 협업 규약"(M28)이 대상 없는 규약이 아니게 된다
- 봉투가 안정돼야 "공개 명세 v1"(M30)이 6개월 뒤 자기 자신과 모순되지 않는다
- 공개 지표(M29)는 **측정 대상이 안 흔들려야** 지표가 된다

#### 지금 여기가 어디까지 왔나

**미착수다. 그리고 그것이 옳다.**

[#3608](https://github.com/edwardkim/rhwp/issues/3608) §8 말미의 원칙 —
**"근거 없는 항목(엔진·RFC·실측 무관)은 넣지 않는다"** — 을 L4 착수에 그대로 적용하면,
오늘 M26~M30 을 시작하는 것은 그 원칙을 스스로 어기는 일이다. 근거가 될 외부 채택 실측이
아직 없다.

#### 무엇이 L4 를 막고 있는가

L3 다. 그 외에 별도의 장애물은 **확인되지 않음**(§9 U-4).

---

## 3. "아래 층이 위 층의 전제"가 구체적으로 무슨 뜻인가

이 문장은 그냥 두면 수사(修辭)다. 세 가지 서로 다른 관계를 구분해야 실제 판단에 쓸 수 있다.

### 3.1 논리적 전제 — 아래가 없으면 위의 명제가 성립하지 않는다

가장 강한 형태다. 위 층이 **주장하는 문장 자체**가 아래 층의 사실에 의존한다.

> **예** — [#3869](https://github.com/edwardkim/rhwp/issues/3869) W2 는
> "WASM 반환값이 CLI `--json` 봉투와 **같은 모양**임을 계약 테스트로 고정"한다고 약속한다.
> 그런데 `export-tables` 는 `-o` 를 주면 봉투 대신 사람 문장을 낸다(§4 B6).
> 그러면 **"같은 모양"의 기준이 두 개**가 된다. 계약 테스트는 둘 중 하나를 골라야 하고,
> 무엇을 고르든 나머지 하나가 반례가 된다. **명제가 성립하지 않는다.**

논리적 전제는 **위에서 우회할 수 없다.** 우회하려면 아래 층의 사실을 바꾸거나
위 층의 명제를 약화시키는 수밖에 없고, 후자는 그 축을 포기하는 것과 같다.

### 3.2 비용 전제 — 아래가 부실하면 위의 비용이 곱해진다

아래 층의 불규칙 1건이 위 층에서 **진입로 수만큼 복제**된다.

> **예** — 봉투에 snake_case 키가 하나 섞여 있다(§4 B1). CLI 한 곳의 결함이지만,
> 이것을 위에서 흡수하려면 Python·Node·C#·Swift·WASM **각 바인딩마다** 예외 처리를
> 한 줄씩 넣어야 한다. [#3879](https://github.com/edwardkim/rhwp/pull/3879) 가 확정한
> 명명 규약 — "원문 키는 어떤 바인딩도 바꾸지 않는다" — 하에서는 그 예외를
> **지울 수도 없다.** 바인딩이 늘수록 예외가 늘어난다.

비용 전제는 **지금 고치면 1, 나중에 고치면 N** 이라는 구조다. N 이 커지기 전에 고쳐야 한다.

### 3.3 신뢰 전제 — 아래가 흔들리면 위의 주장이 검증 불가가 된다

위 층이 하는 말이 틀렸다는 뜻이 아니라, **맞는지 확인할 방법이 없어진다**는 뜻이다.

> **예** — 보안 축(L2)의 소비자 계약은 "봉투를 읽고 판단하라"이다. 그런데
> `info --json` 은 파서가 건너뛴 요소가 있어도 그 사실을 봉투에 싣지 않는다(§4 B3).
> `fonts` 가 부분 목록인데 봉투는 그렇다고 말하지 않는다. 소비자가 봉투를 성실히 읽어도
> **"이 봉투가 완전한가"를 알 방법이 없다.** 보안 문서의 결론이 참인지 거짓인지가 아니라,
> **검증 가능성**이 사라진다.

신뢰 전제는 조용하다. 아무것도 깨지지 않고 exit 0 이 나온다. 그래서 가장 늦게 발견된다.

### 3.4 왜 위에서 고칠 수 없는가 — 일반 원리

상위 층은 하위 층 봉투의 **동형사상**을 판다. "MCP 도구가 CLI 와 같은 어휘를 쓴다",
"바인딩이 CLI 봉투를 그대로 돌려준다", "WASM 이 CLI 와 같은 모양을 낸다" — 전부
"원본과 같다"는 약속이다.

원본이 비정합이면 상위 층에 남는 선택지는 둘뿐이다.

1. **비정합을 그대로 복제한다** → 결함이 진입로 수만큼 늘어난다(비용 전제)
2. **상위에서 몰래 고친다** → 진실이 두 개가 된다. 로그·버그리포트가 CLI 출력과 갈라지고,
   왕복(`to_snake ∘ to_camel = id`)이 깨진다.
   [#3879](https://github.com/edwardkim/rhwp/pull/3879) 가 이 근거로 **일괄 변환을 금지**했다

둘 다 실패다. **그래서 아래에서 고쳐야 한다.** 이것이 §4 의 여덟 실례가 전부
"L1 을 먼저"로 수렴하는 이유다.

---

## 4. L1 의 구멍이 L3 를 막는다 — 실례 8건

> 이 절이 이 문서의 본체다. 각 항목은 **오늘 실측으로 재현**한 것이고,
> 형식은 동일하다: **증상(실측) → 어느 위층을 막는가 → 전제의 형태 → 왜 위에서 못 고치는가**.

### 4.1 한눈에

| # | 증상 | 막는 위층 | 전제 형태 | 상태 |
|---|---|---|---|---|
| B1 | 봉투에 snake_case 1건 | M20 C#/Swift · 전 바인딩 | 비용 | PR [#3882](https://github.com/edwardkim/rhwp/pull/3882) 열림 |
| B2 | 실패 시 stdout 규약 예외 2건 | [#3869](https://github.com/edwardkim/rhwp/issues/3869) W2 봉투 동등성 | 논리 | **미착수** |
| B3 | `info` 가 건너뛴 것을 숨긴다 | L2 소비자 계약 | 신뢰 | PR [#3882](https://github.com/edwardkim/rhwp/pull/3882) 열림 |
| B4 | `--json` 침묵 무시 + exit 사전 이탈 | [#3828](https://github.com/edwardkim/rhwp/issues/3828) B1 키워드 발견 | 논리 | **미착수** |
| B5 | 하위 명령이 자기서술에 없다 (`inspect`·`edit`) | [#3828](https://github.com/edwardkim/rhwp/issues/3828) B2 부트스트랩 | 논리 | **미제기** |
| B6 | `-o` 조합에서 봉투가 사람 문장으로 | [#3869](https://github.com/edwardkim/rhwp/issues/3869) W2 · 전 바인딩 | 논리 | **미착수** |
| B7 | `--password` 선언 7건 미배선 | M4 "완료" 주장 · 세션 보호 문서 | 신뢰 | PR [#3839](https://github.com/edwardkim/rhwp/pull/3839) 열림 |
| B8 | 프로필 등재 누락 14건 | 역할 프로필 도달 경로 | 비용 | PR [#3838](https://github.com/edwardkim/rhwp/pull/3838) 열림 |

---

### 4.2 B1 — 봉투에 snake_case 가 하나 섞여 있다

**증상 (실측)**

```
$ rhwp export-structure --json samples/field-01.hwp | (전체 재귀 순회)
snake keys: ['.structure.node_count']   count 1
```

최상위는 `nodeCount` 인데 중첩된 `structure` 객체만 `node_count` 다.
전체 재귀 순회에서 `_` 가 든 키는 **이것 하나**다(`info` 는 0건 — 실측).

**무엇을 막는가**

M20(C#/Swift). 별칭 조회 계층이 없는 **정적 매핑 언어에서 이 필드는 사라진다.**
`{ [JsonPropertyName("nodeCount")] }` 로 매핑하면 값이 안 들어오고,
안 들어온다는 사실조차 조용하다.

**전제의 형태** — 비용(§3.2). 오늘 CLI 한 곳에서 고치면 1줄이고,
M20 이 시작된 뒤에 고치면 언어 수만큼의 예외가 이미 심어진 상태에서 고쳐야 한다.

**왜 위에서 못 고치나** — [#3879](https://github.com/edwardkim/rhwp/pull/3879) 가
"원문 키는 어떤 바인딩도 바꾸지 않는다"를 명명 규약으로 확정했다. 바인딩이 고치면
그 규약을 깨는 것이고, 규약을 깨면 계획서 왕복(`to_snake ∘ to_camel = id`)이 무너진다.

**상태** — PR [#3882](https://github.com/edwardkim/rhwp/pull/3882) 가 T3 로 닫는다.
회귀 가드가 **"이름만 바꾸고 값을 잃은 수정"까지 막는다**(`nodeCount` 값 존재를 함께 단언).

---

### 4.3 B2 — 실패 시 stdout 규약에 예외가 둘 있다

**증상 (실측)**

자기서술은 이렇게 말한다.

```
$ rhwp capabilities | jq -r '.jsonContract.failure'
단건 명령 실패 시 stdout 0바이트; batch 는 error 레코드 + 최종 exit 1
```

실물은 다르다.

```
$ rhwp info --json <없는 파일>                 → exit 1, stdout   0 B   ← 규약대로
$ rhwp run <없는 입력을 가리키는 계획> --json   → exit 1, stdout 194 B   ← 예외
$ rhwp bench --json                            → exit 1, stdout 407 B   ← 예외(사람용 표 헤더)
```

`run` 의 194바이트는 유효한 JSON 봉투다(`{"error":…,"schemaVersion":"1.0",…}`) —
[#3880](https://github.com/edwardkim/rhwp/issues/3880) T4 가 지적한 **의도된 설계**이며
MCP `hwp_run_plan` 과 저널을 공유하기 위한 것이다.
**`bench --json` 의 407바이트는 JSON 도 아니다** — 사람용 표 머리글이다.
이쪽은 #3880 에 기재되지 않은 항목으로, 본 문서가 새로 확인했다.

**무엇을 막는가**

[#3869](https://github.com/edwardkim/rhwp/issues/3869) W2 — "WASM 반환값이 CLI `--json`
봉투와 같은 모양임을 계약 테스트로 고정". **무엇과 같아야 하는지가 자기서술로 정해지지
않는다.** WASM 에는 프로세스도 exit 코드도 stdout 도 없으므로, 실패 표현은 **CLI 규약을
번역**해야 하는데 그 규약이 실물과 다르다.

**전제의 형태** — 논리(§3.1). 기준이 흔들리면 동등성이라는 명제 자체가 정의되지 않는다.

**왜 위에서 못 고치나** — WASM 쪽이 "우리는 항상 봉투를 낸다"로 정해버리면
`info` 계열(stdout 0B)과 어긋나고, "실패 시 아무것도 안 낸다"로 정하면 `run` 과 어긋난다.
어느 쪽을 골라도 **CLI 의 절반과 다른 표면**이 만들어진다.

**상태** — 미착수. #3880 T4 는 "자기서술이 예외를 적지 않는다"까지 지적했고,
계약 테스트가 걸려 판단이 필요하다고 남겨 뒀다(PR #3882 §남은 T2·T4).
`bench --json` 은 이 문서가 추가로 확인한 것으로 **어느 이슈에도 없다.**

---

### 4.4 B3 — `info` 가 건너뛴 것을 숨긴다

**증상 (실측)**

```
$ rhwp info --json samples/field-01.hwp | jq 'has("warnings")'
false

$ rhwp info --json samples/field-01.hwp | jq -r 'keys[]'
fonts format pageCount paraCount schemaVersion sections sizeBytes source
title untrustedContent untrustedFields version
```

`show_info()` 가 JSON 분기에서 경고 출력에 도달하지 못한다. 결과적으로
**리소스가 조용히 잘린 문서가 exit 0 + 완전해 보이는 봉투**를 낸다.
[#3719](https://github.com/edwardkim/rhwp/issues/3719) §3-5 불변식 **"부분 목록 금지"**
위반이다.

**무엇을 막는가**

L2 전체의 소비자 계약. 보안 축의 결론은
[consumer_guide](../agent_security/consumer_guide.md) 로 요약하면 "봉투를 읽고 판단하라"인데,
그 봉투가 **자신이 부분인지 전부인지 말하지 않는다.**

**전제의 형태** — 신뢰(§3.3). 아무것도 깨지지 않는다. exit 0 이 나오고 JSON 이 유효하다.
그래서 소비자가 잘못 판단한 사실을 **아무도 모른다.**

**왜 위에서 못 고치나** — 위 층은 "봉투에 없는 정보"를 만들어낼 수 없다.
MCP 도구도 바인딩도 WASM 도 `show_info()` 가 내지 않은 경고를 발명할 수 없다.

**상태** — PR [#3882](https://github.com/edwardkim/rhwp/pull/3882) 가 T1 로 닫는다.
회귀 가드가 **"항상 빈 배열을 내는 구현"까지 막는다**(HML 표본에서 실제 경고가 실리는지 확인).
다만 그 PR 이 스스로 **한계를 명시**한다 — 현재 경고 원천은 HML 파서 하나뿐이므로,
빈 배열이 "문서가 온전하다"는 뜻은 아니다.

---

### 4.5 B4 — `--json` 침묵 무시 + exit 사전 이탈

**증상 (실측)**

```
$ rhwp dump samples/field-01.hwp --json   → exit 0, stdout 18,642 B (사람용 텍스트)
$ rhwp diag samples/field-01.hwp --json   → exit 0, stdout    614 B (사람용 텍스트)
```

`capabilities` 는 이 명령들에 `json` 도 `flags` 도 선언하지 않는다(실측: 둘 다 `null`).
**선언 없는 옵션을 조용히 삼킨다.**

더 나아가 exit 사전도 어긋난다.

```
$ rhwp capabilities | jq -r '.exitCodes["2"]'
사용법 오류 (인자 없음, 알 수 없는 옵션/명령, 페이지 범위 초과)

$ rhwp dump --bogus-flag samples/field-01.hwp   → exit 1   ← 사전상 2여야 한다
$ rhwp diag --bogus-flag samples/field-01.hwp   → exit 1   ← 같음
$ rhwp info --json --bogus-flag samples/…       → exit 2   ← 선언 명령은 정상
```

**무엇을 막는가**

[#3828](https://github.com/edwardkim/rhwp/issues/3828) B1 — `capabilities --search <키워드>`.
키워드 검색이 명령을 찾아 주는 순간, 에이전트는 그 명령이 **계약 안에 있다고 가정**한다.
찾아준 명령이 `--json` 을 침묵으로 삼키면 다리는 **에이전트를 계약 밖으로 안내**한 셈이다.

**전제의 형태** — 논리(§3.1). "발견 가능한 것은 호출 가능하다"는 다리의 전제가 거짓이 된다.

**왜 위에서 못 고치나** — 검색은 `capabilities` 선언을 대상으로 한다.
선언에 없는 명령을 검색이 감출 수는 있어도, **선언에 없으면서 실물에는 있는 옵션**을
검색이 알 방법은 없다.

**상태** — 미착수. #3880 T2 로 등록돼 있고,
PR [#3882](https://github.com/edwardkim/rhwp/pull/3882) 가
"`capabilities` 미선언 명령 3종의 정책 결정이 필요"하다고 남겼다.
`dump --bogus-flag` 의 **exit 1** 은 이 문서가 추가로 확인한 것으로 어느 이슈에도 없다.

---

### 4.6 B5 — 하위 명령이 자기서술에 없다

**증상 (실측)**

`capabilities` 는 `inspect` 를 이렇게 선언한다.

```
inspect | json=True | flags=['--json','--threshold-pt','--include-offpage',
                             '--min-confidence','--include-fields','--kind']
```

**하위 명령에 대한 언급이 없다.** 그런데 실물은 하위 명령을 요구한다.

```
$ rhwp inspect --json samples/field-01.hwp
오류: 알 수 없는 inspect 하위 명령입니다 - --json
사용법: rhwp inspect <hidden-text|injection|unicode> <파일.hwp|파일.hwpx> [각 축 옵션]
                                                                     → exit 2, stdout 0 B

$ rhwp inspect hidden-text samples/field-01.hwp --json                → exit 0, stdout 191 B
```

**`edit` 도 같다.** 하위 명령이 여섯인데 선언에는 플래그 23개만 있다.

```
$ rhwp edit
오류: edit 하위 명령을 지정해주세요.
사용법: rhwp edit <fill-fields|replace-text|set-cell|insert-image|redact|sanitize> …
                                                                     → exit 2, stdout 0 B
```

즉 **하위 명령을 갖는 명령 2종이 전부 자기서술에 하위 명령을 적지 않는다.**
(`batch` 는 `--mode` 플래그로 표현하므로 이 문제를 겪지 않는다 — 실측)

`--help` 도 같은 경로로 떨어진다.

```
$ rhwp inspect --help    → exit 2, stdout 0 B  (도움말이 아니라 "알 수 없는 하위 명령")
$ rhwp info    --help    → exit 2, stdout 0 B  ("알 수 없는 옵션: --help")
$ rhwp help info         → exit 2, stdout 0 B  (`help` 하위 명령 없음)
```

명령별 도움말 진입점이 `rhwp --help`(29,590 B 통짜 출력) 하나뿐이다.

**무엇을 막는가**

[#3828](https://github.com/edwardkim/rhwp/issues/3828) B2 — `export-agent-manifest --json`
("왕복 1회로 부트스트랩"). 매니페스트가 `capabilities` 를 그대로 실으면 **실행 불가능한
호출 형태를 가르친다.** 에이전트는 선언대로 `rhwp inspect --json <파일>` 을 만들고 exit 2 를 받는다.

`inspect` 는 [#3787](https://github.com/edwardkim/rhwp/issues/3787) S2~S4 의 산출물이므로,
[#3828](https://github.com/edwardkim/rhwp/issues/3828) B3 레시피(문서 안전성 점검)도 같은
경로를 밟는다.

**전제의 형태** — 논리(§3.1). 자기서술의 목적은 "이것을 읽으면 호출을 만들 수 있다"인데
그 명제가 거짓이다.

**왜 위에서 못 고치나** — MCP 도구는 `hwp_inspect_*` 형태로 하위 명령을 도구 이름에
접었으므로 이 문제를 겪지 않는다. 그러나 **CLI 를 직접 부르는 소비자**(레시피·바인딩·
셸 파이프라인)에게는 `capabilities` 가 유일한 사양이다. 상위에서 하위 명령 목록을
발명할 수 없다.

**상태** — **어느 이슈에도 없다.** 이 문서가 처음 기록한다.

---

### 4.7 B6 — `-o` 를 붙이면 봉투가 사람 문장이 된다

**증상 (실측)**

```
$ rhwp export-tables samples/field-01.hwp --json
{"schemaVersion":"1.0","source":"samples/field-01.hwp","tableCount":0,
 "tables":[],"untrustedContent":false,"untrustedFields":[]}          → exit 0, 128 B

$ rhwp export-tables samples/field-01.hwp -o <출력디렉터리> --json
표 추출 완료: 0개 → <출력디렉터리>                                   → exit 0, 124 B
```

**같은 명령, 같은 `--json`, 옵션 하나 차이로 봉투가 사라진다.**
`jsonContract.stdout` 이 못박은 "데이터(JSON/NDJSON)만 — 진단·진행·요약은 stderr" 위반이다.

**무엇을 막는가**

전 바인딩과 [#3869](https://github.com/edwardkim/rhwp/issues/3869) W2.
[#3879](https://github.com/edwardkim/rhwp/pull/3879) 는 이 결함 때문에 **바인딩이 `-o`
옵션을 아예 닫았다**고 기록했다 — "회피일 뿐 수정이 아니다".
같은 유형의 결함이 `convert(out=)`·`export_hwpx(out=)` 에서는 **항상 `UsageError`(exit 2)**
로 나타난다(D-1, 실행으로 확인됨).

**전제의 형태** — 논리(§3.1). 동일 명령이 옵션에 따라 두 가지 출력 계약을 가지면,
"CLI 봉투와 같은 모양"이라는 약속의 **대상이 정의되지 않는다.**

**왜 위에서 못 고치나** — 바인딩이 실제로 해 본 유일한 대응이 **기능 제거**였다.
상위 층에서 사람 문장을 봉투로 되돌릴 방법은 없다.

**상태** — [#3879](https://github.com/edwardkim/rhwp/pull/3879) 가 "본체 결함 2건 (별도
이슈감)"으로 지목했다. **이슈로 승격되지 않았다.**
#3880 §1 의 T1~T4 에도 들어 있지 않다.

---

### 4.8 B7 — `--password` 를 선언하고도 암호 문서를 못 연다

**증상**

PR [#3839](https://github.com/edwardkim/rhwp/pull/3839) 제목이 그대로 증상이다 —
"`--password` 를 선언하고도 암호 문서를 못 열던 명령 **7건** + 전수 가드".

**무엇을 막는가**

두 가지다.

1. **M4("보호 문서")의 완료 주장.** [#3719](https://github.com/edwardkim/rhwp/issues/3719)
   §8 은 M4 를 **✅ 완료**로 적었다. 선언과 실물이 7건 어긋나는 상태에서의 "완료"다
2. **세션·바인딩·WASM 의 보호 문서 경로.** 상위 진입로는 CLI 인자를 조립해 넘긴다.
   아래에서 안 열리면 위에서도 안 열린다

**전제의 형태** — 신뢰(§3.3). 선언을 읽고 능력을 판단한 소비자가 틀린 결론에 도달한다.

**왜 위에서 못 고치나** — 자기서술이 능력을 광고하고 실물이 거부하는 구조에서,
상위 층이 할 수 있는 일은 "우리도 광고하고 우리도 거부한다"뿐이다.

**상태** — PR [#3839](https://github.com/edwardkim/rhwp/pull/3839) 열림 (2026-08-03).

---

### 4.9 B8 — 프로필 등재 누락 14건

**증상**

PR [#3838](https://github.com/edwardkim/rhwp/pull/3838) — "프로필 도구 등재 누락 **14건**
+ 재발 방지 가드". 오늘 실측한 프로필별 노출 수는 다음과 같다.

```
경영보고 6 · 행정서식 20 · 데이터분석 6 · 콘텐츠제작 6 ·
아카이브검색 15 · 품질검증 6 · 개발통합 51
```

[#3719](https://github.com/edwardkim/rhwp/issues/3719) §2-1 의 2026-08-01 실측
(경영보고 6 · 행정서식 20 · 데이터분석 5 · 콘텐츠제작 6 · 아카이브검색 19 · 품질검증 6 ·
개발통합 36)과 비교하면 **아카이브검색이 19 → 15 로 줄었다.**
서버 총 노출은 35 → 51 로 늘었는데 특정 프로필만 줄어든 원인은 **확인되지 않음**(§9 U-3).

**무엇을 막는가**

역할 프로필은 "이 직무에 필요한 도구만 보여준다"는 **도달 경로의 축소판**이다.
도구가 늘어도 프로필에 등재되지 않으면 그 역할의 에이전트에게는 **존재하지 않는 것과 같다.**

**전제의 형태** — 비용(§3.2). 도구가 늘 때마다 7개 프로필에 대해 판정이 필요하고,
가드가 없으면 누락이 누적된다.

**왜 위에서 못 고치나** — 프로필 등재는 선언 단일 출처에서 나온다.
상위 층이 프로필을 다시 정의하면 그 순간 목록이 복제되고,
[#3719](https://github.com/edwardkim/rhwp/issues/3719) §4 불변식 1("단일 출처")을 깬다.

---

### 4.10 여덟 실례가 말하는 것

여덟 건 중 **네 건은 오늘 열린 PR 이 닫는다**(B1·B3·B7·B8).
**네 건은 어디에도 없거나 판단이 남아 있다**(B2·B4·B5·B6).

그리고 네 건 전부 **크기가 작다.** 봉투 키 하나, 자기서술 문장 하나, 하위 명령 목록 하나,
`-o` 분기 하나. 어느 것도 새 엔진이나 새 설계를 요구하지 않는다.

**작은 것이 큰 것을 막는 구조**란 이런 뜻이다 — 막는 쪽의 크기와 막히는 쪽의 크기는
아무 관계가 없다. 그래서 크기로 우선순위를 정하면 반드시 틀린다.

---

## 5. 층별 현황 종합

| 층 | 있는가 | 실측 근거 | 막는 것 |
|---|---|---|---|
| **L1 표면** | 넓다 (명령 61 / json 31 / MCP 39 / 세션 12) | `capabilities`, `tools/list` | §4 여덟 구멍 중 4건 미착수 |
| **L2 신뢰** | 문서 11편 5,626줄 · 탐지 3축 동작 | 워크트리 존재, `inspect` 실행 | 퍼징 CI 0 · 코퍼스 미착지 · 선검사 오탐 |
| **L3 도달** | 바인딩 2종만 | `bindings/python` 36 · `node` 48 | 다리 4개 · WASM 전부 열린 PR |
| **L4 표준** | 없다 | — | L3 |

### 5.1 층별 진행률을 숫자 하나로 말하지 않는 이유

[#3608](https://github.com/edwardkim/rhwp/issues/3608) §7 은 체크리스트를
**"진행률의 유일 기준"** 으로 못박았다. 오늘 그 체크리스트를 세면 **8 / 196 (4.1%)** 이다.

그런데 [#3719](https://github.com/edwardkim/rhwp/issues/3719) §8 은 M1·M2·M4·M16 을
**✅ 완료**로 적는다. 실측은 두 문서 사이에 있다 — M2(세션 조회·렌더)는 세션 도구 12종이
실제로 뜨므로 참이고, M4(보호 문서)는 `--password` 미배선 7건이 있으므로 거짓이다(§4 B7).

**같은 대상에 대해 세 개의 답이 있다.** 그래서 이 문서는 층별 진행률을 백분율로 적지 않고
**"무엇이 있고 무엇이 막고 있는가"** 로만 적는다. 이 모순의 전문은
[로드맵 지도 §3.3](roadmap_atlas.md#33-d3--진행률의-유일-기준이-셋이다)에 있다.

---

## 6. 층 판정 절차 — 새 조각이 오면 어디에 넣나

새 이슈·PR 이 들어왔을 때 **세 질문을 순서대로** 묻는다. 먼저 참이 되는 곳이 그 조각의 층이다.

1. **이것이 없으면 기존 봉투·자기서술·exit 계약이 실물과 어긋나는가?**
   → 그렇다면 **L1**. 새 기능처럼 보여도 L1 이다
2. **이것이 없으면 기존 표면의 결과를 믿을 근거가 약해지는가?**
   → 그렇다면 **L2**
3. **이것이 없으면 새 소비자가 도달할 수 없는가?**
   → 그렇다면 **L3**
4. 셋 다 아니면 → **L4**. 그리고 L4 는 지금 착수하지 않는다(§2.5)

**판정 예시**

| 조각 | 판정 | 근거 |
|---|---|---|
| `capabilities --search`([#3836](https://github.com/edwardkim/rhwp/pull/3836)) | L3 | 1·2 거짓, 3 참(이름을 모르면 도달 불가) |
| `inspect` 하위 명령 선언(§4 B5) | **L1** | 1 참 — 자기서술과 실물이 어긋난다 |
| 악성 코퍼스([#3867](https://github.com/edwardkim/rhwp/pull/3867)) | L2 | 1 거짓, 2 참(탐지 주장의 근거) |
| WASM 봉투 동등성([#3869](https://github.com/edwardkim/rhwp/issues/3869) W2) | L3 | 3 참. 단 **L1 B2·B6 가 논리적 전제** |
| 퍼징 CI 편입 | L2 | 2 참 — 견고성 주장의 근거 |

**주의** — 1번 질문이 가장 자주 놓친다. `inspect` 하위 명령 선언은 "발견 기능"처럼 보여
L3 로 분류되기 쉽지만, **선언과 실물의 불일치**이므로 L1 이다.

---

## 7. 지금 무엇을 먼저 해야 하는가

[#3880](https://github.com/edwardkim/rhwp/issues/3880) §3 의 순서를 이 문서의 실측으로
보강한 것이다. **근거가 없는 순서는 취향이므로, 각 항목에 근거를 붙인다.**

| 순서 | 무엇 | 근거 |
|---|---|---|
| **1** | §4 여덟 구멍 중 **미착수 4건**(B2·B4·B5·B6) | L3 전체의 논리적 전제(§3.1). 넷 다 작고 독립적 |
| **2** | 열린 PR 22 → 10 이하로 축소 | [#3719](https://github.com/edwardkim/rhwp/issues/3719) §7 · [#3796](https://github.com/edwardkim/rhwp/issues/3796) §6 이 못박은 자기 규율. **오늘 위반 중** |
| **3** | [#3796](https://github.com/edwardkim/rhwp/issues/3796) 선검사를 나머지 여섯 로드맵에 명시 | 이미 만들어 devel 에 있는데 여섯 곳 어디에도 없다 |
| **4** | WASM 축 통합 판정([#3869](https://github.com/edwardkim/rhwp/issues/3869) ↔ M24) | 문서 디렉터리 2개가 동시에 열려 있다([D2](roadmap_atlas.md#32-d2--wasm-축이-두-곳에서-자란다)) |
| **5** | M21 퍼징 CI 편입 | 타깃 6종 존재, `grep -ril fuzz .github/` = **0** |
| **6** | M18~M20 표류 20건 정리 | 바인딩이 늘기 전에. 치명 3건 실행 확인([#3879](https://github.com/edwardkim/rhwp/pull/3879)) |
| **7** | L4 착수 판단 | L3 가 굳은 뒤 다시 본다 |

#3880 §3 과의 차이는 두 곳이다.

- **순서 2(PR 큐 축소)를 새로 넣었다** — 열린 PR 22건은 L3 조각을 전부 담고 있다.
  큐가 안 빠지면 순서 1 을 아무리 잘 해도 L3 는 devel 에 도달하지 않는다
- **순서 1 의 범위를 좁혔다** — #3880 은 T1~T4 넷을 묶었으나, T1·T3 은 PR
  [#3882](https://github.com/edwardkim/rhwp/pull/3882) 로 이미 열렸다.
  남은 것은 T2·T4 와 이 문서가 새로 찾은 B5·B6 이다

---

## 8. 반증 조건 — 이 모델이 틀렸다고 인정하는 경우

아키텍처 문서가 반증 불가능하면 그건 신념이지 모델이 아니다. 다음 중 하나가 관측되면
이 4층 모델을 수정한다.

1. **L1 구멍을 고치지 않고도 L3 조각이 정상 착지한다** — 예컨대 §4 B2·B6 이 열린 채로
   WASM 봉투 동등성 계약 테스트가 red→green 으로 고정되고, 그 계약이 CLI 의 두 가지
   실패 표현을 모두 만족한다면, "L1 이 L3 의 논리적 전제"라는 §3.1 주장은 틀린 것이다
2. **L2 없이 L3 가 채택된다** — 퍼징도 코퍼스도 없는 상태에서 외부 소비자가 붙고
   문제가 생기지 않는 기간이 충분히 관측되면, L2 를 L3 의 전제로 둔 배치가 과했던 것이다
3. **층 판정이 실제 조각에서 갈린다** — §6 의 세 질문으로 같은 조각을 두 사람이 다르게
   분류하는 사례가 반복되면, 층 경계 자체가 잘못 그어진 것이다

---

## 9. 확인되지 않음

이 문서가 **근거를 대지 못한 것**들이다. 사실처럼 쓰지 않는다.

| # | 항목 | 왜 확인 못 했나 |
|---|---|---|
| **U-1** | 측정에 쓴 로컬 릴리스 빌드가 `upstream/devel` 과 **동일 커밋**인지 | 이 작업에서 git 명령을 쓰지 않았다. 간접 근거만 있다 — 측정된 명령 61종이 전부 워크트리 `src/main.rs` 에 존재하고, 열린 PR 의 신규 명령(`explain`·`export-agent-manifest`·`capabilities --search`)은 바이너리에도 소스에도 **없다** |
| **U-2** | 닫힌 PR 167건 중 실제로 devel 에 반영된 것이 몇 건인지 | GitHub API 상 `mergedAt` 이 있는 것은 **11건뿐**이고 나머지 167건은 병합 표시 없이 CLOSED 다. 개별 산출물의 워크트리 존재로 확인한 것만 이 문서에 근거로 썼다(`agent_security/` 11편, `tools/agent_preflight.py`, `bindings/python`·`node`) |
| **U-3** | 프로필 `아카이브검색` 이 19 → 15 로 **줄어든** 원인 | 서버 총 노출은 35 → 51 로 늘었는데 이 프로필만 감소했다. 프로필 정의 변경인지 등재 누락([#3838](https://github.com/edwardkim/rhwp/pull/3838))인지 판별하지 못했다 |
| **U-4** | L4(M26~M30)에 **L3 외의 장애물**이 있는지 | M26~M30 은 착수된 적이 없어 실측할 대상이 없다 |
| **U-5** | Node 바인딩의 **실행** 동작 | [#3879](https://github.com/edwardkim/rhwp/pull/3879) 가 이미 명시했다 — 이 PC 에 `node_modules` 가 없어 `vitest`/`tsc` 미실행. 이 문서의 Node 관련 서술은 전부 그 PR 의 코드 경로 인용이다 |
| **U-6** | §4 B4 의 `dump --bogus-flag` → exit 1 이 **의도된 설계인지 결함인지** | 해당 명령은 기계 계약 명시 제외 대상이므로, exit 사전이 제외 명령에도 적용되는지가 문서에 없다 |

---

## 10. 관련 문서

### 이 축

- [축 지도 · 읽는 순서 · 지금 할 일](README.md)
- [로드맵 7개 전수 지도](roadmap_atlas.md) — 중복·모순 표 포함

### 인접 축

- [에이전트 보안 문서 지도](../agent_security/README.md) — L2 의 권위.
  특히 [위협 모델](../agent_security/threat_model.md)·[공격 표면](../agent_security/attack_surface.md)·
  [소비 에이전트 가이드](../agent_security/consumer_guide.md)
- [경량 에이전트 내성 — CLI·MCP 계약 확장 4건](../weak_agent_proofing.md) — L1 내성 계약의 전신
- [에이전트 경계 무결성 계약](../agent_boundary_contract.md) — L2 경계 계약

### 절차

- [에이전트 표면 플레이북](../../manual/agent_surface_playbook.md) — 표면 추가의 절차·수용 기준
- [에이전트 선검사 가이드](../../manual/agent_preflight_guide.md) — [#3796](https://github.com/edwardkim/rhwp/issues/3796) 의 도구
- [에이전트 지식 지도](../../manual/agent_knowledge_map.md) · [에이전트 실패 사전](../../manual/agent_troubleshooting_guide.md)
- [CLI 명령 레퍼런스](../../manual/cli_commands.md) — 현재 동작은 항상 `rhwp capabilities` 로 재확인

### 이슈

- [#3880](https://github.com/edwardkim/rhwp/issues/3880) 탑다운 로드맵 — 이 축의 발원
- 일곱 로드맵: [#3608](https://github.com/edwardkim/rhwp/issues/3608) ·
  [#3719](https://github.com/edwardkim/rhwp/issues/3719) ·
  [#3787](https://github.com/edwardkim/rhwp/issues/3787) ·
  [#3793](https://github.com/edwardkim/rhwp/issues/3793) ·
  [#3796](https://github.com/edwardkim/rhwp/issues/3796) ·
  [#3828](https://github.com/edwardkim/rhwp/issues/3828) ·
  [#3869](https://github.com/edwardkim/rhwp/issues/3869)
