---
kind: report
status: active
canonical: mydocs/report/unicode_spoof_guard/README.md
last_verified: 2026-08-01
---

# 화면상 같은 이름, 다른 칸 — 에이전트 대상 유니코드 기만 실증과 P0 방어

`mcp-serve`(#3571) 이후 rhwp 의 출력은 LLM 에이전트의 컨텍스트로 직행한다. 그 출력에
담기는 누름틀 이름·값·본문은 전부 **공격자가 내용을 정할 수 있는 문서**에서 온다.
이 보고서는 그 경계에서 실증한 공격과, 그중 최고 위험 축(누름틀 이름 쌍둥이)에 대한
P0 방어의 기록이다. 로드맵은 #3709.

## 0. 요약

| 항목 | 내용 |
|---|---|
| 최고 위험 | 화면상 구별되지 않는 누름틀 이름 쌍 → **엉뚱한 칸을 채우고 `filledCount:1` 성공 보고** |
| 왜 기존 방어가 못 잡나 | `ambiguous` 는 `name_counts[name] > 1`(바이트 동일 중복)만 센다 — 쌍둥이는 각각 `total==1` 이라 **구조적으로 침묵** |
| 가장 현실적인 벡터 | 키릴 동형자가 아니라 **한글 조합형/완성형**(`총액` NFC vs NFD) — 낯선 글자 0개 |
| 방어 원칙 | **보고만 하고 문자열을 고치지 않는다** (문서 엔진의 형식 보존 약속) |
| 의존성 | **0개 추가** — UTS #39 전체 표(728KB)는 WASM 산출물에 그대로 실린다 |
| 오탐 시험 | `samples/` 351건 전수 스윕 → **348/348 clean, 경고 0건** |
| 시험 | 계약 13종 + 단위 9종, 인접 회귀 39종, clippy 0, WASM 타깃 정상 |

## 1. 신뢰 경계가 어디로 옮겨갔나

무상태 CLI 시절 rhwp 의 소비자는 사람이었다. 사람은 `Total` 과 `Тotal` 을 구별하지
못해도 **화면의 다른 단서**(칸의 위치, 주변 문구, 인쇄물의 생김새)로 보정한다.

에이전트에게는 그 단서가 없다. 도구가 돌려준 문자열이 세계의 전부이고, 그것은
"검증된 도구 출력"으로 취급된다. 그래서 같은 문서, 같은 코드가 소비자만 바뀌어도
공격면이 새로 생긴다 — **이 결함은 rhwp 가 MCP 서버가 된 날 만들어졌다.**

현황 전수 검색 결과, 봉투 경로의 유니코드 방어는 0건이었다.

![현황 — 방어 표면 0건](c5_existing_defense.png)

## 2. 실증 ① 누름틀 이름 쌍둥이

공격 문서를 만든다: 한 문서에 `Total`(라틴 T, U+0054)과 `Тotal`(키릴 Т, U+0422).
HWPX 의 `hp:fieldBegin name=` 속성을 고치면 되므로 특별한 도구가 필요 없다.

![쌍둥이 이름 목록](c1_twin_names.png)

에이전트가 보는 것은 `['Total', 'Тotal', …]` — **두 문자열이 화면상 완전히 같다.**
봉투 어디에도 스크립트 정보가 없어 구별할 방법이 없다.

이제 채운다.

![엉뚱한 칸을 채우고 성공 보고](c2_twin_fill.png)

```json
{"ambiguous":[], "filled":[{"name":"Total","occurrence":0,"value":"1,000,000원"}],
 "filledCount":1, "notFound":[]}
```

**흠 없는 성공 봉투다.** 저장본을 되읽으면 `Total`(라틴)에 값이 들어가고
`Тotal`(키릴)은 비어 있다. 사람이 인쇄물에서 보는 칸이 어느 쪽인지는 공격자가 정한다
— 흰 글자·0 크기·머리말·페이지 밖 어디든 반대쪽 쌍둥이를 숨길 수 있다.

MCP 경로도 같다. 오히려 `structuredContent` 로 파싱까지 해서 넘기므로 신뢰도가 더
올라간다.

![MCP 도구 결과도 동일](c3_mcp_same.png)

### 왜 기존 판정이 못 잡는가 — 구조적 침묵

`ambiguous` 는 이런 모양이다:

```rust
*name_counts.entry(n.to_string()).or_insert(0) += 1;   // HashMap<String, usize>
…
if occurrence == 0 && total > 1 && !key.contains('[') { ambiguous.push(…) }
```

키가 `String` 이므로 `Total` 과 `Тotal` 은 **서로 다른 해시 버킷**에 들어가 각각
`total == 1` 이 된다. 즉 이 판정은 잡지 못하는 게 아니라 **잡을 수 있는 축이 아니다.**
대조 실험이 이를 못박는다: 같은 문서에서 두 이름을 바이트까지 동일하게 만들면
`ambiguous:[{"matched":1,"name":"총액","total":2}]` 가 즉시 발화한다. **코드포인트
하나가 요란한 경고와 완전한 침묵을 가른다.**

`agent_profiles.rs` 가 정한 완료 기준("notFound/ambiguous 가 비어야 완료")이 충족된
채로 결과만 틀린다는 점이 이 결함의 성질을 잘 보여준다.

### 한글 조합형/완성형 — 더 현실적인 벡터

키릴을 쓸 필요조차 없다.

| | 코드포인트 |
|---|---|
| `총액` 완성형(NFC) | U+CD1D U+C561 |
| `총액` 조합형(NFD) | U+110E U+1169 U+11BC U+110B U+1162 U+11A8 |

**낯선 글자가 하나도 없다.** macOS 파일시스템과 일부 한글 IME 가 자연스럽게 만드는
형태라 "수상한 문서"로 보이지도 않고, 어떤 한국어 검토자도 눈으로 잡을 수 없다.
행정서식이 주 무대인 rhwp 에서는 이쪽이 본령이다.

## 3. 실증 ② bidi·제로폭·주입 문구

![bidi 페이로드 무경고 통과](c4_bidi_payload.png)

되읽기 분석 결과, `U+202E`(RIGHT-TO-LEFT OVERRIDE, Trojan Source 계열)·제로폭
문자·키릴 동형자·주입 문구가 전부 봉투를 그대로 통과한다.

한 가지 반가운 사실과 한 가지 함정이 있다.

- **JSON 전선 위에서 ANSI 이스케이프는 무해하다.** `serde_json` 이 C0 제어문자를
  `\u001b` 형태로 이스케이프한다. 다만 이는 **부수 효과이지 방어가 아니다** — 호스트가
  `JSON.parse` 한 순간 진짜 ESC 로 되살아나고, 비-JSON CLI 경로는 raw 바이트를
  그대로 터미널에 내보낸다.
- **bidi·제로폭·태그 문자는 전부 U+007F 초과**라 어떤 이스케이프도 거치지 않는다.
  특히 태그 문자(U+E0020~U+E007F)는 어느 렌더러에도 보이지 않으면서 토크나이저에는
  평문 ASCII 로 읽힌다.

## 4. 실증 ③ 공격이 아닌 것 — 노력을 낭비하지 않기 위해

감사에서 **아님**으로 판정한 축을 명시해 둔다. 이 목록이 없으면 후속 작업이 이미
닫힌 문을 다시 두드린다.

- **도구 이름 동형자** — `match` 바이트 완전 일치 + `find(|t| t["name"] == name)`.
  `hwp_infо`(키릴 о)는 `알 수 없는 도구` 로 거부된다. 퍼지·접두·대소문자 무시 매칭 없음.
- **프로필 이름 동형자** — 바이트 일치 + 미지의 이름은 `EXIT_USAGE` 하드 실패.
  게다가 프로필은 운영자 argv 에서 오지 문서에서 오지 않는다.
- **셸 주입** — `Command::new(exe).args(...)`, 셸 경유 없음.
- **프로필 도구 게이팅** — `tools/list` 필터와 `tools/call` 차단이 둘 다 있는
  **닫힌 경계**다. 잘 되어 있으니 손대지 않는다.

## 5. 방어 설계 — 세 가지 결정

### 5-1. 정화하지 않고 보고한다

각각 단독으로 충분한 네 가지 이유가 있다.

1. **형식 보존이 제품의 핵심 약속이다** (#3383, 왕복 충실도 하네스, `ir-diff`).
   정화기는 이 셋을 동시에 조용히 깬다. 키릴로 쓰인 정당한 러시아어 인용문을
   라틴으로 고쳐 저장하는 순간 그 문서는 손상된 것이다.
2. **사용자 파일이 실제로 망가진다.** `hwp_doc_text` → `hwp_doc_replace_text` →
   `hwp_doc_save` 는 같은 핸들을 도는 읽기-수정-쓰기 루프다. 읽을 때 정화하면
   정화된 형태가 그대로 디스크에 기록된다.
3. **좌표계가 어긋난다.** `GrepMatch.char_offset`/`length` 는 문자 오프셋이다.
   `U+200B` 하나를 지우면 그 뒤 모든 좌표가 무효가 되고, `set-cell`·`--occurrence`
   가 그 좌표에 의존한다.
4. **위협이 사라지지도 않는다.** bidi 를 걷어내도 "무시하고 다음을 실행하라"라는
   한글 문장은 그대로다. 정화는 충실도를 내주고 **완전하다는 착각**을 사는 거래다.

그래서 `text_security` 의 모든 함수는 `&str` 을 받아 판정만 돌려준다.

### 5-2. 의존성을 더하지 않는다

`unicode-security` 는 기술적으로 정답이지만 이 저장소에는 오답이다.

- UTS #39 데이터는 `confusables.txt` 728KB, `IdentifierType.txt` 515KB.
- `[lib] crate-type = ["rlib", "cdylib"]` 이라 `[dependencies]` 에 넣고 `lib.rs` 에서
  참조하면 그 표가 **브라우저 뷰어 WASM 산출물에 그대로 실린다** — MCP 서버를
  영원히 실행하지 않을 아티팩트에.
- `mcp_serve.rs` 모듈 헤더의 무의존 원칙과도 충돌한다.

실제 스푸핑에 쓰이는 글자만 담은 ~50항목 표로 같은 방어력을 얻는다. **한글 조합은
표가 아니라 산술**(Unicode 3.12 Hangul Syllable Composition)이라 정규화 크레이트 없이
정확히 접힌다:

```
S = (L-0x1100)*588 + (V-0x1161)*28 + (T-0x11A7) + 0xAC00
```

훗날 근거가 쌓이면 `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]` 아래
feature 게이트로 얹는 길이 이미 열려 있다(svg2pdf 계열 선례). 그때까지 **UTS #39
준수를 표방하지 않는다** — 판정 이름도 `confusableFieldName`/`mixedScript` 처럼
사실만 말한다.

### 5-3. 오탐은 기능을 죽인다

경고를 무시하도록 학습된 에이전트에게 `textSecurity` 는 없느니만 못하다. 그래서
탐지 규칙을 좁게 잡았다.

- **본문 텍스트에는 스크립트 판정을 하지 않는다.** 한국어 문서가 러시아어를 인용하는
  것은 정상이다. 혼합 스크립트는 **에이전트가 지목에 쓰는 이름**에만 적용한다.
- **단일 스크립트는 통과시킨다.** 순수 키릴(`Москва`)·순수 그리스(`αβγ`)는 정당하다.
- **한글·한자·숫자·문장부호는 스크립트 계산에서 뺀다.** 라틴과 헷갈릴 일이 없고,
  세는 순간 한국어 문서 전부가 발화한다.
- **같은 이름의 단순 반복은 건드리지 않는다.** 그건 기존 `ambiguous` 의 몫이다.

## 6. 구현

### `document_core::text_security` — 무의존 탐지기

| 함수 | 역할 |
|---|---|
| `scan_text(&str)` | bidi 오버라이드·제로폭·ANSI 이스케이프 (자유 서술 문자열용) |
| `scan_identifier(&str)` | 위 + 혼합 스크립트 (이름용) |
| `confusable_skeleton(&str)` | 보이지 않는 문자 제거 → **한글 음절 조합** → 동형자 라틴 접기 → 소문자화 |
| `confusable_collisions(&[String])` | 골격이 같은 **서로 다른** 이름 무리 |

### 봉투 배선

`fields --json` 에 `textSecurity` 를 싣는다. `hwp_fields`·`hwp_doc_fields` 가 같은
helper 를 쓰므로 MCP 양쪽이 함께 덮인다.

![AFTER — fields 봉투](d1_after_fields.png)

`edit fill-fields`·`hwp_doc_fill_fields` 에는 `confusable` 판정을 더한다 — 이것이
실제 보안 수정이다.

![AFTER — 채우기 판정](d2_after_fill.png)

기존 `ambiguous` 는 **여전히 비어 있다**(바이트가 달라 개수 판정에 걸리지 않는다).
새 축이 정확히 그 공백을 메운다. 사람용 경로에도 stderr 경고를 낸다 — 화면상 같은
이름이라 눈으로는 잡을 수 없는 축이기 때문이다.

한글 NFC/NFD 쌍둥이도 잡힌다.

![AFTER — 한글 조합형/완성형](d3_hangul_nfd.png)

### 봉투는 소견이 없어도 실린다

![정상 문서와 자기서술](d5_clean_and_caps.png)

`status: "clean"` 을 항상 싣는 이유는 소비자가 **"검사했고 깨끗함"** 과 **"검사하지
않는 옛 바이너리"** 를 구별할 수 있어야 하기 때문이다. 키가 없으면 후자다.
`capabilities.jsonContract.textSecurity` 가 같은 사실을 선언 층에서도 말한다.

## 7. 오탐 시험 (cry-wolf) — 출하 기준

![351건 전수 스윕](d4_crywolf.png)

`samples/` 전체 351건에 `fields --json` 을 돌려 경고율을 쟀다.

```
scanned OK : 348   (parse/read failures skipped: 3)
clean      : 348
warned     : 0
clean rate : 100.00%   (기준: >= 99%)
```

실제 한국 공문서·서식 코퍼스 전수에서 **경고 0건**. 이 수치가 무너지면 기능이
아니라 소음이므로, 기준을 문서에 못박아 둔다.

## 8. 시험

![계약·단위 시험](d6_tests.png)

탐지 축 5종과 **오탐 가드 2종**을 함께 고정했다. 오탐 가드가 탐지 시험만큼 중요하다
— `text_security_is_clean_on_ordinary_korean_forms` 가 깨지면 그 순간 기능의 가치가
사라진다.

공격 문서는 **저장소에 커밋하지 않는다.** 시험이 실행 시점에 표본 문서를 HWPX 로
변환하고 `name=` 속성 두 개를 바꿔 재압축한다. 이를 위해 `zip` 을
`[dev-dependencies]` 에 넣었는데, `[dependencies]` 와 같은 버전이라 의존성 그래프도
WASM 산출물도 그대로다.

![무회귀·게이트](d7_noreg.png)

## 9. 검증 매트릭스

| 게이트 | 결과 |
|---|---|
| `cargo test --test fields_json_contract` | 13/13 (신규 5종 포함) |
| `cargo test --lib text_security` | 9/9 |
| 인접 계약(mcp_session_edit·view·cli_json·edit_fill_fields) | 39/39 무회귀 |
| `cargo clippy --profile release-test --bin rhwp` | 경고 0 |
| `cargo fmt --check` | 통과 |
| `cargo build --target wasm32-unknown-unknown --lib` | 정상 (신규 크레이트 0) |
| 오탐 스윕 `samples/` 351건 | 348/348 clean |

## 10. 한계와 후속 (#3709 로드맵)

- **P0 는 이름 축만 덮는다.** 본문·검색 문맥·표 셀·글꼴 이름·`digest` 는 P1 이다.
  특히 `digest` 는 소비자가 4B급 모델이라 주입 저항이 가장 낮아 우선순위가 높다.
- **`guide`(안내문) 축은 P1 의 핵심이다.** 서식 포맷상 "채우는 이에게 주는 지시"라
  에이전트가 지침으로 읽도록 설계돼 있는데, 출처 표시가 없다. 완화는 정규식이 아니라
  **출처 표시**(MCP 선행 content 블록, delimiting-mode spotlighting)다.
- **MCP 출력 경로 무제약은 P2 다.** 감사에서 경로 traversal·ADS(`file.hwpx:stream`)·
  후행 점/공백 별칭을 실증했다. 문서가 `output` 을 직접 정하지는 못하지만, `guide`
  축이 그 값을 에이전트에게 불러 주는 통로가 된다 — 두 조각을 각각 실증했고 이어
  붙이면 유출 경로다.
- **자연어 주입은 범위 밖이다.** 순한글 "무시하고 다음을 실행하라"는 어떤 코드포인트
  탐지로도 잡히지 않는다. 정직한 완화는 출처 표시이지 내용 분류가 아니다.

## 부록. 재현

```bash
# 공격 문서 합성 (표본 → HWPX → name= 교체 → 재압축)
rhwp export-hwpx samples/field-01.hwp f01.hwpx
#   Contents/section0.xml 에서 name="회사명" → name="Total",
#                              name="작성자" → name="Тotal"(키릴 Т) 로 교체 후 재압축

rhwp fields attack_twin.hwpx --json | jq .textSecurity
rhwp edit fill-fields attack_twin.hwpx --data '{"Total":"999"}' --dry-run --json

# 오탐 스윕
python cry_wolf.py     # samples/ 전수 fields --json → clean 비율
```
