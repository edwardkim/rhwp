# `rhwp inspect unicode` — 문서 텍스트의 유니코드 기만 탐지 (#3787 S4)

## 왜 이것이 문서 엔진의 일인가

rhwp 의 `--json` 봉투와 MCP 도구 결과는 LLM 에이전트가 **검증된 도구 출력**으로 읽는다.
그런데 그 안에 담기는 본문 텍스트는 전부 공격자가 내용을 정할 수 있는 문서에서 온다 —
민원인이 올린 서식, 웹에서 받은 공고문, 메일에 붙어 온 계약서.

사람은 그 문서를 **화면으로** 보고 안전하다고 판단한다. 에이전트는 **바이트로** 읽는다.
둘이 어긋나면 사람의 승인은 자기가 본 적 없는 내용에 대한 승인이 된다.

```
화면(사람):   첨부 exe.doc 를 확인하세요
바이트(LLM):  첨부 <U+202E>cod.exe<U+202C> 를 확인하세요
```

같은 문단이다. 사람은 문서 파일을 승인했고 에이전트는 실행 파일을 읽었다.
`rhwp inspect unicode` 는 이 어긋남을 찾아 **양쪽을 나란히** 내놓는다.

## 명령

```console
$ rhwp inspect unicode <파일.hwp|파일.hwpx> [--json] [--kind zero-width|bidi|tag|confusable|all]
```

읽기 전용이다. 출력 경로가 없고, IR 을 고치지 않으며, 스캔 전후 파일 바이트가 같다
(`scanning_does_not_touch_the_document` 가 해시로 고정).

### 봉투

```json
{"schemaVersion":"1.0","source":"공고문.hwp","kindFilter":"all","scannedChars":2943,
 "findings":[
   {"kind":"bidi_override","codepoint":"U+202E","severity":"high",
    "section":0,"paragraph":3,"location":"body","charOffset":12,"runLength":1,
    "excerpt":"…제출 <U+202E>cod.exe<U+202C> : 대회 홈페이지…",
    "rendered":"제출 exe.doc : 대회 홈페이지",
    "raw":"제출 <U+202E>cod.exe<U+202C> : 대회 홈페이지",
    "why":"표시 순서를 뒤집는 제어문자입니다 — 화면에 보이는 순서와 실제 문자 순서가 다릅니다"}],
 "findingCount":1,"clean":false,
 "severityCounts":{"high":1,"medium":0,"low":0},
 "kindCounts":{"zero_width":0,"bidi_override":1,"tag_char":0,"confusable":0}}
```

0건이면 `findings: []` · `clean: true` 다. 키를 빼지 않는다 — 소비자가 **"검사했는데
깨끗함"과 "검사하지 않음"** 을 구별할 수 있어야 한다.

`codepoint` 는 숫자가 아니라 `U+202E` 문자열이다. 숫자로 주면 사람이 못 읽고, 사람이
못 읽는 보안 경보는 없는 것과 같다.

### `rendered` vs `raw` — 이 명령의 전부

| 필드 | 뜻 | 만드는 방법 |
|---|---|---|
| `rendered` | **화면에 보이는 모습** | 보이지 않는 문자를 지우고, 방향 제어를 실제로 적용해 재배열 |
| `raw` | **실제 순서** | 논리 순서 그대로. 제어문자는 `<U+XXXX>` 표기로 드러냄 |
| `excerpt` | 넓은 문맥(±40자) | 논리 순서 + 표기. 어디쯤인지 사람이 찾아가는 용도 |

`rendered` 와 `raw` 는 **같은 창(±24자)** 에서 만든다. 창이 다르면 나란히 놓고 비교할 수 없다.

봉투에는 원문 제어문자를 그대로 싣지 않는다. 탐지 결과를 날것으로 출력하면 **그 출력을 읽는
터미널과 에이전트가 같은 속임수에 다시 걸린다** — 보안 보고가 스스로 공격 매개가 되는 형태다.
그래서 보이지 않는 문자·방향 제어·태그 문자·C0 제어는 전부 `<U+XXXX>` 로 바꿔 내보낸다
(`bidi_finding_shows_rendered_and_raw_disagreeing` 이 원문 U+202E 유출을 금지로 고정).

#### rendered 계산: 묶음 단위 재배열

UAX #9 전체(문자 고유 방향성·중립 문자 해소)를 구현하지 않는다. 목적이 조판이 아니라
**"제어문자가 순서를 뒤집었는가"를 눈에 보이게 하는 것**이고, 그 판정에는 명시적 방향
제어만 있으면 충분하다. 아랍어·히브리어 본문의 고유 방향성은 다루지 않는다.

레벨 스택을 들고 한 번 훑는다.

| 입력 | 동작 |
|---|---|
| RLE `U+202B` · RLO `U+202E` · RLI `U+2067` | RTL 프레임 push |
| LRE `U+202A` · LRO `U+202D` · LRI `U+2066` · FSI `U+2068` | LTR 프레임 push |
| PDF `U+202C` · PDI `U+2069` | 프레임 pop → 접어서 부모에 **한 묶음으로** 붙임 |
| 보이지 않는 문자·태그 문자·C0 | 버림(화면에 없다) |
| 그 외 | 현재 프레임에 1글자 묶음으로 push |

RTL 프레임을 접을 때 **글자가 아니라 묶음의 순서**를 뒤집는다. 글자 단위로 뒤집으면
중첩 레벨이 깨진다:

```
논리:      RLO a LRO b c PDF d PDF
글자 단위: dcba   ← 틀림 (안쪽 LTR 구간까지 거꾸로)
묶음 단위: dbca   ← 맞음
```

`nested_bidi_levels_reorder_by_run_not_by_char` 가 이 구분을 고정한다.
창 안에서 닫히지 않은 레벨(창 밖에서 열린 경우)도 끝에서 접는다 — 안 접으면 그 구간이
보고에서 통째로 사라져 "아무 일 없음"처럼 보인다.

---

## 코드포인트 판정표

각 축마다 **정상 용도**와 **악용 방식**을 나눠 적는다. 정상 용도가 실재하는 코드포인트를
무조건 위협으로 올리면 경보는 오탐 한 건이면 통째로 무시된다.

### ① 제로폭 — `zero_width`

| 코드포인트 | 이름 | 정상 용도 | 악용 방식 | 판정 |
|---|---|---|---|---|
| `U+200B` | ZERO WIDTH SPACE | 긴 URL·옛한글 낱자 사이 줄바꿈 지점 지정 | 탐지 규칙 우회(`무시`→`무<200B>시`), 문자 단위 은닉 인코딩 | 조건부 |
| `U+200C` | ZWNJ | 아랍·인도계 문자 결합 억제 | 〃 | 조건부 |
| `U+200D` | ZWJ | 이모지 결합(👨‍👩‍👧), 인도계 결합 | 〃 | 조건부 |
| `U+2060` | WORD JOINER | 줄바꿈 금지(BOM 대체 권장형) | 〃 | 조건부 |
| `U+FEFF` | ZWNBSP / BOM | **맨 앞이면 바이트 순서 표식** | 본문 중간 삽입 시 은닉 채널 | 맨 앞 정상, 중간 조건부 |

정상 용도가 실재하므로 등급을 나눈다.

| 조건 | 등급 | 근거 |
|---|---|---|
| 3자 이상 연속 | `high` | 은닉 **데이터**의 형태다. 조판 보조는 이렇게 뭉치지 않는다 |
| ±48자 안에 지시문 표지 | `high` | 은닉 **지시**의 형태다 (`무시하`·`프롬프트`·`ignore`·`system prompt` 등) |
| 2자 연속 | `medium` | |
| 낱개 1자 | `low` | 조판 보조일 가능성이 실재한다 |
| 맨 앞 `U+FEFF` | 보고 안 함 | BOM 이다 |
| **앞뒤 어느 한쪽이 PUA** | 보고 안 함 | 한/글 옛한글 조판 — 아래 참조 |

연속된 열은 **1건으로 묶어** 보고하고 `runLength` 를 함께 준다. 글자마다 1건이면
50자 은닉 문자열이 findings 50건이 되어 봉투가 노이즈로 덮인다.

#### 좁힘의 근거 — 한/글 옛한글(PUA) 조판

`samples/*.hwp|hwpx` 348개 전수 스윕에서 **유일하게** 걸린 파일이
`samples/exam_kor.hwp`(국어 시험지, 중세 국어)였고, 24건 **전부** 같은 형태였다.

```
raw: …'쥭<U+F152><U+200B>'(중의)처럼 이어 적기도 하고, '<U+F1C4><U+200B><U+200B>으란'(중은)처럼…
        ^^^^^^^^ 옛한글 낱자(PUA)   ^^^^^^^^^^ 그 뒤에 붙은 제로폭
```

한/글은 유니코드에 자리가 없는 옛한글 낱자와 조판부호를 PUA 코드포인트로 싣고
(BMP `U+E000~F8FF`, 15면 보충 `U+F0000~`), 그 낱자 조합을 끊어 줄바꿈·자간을 잡는 데
`U+200B` 를 쓴다. **은닉 채널이 아니라 조판의 부산물**이다.

그래서 제로폭 축에서만, 앞뒤 어느 한쪽이 PUA 글자면 보고하지 않는다.
방향 제어·태그 문자에는 이 완화를 주지 않는다 — 그쪽은 PUA 곁이라 해도 정당한 용도가 없다
(`zero_width_next_to_hangul_pua_is_typesetting_not_deception` 이 양쪽을 함께 고정).

남는 위험: 공격자가 옛한글 문서 안에서 PUA 글자에 잇대어 제로폭을 심으면 이 축을 피한다.
다만 그러려면 눈에 보이는 옛한글 글자를 본문에 함께 심어야 하므로 은닉성이 크게 떨어진다.
경보 전체를 살리는 대가로 이 잔여 위험을 받아들였다.

### ② 방향 오버라이드 — `bidi_override`

Trojan Source(CVE-2021-42574) 계열. 화면 순서와 논리 순서를 어긋나게 만든다.

| 코드포인트 | 이름 | 정상 용도 | 악용 방식 | 등급 |
|---|---|---|---|---|
| `U+202D` | LRO | 사실상 없음(격리 문자로 대체됨) | 순서 강제 뒤집기 | `high` |
| `U+202E` | RLO | 〃 | `cod.exe` → 화면엔 `exe.doc` | `high` |
| `U+202A` | LRE | 히브리·아랍 혼용 문단의 구식 임베딩 | 구간 방향 조작 | `medium` |
| `U+202B` | RLE | 〃 | 〃 | `medium` |
| `U+2066` | LRI | RTL 문맥에 LTR 조각 삽입(권장형) | 〃 | `medium` |
| `U+2067` | RLI | 그 반대 | 〃 | `medium` |
| `U+2068` | FSI | 방향을 모르는 조각 삽입 | 〃 | `medium` |
| `U+202C` | PDF | 위 임베딩 닫기 | — | `low` |
| `U+2069` | PDI | 위 격리 닫기 | — | `low` |

닫는 문자를 `low` 로 낮추는 이유: 여는 쪽이 이미 보고됐다. 같은 사건을 두 번 `high` 로
올리면 건수가 부풀어 우선순위가 흐려진다. 그래도 보고는 한다 — 짝이 없는 닫기 문자는
**그 뒤 문단 전체**의 렌더링을 바꾸므로 사람이 알아야 한다.

한국어 문서에 방향 제어가 정당하게 필요한 경우는 사실상 없다. 아랍어·히브리어를 인용하는
문서라면 `medium` 이 뜨는데, 그 판단은 사람 몫으로 남긴다(자동 억제하지 않는다).

### ③ 태그 문자 — `tag_char`

`U+E0000~U+E007F`. **렌더링되지 않는데 텍스트에는 남는다.** 원래 언어 태그용으로 도입됐다
폐기됐고, 현재 유일한 정상 용도는 국기 이모지의 하위 지역 표기다.

| 범위 | 이름 | 정상 용도 | 악용 방식 | 등급 |
|---|---|---|---|---|
| `U+E0001` | LANGUAGE TAG | 폐기됨 | — | `high` |
| `U+E0020~E007E` | TAG SPACE~TILDE | 국기 이모지 시퀀스의 지역 코드 | **ASCII 전문을 통째로 은닉** | `high` |
| `U+E007F` | CANCEL TAG | 국기 이모지 시퀀스 종료 | 〃 | `high` |

`U+E0020~E007E` 는 ASCII `0x20~0x7E` 와 1:1 대응이라(`cp - 0xE0000`), 임의의 영문
지시문을 화면에 흔적 없이 실을 수 있다. 그래서 이 축은 **복원해서 보여준다**:

```json
{"kind":"tag_char","codepoint":"U+E0049","runLength":6,"severity":"high",
 "rendered":"보고서","hidden":"Ignore"}
```

`rendered` 에는 흔적이 없고 `hidden` 에 숨은 지시가 있다. "무엇이 숨었는지"까지 보여야
사람이 판단할 수 있다.

**정상 용도 예외 1건**: 지역 국기 이모지(🏴󠁧󠁢󠁳󠁣󠁴󠁿 = `U+1F3F4` + 태그열 + `U+E007F`)는
잡지 않는다. 앞이 `U+1F3F4` 이고 열이 `U+E007F` 로 끝나는 형태만 예외로 인정한다
(`emoji_tag_sequence_is_not_a_finding`).

### ④ 혼동 문자 — `confusable`

같은 글리프로 렌더되는 다른 코드포인트. 기존 `confusable_collisions`(누름틀 **이름** 축)와
같은 동형자 표를 공유하고, 이 축은 **본문 낱말**을 본다.

| 예 | 코드포인트 | 화면 | 정상 용도 | 악용 방식 |
|---|---|---|---|---|
| 키릴 `а` | `U+0430` | 라틴 `a` 와 동일 | 러시아어 본문 | 라틴 낱말 위장 |
| 키릴 `Т` | `U+0422` | 라틴 `T` 와 동일 | 〃 | `Тotal` ≠ `Total` |
| 그리스 `ο` | `U+03BF` | 라틴 `o` 와 동일 | 그리스어·수식 | 〃 |

전체 표는 `src/document_core/text_security.rs::confusable_to_latin` — 실제 스푸핑에 쓰이는
고빈도 동형자만 담는다(UTS #39 전체 표는 수만 항목이고 WASM 산출물 크기에 그대로 얹힌다).

**라틴 낱말로 위장한 경우만** 잡는다. 판정 조건 2개를 모두 만족해야 한다.

1. 낱말 안에 라틴 글자가 **2자 이상**
2. 그 낱말에 라틴 동형자를 가진 **비라틴 글자**가 1자 이상

이 두 조건이 한국어·학술 문서의 오탐 대부분을 막는다.

| 입력 | 판정 | 이유 |
|---|---|---|
| `Тotal` (키릴 Т) | 탐지 | 라틴 4자 + 동형자 키릴 1자 |
| `Москва` | 통과 | 라틴 0자 — 정당한 러시아어 |
| `αβγ` | 통과 | 라틴 0자 — 그리스 수식 기호 |
| `Δt` | 통과 | 라틴 1자, `Δ` 는 동형자 표에 없음 |
| `총액 α 값` | 통과 | 라틴 0자 |
| `E-mail` · `URL` · `pH` | 통과 | 순수 라틴 |

낱말 경계는 라틴·키릴·그리스 글자의 연속으로 잡는다. **보이지 않는 문자는 낱말을 끊지
않는다** — `Т<U+200B>otal` 처럼 제로폭으로 낱말을 갈라 판정을 피하는 우회를 막기 위해서다.

`rendered`/`raw` 의 뜻이 이 축에서만 다르다. 어긋나는 것이 순서가 아니라 **정체**이므로,
`rendered` 는 라틴으로 접었을 때의 모습(`Total`), `raw` 는 실제 글자와 코드포인트
(`Т<U+0422>otal`)다.

등급은 `medium` 고정이다. 이 축 단독으로는 "화면상 구별되지 않는다"까지만 말할 수 있고,
같은 문서 안에 실제 쌍둥이가 있는지는 이름 축의 `confusable_collisions` 가 판정한다.

---

## 일부러 보지 않는 것

| 코드포인트 | 이름 | 왜 빼는가 |
|---|---|---|
| `U+00AD` | SOFT HYPHEN | 정당한 하이픈 조판 보조. 본문 전수 스캔에서 오탐 비용이 탐지 이득을 넘는다 |
| `U+180E` | MONGOLIAN VOWEL SEP | 〃 |
| `U+200E`/`U+200F` | LRM/RLM | 약한 방향 표식. 이 명령의 축 정의(오버라이드·임베딩·격리)에 없다 |
| `U+1100~U+11FF` | **한글 자모** | **정상이다. 절대 잡지 않는다** — 아래 참조 |

### 한글 자모는 정상이다

`U+1100` 계열(초성·중성·종성 낱자)은 한글 조합형의 정규 표현이다. macOS 파일시스템과
일부 IME 가 자연스럽게 만들어 낸다. `script_of` 가 이들에 대해 `None` 을 돌려주므로
혼동 축의 낱말 경계에도 들어가지 않고, 어느 축에서도 탐지 대상이 아니다.

```
scan_deception("\u{110E}\u{1169}\u{11BC}\u{110B}\u{1162}\u{11A8}") → []   // "총액"의 조합형
```

`ordinary_korean_text_is_clean` 이 이것을 명시적으로 고정한다.

다만 조합형/완성형이 **같은 문서 안에 쌍으로** 있으면 그것은 별개의 문제이고, 이미
`confusable_skeleton` 의 `compose_hangul` 이 접어 `confusable_collisions` 가 이름 축에서
보고한다(`hangul_nfc_and_nfd_share_a_skeleton`). 이 명령은 그 축을 중복 보고하지 않는다.

---

## 성능 — 코드포인트 1패스

문서 전체를 훑으므로 비용은 문자 수에 선형이어야 한다. 글자마다 정규식을 돌리는 구현은
쓰지 않았다.

- 문자열 1개당 `Vec<char>` 로 한 번 수집하고 인덱스로 한 번 훑는다 — O(n) 두 번, 즉 O(n).
- 낱말 경계·연속 열은 훑는 도중에 상태로 들고 간다. 되돌아가지 않는다.
- 발췌/재배열/지시문 표지 검사는 **탐지 1건당 고정 크기 창**(±40자, ±48자)에 묶여 있어
  문서 크기와 무관하다. 탐지가 0건이면 이 비용 자체가 발생하지 않는다.

실측은 `evidence.txt` 참조. 요약: 348개 샘플 781만 자를 54.7초에 전수 스윕했고(파일당
평균 0.157초, 대부분이 파싱·조판 비용), 104만 자 문서에서도 `inspect` 와 `info`(파싱만)의
차이가 프로세스 노이즈 안에 있다. 차수 판정은 코어를 직접 부르는 크기 사다리로 재고
(`scan_cost_stays_linear_as_input_grows`), 프로세스 단위 상한은
`scan_cost_is_linear_in_document_size` 가 건다.

---

## 드리프트 가드

이 명령이 나중에 조용히 거짓말하게 되는 경로를 모두 테스트로 막았다.

| 가드 | 무엇을 막는가 | 테스트 |
|---|---|---|
| MCP `inputSchema` 에 `required` 배열 | 자동 등록 클라이언트가 스키마를 못 읽는 상태 | `mcp_tool_declares_required_and_wires_every_property` |
| 선언 속성 전부 CLI 배선 | 서버가 인자를 조용히 버리고 성공 보고 | 〃 + `every_declared_input_property_is_wired_to_the_cli` |
| `--json` → MCP 도구 필수 | 계약 명령이 에이전트에게 안 보이는 상태 | `capabilities_mcp_covers_every_json_command` |
| capabilities ↔ `--help` 양방향 | 한쪽에만 있는 명령 | `capabilities_covers_every_help_command` · `help_covers_every_capabilities_command` · `capabilities_and_help_both_carry_inspect` |
| 선언 flags 실재 | 못 쓰는 플래그를 광고 | `capabilities_and_help_both_carry_inspect` |
| **선언 enum 값을 CLI 가 수용** | 스키마대로 보냈는데 usage 오류 | `declared_kind_enum_is_accepted_by_the_cli` |
| 실패 시 stdout 0바이트 | 절반쯤 쓰인 봉투가 성공으로 파싱됨 | `failures_keep_stdout_empty` (8경로) |
| 문서 무변경 | 검사가 원본을 건드림 | `scanning_does_not_touch_the_document` |
| 정상 문서 오탐 0 | 경보가 통째로 무시됨 | `ordinary_korean_documents_are_clean` |

`--kind` 허용값의 단일 출처는 `DeceptionKind` 하나다. `inspect_unicode_kind_enum()` 이
MCP 스키마의 `enum` 을 그 목록에서 생성하므로, 축을 추가할 곳은 코어 하나뿐이다.

## 남은 것 — 의도적으로 이번 범위 밖에 둔 것

경계를 적어 두지 않으면 다음 사람이 "검사했다"를 "전부 검사했다"로 읽는다.

| 항목 | 지금 상태 | 왜 미뤘나 |
|---|---|---|
| **머리말·꼬리말·각주·미주** | 스캔 안 함 | 순회 범위는 본문 문단 + 표 셀 + 글상자 + 수식 스크립트다(`hwp_search` 와 같은 범위). `Control::Header`/`Footer`/`Footnote`/`Endnote` 안의 문단은 아직 안 들어간다 — 축을 넓히면 주소 체계(`location`)도 함께 설계해야 해서 별건으로 남긴다 |
| `U+200E`/`U+200F` (LRM/RLM) | 탐지 안 함 | 약한 방향 표식. 이 명령의 축 정의(오버라이드·임베딩·격리)에 없다. 짧은 이름 축은 기존 `scan_text` 의 `InvisibleChar` 가 계속 잡는다 |
| 역방향 동형자 위장 | 탐지 안 함 | 키릴 낱말에 라틴 글자를 심는 반대 방향(`Мoсква`). 동형자 표가 키릴·그리스 → 라틴 단방향이라 지금 표로는 판정 근거가 없다 |
| PUA 인접 제로폭 | 보고 안 함 | 위 「좁힘의 근거」의 잔여 위험. 옛한글 문서 안에서 PUA 글자에 잇대면 이 축을 피한다 |
| 탐지 시 종료 코드 | 항상 0 | 탐지는 실행 실패가 아니다(#2707 계약). 게이트로 쓰고 싶다는 수요가 실증되면 `--fail-on <severity>` 로 별건 승격 |

### 로컬 검증 환경 제약

이 PC 의 rustc 는 **전체 테스트 타깃을 한꺼번에 LTO 로 빌드하면 크래시한다**
(`STATUS_ACCESS_VIOLATION` / `STATUS_STACK_BUFFER_OVERRUN`, 소스 진단 0건 — 코드 오류가 아니다).
그래서 검증은 대상을 지정해 돌렸다.

```console
$ cargo test --release --lib -j 2 text_security          # 기존 회귀 + 신규 단위
$ cargo test --release -j 2 --test unicode_deception_contract \
      --test cli_json_contract --test mcp_server_contract
```

`cargo test --release <필터>` 처럼 타깃을 지정하지 않으면 무관한 test 타깃 수십 개까지
LTO 로 빌드하다 rustc 가 죽는다. 최종 판정은 CI 몫이다.

## 손댄 파일

| 파일 | 내용 |
|---|---|
| `src/document_core/text_security.rs` | 탐지 코어 확장 — `DeceptionKind`/`Severity`/`DeceptionFinding`/`scan_deception`. 기존 `confusable_collisions`·`scan_identifier`·`confusable_skeleton` 과 동형자 표·`is_bidi_control`·`script_of` 를 공유하고 시그니처를 바꾸지 않았다 |
| `src/main.rs` | `inspect` 라우터 + `inspect unicode` 구현, capabilities 항목, MCP `hwp_inspect_unicode`, `--help` |
| `tests/unicode_deception_contract.rs` | 계약·드리프트·오탐 0·무변경·선형성 |
