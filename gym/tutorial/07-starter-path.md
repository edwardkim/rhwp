---
kind: guide
status: active
canonical: gym/tutorial/README.md
last_verified: 2026-08-18
---

# 7. starter 길 — casual 바깥 첫 입문

입문존(`family`)을 닫은 다음 자연스러운 걸음이다. `starter` 는
`core-cli` 와 `self-description` 두 pack 을 고른다. 부모님용 회전목마는
아니지만, 여전히 "읽고 숫자 하나"에 가까운 과제가 앞에 있다.

이 페이지는 기존 과제 JSON 을 **읽기만** 한다. 새 과제를 추가하지
않고, 다른 열린 PR 의 pack 과제 파일도 건드리지 않는다.

돌아가기: [README.md](README.md) · 프로파일: [06-profiles.md](06-profiles.md)

## 왜 이 두 pack 인가

`gym/profiles/starter.json` 이 그렇게 묶는다.

- `core-cli` — 조사·추출·편집·검증의 최소 코어. T01 은 쪽수다.
- `self-description` — 도구가 자기를 설명하는 계약. SD01 은 명령 개수다.

둘 다 "문서를 고치기 전에 도구와 입력을 읽는다"는 결이다. 편집은
`editor` 로 미룬다.

```bash
python gym/score.py --agent 나 --profile starter
```

또는 pack 을 하나씩:

```bash
python gym/score.py --agent 나 --pack core-cli
python gym/score.py --agent 나 --pack self-description
```

## 첫 과제 T01 — 문서 신상

정본: `gym/packs/core-cli/tasks/T01.json`

| 항목 | 값 |
|---|---|
| id | `T01` |
| tier | 1 |
| 입력 | `samples/basic/issue2007_nested_cell_pagination_42065.hwp` |
| 답 키 | `pages` |
| 오라클 | `rhwp info {input} --json` 의 `pageCount` |

CR01 과 같은 동작이다. 입력 문서만 다르다. 입문존에서 익힌 결을 다른
샘플에 적용하는 것이 첫 숙제다.

```bash
rhwp info samples/basic/issue2007_nested_cell_pagination_42065.hwp --json
mkdir -p gym/submissions/나/core-cli/T01
```

`gym/submissions/나/core-cli/T01/answer.json`:

```json
{"pages": 0}
```

`0` 은 자리 표시다. 네 `info` 출력을 적어라. 폴더가 `casual-rides` 가
아니라 `core-cli` 인 것을 놓치지 마라.

## 두 번째 맛보기 T02 — 전수 검색

정본: `gym/packs/core-cli/tasks/T02.json`

| 항목 | 값 |
|---|---|
| id | `T02` |
| 입력 | `samples/2022년 국립국어원 업무계획.hwp` |
| 답 키 | `matchCount` |
| 오라클 | `rhwp search {input} 국어 --json` 의 `matches` **길이** |

CR04 와 비슷해 보이지만 연산자가 `answer_eq` 가 아니라
`len_answer_eq` 다. 답 키 `matchCount` 는 `matches` 배열의 길이와
같아야 한다. 연산자 정의는 `gym/core/checks.py` 에 있다. 휴게실이 그
정의를 바꾸지 않는다.

```bash
rhwp search "samples/2022년 국립국어원 업무계획.hwp" 국어 --json
mkdir -p gym/submissions/나/core-cli/T02
```

경로에 공백·한글이 있다. 따옴표를 빼면 셸이 파일을 둘로 나눈다.
Windows 는 [19-windows.md](19-windows.md).

## 자기서술 SD01 — 명령 표면 계수

정본: `gym/packs/self-description/tasks/SD01.json`

| 항목 | 값 |
|---|---|
| id | `SD01` |
| 입력 | `samples/table-001.hwp` (이 과제는 문서보다 도구가 대상) |
| 답 키 | `commands` |
| 오라클 | `rhwp capabilities` 의 `commands` **길이** |

```bash
rhwp capabilities
mkdir -p gym/submissions/나/self-description/SD01
```

`gym/submissions/나/self-description/SD01/answer.json`:

```json
{"commands": 0}
```

`0` 은 자리 표시다. `capabilities` 봉투의 `commands` 배열 길이를
적어라. 바이너리가 새로워지면 수가 늘어날 수 있다. 그래서 골든을
박제하지 않는다.

`self-description` pack 은 `capabilities`, `export-agent-manifest`,
`export-ontology`, `export-plan-schema`, `export-provenance-map` 을
요구한다. 하나라도 없으면 pack 전체가 `unavailable` 이다.

## starter 가 아직 아닌 것

- 본문을 치환하는 일 (`text-editing` TE01) — [08-editor-path.md](08-editor-path.md)
- HWPX 로 내보내는 일 (`serialization` SR01) — [09-publisher-path.md](09-publisher-path.md)
- 사다리 10단 (`automation`) — [10-operator-path.md](10-operator-path.md)
- L5 완주 (`expert-challenges` XC01) — [11-boss-path.md](11-boss-path.md)

`core-cli` 에는 T07 이후 편집·변환·하네스 과제도 있다. 그것들은
starter 묶음 안에 들어 있지만, 난도가 입문 숫자 옮기기보다 높다.
처음부터 T13/T14 로 뛰지 않아도 된다. T01·T02·SD01 이 닫히면
`starter` 길의 입구는 통과한 것이다.

## 다음

문서를 고치고 싶으면 [08-editor-path.md](08-editor-path.md).
배포 전 확인이면 [09-publisher-path.md](09-publisher-path.md).
