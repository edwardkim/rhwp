---
kind: guide
status: active
canonical: mydocs/tech/agent_architecture/provenance_chain.md
last_verified: 2026-08-08
---

# 검증 가능한 문서 이력 체인 (Verifiable Document Provenance Chain)

> 문서 하나에 대해 "**어떤 입력에서, 어떤 조작 열을 거쳐, 어떤 출력이 되었는가**"를
> 기계가 재검증할 수 있는 형태로 남기는 축의 설계와 P1 실물.
> 온톨로지 축(무엇이 있는가)의 다음 층 — **누가/무엇이/어떻게 바꿨는가의 시간·신뢰층**이다.
> 층 배치는 [4층 성숙도 모델](layer_model.md), 전체 지도는 [로드맵 지도](roadmap_atlas.md),
> 봉투 신뢰 경계는 [봉투 출처 표지](../envelope_provenance.md) 참조.
>
> **이 문서는 제안이다. 채택 여부·우선순위·표면 확장은 전적으로 메인테이너 판단에 따른다.**
> refs [#3907](https://github.com/edwardkim/rhwp/issues/3907)
> · [#4229](https://github.com/edwardkim/rhwp/issues/4229)

이웃 문서와 같은 규율을 따른다: 기술 주장에는 **이슈/PR 번호 · 코드 경로(줄번호) ·
실측 출력 · 출처 URL(접근일)** 중 하나가 붙는다. 확인하지 못한 것은 확인하지
못했다고 적는다.

---

## §1 논지 — 5년 시계의 규제·수요 곡선

### 1.1 미디어에는 이미 정착 중이다 (C2PA)

미디어(이미지·영상) 도메인에는 콘텐츠 출처 자격증명 표준 C2PA(Coalition for
Content Provenance and Authenticity)가 정착 중이다. 활발히 개정되는 살아있는
표준이다:

- C2PA Technical Specification **2.2** — 2025-05-01 발행
  ([spec.c2pa.org 2.2](https://spec.c2pa.org/specifications/specifications/2.2/specs/_attachments/C2PA_Specification.pdf), 접근 2026-08-08)
- C2PA Technical Specification **2.3** — 2026-01-05 발행
  ([spec.c2pa.org 2.3](https://spec.c2pa.org/specifications/specifications/2.3/specs/_attachments/C2PA_Specification.pdf), 접근 2026-08-08)

구조는 우리 논의에 그대로 참고가 된다: 자산(asset)에 **매니페스트**를 결속하고,
매니페스트는 자산의 **암호학적 해시 바인딩** + 제작·변형 이력 주장(assertion) +
PKI 서명으로 구성된다. 컨테이너는 ISO 표준(JUMBF, ISO 19566-5)을 쓴다.

### 1.2 AI 산출물 표시는 의무가 되고 있다 (EU·한국)

- **EU AI Act 50조** — 생성형 AI 시스템 제공자는 산출물(텍스트 포함)을
  **기계판독 가능한 형식으로 표시**하고 인공 생성물임을 탐지 가능하게 해야 한다.
  적용 개시 **2026-08-02**
  ([Article 50 전문](https://artificialintelligenceact.eu/article/50/), 접근 2026-08-08).
  이행 기법으로 유럽연합 집행위 FAQ는 "워터마크, 메타데이터 식별, **출처·진본성
  증명을 위한 암호학적 방법**, 로깅, 핑거프린트" 등을 열거한다
  ([EC digital-strategy FAQ](https://digital-strategy.ec.europa.eu/en/faqs/transparency-obligations-under-article-50-ai-act), 접근 2026-08-08).
  2026-06-10 에는 투명성 실천규범(Code of Practice on Transparency of
  AI-Generated Content) 최종본이 공표되었다
  ([EC 가이드라인 페이지](https://digital-strategy.ec.europa.eu/en/policies/guidelines-transparency-ai-generated-content), 접근 2026-08-08).
  (2026-05 AI Omnibus 잠정 합의 기준, 기존 출시 시스템의 기계판독 표시는
  2026-12-02 까지 유예 — 보도 시점 기준 잠정.)
- **한국 AI 기본법**(인공지능 발전과 신뢰 기반 조성 등에 관한 기본법) — **2026-01-22
  시행**. 생성형 AI 결과물에 대한 표시 의무를 부과하며, 위반 시 시정명령·과태료
  (최대 3천만 원)가 규정되어 있다
  ([법률신문 해설](https://www.lawtimes.co.kr/news/articleView.html?idxno=216500),
  [법률사무소 칼럼: 표기 의무 정리](https://bh-law.kr/ko/news/column/ai-content-labeling-obligation-guide), 접근 2026-08-08).

규제의 공통 방향은 명확하다: "AI 가 만들었다"는 **선언**을 넘어, 그 선언을
**기계가 검증할 수 있는 형태**(메타데이터·암호학적 방법·로깅)로 요구하는 쪽으로
움직이고 있다.

### 1.3 사무 문서 도메인은 비어 있다

C2PA 의 자산 포맷 지원 목록에는 JPEG·PNG·MP4·PDF·WebP 등 미디어 포맷이
올라 있으나, **워드프로세서 계열 사무 문서 포맷(OOXML 계열·HWP 계열)은 지원
목록에 없다** ([C2PA FAQ](https://c2pa.org/faqs/), 접근 2026-08-08).

HWP/HWPX 도메인에서 "편집 이력의 기계 증명"을 구현한 선행 사례는 이번 조사
범위(공개 웹, 2026-08-08)에서 **발견하지 못했다**. 부재의 증명은 불가능하므로
"없다"가 아니라 "찾지 못했다"로 적는다 — 다만 표준 지원 목록의 공백과 합치면,
이 도메인이 선점되지 않은 상태라는 판단의 근거로는 충분하다.

### 1.4 설계 관례는 이미 있다 (소프트웨어 공급망)

빌드 산출물 도메인은 같은 문제를 먼저 풀었다:

- **in-toto attestation** — Statement = **subject**(산출물 해시) +
  **predicate**(맥락 주장), 서명은 DSSE 봉투로 분리
  ([in-toto and SLSA](https://slsa.dev/blog/2023/05/in-toto-and-slsa), 접근 2026-08-08).
- **SLSA Provenance** — "이 산출물이 어떤 빌더가 어떤 입력·명령으로 만들었는가"를
  predicate 로 고정한 규격 ([slsa.dev v1.1](https://slsa.dev/spec/v1.1/faq), 접근 2026-08-08).

"입력 해시 → 조작 → 출력 해시 + 도구 신원"이라는 뼈대는 검증된 관례다. 문서
편집은 빌드와 동형이다: 입력 문서 = 소스, 편집 계획 = 빌드 명령, 산출 문서 =
아티팩트.

### 1.5 왜 5년 축인가

AI 에이전트가 공공·기관 문서를 대량 생산·수정하는 향후 5년, 문서 엔진의 질문은
"무엇이 들어 있나"(온톨로지)에서 "**이 내용은 어떻게 여기 왔나**"로 이동한다.
규제(§1.2)는 이를 의무로 만들고, 표준(§1.1)은 미디어에서 먼저 관례를 만들었으며,
사무 문서 도메인(§1.3)은 비어 있다. rhwp 는 아래 §2 의 재료 4종을 이미 갖췄다 —
묶는 층 하나가 없을 뿐이다.

---

## §2 rhwp 현황 대조 — 재료 4종과 공백 (실측)

측정 기준: 이 워크트리, `upstream/devel` `30bad7c1d`, 2026-08-08.

| 재료 | 무엇 | 실측 위치 | 상태 |
|---|---|---|---|
| ① run 계획서 저널 | 조작 열의 1차 기록 | `src/main.rs:345`(디스패치) · `:14039`(`cmd_run_plan`) · `:14203`(`run_plan_engine`) | devel 머지됨 (#3703) |
| ② 독립 사후검증 | "됐다"의 기계 판정 | PR [#4186](https://github.com/edwardkim/rhwp/pull/4186) `rhwp verify --expect` 5축, `tests/verify_contract.rs` 5본 | **OPEN** (refs #4113) |
| ③ 봉투 출처 표지 | 값의 신뢰 경계 | `src/provenance.rs:74`(`MAP`) · `:507`(`marked`) · `src/main.rs:278`(`export-provenance-map`) · `tests/provenance_contract.rs` | devel 머지됨 (#3787) |
| ④ 결정론 렌더 | "같은 입력 = 같은 렌더" 실측 | PR [#4200](https://github.com/edwardkim/rhwp/pull/4200) render-diff 자기 pair 결정성(반복 측정 봉투 완전 동일·비결정 0건), `tests/edit_render_diff_gate.rs` 5계약 | **OPEN** (refs #4199) |

각 재료가 체인에 주는 것:

- **① run 저널**이 이미 조작 열을 데이터로 남긴다 — `steps[]`(건너뜀 사유 포함) ·
  `verify`(저장 직후 자기검증) · `changedPages` · `assertions`
  (`src/main.rs:14698-14704`). 매니페스트의 "조작 열" 필드는 이 저널을 **재사용**
  하면 되고, 새 기록 체계를 발명할 필요가 없다.
- **② verify**(#4186)는 "출력이 기대를 만족하는가"의 사후 판정을 준다. 체인은
  여기에 "그 출력이 **그 입력에서** 나왔는가"라는 직교 축을 더한다.
- **③ provenance 표지**(#3787)는 봉투 값의 신뢰 경계(엔진 산출 vs 문서 파생)를
  이미 판정한다. 체인 매니페스트는 같은 신뢰 구분 위에 서고, run 저널을 임베드할
  때 이 표지가 함께 따라온다.
- **④ 결정론 렌더**(#4200)는 "같은 바이트 = 같은 화면"의 실측 근거다. 콘텐츠
  해시가 의미를 갖는 전제 — 해시가 같으면 보이는 것도 같다 — 를 렌더 측에서
  받쳐 준다.

**공백 (실측)**: 이들을 묶는 **콘텐츠 해시**가 본체에 없다.

```
$ grep -c "sha256" src/main.rs
0
```

run 저널의 `input`/`output` 은 **경로 문자열**이다(`src/main.rs:14700`). 경로는
가리키는 파일이 바뀌면 같이 거짓말을 한다. 저널이 "무엇을 했는지"는 남기지만
"어떤 바이트에서 어떤 바이트로 갔는지"는 남기지 않는다 — 사후에 저널·산출물
어느 쪽이 변조되어도 검출할 방법이 없다. 이 공백을 메우는 것이 매니페스트다.

---

## §3 매니페스트 설계

### 3.1 원칙

1. **재사용** — 조작 열은 run 저널을 임베드한다. 저널이 없는 편집(단건 CLI)은
   조작 서술 문자열로 정직하게 격을 낮춰 기록한다(`operations.source` 로 구분).
2. **정직 절단** — 1차는 **무결성 체인(변조 검출)까지**다. 서명이 없으므로
   매니페스트와 실물을 **함께** 바꾸는 위조는 검출하지 못한다. 부인 방지는 P2
   (서명) 이후에만 주장할 수 있고, 그때도 키 관리는 범위 밖이다(§5).
3. **판정은 데이터** — 검증 결과는 불일치 목록으로 열거하고 종료 코드는 요약이다
   (0 전부 일치 / 1 불일치 / 2 조립 오류). 본체 CLI 의 계약(#2707 계열)과 같은
   문법을 따른다.

### 3.2 매니페스트 JSON

| 필드 | 내용 | 출처 |
|---|---|---|
| `manifestVersion` | 스키마 버전 (`"1.0"`) | 고정 |
| `createdAt` | 생성 시각 (UTC, ISO-8601) | 생성기 |
| `generator` | 매니페스트 생성 도구 | 고정 |
| `tool.versionString` | 편집 도구 버전. `RHWP_BIN --version` **실행 결과** — 실행 불가 시 `null` + 사유 | 실측 |
| `input.{path,bytes,sha256}` | 입력 파일의 경로·크기·해시 | 실측 |
| `output.{path,bytes,sha256}` | 출력 파일의 경로·크기·해시 | 실측 |
| `operations.source` | `"run-journal"`(1차 출처) 또는 `"description"`(서술 대체) | 인자 |
| `operations.steps` | run 저널의 `steps[]` 그대로, 또는 서술 목록 | 저널/인자 |
| `operations.journal.{path,sha256}` | 저널 파일 자체의 해시 (저널 변조 검출) | 실측 |
| `prev.{path,sha256}` | **이전 매니페스트 파일 바이트의 해시** — 체인 링크 | 실측 |

in-toto 와의 대응: `input`/`output` = subject, `operations`+`tool` = predicate,
서명(DSSE 상당)은 P2. C2PA 와의 대응: 매니페스트 = manifest, `sha256` = 해시
바인딩, `prev` = 이력 체인(C2PA 의 ingredient 관계에 상당), 서명 = P2.

실물 예시 (§6 실증에서 실제 생성된 첫 매니페스트, 경로만 축약):

```json
{
  "manifestVersion": "1.0",
  "createdAt": "2026-08-08T03:17:31Z",
  "generator": "tools/provenance_chain.py",
  "tool": { "versionString": "rhwp v0.8.2", "versionSource": "rhwp.exe --version" },
  "input":  { "path": "samples/2010-01-06.hwp", "bytes": 31232,
              "sha256": "d2562d9219fc1d49…" },
  "output": { "path": "output/provdemo/step1.hwp", "bytes": 18432,
              "sha256": "47c0f438ccc0b8fe…" },
  "operations": {
    "source": "run-journal",
    "steps": [ { "action": "replace_text", "find": "잠정", "replacedCount": 6, "step": 0 } ],
    "journal": { "path": "journal1.json", "sha256": "8673e4f7d47ac9c5…" }
  },
  "prev": null
}
```

### 3.3 체인 규칙

1. **링크** — 매니페스트 N 의 `prev.sha256` = 매니페스트 N−1 **파일 바이트**의
   sha256. 과거 매니페스트를 고치면 이후 모든 링크가 깨진다(해시 체인의 표준
   성질).
2. **연속성** — 매니페스트 N 의 `input.sha256` = 매니페스트 N−1 의
   `output.sha256`. "이전 산출물이 곧 이번 입력"이 데이터로 강제된다. 중간
   산출물 하나를 변조하면 N−1 의 출력 검증과 N 의 입력 검증 **두 곳**에서
   걸린다(§6 red 실측).
3. **검증 대상** — `verify` 는 매니페스트 1개의 기록(입력·출력·저널·prev)을
   실물과 재대조하고, `chain` 은 열 전체의 링크·연속성을 검증한다
   (`--files` 로 실물 대조 포함).

### 3.4 위협 모델 (1차가 막는 것 / 못 막는 것)

| 시나리오 | 1차(무결성 체인) | P2(서명) 이후 |
|---|---|---|
| 산출물 사후 변조 (매니페스트는 그대로) | **검출** (§6 red 1) | 검출 |
| 저널 사후 변조 | **검출** (`operations.journal.sha256`) | 검출 |
| 과거 매니페스트 조작 (이력 세탁) | **검출** (§6 red 2, prev 링크) | 검출 |
| 매니페스트+실물 **동시** 위조 | 검출 불가 — 정직하게 명시 | 서명 키 없이는 불가 |
| 발행자 신원 위장 | 범위 밖 | PKI 검증 (키 관리는 그때도 범위 밖) |

---

## §4 표면 제안 — P1~P4

| 단계 | 표면 | 착수 게이트 | DoD |
|---|---|---|---|
| **P1 (이번)** | `tools/provenance_chain.py` — `create`/`verify`/`chain`, 표준 라이브러리만 | — | 실물 문서 CLI 편집에 걸어 create→verify green, 1바이트 변조 red 실측 (§6 완료) |
| P2 | 본체 명령 승격(`rhwp provenance create/verify`) + `edit`/`run` 의 `--emit-provenance` 옵트인 + 외부 키 서명(알고리즘 명기, 키 관리 범위 밖) | P1 사용 피드백 + 메인테이너 승인 | 계약 테스트(green/red/조립 오류)·provenance MAP 등재·capabilities 등재 |
| P3 | 매니페스트를 HWPX 컨테이너에 동봉 | P2 + **호환 무해 위치 실측** (OCF zip 내 추가 파트를 타 소비자가 무해하게 무시하는지 — 조사 필요, 미확인) | 동봉 문서가 기존 뷰어에서 무변화 렌더됨을 실측 |
| P4 | 검증 API·MCP 도구(`hwp_provenance_verify`)·뷰어 표면 | P2 안정화 | MCP 왕복 계약 + 저지연 검증 |

P1 을 도구(tools/)에 두는 이유: 본체 표면(명령·capabilities·MCP)은 등재 비용이
크고 되돌리기 어렵다. 스키마가 사용으로 검증되기 전에는 본체를 건드리지 않는다 —
스킬군 승격 관례(tools 검증 → 본체 편입)와 같은 순서다.

---

## §5 하지 않는 것

- **법적 효력 주장 금지** — 이 체인은 전자서명법·각국 증거법상 효력을 주장하지
  않는다. "무결성 검증 데이터"이지 "법적 증명"이 아니다.
- **규제 준수 보장 주장 금지** — §1.2 의 표시 의무를 "이 도구로 준수된다"고
  주장하지 않는다. 규제 대응의 **재료**가 될 수 있다는 것까지가 이 문서의 주장이다.
- **키/PKI 운영 없음** — P2 서명 단계에서도 키 생성·보관·배포·폐기는 사용자
  몫이다. rhwp 는 검증 알고리즘만 구현한다.
- **포맷 개조 없음** — HWP/HWPX 바이트 구조를 바꾸지 않는다. P3 동봉도 기존
  소비자가 무시하는 위치가 실측으로 확인될 때만 진행한다.
- **자동 발행 없음** — 매니페스트 생성은 옵트인이다. 기본 동작(편집 명령의
  산출물)은 변하지 않는다.

---

## §6 P1 실증 (2026-08-08 실측)

환경: Windows 11 · Python 3.11 · rhwp **v0.8.2** release 바이너리
(`target/release/rhwp.exe`). 정직 기록: 이 바이너리는 devel 헤드가 아니라 기존
빌드 산물이다(이번 작업은 Rust 컴파일 없이 진행). HWPX 샘플
(`tac-host-spacing.hwpx`)은 이 구버전에서 직렬화 실패(`styleIDRef` 미등록 오류)
하여 **HWP5 샘플로 실증**했다 — 편집은 파일 복사 대체가 아니라 **실제 rhwp CLI
편집**이다.

재현: `python output/provdemo/driver.py` (실증 산출물은 무시 경로 `output/` 에
있으며 커밋하지 않는다). 도구는 [provenance_chain.py](../../../tools/provenance_chain.py).

**1) 편집 1 + 매니페스트 + 검증 (green)**

```
$ rhwp run plan1.json --json          # replace_text 잠정→시험, assertions.verify=true
exit=0  steps=1 replacedCount=6  verify={diffCount:0, identical:true}
        입력  samples/2010-01-06.hwp        31,232 B  sha256 d2562d9219fc1d49…
        출력  output/provdemo/step1.hwp     18,432 B  sha256 47c0f438ccc0b8fe…
$ provenance_chain.py create --input … --output … --journal journal1.json --manifest m1.json
매니페스트 생성: m1.json (sha256 43178438a932d8f2…)
$ provenance_chain.py verify m1.json
  일치: input …/2010-01-06.hwp (d2562d9219fc1d49…)
  일치: output …/step1.hwp (47c0f438ccc0b8fe…)
  일치: journal journal1.json (8673e4f7d47ac9c5…)
검증 통과: 기록된 해시 전부 실물과 일치        → exit 0
```

**2) 편집 2 + 체인 (green)** — 두 번째 `rhwp run`(경상수입→경상수입A)으로
step2.hwp(sha256 `e7f2d136…`)를 만들고 `--prev m1.json` 으로 m2 를 생성:

```
$ provenance_chain.py chain m1.json m2.json --files
[1] m2.json
  연결: prev 해시 일치 (43178438a932d8f2…)
  연속: 이전 출력 = 이번 입력 (47c0f438ccc0b8fe…)
체인 검증 통과: 매니페스트 2개 연결·해시 전부 일치   → exit 0
```

**3) 산출물 1바이트 변조 (red)** — step1.hwp 오프셋 9,216 의 1바이트를 XOR:

```
$ provenance_chain.py verify m1.json
실패 1건: output: sha256 불일치
  기록 47c0f438ccc0b8fe456b3e25a30e49ee005a8dfc616a9f3f2d20c1dcd8161060
  실물 52e6229999154fe87664816f4f29ab81196bd210e04fee881479e44b09d7b0bf
→ exit 1
$ provenance_chain.py chain m1.json m2.json --files
체인 검증 실패 2건: m1 의 output + m2 의 input     → exit 1
```

같은 1바이트가 §3.3-2 연속성 규칙 때문에 **두 곳에서** 검출된다. 복원 후 재검증
exit 0.

**4) 매니페스트 변조 (red)** — m1.json 을 1문자 수정(이력 세탁 시도):

```
$ provenance_chain.py chain m1.json m2.json
실패 1건: [1] prev 해시 불일치 - 이전 매니페스트(m1.json)가 변조되었거나 다른 파일입니다
  기록 43178438a932d8f2…  실물 22724ebc3c7601c3…
→ exit 1
```

복원 후 exit 0.

**요약**: green 4/4 (run×2·create×2·verify·chain), red 2/2 (산출물 변조·매니페스트
변조 모두 exit 1 + 불일치 데이터 열거), 복원 후 전부 green 복귀.

---

## §7 제안 성격과 다음 판단

이 문서와 P1 도구는 **축의 존재 증명**이다: 재료 4종(§2) 위에 얇은 해시 체인
하나를 얹으면 "검증 가능한 문서 이력"이 실물로 성립함을 실측(§6)으로 보였다.
P2 이후의 본체 편입 여부·표면 형태·우선순위는 메인테이너 판단에 따르며,
①(#3703)은 머지됨·②(#4186)·④(#4200)는 이 문서 작성 시점에 OPEN 이므로 이 축의
착수 게이트도 그 판정들을 따라간다.

refs [#3907](https://github.com/edwardkim/rhwp/issues/3907)
· [#4229](https://github.com/edwardkim/rhwp/issues/4229)
