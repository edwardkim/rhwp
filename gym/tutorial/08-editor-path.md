---
kind: guide
status: active
canonical: gym/tutorial/README.md
last_verified: 2026-08-18
---

# 8. editor 길 — 문서를 실제로 고친다

`editor` 프로파일은 `core-cli`, `text-editing`, `table-editing`,
`objects-media` 를 고른다. 정본은 `gym/profiles/editor.json`. 이
페이지는 각 pack 의 **첫 과제만** 안내한다. 과제 JSON 을 늘리거나
고치지 않는다.

돌아가기: [README.md](README.md) · 프로파일: [06-profiles.md](06-profiles.md)

## 한 줄로 고르기

```bash
python gym/score.py --agent 나 --profile editor
```

편집 과제는 산출물을 `-o` 로 새로 만든다. **원본 샘플을 덮어쓰지
마라.** 제출은 `gym/submissions/나/<pack>/<과제id>/` 아래다.

## TE01 — 문구 치환 왕복

정본: `gym/packs/text-editing/tasks/TE01.json`

입력 `samples/basic/issue2007_nested_cell_pagination_42065.hwp` 에서
'규제' 를 모두 '점검' 으로 바꾼 `edited.hwp` 를 낸다.

채점 세 칸 (연산자는 이미 있는 것들이다):

1. `value_eq` — 산출물에서 '규제' 의 `matchCount` 가 0
2. `value_ge` — 산출물에서 '점검' 의 `matchCount` 가 1 이상
3. `differs_from_input` — 산출물 바이트가 입력과 다름

세 번째가 없으면 원본을 이름만 바꿔 내는 제출이 통과한다. 그 연산자를
휴게실이 추가하는 것이 아니다. 과제에 이미 있다.

힌트 명령은 `rhwp edit replace-text` 다. 정확한 플래그는 과제
`instructions` 와 `rhwp capabilities` 를 본다. 이 안내가 CLI 표면을
새로 만들지 않는다.

```bash
mkdir -p gym/submissions/나/text-editing/TE01
# 산출물은 이 폴더의 edited.hwp 로 낸다. 원본 경로는 -o 로 피한다.
```

## TB01 — 표 좌표 조사

정본: `gym/packs/table-editing/tasks/TB01.json`

아직 셀을 고치지 않는다. 첫 표의 행·열 수를 읽는다.

```bash
rhwp export-tables samples/basic/issue2007_nested_cell_pagination_42065.hwp --json
mkdir -p gym/submissions/나/table-editing/TB01
```

`answer.json`:

```json
{"rows": 0, "cols": 0}
```

`0` 은 자리 표시다. `tables[0].rows` 와 `tables[0].cols` 를 적어라.
CR03 이 표 **개수**였다면, TB01 은 첫 표의 **모양**이다.

## OM01 — 누름틀 전수 조사

정본: `gym/packs/objects-media/tasks/OM01.json`

```bash
rhwp fields samples/field-01.hwp --json
mkdir -p gym/submissions/나/objects-media/OM01
```

`answer.json` 의 키는 `fields`, 오라클 경로는 `fieldCount` 다.

이 pack 의 뒤 과제들은 필드 채움·개체 렌더로 올라간다. 첫 과제는
조사다. 조사 없이 채우면 좌표가 틀린다.

## 원본 무훼손

편집 길의 공통 규칙이다.

- 입력은 `samples/` 아래 픽스처다. 커밋된 바이트를 바꾸지 마라.
- 산출은 항상 `-o` (또는 동등한 출력 옵션)로 제출 폴더에 만든다.
- `differs_from_input` 이 있는 과제는 무편집 복사를 거절한다.
- `.hwp`/`.hwpx` 산출은 보통 커밋하지 않는다(`.gitignore`).
  재실행하면 다시 만들 수 있어야 한다.

이 규칙은 `gym/README.md` 제출 형식 절과 같다. 휴게실이 새 규칙을
발명하지 않는다.

## editor 다음에

배포·변환·보안은 [09-publisher-path.md](09-publisher-path.md).
표만 CSV 로 왕복하고 싶으면 `table-csv` pack 이 따로 있다. 그 pack 은
`editor` 프로파일에 묶여 있지 않다. `--pack table-csv` 로 고른다.
프로파일에 없는 pack 을 몰래 넣지 않는 것이
[06-profiles.md](06-profiles.md) 의 결이다.
