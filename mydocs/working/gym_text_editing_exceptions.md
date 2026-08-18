---
kind: investigation
status: active
canonical: gym/packs/text-editing/README.md
last_verified: 2026-08-18
---

# text-editing pack 예외·가장자리 노트

이 문서는 본문 편집 과제가 **일부러 박제하지 않는 값**과, 현장에서
반복되는 가장자리 실패를 기록한다. 여정 지도는
[gym/packs/text-editing/README.md](../../gym/packs/text-editing/README.md),
작업 계보는 [gym_text_editing.md](gym_text_editing.md).

값은 표본 개정·검색 구현·문단 분할에 흔들린다. 그래서 과제 JSON에
건수·쪽수·문단 번호를 넣지 않고, 채점 시점에 rhwp가 같은 명령을 다시
돌린다. 아래는 "흔들린다"가 정확히 무엇을 뜻하는지의 목록이다.

## E1. occurrence 는 0 기준이고, 잔여 검사는 느슨하다

`edit replace-text --occurrence N` 은 문서 순서 **0 기준 N번째** 한 건만
바꾼다. 코어 테스트 `replace_occurrence_contract` 가 이 계약을 고정한다.
문서 일부는 "1 기준 k번째"로 읽히지만, 이 pack은 0 기준을 따른다.

잔여 옛 문구는 `value_ge 1` 이다. 이유:

- 전건 치환(잔여 0)만 확실히 거절하면 된다.
- 총 건수를 6으로 박제하면 표본이 바뀌는 순간 과제가 죽는다.
- 두 건을 바꿔도 잔여가 있으면 통과한다. 이게 검사의 한계다.

더 촘촘히 보려면 예정 건수 dry-run(`replacedCount`)과 산출 후 잔여를
함께 보거나, `files_differ` 로 전건 산출과 한 건 산출을 맞대면 된다.
이번 확장은 그 두 산출을 한 과제에 넣지 않았다. 한 과제 한 계약.

occurrence 가 총 건수 이상이면 치환 0건 → 산출 파일이 없다.
`differs_from_input` 전에 이미 기준풀이가 실패한다. TE16(occurrence 2)은
중첩셀 표본의 '규제' 가 3건 이상이라는 전제 위에 있다. TE11 이 잔여를
요구하므로 최소 2건은 실측된 셈이다. 셋째(2)는 그보다 한 단계 더
요구한다.

## E2. replacedCount 0 은 산출이 없다

`edit replace-text` 는 치환 0건이면 출력 파일을 만들지 않는다.
에이전트가 `-o out.hwp` 를 줘도 파일이 없고, 원본을 복사해 제출하면
`differs_from_input` 이 거절한다.

함정:

- 검색어 오타 → 0건 → 무산출
- 이미 바뀐 산출물에 같은 치환을 다시 적용 → 0건
- HWPX/HWP 경로를 바꿔 다른 본문을 연다

dry-run(TE08/TE25/TE26/TE90)은 이 값을 파일 없이 읽는다. 0 이 정답일
수도 있다. 박제하지 마라.

## E3. insert-text 좌표는 자르지 않는다

`--section` / `--para` / `--offset` 은 전부 0 기준이다.

- offset == 문단 길이 → 끝에 붙인다 (허용)
- offset > 문단 길이 → exit 2, 실제 길이를 안내, 원본 불변
- 없는 구역·문단 → exit 2

조용히 클램프하지 않는다. TE13/TE27–TE50 이 (0,0,0) 을 쓰는 이유는
그 좌표가 단순 표본에 존재한다고 가정하기 때문이다. 빈 첫 문단
(field-01)에도 0,0,0 삽입은 성립한다 — 빈 문단의 길이 0 에 offset 0 은
끝에 붙이는 계약과 같다.

표지 문자열은 문서에 없던 고유 토큰(`짐표지TEnn`)이다. 이미 있는
단어를 넣으면 `matchCount == 1` 이 깨진다.

HWPX 입력은 산출도 `.hwpx` 다(TE47). 확장자만 `.hwp` 로 바꾸면
형식 검사가 실패한다.

## E4. search 주소와 edit 주소는 같지만, 쪽 개수와는 다르다

같은 단어 "page" 가 두 가지를 가리킨다.

1. `search` 의 `matches[].page` — **0 기준 쪽 번호**
2. `info` 의 `pageCount` — **개수** (기준이 아님)

`extract-pages --from/--to` 는 **1 기준**이지만 이 pack은 그 명령을
쓰지 않는다. 다른 pack에서 습관을 가져오면 TE48/TE53/TE72 의 첫 쪽
답이 한 칸 밀린다.

`matches[0].paragraph` 와 `insert-text --para` 는 같은 0 기준 문단
번호다. `matches[0].offset` 과 `--offset` 도 같다. 삽입 뒤 되읽기가
성립하는 이유다.

0건이면 `matches[0]` 경로가 없다. 그래서 재검색 과제들은 새 문구
`value_ge 1` 을 먼저 둔다.

## E5. digest.paraCount ≠ explain.paragraphCount

두 필드는 이름이 비슷하고 값이 다를 수 있다.

- `digest` 의 `paraCount` — 발췌에 잡힌 문단
- `explain` 의 `paragraphCount` — 문서 설명의 문단 수

TE05 와 TE06 이 같은 표본에서 둘을 가른다. TE65/TE66/TE75/TE85 도
필드 이름을 힌트에 박아 두었다. 한 값을 다른 과제에 재사용하지 마라.

`export-structure` 의 `nodeCount` 는 또 다른 축이다. TE07 은 중첩셀,
TE69 는 표, TE70 은 문단, TE88 은 hwp3. 같은 필드라도 표본이 바뀌면
다른 계약이다. 이것을 TE07 복제로 보지 않는다. T07 복제는
`fill-fields` 를 베끼는 일이다.

## E6. 치환어가 검색어를 포함하면 잔여 0 이 불가능하다

`표` → `도표` 로 전건 치환한 뒤 `search 표` 는 `도표` 안의 `표` 를
다시 센다. 옛 문구 `value_eq 0` 이 영원히 실패한다.

이 pack이 쓰는 쌍:

| 찾음 | 바꿈 | 겹침 |
|---|---|---|
| 규제 | 점검 / 심사 / 감독 | 없음 |
| 보험료 | 납입금 / 보험금 | 없음 |
| 국어 | 언어 | 없음 |
| 의 | ◎ | 없음 |

`보험료` → `보험금` 은 앞 두 글자가 같지만, 검색 바늘이 세 글자라
`보험금` 안에서 `보험료` 가 나오지 않는다.

## E7. HWP3 · HWPX · 형식 표기

`info.format` 은 추측하지 않는다.

- 일반 편집 가능 HWP5 → 보통 `hwp5`
- HWPX → `hwpx`
- hwp3-sample → 라이브로 읽는다 (TE87)

sanitize 산출물도 같은 형식으로 열려야 한다(TE03/TE77–TE79).
형식 문자열을 과제 JSON 에 `hwp5` 로 박은 곳은 **계약이 형식 자체**인
경우뿐이다. 쪽수·건수는 박제하지 않았다.

## E8. 라이브 오라클이 따라가는 부작용

TE12/TE51 의 무관 문구 `ⅰ` 는 치환이 그 글자를 건드리면 채점이
그 결과를 따른다. 값을 박제하지 않았기 때문이다. 의도된 동작이다.

삽입 표지 `짐표지TEnn` 이 원본에 이미 있으면 1건 검사가 실패한다.
현재 표본에는 없다. 표본을 교체할 때 표지 충돌을 확인하라.

## E9. 이 pack 이 받지 않는 실패

다음을 실패로 위장하지 않는다.

- 바이너리에 `edit` 가 없음 → pack unavailable (0점이 아님)
- `--verify` 차이 exit 3 → 이번 과제들은 `--verify` 를 켜지 않음
- 폰트·조판 픽셀 차이 → 본문 검색·형식만 본다
- 누름틀이 비어 있음 → 이 pack 의 축이 아님 (T07)

## E10. 기준풀이 자리표

reference 는 `{sub:파일}` 을 쓰고 `{file:파일}` 을 쓰지 않는다.
채점기 쪽 검사는 `{file:제출}` 로 제출물을 연다. 기준풀이가 `{file:}`
을 쓰면 채점 워크스페이스가 아니라 엉뚱한 자리를 연다.

`{input}` 은 과제 입력 표본이다. dry-run · 조사 과제는 `{input}` 만
쓴다.
