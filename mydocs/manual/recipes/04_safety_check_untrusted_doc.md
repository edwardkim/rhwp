---
kind: guide
status: active
canonical: mydocs/manual/recipes/04_safety_check_untrusted_doc.md
last_verified: 2026-08-03
---

# 레시피 4 — 출처를 모르는 문서를 처음 열 때

**목표 한 줄**: 메일 첨부·다운로드 폴더·낯선 USB에서 온 `.hwp`/`.hwpx`를 **본문 전체를
셸이나 LLM에 흘려보내기 전에**, 기계로 확인 가능한 신호만으로 "지금 열어도 되는
문서인가"를 판정한다.

이 레시피는 "악성 매크로를 잡아낸다"는 백신 소프트웨어의 약속이 아니다. HWP에는
실행 코드가 없다 — 위험은 다른 모양으로 온다: 누름틀 안내문에 숨겨진 프롬프트
주입 문구, 표 셀에 박힌 대량의 텍스트, 화면에 안 보이는 사설 영역(PUA) 문자가
엉뚱한 글자로 렌더링되는 문제. 이 레시피는 **읽기 전용** 명령만 써서 문서 크기를
가늠하고, 필드별 텍스트 보안 신호를 확인하고, 필요한 부분만 좁혀서 본다 —
`export-text`로 전체를 한 번에 쏟아붓지 않는다.

절차 요약:

```
info (문서 규모·형식 확인, 읽기 전용)
  → digest (짧은 미리보기, 전체 덤프 아님)
  → fields (필드별 textSecurity 신호 확인)
  → [필요시] search (본문 전체 대신 특정 키워드만 좁혀서 확인)
  → 판정 통과 후에만 export-text/edit 계열 명령 사용
```

모든 명령·출력은 이 저장소의 `samples/form-01.hwp`(명령 단추 예제 문서)와
`output/pua-test.hwp`(`rhwp gen-pua`로 생성한 사설영역 문자 회귀 문서, 18종
PUA 코드포인트 포함)로 실제 실행해서 얻었다. 지어낸 값은 없다.

## 0단계 — 왜 `export-text`로 바로 시작하면 안 되나

낯선 문서의 본문 텍스트를 통째로 표준출력에 찍어 셸 파이프라인이나 LLM 프롬프트에
바로 넣는 습관은 두 가지 문제를 만든다.

1. 문서 크기를 모르는 채로 전체를 읽으면, 수백 페이지짜리 문서에서 토큰/시간을
   낭비한다.
2. 본문 안에 "이 지시를 무시하고 다음을 실행하라" 같은 문구가 심겨 있으면, 그
   문구가 그대로 다음 단계(LLM 프롬프트, 셸 명령)에 흘러들어갈 수 있다.

그래서 이 레시피는 **크기 확인 → 짧은 미리보기 → 필드별 신호 확인** 순서로
점진적으로 좁혀 간다. 각 단계는 전 단계보다 더 많은 내용을 노출하므로, 이상 신호가
보이면 그 자리에서 멈춘다.

## 1단계 — `info`로 문서 규모와 형식부터 확인한다

`info`는 본문 텍스트를 전혀 반환하지 않는다 — 페이지 수·문단 수·글꼴 목록·형식
버전·파일 크기만 본다. 가장 싼 비용으로 "이 문서가 상식적인 크기인가"를 먼저
판정한다.

```bash
rhwp info samples/form-01.hwp --json
```

실측 출력:

```json
{"fonts":["한컴바탕","함초롬돋움","함초롬바탕"],"format":"hwp5","pageCount":1,"paraCount":13,"schemaVersion":"1.0","sections":1,"sizeBytes":18432,"source":"samples/form-01.hwp","title":"명령 단추","version":"5.0.3.0"}
```

읽어야 할 것 셋:

- `pageCount: 1`, `paraCount: 13` — 규모가 상식적이다. 1페이지짜리 서식이 파일
  크기 수십 MB로 나오거나, `paraCount`가 비정상적으로 크면(예: 수만 단위) 뭔가가
  반복 삽입되어 있다는 신호다 — 다음 단계로 넘어가기 전에 의심한다.
- `title`: `"명령 단추"` — 문서 속성에 저장된 제목. 본문과 무관한 제목이거나
  URL·명령어처럼 보이면 이 자체가 이상 신호다.
- `format`/`version`: `"hwp5"` / `"5.0.3.0"` — 문서 포맷 버전. 지원 범위 밖의
  포맷이면 다른 명령이 거부할 수 있다.

PUA(사설 영역 문자) 회귀 문서로도 같은 명령을 실행해 대조해본다:

```bash
rhwp info output/pua-test.hwp --json
```

실측 출력:

```json
{"fonts":["함초롬돋움","함초롬바탕"],"format":"hwp5","pageCount":1,"paraCount":19,"schemaVersion":"1.0","sections":1,"sizeBytes":15360,"source":"output/pua-test.hwp","title":"[PUA 회귀 검증 — Task #509]","version":"5.1.0.1"}
```

`title`에 대괄호로 싸인 내부 태그(`[PUA 회귀 검증 — Task #509]`)가 그대로 들어
있다 — 이런 문서는 사람이 작성한 최종 문서가 아니라 테스트/회귀 자산일 가능성이
높다는 신호다. `info` 한 번으로 이런 맥락 정보까지 공짜로 얻는다.

## 2단계 — `digest`로 짧은 미리보기만 본다

`digest`는 `export-text`처럼 전체를 다 뱉지 않는다 — 앞부분 일부만 잘라 보여주고,
`truncated` 필드로 "잘렸다"는 사실을 명시한다. 본문 전체를 노출하지 않고도 "이
문서가 예상한 종류의 문서인가"를 판정하기에 충분하다.

```bash
rhwp digest samples/form-01.hwp --json
```

실측 출력:

```json
{"excerpt":"명령 단추\n\n선택 상자\n\n계절 선택\n\n라디오 단추\n\n\n\n여기에 입력\n\n","format":"hwp5","pageCount":1,"paraCount":13,"schemaVersion":"1.0","source":"samples/form-01.hwp","truncated":false,"nextStep":"더 읽으려면 export-text --json -p <쪽>, 찾으려면 search --json"}
```

`truncated: false` — 이 짧은 문서는 통째로 다 보였다는 뜻이다. 큰 문서라면
`truncated: true`가 나오고, 그 다음 어떻게 더 읽을지는 `nextStep` 필드가 직접
알려준다(예: 특정 쪽만 `export-text --json -p <쪽>`, 아니면 `search --json`으로
키워드만 좁혀서).

PUA 문서로 같은 명령을 돌리면 어떤 신호가 섞여 있는지 미리 볼 수 있다:

```bash
rhwp digest output/pua-test.hwp --json --max-chars 500
```

실측 출력(발췌, `excerpt`만):

```json
{"excerpt":"[PUA 회귀 검증 — Task #509]\nU+0F076 (Basic, mel-001, ❖ U+2756): ❖  ← 한컴 PDF 정답지\nU+0F09F (Basic, biz_plan, • U+2022): •  ← 한컴 PDF 정답지\nU+0F0A0 (Basic, synam-001, ▪ U+25AA): ·  ← 한컴 PDF 정답지\n...","truncated":true}
```

`truncated: true`가 뜬 이유는 `--max-chars 500`로 직접 잘랐기 때문이다 — 큰
문서를 다룰 때 이렇게 명시적으로 상한을 걸어 미리보기 비용을 통제할 수 있다. 본문에
`U+F0xx`대 코드포인트가 반복해서 나오는 것 자체가 "이 문서는 사설 영역 문자를
쓰고 있으니, 화면에 보이는 글자와 실제 저장된 코드포인트가 다를 수 있다"는
신호다 — 이 문서를 신뢰해서 값으로 쓰기 전에 `export-svg`로 실제 렌더링을 눈으로
대조하는 편이 안전하다.

## 3단계 — `fields`로 필드별 `textSecurity` 신호를 확인한다

누름틀이 있는 서식이라면, `fields --json`의 각 필드 항목에 `textSecurity`가
붙어 나온다. 이 필드는 안내문·현재값 안에 숨은 콘텐츠나 프롬프트 주입으로 보이는
패턴이 있는지를 요약한 판정이다.

```bash
rhwp fields --json samples/form-01.hwp
```

실측 출력:

```json
{"fieldCount":1,"fields":[{"command":"Clickhere:set:48:Direction:wstring:6:여기에 입력 HelpState:wstring:0:  ","editableInForm":true,"fieldId":2110609883,"fieldType":"ClickHere","guide":"여기에 입력","location":{"nested":[],"paragraph":10,"section":0},"memo":"","name":"myMsg01","value":""}],"schemaVersion":"1.0","source":"samples/form-01.hwp","textSecurity":{"status":"clean"}}
```

`textSecurity.status: "clean"` — 이 필드의 안내문·현재값에 이상 신호가 없다는
뜻이다. `"clean"`이 아닌 상태를 만나면(예: 의심스러운 지시문 패턴이 잡힌 경우),
[레시피 1의 0단계](01_fill_form_and_submit.md#0단계--이-서식이-누름틀-기반인지-먼저-확인한다)로
돌아가 그 필드를 값으로 채우거나 그 값을 그대로 다음 단계(LLM 프롬프트 등)에
넘기지 않는다 — 사람이 먼저 `guide`/`value` 원문을 눈으로 읽고 판단한다.

이 문서는 `fieldCount: 1`이라 누름틀 서식이지만, 표 칸 기반 서식(`edit
set-cell` 축)이라면 `fields`가 `fieldCount: 0`을 돌려준다 — 그 경우 표 셀
텍스트를 신뢰하기 전 검토는 [레시피 2의 판정
표](02_table_csv_roundtrip.md)를 참고한다.

## 4단계(선택) — `search`로 특정 키워드만 좁혀서 확인한다

본문 전체를 보는 대신, 걱정되는 패턴(예: URL, 명령어처럼 보이는 문자열)만
targeted로 찾을 수 있다. `search`는 매치된 위치만 반환하고 문서 전체를 노출하지
않는다.

```bash
rhwp search samples/form-01.hwp "입력" --json
```

실측 출력:

```json
{"caseSensitive":true,"matchCount":0,"matches":[],"query":"입력","schemaVersion":"1.0","source":"samples/form-01.hwp","totalMatchCount":0,"truncated":false}
```

이 문서 본문에는 정확히 `"입력"`이라는 문자열이 없다 — `matchCount: 0`이 그
사실을 그대로 보여준다(`digest`의 미리보기에 보인 "여기에 입력"은 누름틀
`guide` 텍스트라 본문 검색 인덱스와 다른 경로로 저장돼 있을 수 있다는 뜻이기도
하다 — 필드 관련 텍스트는 `fields --json`으로 별도 확인). 기본은 대소문자
구분(`caseSensitive: true`)이며 `--ignore-case`로 끌 수 있다.

실제로 매치가 있는 문서로 대조해본다 — `output/pua-test.hwp`에서 반복 등장하는
`"정답지"`를 찾고, `--limit`으로 상한을 건다:

```bash
rhwp search output/pua-test.hwp "정답지" --json --limit 2
```

실측 출력:

```json
{"caseSensitive":true,"matchCount":2,"matches":[{"charOffset":48,"context":"…(Basic, mel-001, ❖ U+2756):   ← 한컴 PDF 정답지","length":3,"page":0,"paragraph":1,"section":0,"text":"U+0F076 (Basic, mel-001, ❖ U+2756):   ← 한컴 PDF 정답지"},{"charOffset":49,"context":"…Basic, biz_plan, • U+2022):   ← 한컴 PDF 정답지","length":3,"page":0,"paragraph":2,"section":0,"text":"U+0F09F (Basic, biz_plan, • U+2022):   ← 한컴 PDF 정답지"}],"query":"정답지","schemaVersion":"1.0","source":"output/pua-test.hwp","totalMatchCount":18,"truncated":true}
```

`matchCount: 2`인데 `totalMatchCount: 18`이다 — `--limit 2`로 반환 개수만
잘랐을 뿐, 실제로는 18곳에 매치가 있다는 뜻이다(`truncated: true`가 이 사실을
명시한다). 이렇게 "실제 매치 수"와 "반환된 매치 수"가 분리돼 있어서, 상한을
낮게 걸어도 전체 규모를 놓치지 않는다. 각 매치의 `context`는 매치 지점 주변
텍스트만 잘라 보여준다 — 본문 전체를 노출하지 않고도 그 문자열이 어떤 맥락에서
쓰였는지 판단할 수 있다.

## 5단계(선택) — 여러 문서를 한 번에 점검한다: `batch`

메일함 첨부 여러 개, 다운로드 폴더 전체처럼 문서가 여러 개 쌓여 있을 때는 하나씩
명령을 반복하는 대신 `batch`에 파일 목록을 표준입력으로 흘려보낸다. `batch`가
지원하는 하위 명령은 `export-text·info·export-structure·export-tables·fields·
search·convert` 다. 여기서는 이 레시피에서 쓴 `fields`와 `info`를 배치로
돌린다.

점검 대상 목록을 만든다(파일 하나당 한 줄):

```bash
printf 'samples/form-01.hwp\nsamples/form-02.hwp\noutput/pua-test.hwp\n' > check_list.txt
```

```bash
rhwp batch fields --json < check_list.txt
```

실측 출력 — 파일마다 한 줄씩 JSON이 나오고, 마지막에 사람이 읽는 요약이 붙는다:

```json
{"fieldCount":1,"fields":[{"command":"Clickhere:set:48:Direction:wstring:6:여기에 입력 HelpState:wstring:0:  ","editableInForm":true,"fieldId":2110609883,"fieldType":"ClickHere","guide":"여기에 입력","location":{"nested":[],"paragraph":10,"section":0},"memo":"","name":"myMsg01","value":""}],"schemaVersion":"1.0","source":"samples/form-01.hwp","textSecurity":{"status":"clean"}}
{"fieldCount":1,"fields":[{"command":"Clickhere:set:48:Direction:wstring:6:여기에 입력 HelpState:wstring:0:  ","editableInForm":true,"fieldId":2110609883,"fieldType":"ClickHere","guide":"여기에 입력","location":{"nested":[],"paragraph":10,"section":0},"memo":"","name":"myMsg01","value":""}],"schemaVersion":"1.0","source":"samples/form-02.hwp","textSecurity":{"status":"clean"}}
{"fieldCount":0,"fields":[],"schemaVersion":"1.0","source":"output/pua-test.hwp","textSecurity":{"status":"clean"}}
```

```
batch: 3건 중 3 성공, 0 실패 (3ms, threads=32)
```

`fieldCount: 0`인 `output/pua-test.hwp`는 애초에 누름틀이 없는 문서다 — 이
결과 하나만으로는 그 문서가 안전하다는 뜻이 아니라, "이 축(`fields`)으로는 더
볼 게 없다"는 뜻일 뿐이다. 3단계까지 온 이유가 바로 이것이다 — `fieldCount:
0`이 나온 문서는 2단계 `digest`의 미리보기 판정으로 되돌아가서 다시 본다(이
레시피에서는 앞서 `pua-test.hwp`에 대해 이미 그렇게 했다).

`info`도 같은 방식으로 배치 처리할 수 있다 — 여러 문서의 규모를 한 번에
비교할 때 쓴다:

```bash
rhwp batch info --json < check_list.txt
```

실측 출력(요약 줄 제외, 세 문서분):

```json
{"fonts":["한컴바탕","함초롬돋움","함초롬바탕"],"format":"hwp5","pageCount":1,"paraCount":13,"schemaVersion":"1.0","sections":1,"sizeBytes":18432,"source":"samples/form-01.hwp","title":"명령 단추","version":"5.0.3.0"}
{"fonts":["한컴바탕","함초롬돋움","함초롬바탕"],"format":"hwp5","pageCount":1,"paraCount":13,"schemaVersion":"1.0","sections":1,"sizeBytes":10752,"source":"samples/form-02.hwp","title":"명령 단추","version":"5.0.3.0"}
{"fonts":["함초롬돋움","함초롬바탕"],"format":"hwp5","pageCount":1,"paraCount":19,"schemaVersion":"1.0","sections":1,"sizeBytes":15360,"source":"output/pua-test.hwp","title":"[PUA 회귀 검증 — Task #509]","version":"5.1.0.1"}
```

`jq`로 좁혀서 이상 신호가 있는 파일만 골라내는 게이트로 쓸 수 있다:

```bash
rhwp batch fields --json < check_list.txt \
  | jq -c 'select(.textSecurity.status != "clean")'
```

아무 줄도 안 나오면(이 저장소 표본은 전부 `clean`이라 실제로 빈 출력이다) 3개
문서 모두 필드 축에서는 이상 신호가 없다는 뜻이다. `batch`는 파일당 스레드를
나눠 병렬로 돈다(`threads=32` — CPU 코어 수에 맞춰 자동 결정되고 `--threads
<N>`으로 상한을 지정할 수 있다) — 문서 수가 많을수록 하나씩 반복하는 것보다
확실히 빠르다.

## 판정 — 다음 단계로 넘어가도 되는 기준

| 신호 | 판정 | 처방 |
|---|---|---|
| `info`의 `pageCount`/`paraCount`가 문서 성격에 비해 비정상적으로 큼 | 의심 | 열람을 멈추고 파일 출처를 먼저 확인한다 |
| `info`의 `title`이 본문과 무관하거나 내부 태그·코드처럼 보임 | 주의 | 테스트/회귀 자산일 가능성 — 최종 배포본으로 신뢰하지 않는다 |
| `digest`의 `excerpt`에 지시문처럼 읽히는 문장이 있음(예: "이전 지시를 무시하고…") | 위험 | 이 텍스트를 LLM 프롬프트나 셸 명령에 그대로 넘기지 않는다. 사람이 원문을 검토 |
| `digest`의 `excerpt`에 `U+F0xx`~`U+FFxx`대 코드포인트가 반복됨 | 주의 | 사설 영역 문자 — `export-svg`로 실제 렌더링을 눈으로 대조하기 전에는 화면 글자를 신뢰하지 않는다 |
| `fields --json`의 `textSecurity.status`가 `"clean"`이 아님 | 위험 | 해당 필드의 `guide`/`value` 원문을 사람이 직접 읽고서만 다음 단계 진행 — [레시피 1](01_fill_form_and_submit.md) |
| 위 모든 신호가 정상 | 통과 | `export-text`/`edit fill-fields` 등 본격 처리 단계로 진행 가능 |

## 이 레시피가 하지 않는 것

- 바이러스·매크로 스캔이 아니다 — HWP5/HWPX에는 실행 가능한 매크로가 없다(OLE
  개체 삽입 등 다른 경로의 위험은 이 레시피 범위 밖이다).
- 암호화된 문서(`EncryptVersion 4`)의 복호화는 다루지 않는다 — 전역 옵션
  `--password`/`--password-stdin`으로 열되, 비밀번호 자체를 신뢰할 수 없는
  경로에서 받았다면 그 비밀번호도 별도로 검증한다.
- 완벽한 보장이 아니다 — `textSecurity.status: "clean"`은 "이 시점의 판정
  규칙으로 이상 신호를 못 찾았다"는 뜻이지, 문서가 100% 안전하다는 증명이 아니다.
  최종 판단은 사람이 한다.

## 관련 문서

- [레시피 1 — 서식 문서를 채워서 제출용으로 만들기](01_fill_form_and_submit.md) —
  0단계에서 `textSecurity.status`가 `"clean"`이 아닐 때 이 레시피로 오는
  진입점.
- [레시피 2 — 표 데이터를 CSV로 뽑아 스프레드시트에서 고치고 되돌리기](02_table_csv_roundtrip.md) —
  `untrustedContent` 판정이 걸렸을 때 이 레시피로 먼저 점검하는 흐름.
- [CLI 명령어 매뉴얼](../cli_commands.md) — `info`·`digest`·`fields`·`search`의
  전체 옵션과 종료 코드 계약.
- [문서 진단 도구](../document_diagnostics_tool_manual.md) — `info → dump →
  diag` 순서로 문서 자체의 구조적 이상을 좁혀 가는 절차(이 레시피보다 더 깊은
  진단이 필요할 때).
- [에이전트 실패 사전](../agent_troubleshooting_guide.md) — 오류 문자열별
  원인·처방.
- [레시피 5 — 서식 하나에 여러 사람 데이터를 한 번에 채우기](05_mail_merge_batch_fill.md) —
  이 레시피로 안전 판정을 통과한 서식을 대량으로 채울 때.
- [레시피 6 — 편집 전후를 눈이 아니라 숫자로 비교하기](06_visual_regression_before_after.md) —
  `--verify`로 못 잡는 렌더링 차이까지 정량화할 때.
