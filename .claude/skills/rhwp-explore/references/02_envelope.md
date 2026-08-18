# 02 — explore 봉투

성공 시 stdout 은 순수 JSON 한 객체다. 실패 시 stdout 은 비고 이유는
stderr. `schemaVersion` 은 `"1.0"`.

## 최상위 키

| 키 | 형 | 의미 |
| --- | --- | --- |
| schemaVersion | string | 항상 1.0 |
| source | string | 입력 경로 그대로 |
| format | string | HWP5 / HWPX / HWP3 / HML / DRM / 빈 파일 / 알 수 없음 |
| pageCount | number | 조판 쪽수 |
| encrypted | boolean | header.encrypted |
| affordanceCount | number | menu 길이. triage-overview 포함 |
| menu | array | 우선순위 내림차순 항목 |
| note | string | 정직성 고지 (고정 문장) |

출처 표지: `untrustedContent` 는 **false**. `why` 는 엔진 개수·형식
레이블이라 문서 원문을 싣지 않는다. `untrustedFields` 는 비어 있다.

## menu[] 항목

| 키 | 형 | 의미 |
| --- | --- | --- |
| affordance | string | 고정 어휘 8개 중 하나 |
| why | string | 이 문서에서 켠 개수 근거 |
| command | string | 다음 rhwp 명령. 경로 자리는 <file> |
| skill | string | .claude/skills 이름 |
| confidence | string | high / medium / low |

에이전트는 `command` 를 실행하고 `skill` 로 인계한다. `why` 를 도구
인자로 넣지 않는다.

## 고정 note

```
정직한 휴리스틱 안내다 — 이 문서에 적용 가능한 rhwp 행동을 개수 근거와 함께 제안할 뿐, 완전성을 보장하지 않는다. 각 항목은 '해 볼 수 있는' 다음 명령이며 증거(why)는 엔진이 센 값이다. explain(문서가 무엇인지)·capabilities(도구 일반)와 달리 explore 는 이 문서로 무엇을 할 수 있는지를 라우팅한다.
```

이 문장을 줄이거나 번역해 계약을 바꾸지 않는다. 사람에게 보여 주는
고지다.

## 없는 키

셀 텍스트, 누름틀 값, 주입 원문, 숨은 글자 원문은 이 봉투에 없다.
그것들은 각 조회 명령의 봉투다. explore 가 그것들을 삼키면
`untrustedContent:false` 계약이 깨진다.
