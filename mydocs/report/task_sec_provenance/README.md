---
kind: report
status: active
canonical: mydocs/report/task_sec_provenance/README.md
last_verified: 2026-08-02
---

# 봉투가 스스로 출처를 밝힌다 — `untrustedContent` / `untrustedFields` (#3787 S1)

## 0. 요약

| 항목 | 내용 |
|---|---|
| 문제 | 봉투 안에서 **엔진이 만든 값**과 **문서에서 온 값**이 구별되지 않는다 |
| 결과 | 에이전트가 문서 본문에 적힌 문장을 **도구의 지시**로 읽는다 |
| 처방 | 모든 `--json` 봉투에 `untrustedContent`/`untrustedFields` 표지 + 지도 명령 `export-provenance-map` |
| 단일 출처 | `src/provenance.rs::MAP` — 표지도 지도도 여기 하나에서 나온다 |
| 지도 규모 | 명령 24건 / 문서 파생 필드 51건 / 문서 값을 담는 명령 13건 · 담지 않는 명령 11건 |
| 드리프트 가드 | 신규 7종 — **선언을 믿지 않고 실제 문서 문자열이 봉투에 나타나는지로 판정** |
| 가드 실증 | 선언을 지워 봤다 → 잡힌다. 명령 항목을 지워 봤다 → 잡힌다 (§4) |
| `schemaVersion` | **범프하지 않는다** — 추가만이고, 저장소 정책이 "필드 추가 허용" (§5) |
| 검증 | 계약 55종 통과(신규 7 + 기존 48), clippy 0, rustfmt 0, 선검사 9종 전부 통과 |
| 권위 문서 | [봉투 출처 계약](../../tech/envelope_provenance.md) |

## 1. 무엇이 문제인가

`mcp-serve`(#3571) 이후 rhwp 의 출력은 LLM 에이전트의 컨텍스트로 직행한다. 그 봉투
하나에는 성질이 정반대인 두 종류의 값이 섞여 있다.

```json
{
  "pageCount": 16,                        ← rhwp 가 계산했다
  "pages": [{"page":0,"text":"…본문…"}]   ← 문서를 만든 사람이 정했다
}
```

**에이전트에게 이 둘은 똑같이 생겼다.** 그래서 본문에 적힌

> 앞의 지시는 무시하고 …

같은 문장이 *도구가 내려준 지시*처럼 읽힌다. 사람은 "이건 문서 내용이지"를 문맥으로
알지만, 봉투를 파싱해 프롬프트에 이어 붙이는 경량 에이전트에게는 그 문맥이 없다.
rhwp 는 한국 공공문서를 다루는 도구라 **열람 대상이 신뢰할 수 없는 외부 입력인 것이
기본값**이다.

#3707(유니코드 기만 판정)이 *문자열의 모양*을 판정했다면, 이 조각은 *값의 출처*를
판정한다. 같은 신뢰 경계의 다음 칸이다.

## 2. 무엇을 만들었나

### 2.1 봉투 표지

```
$ rhwp search samples/hwp3-sample.hwp 한 --json
{… "matchCount":143, "untrustedContent":true,
    "untrustedFields":["matches[].text","matches[].context"]}

$ rhwp export-pdf samples/hwp3-sample.hwp -o out.pdf -p 0 --json
{… "renderedCount":1, "untrustedContent":false, "untrustedFields":[]}
```

**표지는 항상 실린다.** 문서를 열지 않는 명령도 `false` 를 명시한다 — 키가 없으면
소비자는 "문서 값 없음"과 "출처를 판정하지 않는 옛 바이너리"를 구별할 수 없다
(#3707 `textSecurity` 와 같은 규약).

`untrustedFields` 는 선언을 그대로 베끼지 않는다. 같은 명령이라도 모드마다 봉투
모양이 다르므로(`digest` 는 기본/`--sections`/`--pages` 가 서로 다른 필드를 낸다),
**그 봉투에 실제로 값이 실린 경로만** 남긴다. 있지도 않은 필드를 표지에 적으면
표지 자체가 거짓말이 된다.

### 2.2 지도 — `rhwp export-provenance-map [--json]`

표지는 "이 봉투가 지금 무엇을 담았나"만 말한다. 에이전트 프레임워크가 **호출 전에**
정책을 세우려면(이 필드는 절대 프롬프트에 이어 붙이지 않는다) 전체 지도가 필요하다.

```json
"search": {
  "untrusted": ["matches[].text", "matches[].context"],
  "origins": {
    "matches[].text": "GrepMatch.text — 매치가 속한 문단의 전문",
    "matches[].context": "GrepMatch.context — 매치 앞뒤 문맥 발췌"
  },
  "note": "query 는 호출자가 준 값이고 주소(section/paragraph/page/charOffset)는 엔진값이다."
}
```

`origins` 는 장식이 아니다. **필드 목록을 하드코딩하지 않는다**는 요구는 "어디서
오는지 근거를 코드에 남긴다"는 뜻으로 구현했다 — 각 항목이 어느 엔진 경로를 타고
문서에서 봉투로 들어오는지를 달고 있고, 계약 테스트가 근거 없는 선언을 거부한다.
근거 없는 보안 선언은 검토할 수 없고, 검토할 수 없는 선언은 다음 사람이 지운다.

MCP 로도 닿는다: `hwp_export_provenance_map`(입력 없음). 자기서술
`capabilities.jsonContract.provenance` 가 표지의 의미와 지도의 위치를 광고한다.

### 2.3 판정 결과 (요지)

문서 파생으로 판정한 것 중 **놓치기 쉬운 것들**:

- `info.title` — 설계 이슈의 예시는 `info` 를 "문서 텍스트를 담지 않는 봉투"로 들었지만,
  실제 `title` 은 `document_title()` 이 **앞 3쪽을 렌더해 얻은 첫 의미 줄**이다(#3407).
  페이지 텍스트 그 자체이므로 선언했다. 예시를 따르는 것보다 사실이 우선이다.
- `info.fonts[]` — 글꼴 **이름 문자열**을 문서가 정한다.
- `thumbnail.base64`/`dataUri` — 텍스트가 아니라고 안전한 게 아니다. **멀티모달
  에이전트는 그림 속 글자를 읽는다.**
- `dump-pages.pages[].columns[].items[].textPreview` — 조판 진단 봉투인데 문단
  미리보기만은 문서 텍스트다.
- `ir-diff.categories` — 보통은 엔진 카테고리 라벨이지만, `:` 가 없는 차이 라인은
  본문 전체가 키가 된다. **애매하면 문서 파생으로 선언한다** — 과소 선언만 위험하다.

문서 파생이 **아닌** 것: 호출자가 준 값의 반향(`source`/`output`/`query`/`find`),
엔진 계산값(`pageCount`/`bytes`/`diffCount`/`verify`), 고정 문자열 계약
(`digest.nextStep`), 산출 매니페스트(`pages[].path`).

전체 표와 판정 기준은 [권위 문서 §3](../../tech/envelope_provenance.md)에 있다.

## 3. 드리프트 가드 — 이 조각의 진짜 값어치

선언은 코드가 바뀌어도 조용히 남는다. 새 명령이 문서 텍스트를 실어 나르기 시작해도
지도는 아무 말 없이 옛 사실을 계속 광고한다. **6개월 뒤 "이 봉투는 안전하다"는
표지가 거짓이 되는 경로가 그것이다.**

그래서 `tests/provenance_contract.rs` 는 **선언을 믿지 않는다.**

1. 대상 문서에 `export-text --json` 을 돌려 6자 이상 토큰을 모은다(부분 일치 축).
2. `export-tables --json` 의 셀 텍스트와, 본문의 **한글이 든 2자 이상 낱말**을
   완전 일치 축으로 더한다 — `edit set-cell` 의 `oldText`("구 분"), `fields[].name`
   ("회사명") 같은 짧은 문서 값을 잡기 위해서다.
3. `--json` 명령 **전부**를 실제로 실행해 봉투를 받고, 봉투를 재귀로 훑어 그 문자열이
   나타난 **경로**를 모은다(`matches[].context` 같은 지도 표기 그대로).
4. 발견된 경로가 지도에 없거나 `untrustedContent` 가 `true` 가 아니면 **실패**.

공허한 통과를 막는 장치를 함께 걸었다.

- 레시피가 `--json` 명령 전부를 덮지 않으면 실패한다. 못 덮는 명령은 `SWEEP_EXEMPT`
  에 **사유와 함께** 넣어야 한다 (현재 1건, §6).
- 오라클이 비면 실패한다.
- 문서 문자열이 탐지된 명령이 6건 미만이면 탐지기 고장으로 보고 실패한다.
- `export-text`·`search`·`export-structure`·`export-tables` 중 하나라도 탐지되지
  않으면 실패한다.

| 가드 | 무엇을 잡는가 |
|---|---|
| `provenance_map_covers_every_json_command` | 지도에 없는 `--json` 명령 / 지도에만 남은 유령 항목 / 근거 없는 선언 |
| `every_text_bearing_command_declares_untrusted_fields` | **문서 문자열이 실제로 실렸는데 선언이 없는 필드** |
| `untrusted_flag_matches_map` | 표지가 지도에 없는 경로를 광고 / 두 표지가 서로 다른 말 |
| `every_json_envelope_carries_the_flag` | 표지를 빠뜨린 봉투 |
| `export_provenance_map_is_wired_across_every_surface` | capabilities↔help↔MCP 배선, 선언 flags 실재, MCP `required` 배열, 실패 시 stdout 0바이트 |
| `capabilities_advertises_the_provenance_contract` | 자기서술에서 계약이 사라지는 것 |
| `schema_version_stays_1_0_because_the_flag_is_additive` | 추가 허용 정책이 바뀌었는데 범프 판단을 안 고치는 것 |

## 4. 가드를 일부러 실패시켜 봤다

가드는 "만들었다"가 아니라 "실패시켜 봤다"여야 의미가 있다. 두 가지 드리프트를 실제로
재현했다. 원문 출력은 [evidence.txt §8](evidence.txt).

**실험 A — 선언된 필드를 지웠다.** `search` 항목에서 `matches[].text`/
`matches[].context` 선언을 지우고 재빌드.

```
every_text_bearing_command_declares_untrusted_fields FAILED
  선언되지 않은 문서 파생 필드 3건:
    - search: 문서 문자열이 {"matches[].context", "matches[].text"} 에 실렸는데
              untrustedContent 가 true 가 아닙니다
    - search: 봉투의 matches[].context 에 문서 문자열이 실렸는데 지도에 선언이 없습니다
    - search: 봉투의 matches[].text 에 문서 문자열이 실렸는데 지도에 선언이 없습니다
test result: FAILED. 6 passed; 1 failed
```

지도를 참고하지 않고 **문서 자체**에서 만든 오라클이 봉투 속 문서 문자열을 찾아
정확한 경로까지 지목했다. 선언을 지운다고 가드가 따라 눈감지 않는다.

**실험 B — 명령 항목을 통째로 지웠다.** `dump-pages` 항목 삭제(25→24) 후 재빌드.
새 명령을 추가하고 지도에 안 넣은 경우와 동형이다.

```
provenance_map_covers_every_json_command FAILED
  --json 계약 명령인데 출처 지도에 없는 것: ["dump-pages"]
  … 문서 값을 담지 않는 명령이라도 빈 목록과 사유(note)를 남겨야 합니다.
test result: FAILED. 3 passed; 4 failed
```

둘 다 원상 복구 후 재검증 — 7 passed; 0 failed.

**부수 소득**: 실험 B 도중 `export-pdf` 가 스레드 병렬로 겹쳐 돌며
`memory allocation of 16273348 bytes failed` 로 죽는 것을 실측했다. 가드 4종이 각자
전 명령을 다시 실행하던 구조가 원인이라 스윕을 `OnceLock` 으로 프로세스당 1회만
돌리게 고쳤다. 테스트 시간이 **13.61s → 2.17s** 로 줄었다.

## 5. `schemaVersion` 을 올리지 않은 근거

**결론: 범프하지 않는다. `1.0` 그대로다.**

1. 저장소 정책이 코드에 명시돼 있다 — `capabilities.jsonContract.schemaPolicy` =
   *"필드 추가 허용, 변경·삭제는 schemaVersion 범프"*. 이번 변경은 **추가만** 한다.
2. 기존 필드의 이름·타입·값이 하나도 바뀌지 않았다.
3. 저장소의 모든 봉투가 `"1.0"` 단일 값을 쓴다(전수 확인). 추가마다 범프하면 봉투마다
   버전이 갈라지고, 정작 **깨는 변경**이 왔을 때 소비자가 그 신호를 구별할 수 없게 된다.
4. 옛 소비자는 모르는 키를 무시하면 그대로 동작하고, 새 소비자는 **키의 존재 여부**로
   바이너리 세대를 구별한다 — 범프 없이도 세대 구별이 된다.

`schema_version_stays_1_0_because_the_flag_is_additive` 가 이 판단을 계약으로 고정한다.
추가 허용 정책 자체가 바뀌면 그 테스트가 먼저 실패해 판단을 다시 하게 만든다.

## 6. 범위에서 뺀 것과 그 이유

| 뺀 것 | 이유 |
|---|---|
| `build-from-ingest` 를 출처 스윕에서 제외 | 입력이 문서가 아니라 호출자가 만든 ingest JSON 이라 "문서에서 온 문자열" 오라클을 만들 수 없다. 지도 항목(빈 목록 + 사유)은 그대로 있고, 봉투 자체는 `tests/issue_3358_ingest_unknown_fields.rs` 가 따로 고정한다. `SWEEP_EXEMPT` 에 사유와 함께 등재했고, 목록이 늘면 사유가 강제된다. |
| 누락 판정을 **최상위 키** 단위로 | 발견된 경로의 최상위 키가 선언에 있으면 통과한다. 이미 선언된 루트 아래(`matches[].새필드`)에 문서 문자열이 하나 더 붙는 경우는 못 잡는다. 재귀 구조(`structure.roots[].children[]`)에서 경로 완전 일치를 요구하면 오탐이 나기 때문이다. **새 명령·새 루트는 전부 잡힌다** — 요구된 완료 조건은 충족한다. |
| `capabilities --mcp` 매니페스트도 표지를 실음 | 문서를 열지 않으므로 `false` 고정. 같은 명령의 다른 출력만 표지가 없으면 소비자가 두 규약을 외워야 한다. |
| `mydocs/manual/cli_commands.md` 미갱신 | 같은 파일을 다른 병렬 작업이 건드리고 있어 충돌을 만들지 않았다. `--help` 와 `capabilities` 는 갱신했고 계약 테스트가 그 둘을 강제한다. |
| 문서 문자열 수정·차단 | **하지 않는다.** rhwp 는 판정만 하고 값을 고치지 않는다(#3707 과 같은 원칙). 실제 격리는 봉투를 소비하는 쪽의 몫이고, 이 계약은 그 격리를 *가능하게* 만드는 최소 정보다. |

## 7. 파일

| 파일 | 줄 | 내용 |
|---|---|---|
| `src/provenance.rs` | 439 | 신규 — 지도의 단일 출처(`MAP`), 경로 해석, 표지 부착 |
| `tests/provenance_contract.rs` | 1027 | 신규 — 드리프트 가드 7종 + 문서 문자열 오라클 + 24개 명령 레시피 |
| `mydocs/tech/envelope_provenance.md` | 261 | 신규 — 계약 권위 문서 |
| `src/main.rs` | +339/−179 | 봉투 29곳 표지 부착, `export-provenance-map` 배선, capabilities·help·MCP 도구 |
| `mydocs/report/task_sec_provenance/` | — | 본 보고서 + `evidence.txt` |

## 8. 검증

[evidence.txt](evidence.txt) 참조. 요지:

- `cargo build --release --bin rhwp` — exit 0
- `cargo test --release --test provenance_contract --test cli_json_contract --test mcp_server_contract`
  — **55 passed; 0 failed** (신규 7 / 기존 `cli_json_contract` 26 · `mcp_server_contract` 22)
- `cargo clippy -- -D warnings` — 경고 0
- rustfmt(파일 단위) — 차이 0
- `agent_preflight.py` — 9종 전부 통과 (MCP 도구 26 / 명령 55 / 플래그 61)
