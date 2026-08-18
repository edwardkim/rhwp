---
kind: guide
status: active
canonical: gym/tutorial/README.md
last_verified: 2026-08-18
---

# 4. 🎪 CR03 서커스 텐트 — 표가 몇 개인가요?

같은 문서의 표를 센다. 과제 정본은
`gym/packs/casual-rides/tasks/CR03.json`. 이 안내는 JSON 을 고치지
않는다.

돌아가기: [README.md](README.md) · 이전: [03-cr02-ferris.md](03-cr02-ferris.md)

## 과제가 묻는 것

| 항목 | 값 |
|---|---|
| id | `CR03` |
| tier | 1 |
| 제목 | 표가 몇 개인가요? |
| 입력 | `samples/table-001.hwp` |
| 제출 | `answer.json` 의 `tables` |
| 라이브 오라클 | `rhwp export-tables {input} --json` 의 `tableCount` |

나중에 편집존(`table-editing`)에 가면 같은 `export-tables` 가 좌표
조사의 입구가 된다. 입문존에서는 개수만 묻는다.

## 손으로 타기

```bash
rhwp export-tables samples/table-001.hwp --json
```

`tableCount` 를 찾는다. 표 배열 `tables` 의 길이와 같은 수여야 한다.
그 수를 답 키 `tables` 에 넣는다. 키 이름과 봉투 필드 이름이 같아서
헷갈리기 쉽다. **답 파일의 키는 `tables` 이고, 오라클 경로는
`tableCount` 다.**

```bash
mkdir -p gym/submissions/나/casual-rides/CR03
```

`gym/submissions/나/casual-rides/CR03/answer.json`:

```json
{"tables": 0}
```

`0` 은 자리 표시다. 네 `export-tables` 출력을 적어라.

```bash
python gym/score.py --agent 나 --pack casual-rides
```

## 표를 열어 보지 않아도 되나

된다. 입문존은 숫자를 옮기는 곳이다. 표를 CSV 로 뽑거나 셀을 고치는
일은 `editor` 프로파일의 `table-editing` · `table-csv` 가 맡는다.
지금 단계의 성공 조건은 `tableCount` 와 `tables` 가 같은 것이다.

셀 좌표를 벌써 보고 싶다면 [08-editor-path.md](08-editor-path.md) 의
TB01 안내로 건너뛰어도 된다. 키 제한은 없지만, 입문존 네 개를 먼저
닫는 편이 제출 폴더 결이 몸에 붙는다.

## 자주 하는 실수

1. **답 키를 `tableCount` 로 적는다.** 오라클 경로를 답 키로 착각한
   것이다. CR03 의 답 키는 `tables` 다.
2. **표 배열 전체를 붙여 넣는다.** `answer_eq` 는 숫자(또는 정규화된
   숫자 문자열)를 기대한다. 배열을 넣으면 비교가 실패한다.
3. **원본 샘플을 편집한다.** 입문존은 원본을 건드리지 않는다. 제출은
   `gym/submissions/` 아래 `answer.json` 뿐이다.

## 다음

링 던지기 → [05-cr04-ringtoss.md](05-cr04-ringtoss.md). 같은 문서에서
글자 '표' 가 몇 번 나오는지 센다.
