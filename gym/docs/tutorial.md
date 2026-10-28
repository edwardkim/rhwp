---
kind: guide
status: active
canonical: gym/docs/tutorial.md
last_verified: 2026-09-02
---

# gym 휴게실 · 테마파크 입문 규약

이 문서는 `gym/tutorial/` · `gym/PARK.md` · `gym/INVITE.md` 가
지켜야 하는 **입문 안내 계약**을 고정한다. 작업 기록은
[`mydocs/working/gym_tutorial.md`](../../mydocs/working/archives/gym_tutorial.md).
기계 시험은 `scripts/tests/test_gym_tutorial.py` 다.

채점 논리의 정본은 여기가 아니다. 연산자 등록부는
`gym/core/checks.py`, 채점 절차는 `gym/core/runner.py`, 진입점은
`gym/score.py` 다. 이 규약이 그 세 파일을 고치지 않는다.

## 1. 왜 이 기둥이 필요한가

테마파크 장식(#4664)은 입구·휴게실·초대장만 얇게 남겼다. 처음 온
사람·에이전트는 `PARK.md` 한 장과 `tutorial/README.md` 90줄만 보고
다음 존을 추측해야 했다.

- 프로파일 일곱 이름이 문서마다 다르게 불리기 쉽다 (`casual`,
  `Family`, `beginner`).
- 입문존 네 놀이기구의 답 키(`pages`/`paragraphs`/`tables`/`hits`)가
  오라클 경로와 섞인다.
- Windows 방문자가 bash 줄을 붙여 JSON 에 BOM 을 넣는다.
- 안내를 고치려다 `checks.py` 나 다른 PR 의 pack JSON 을 건드린다.

그래서 휴게실을 동선 문서로 늘리고, 프로파일 이름·상대 링크·채점
불가침을 시험이 잠근다. 새 pack 과 새 연산자는 이 기둥의 산출이
아니다.

## 2. 문서 지도

| 경로 | 역할 |
|---|---|
| `gym/tutorial/README.md` | 5분 첫 방문 + 휴게실 색인 |
| `gym/tutorial/01-admission.md` | 입장 봉투 |
| `gym/tutorial/02-cr01-carousel.md` | CR01 |
| `gym/tutorial/03-cr02-ferris.md` | CR02 |
| `gym/tutorial/04-cr03-circus.md` | CR03 |
| `gym/tutorial/05-cr04-ringtoss.md` | CR04 |
| `gym/tutorial/06-profiles.md` | 일곱 프로파일 |
| `gym/tutorial/07-starter-path.md` | casual 바깥 첫 입문 |
| `gym/tutorial/08-editor-path.md` | editor 첫 과제 |
| `gym/tutorial/09-publisher-path.md` | publisher 첫 과제 |
| `gym/tutorial/10-operator-path.md` | operator 첫 과제 |
| `gym/tutorial/11-boss-path.md` | boss / XC01 |
| `gym/tutorial/12-leaderboard.md` | attest / verify / render |
| `gym/tutorial/13-invite.md` | 초대장 방문 안내 |
| `gym/tutorial/14-submissions.md` | 제출 폴더 |
| `gym/tutorial/15-scoring-honesty.md` | 채점 불가침 |
| `gym/tutorial/16-unavailable.md` | 부재 ≠ 0점 |
| `gym/tutorial/17-faq.md` | FAQ |
| `gym/tutorial/18-troubleshooting.md` | 막힘 |
| `gym/tutorial/19-windows.md` | PowerShell 번역 |
| `gym/tutorial/20-checklist.md` | 첫날 한 장 |
| `gym/docs/tutorial.md` | 이 규약 |
| `mydocs/working/gym_tutorial.md` | 작업 기록 |
| `gym/PARK.md` | 테마파크 한 장 지도 |
| `gym/INVITE.md` | 초대장 정본 |

상대 링크는 파일이 실재해야 한다. 외부 URL 과 문서 내 앵커는
`scripts/check_markdown_links.py` 와 같은 범위로, 이 규약의
필수 검사는 **저장소 내부 상대 경로**다.

## 3. 프로파일 이름 계약

`gym/profiles/*.json` 의 `id` 가 유일 정본이다. 안내 문서는 아래
일곱 철자를 그대로 쓴다.

| id | packs (정본 파일 그대로) |
|---|---|
| `family` | `casual-rides` |
| `starter` | `core-cli`, `self-description` |
| `editor` | `core-cli`, `text-editing`, `table-editing`, `objects-media` |
| `publisher` | `serialization`, `layout-rendering`, `security` |
| `operator` | `corpus-diagnostics`, `automation` |
| `boss` | `expert-challenges` |
| `maintainer` | 저장소의 모든 pack id |

금지 별명: `Family`, `FAMILY`, `casual`, `beginner`, `expert`,
`guest`, `kiddie`, `admin`. 본문에 이 별명을 **프로파일 id 처럼**
쓰면 시험이 실패한다. 설명 문장에서 "입문자"처럼 쓰는 것은 이름이
아니다.

`--profile family` · `--profile starter` · `--profile editor` ·
`--profile publisher` · `--profile operator` · `--profile boss` ·
`--profile maintainer` 가 휴게실 허브(`tutorial/README.md`)와
프로파일 문서에 나타나야 한다.

## 4. 입문존 네 놀이기구 계약

과제 JSON 을 이 규약이 고치지 않는다. 안내가 옮겨 적는 값은 아래와
같아야 한다. 입력이 네 과제 모두 `samples/table-001.hwp` 다.

| id | 명령 | 오라클 경로 | 답 키 |
|---|---|---|---|
| CR01 | `rhwp info samples/table-001.hwp --json` | `pageCount` | `pages` |
| CR02 | `rhwp explain samples/table-001.hwp --json` | `paragraphCount` | `paragraphs` |
| CR03 | `rhwp export-tables samples/table-001.hwp --json` | `tableCount` | `tables` |
| CR04 | `rhwp search samples/table-001.hwp --json -- 표` | `matchCount` | `hits` |

`casual-rides` 의 `requires.commands` 는 `info`, `explain`,
`export-tables`, `search` 다. 하나라도 없으면 pack 은
`unavailable` 이다.

## 5. 채점 불가침

이 기둥이 **금지하는** 변경:

1. `gym/core/checks.py` 의 `REGISTRY` 키를 더하거나 빼기
2. `GLOBAL_SCAN_OPS` 집합 변경
3. `op_*` 함수의 비교 의미 변경
4. 다른 열린 PR 이 만지는 `gym/packs/*/tasks/*.json` 편집
5. `gym/core/runner.py` 의 `verdict` 계산식 변경
6. `score_pack` 의 unavailable 규칙을 0점으로 접기

`REGISTRY` 스냅샷 (현재, 이 규약이 잠그는 집합):

```
same_hash
differs_from_input
file_exists
files_differ
xml_root_eq
json_value_eq
csv_cell_eq
utf8_bom
json_len_eq
csv_row_count_eq
ndjson_count_eq
ndjson_field_eq
json_keys_contain
text_line_eq
json_type_eq
json_len_ge
json_array_item_eq
csv_col_count_eq
csv_header_eq
csv_row_eq
ndjson_keys_contain
ndjson_len_eq
text_line_count_eq
text_line_contains
answer_eq
len_answer_eq
len_ge
value_eq
value_ge
value_in
deep_contains
not_contains
cell_text_eq
text_file_envelope_eq
```

`GLOBAL_SCAN_OPS = {deep_contains, not_contains}`.

안내 문서가 연산자를 **이름만** 인용하는 것은 허용한다. 구현을
복사해 넣거나, 열여덟 번째 연산자를 제안하며 등록부를 고치는 것은
이 기둥 밖이다.

## 6. 입장 봉투

`admission.json` 의 키와 의미는 `gym/core/runner.py` 가 정본이며,
`gym/score.py` 는 하위 호환 진입점이다.

- `kind` = `gymAdmission`
- `verdict` = `packsScored >= 1` 이면 `allow`, 아니면 `deny`
- 만점은 조건이 아니다
- `runner` 신원이 붙는다

휴게실이 이 계산을 재구현하지 않는다. 읽기만 한다.

## 7. 링크 계약

다음 허브는 서로 가리켜야 한다.

- `gym/PARK.md` → `tutorial/README.md`, `INVITE.md`, `docs/tutorial.md`
- `gym/INVITE.md` → `tutorial/README.md`, `PARK.md`
- `gym/tutorial/README.md` → `../PARK.md`, `../INVITE.md`,
  `../docs/tutorial.md`, 01~20 페이지
- `gym/docs/tutorial.md` → `mydocs/working/gym_tutorial.md`

상대 링크 대상 파일이 없으면 시험이 실패한다. 깨진 링크를 통과시키는
허브는 입문 기둥이 아니다.

## 8. 정직 문장

아래 문장이 PARK · 휴게실 정직 문서 · 이 규약에 남아야 한다.
철자 하나가 달라도 안 된다기보다, **이 네 주장이 모두 나타나야**
한다.

1. 점수는 pack 별로 보존되고 총점은 편의값이다
2. 기대값은 채점 시점에 rhwp 로 재계산한다 (라이브 오라클)
3. 부재는 0점이 아니라 `unavailable`
4. 테마는 장식이고 판정 논리와 무관하다

## 9. casual 바깥 입문

이슈 #5263 은 "필요하면 casual 외 입문 문서와 시험"을 허용한다.
허용하는 것은 **문서와 시험**이다. 새 과제 JSON 이 아니다.

`starter` 길이 가리키는 기존 과제:

| pack | id | 답 키 | 오라클 |
|---|---|---|---|
| core-cli | T01 | `pages` | `info` / `pageCount` |
| core-cli | T02 | `matchCount` | `search` / `matches` 길이 |
| self-description | SD01 | `commands` | `capabilities` / `commands` 길이 |

editor / publisher / operator / boss 는 각 pack 의 **이미 있는
1번 과제**만 안내한다. 다른 열린 PR 이 CR05+ 나 EX03+ 를 늘리고
있으면 그 파일은 그 PR 의 것이다.

## 10. Windows 번역

`gym/tutorial/19-windows.md` 는 bash 와 같은 동작을 PowerShell 로
옮긴다. 필수 요소:

- `New-Item -ItemType Directory -Force` (mkdir -p 대응)
- UTF-8 **without BOM** 으로 `answer.json` 쓰기
- `--profile family` 철자 그대로
- 답 키 네 개 그대로

콘솔 코드 페이지가 한글을 `??` 로 바꾸는 문제는 제출 파일 바이트로
우회한다고 적어야 한다. 채점기를 CP949 로 바꾸라고 하면 안 된다.

## 11. 시험이 잠그는 것

`scripts/tests/test_gym_tutorial.py`:

- 필수 문서가 존재한다
- 일곱 프로파일 파일·id·packs 가 문서와 같다
- 상대 링크가 저장소 안에서 풀린다
- CR01~CR04 키와 명령이 휴게실에 있다
- `REGISTRY` 키가 위 스냅샷과 같다
- 안내 문서가 `gym/core/checks.py` 를 "고친다/추가한다"고 말하지
  않는다
- PARK · INVITE · 휴게실이 서로를 가리킨다
- Gym Benchmark Validation이 이 시험을 호출한다

음성 회귀: 임시로 링크를 지운 텍스트는 검사 함수가 문제를 내야
한다. 통과만 보면 가드가 썩는다.

## 12. 하지 않는 것

- 새 CLI 플래그
- 새 pack, 새 과제 JSON
- 새 채점 연산자
- `gym/README.md` 만점 표의 숫자를 이 기둥이 갱신하는 일 (다른
  pack 확장 PR 과 싸운다)
- `cargo fmt --all`

## 13. 재현

```bash
python -m unittest scripts.tests.test_gym_tutorial
python gym/tools/audit.py
```

`audit.py` 는 pack 정합이다. 이 기둥이 pack JSON 을 안 만지면
devel 과 같은 통과여야 한다. 실패하면 작업 트리 오염을 먼저 의심한다.
