---
kind: guide
status: active
canonical: mydocs/report/edit_demo_regulatory/README.md
last_verified: 2026-07-27
---

# 실제 CLI 작동 사례 — 규제영향분석서(157쪽·누름틀 1,070개) 반복 항목 채우기

> 대상: `samples/80168_regulatory_analysis.hwp` — **규제영향분석서**, 정부 제출 법정 서식.
> 앞선 사례: [복학원서](../edit_demo_bokhak/README.md)(표 격자 좌표),
> [보도자료](../edit_demo_hongbo/README.md)(누름틀 + MCP 구동).
> 이번 사례는 **사람이 손으로는 채울 수 없는 규모**를 다루고, 그 과정에서
> `fill-fields` 가 반복 항목을 조용히 놓치던 결함(#3476)을 찾아 함께 고쳤다.

## 왜 이 문서인가

사람들이 실제로 제출하는 서식은 단순하지 않다. 이 문서는:

| 지표 | 값 |
|---|---|
| 쪽수 | **157쪽** |
| 누름틀(필드) | **1,070개** |
| 고유 필드 이름 | **151개** |
| 표 | 154개 (병합 셀 905개, 중첩 표 30개) |
| 최대 표 | 24행×14열, 셀 65개 중 병합 54개 |

**1,070개인데 고유 이름이 151개**라는 게 핵심이다. 같은 이름이 반복되는 것은 서식이
**같은 항목 묶음을 여러 번** 요구하기 때문이다 — `피규제집단명` 이 14번 나오는 것은
규제 대상 집단이 14개이고 각각 이름을 적어야 한다는 뜻이다.

```console
$ rhwp fields samples/80168_regulatory_analysis.hwp --json \
  | jq -r '[.fields[].name]|group_by(.)|map({n:.[0],c:length})|sort_by(-.c)[:4][]|"\(.c)회\t\(.n)"'
14회	피규제집단내용
14회	피규제집단명
14회	피규제집단유형
10회	개요_경쟁영향평가
```

## 이 사례가 찾아낸 결함 (#3476)

### 전 — 첫 번째만 채워지고, 그 사실을 알려주지 않았다

```console
$ rhwp edit fill-fields in.hwp --data '{"피규제집단명":"가상협회 회원사"}' -o out.hwp --json
{"filledCount":1,"notFound":[]}          # ← "요청한 대로 다 됐다"로 읽힌다

$ rhwp fields out.hwp --json | jq -r '[.fields[]|select(.name=="피규제집단명")]|to_entries[]|"[\(.key)] \(.value.value)"'
[0] 가상협회 회원사                        # 첫 번째만 바뀜
[1] 피규제자
...
[13] 이해관계자                            # 나머지 13개는 그대로
```

두 가지가 겹쳐 위험했다. **나머지 13개를 채울 방법이 없었고**, `filledCount:1`·`notFound:[]`
가 성공처럼 보여 **에이전트가 불완전한 문서를 완성본으로 판단**했다. 제출 실패는 문서가
깨져서가 아니라 **빈칸이 남아서** 일어난다.

### 후 — 순번으로 지목하고, 모호하면 알린다

```console
$ rhwp edit fill-fields in.hwp --data @groups.json -o out.hwp --json
{"filledCount":14,"notFound":[],"ambiguous":[]}     # 14개 전부 (실측 274ms)

$ rhwp fields out.hwp --json | jq -r '[.fields[]|select(.name=="피규제집단명")]|to_entries[]|"[\(.key)] \(.value.value)"'
[0] 가상협회 회원사
[1] 가상조합 조합원
[2] 가상공제회 가입자
...
[13] 가상기관 위탁사업자
```

순번을 빼면 **몇 개 중 몇 개를 채웠는지 보고**한다:

```console
$ rhwp edit fill-fields in.hwp --data '{"피규제집단명":"가상협회 회원사","피규제집단유형":"사업자"}' \
    -o x.hwp --dry-run --json | jq -c '{filledCount, ambiguous}'
{"filledCount":2,"ambiguous":[{"matched":1,"name":"피규제집단명","total":14},
                              {"matched":1,"name":"피규제집단유형","total":14}]}
```

## 재현

```bash
set -e
cp samples/80168_regulatory_analysis.hwp reg-work.hwp

# 1) 이 서식이 무엇을 몇 개씩 요구하는지 파악한다
rhwp fields reg-work.hwp --json \
  | jq -r '[.fields[].name]|group_by(.)|map({n:.[0],c:length})|sort_by(-.c)[:10][]|"\(.c)회\t\(.n)"'

# 2) 반복 항목을 순번으로 지목해 채운다 (UTF-8 로 저장할 것)
python - > groups.json <<'PY'
import json, io, sys
names = ["가상협회 회원사","가상조합 조합원","가상공제회 가입자","가상학회 정회원",
         "가상연합 소속기관","가상센터 이용자","가상재단 수혜자","가상공단 계약업체",
         "가상진흥원 참여기업","가상원 위탁기관","가상단체 등록회원","가상협의회 참여자",
         "가상기금 출연자","가상기관 위탁사업자"]
data = {f"피규제집단명[{i}]": n for i, n in enumerate(names)}
io.open(sys.argv[0] if False else 1, 'w', encoding='utf-8').write(
    json.dumps(data, ensure_ascii=False, indent=1))
PY

# 3) 먼저 --dry-run 으로 확인 (파일을 만들지 않는다)
rhwp edit fill-fields reg-work.hwp --data @groups.json -o reg-filled.hwp --dry-run --json \
  | jq -c '{dryRun, filledCount, notFound, ambiguous}'

# 4) 실제로 채우고, 산출물을 다시 읽어 대조한다
rhwp edit fill-fields reg-work.hwp --data @groups.json -o reg-filled.hwp --json \
  | jq -c '{filledCount, notFound, ambiguous}'
rhwp fields reg-filled.hwp --json \
  | jq -r '[.fields[]|select(.name=="피규제집단명")]|to_entries[]|"[\(.key)] \(.value.value)"'
```

> **주의 — `--data @파일` 은 UTF-8 이어야 한다.** 한국어 Windows 에서 스크립트가 CP949 로
> 파일을 쓰면 `stream did not contain valid UTF-8` 로 exit 1 이 난다(실제로 겪었다).
> 이것은 도구의 결함이 아니라 인코딩 문제이며, 오류 메시지가 원인을 정확히 지목한다.

## 성능 실측 (157쪽·1,070필드)

| 작업 | 소요 |
|---|---|
| `fields --json` (1,070개 열거) | **292ms** |
| `edit fill-fields` (14개 채우기 + 저장) | **274ms** |

## 남은 한계

- `ir-diff` 로 편집 전후를 검증하려면 [#3471](https://github.com/edwardkim/rhwp/pull/3471)
  (표 셀 재귀)이 필요하다. 이 서식의 필드는 대부분 표 셀 안에 있어, 그 수정 전에는
  `identical:true` 로 보고된다.
- 이름이 **빈 문자열**인 필드가 63개 있다(`fields --json` 에서 `name:""`).
  이들은 이름으로 지목할 수 없으므로 현재는 채울 수 없다 — 위치 기반 지목은 별도 과제다.
