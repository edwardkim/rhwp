---
kind: guide
status: active
canonical: gym/packs/form-journeys/README.md
last_verified: 2026-08-18
---

# form-journeys — 서식 여정 (이름·순번·dry-run·재독)

## 왜 이 pack 인가

에이전트가 실제 서식을 채울 때 빠지는 길은 짧다. 첫 칸에 이름을 넣는 시연
(core-cli T07, `fields[0].value == 홍길동`)은 그 길이 아니다. 이 pack 은
다른 표본·다른 좌표·다른 축만 둔다.

1. 이름 있는 칸을 지목한다 (`전화번호`·`작성자`·`부서명`·`이메일`·`제목`)
2. 같은 이름 다섯 칸은 `이름[N]` 순번으로 고른다
3. `--dry-run` 은 파일을 만들지 않고 봉투만 남긴다
4. 본문을 바꾼 뒤에는 `search` 로 옛 문구 0·새 문구 ≥1 을 닫는다
5. 제출본은 `sanitize` 한 뒤 한 번 더 돌려 `removedCount == 0` 을 본다

새 CLI 는 없다. `fields` · `edit` · `search` 만 쓴다. 기존 연산자만 고른다.
T07 의 `fields[0]==홍길동` 판정은 복제하지 않는다.

권위 출처: 스킬 `rhwp-form-fill`, `mydocs/manual/form_filling_guide.md`,
레시피 01·05, `edit_fill_fields_contract`.

## 이 확장이 지키는 규칙

1. **기존 명령만.** `pack.json` requires 는 `edit` · `fields` · `search`
   뿐이다. `batch` · `run` · `gate` · `export-tables` 를 부르지 않는다.
2. **기존 연산자만.** `value_eq` · `value_ge` · `value_in` · `answer_eq` ·
   `len_answer_eq` · `differs_from_input` · `file_exists` · `files_differ`.
   전역 훑기(`deep_contains` · `not_contains`)는 쓰지 않는다.
3. **기존 표본만.** `field-01` · `field-01-memo` · `form-01` · `form-02`.
   `samples/` 밖 파일을 만들지 않는다.
4. **라이브 오라클.** fieldCount·notFound·ambiguous·guide·memo 같은 숫자는
   과제에 박제하지 않는다. 채점기가 같은 명령을 다시 돌린다.
5. **T07 을 복제하지 않는다.** 첫 필드에 홍길동을 쓰지 않는다. 채움 과제는
   회사명을 비우고, 조회 과제는 이름·타입만 읽는다.
6. **원본을 덮지 않는다.** 산출은 제출 폴더의 `-o` 뿐이다.
7. **새 CLI 없음.** 연산자 등록부도 키우지 않는다.

## 명령 표면 (pack.json requires)

| 명령 | 이 pack 에서 하는 일 | 읽는 봉투 |
|------|----------------------|-----------|
| `fields` | 누름틀 대장·재독 | `fieldCount` · `fields[].name/value/memo/guide` · `textSecurity` |
| `edit fill-fields` | 단건 채움·dry-run·verify | `dryRun` · `filledCount` · `filled[]` · `notFound` · `ambiguous` · `verify` |
| `edit replace-text` | 본문 치환 | 산출물 `search.matchCount` 로 닫음 |
| `edit sanitize` | 작성자·미리보기 제거 | `removedCount` |
| `search` | 옛/새 문구 재검색 | `matchCount` |

`batch fill` 메일머지는 이 pack 의 일이 아니다. 단건 여정만 둔다.

## 함정 (실측, 과제에 녹여 둔 것)

- **순번 없는 반복 이름**은 첫 칸만 채우고 `ambiguous` 가 남는다 (FJ24·FJ03).
- **오타 키**는 `notFound` 로 보고되고 조용히 무시되지 않는다 (FJ23·FJ34).
- **`--dry-run` 은 파일을 만들지 않는다.** 부재를 통과로 치지 마라 (FJ02·FJ21).
- **보고만 믿지 마라.** 산출물은 `fields` / `search` 로 다시 읽는다.
- **sanitize 한 번 더** 돌려 `removedCount==0` 이어야 제출본이다 (FJ05·FJ20).
- **T07 금지.** `fields[0].value == 홍길동` 을 맞추려 하지 마라. 채움 과제는
  회사명 공란을 함께 검사한다.
- **치환 0건은 산출 파일을 만들지 않는다.** 없는 문구를 바꾸면 제출이 없다.

## 과제 지도

난도 1=입문 · 2=초급 · 3=중급 · 4=고급. 보스(5) 사다리 완주는 XC 의 일이다.

### FJ01–FJ05 — 개장 코어 (초안)

| ID | 티어 | 질문 | 표본 | 판정 |
|----|------|------|------|------|
| FJ01 | 3 | 전화번호만 | field-01 | fields[3]==02-720-8899, 회사명 공란 |
| FJ02 | 2 | dry-run 조회 | form-01 | dryRun·filledCount 라이브 |
| FJ03 | 3 | 목차1[2] | field-01 | fields[7]==세번째목차여정, [0] 공란 |
| FJ04 | 2 | 마케팅 치환 | field-01 | 마케팅 0, 여정기획 ≥1 |
| FJ05 | 3 | 채움+sanitize | form-01 | 배포전값 재검색, 재sanitize 0 |

### FJ06+ — 이름 있는 칸 지목 (회사명 공란)

| ID | 티어 | 질문 | 표본 | 판정 |
|----|------|------|------|------|
| FJ06 | 2 | 이름 지목 채움 (작성자) | field-01.hwp | fields[1]==박서연기획, 회사명·부서 공란 |
| FJ07 | 2 | 이름 지목 채움 (부서명) | field-01.hwp | fields[2]==여정운영팀, 회사명 공란 |
| FJ08 | 2 | 이름 지목 채움 (이메일) | field-01.hwp | fields[4]==journey@example.go.kr, 회사명 공란 |
| FJ09 | 2 | 이름 지목 채움 (제목) | field-01.hwp | fields[10]==서식여정점검표, 회사명 공란 |
| FJ10 | 3 | 두 칸 동시 지목 (작성자+부서) | field-01.hwp | 작성자+부서 채움, 회사명·이메일 공란 |
| FJ11 | 3 | 두 칸 동시 지목 (이메일+전화) | field-01.hwp | 이메일+전화(031-900-4411), 회사명 공란 |
| FJ50 | 4 | 작성자·이메일·제목 삼칸, 회사명·목차 공란 | field-01.hwp | 삼칸 채움, 회사명·목차1[0] 공란 |
| FJ54 | 3 | 이메일 채움 뒤 칸 이름 불변 | field-01.hwp | 이메일 값+이름 불변, 회사명 공란 |

### FJ06+ — 반복 누름틀 순번

| ID | 티어 | 질문 | 표본 | 판정 |
|----|------|------|------|------|
| FJ12 | 3 | 반복 순번 지목 (목차1[0] 첫 칸) | field-01.hwp | 목차1[0]==첫번째목차여정, 회사명 공란 |
| FJ13 | 3 | 반복 순번 지목 (목차1[1] 둘째 칸) | field-01.hwp | 목차1[1]==두번째목차여정, 회사명 공란 |
| FJ14 | 3 | 반복 순번 지목 (목차1[3] 넷째 칸) | field-01.hwp | 목차1[3]==네번째목차여정, 회사명 공란 |
| FJ15 | 3 | 반복 순번 지목 (목차1[4] 다섯째 칸) | field-01.hwp | 목차1[4]==다섯째목차여정, 회사명 공란 |
| FJ16 | 4 | 반복 양끝만 채움 (목차1[0]·[4]) | field-01.hwp | [0]·[4]만 채움, 가운데·회사명 공란 |
| FJ17 | 4 | 반복 다섯 칸 전부 순번 지목 | field-01.hwp | 목차1 다섯 칸 서로 다른 값, 회사명 공란 |

### FJ06+ — form-01 / form-02 단칸

| ID | 티어 | 질문 | 표본 | 판정 |
|----|------|------|------|------|
| FJ18 | 2 | 다른 표본 단칸 채움 (form-02) | form-02.hwp | form-02 myMsg01==이준호 귀하 |
| FJ19 | 3 | form-01 채움 후 본문 재검색 | form-01.hwp | form-01 제출검토값 재독+재검색 |

### FJ06+ — sanitize 제출본

| ID | 티어 | 질문 | 표본 | 판정 |
|----|------|------|------|------|
| FJ20 | 3 | form-02 채움 뒤 sanitize 제출본 | form-02.hwp | form-02 배포후확인 + 재sanitize 0 |
| FJ41 | 3 | form-01 sanitize 만 (채움 없음) | form-01.hwp | form-01 sanitize만, 재실행 0 |
| FJ42 | 3 | form-02 sanitize 만 (채움 없음) | form-02.hwp | form-02 sanitize만, 재실행 0 |

### FJ06+ — dry-run 라이브 오라클

| ID | 티어 | 질문 | 표본 | 판정 |
|----|------|------|------|------|
| FJ21 | 2 | field-01 작성자 dry-run 오라클 | field-01.hwp | field-01 작성자 dryRun·filledCount 라이브 |
| FJ22 | 2 | form-02 dry-run 오라클 | form-02.hwp | form-02 dryRun·filledCount 라이브 |
| FJ33 | 3 | 세 칸 dry-run filledCount 오라클 | field-01.hwp | 3키 dry-run filledCount 라이브 |
| FJ55 | 2 | dry-run filled[0].name 오라클 | field-01.hwp | dry-run filled[0].name 라이브 |
| FJ56 | 3 | dry-run 목차1[2] occurrence 오라클 | field-01.hwp | dry-run filled[0].occurrence 라이브 |

### FJ06+ — notFound 오라클

| ID | 티어 | 질문 | 표본 | 판정 |
|----|------|------|------|------|
| FJ23 | 3 | 없는 필드 notFound 오라클 | field-01.hwp | notFound[0] 라이브 (회사명칭) |
| FJ34 | 3 | 오타 두 키 notFound 길이 오라클 | field-01.hwp | notFound 길이 라이브 |

### FJ06+ — ambiguous 오라클

| ID | 티어 | 질문 | 표본 | 판정 |
|----|------|------|------|------|
| FJ24 | 3 | 순번 없는 목차1 ambiguous 오라클 | field-01.hwp | ambiguous[0].total 라이브 |

### FJ06+ — 읽기 전용 라이브 오라클

| ID | 티어 | 질문 | 표본 | 판정 |
|----|------|------|------|------|
| FJ25 | 1 | field-01 fieldCount 오라클 | field-01.hwp | samples/field-01.hwp fieldCount 라이브 |
| FJ26 | 1 | form-01 fieldCount 오라클 | form-01.hwp | samples/form-01.hwp fieldCount 라이브 |
| FJ27 | 1 | form-02 fieldCount 오라클 | form-02.hwp | samples/form-02.hwp fieldCount 라이브 |
| FJ28 | 1 | field-01-memo fieldCount 오라클 | field-01-memo.hwp | samples/field-01-memo.hwp fieldCount 라이브 |
| FJ29 | 1 | 첫 칸 이름은 회사명 (읽기만) | field-01.hwp | fields[0].name 라이브 (채움 없음) |
| FJ30 | 1 | textSecurity.status 오라클 | field-01.hwp | textSecurity.status 라이브 |
| FJ31 | 2 | field-01-memo 첫 칸 memo 오라클 | field-01-memo.hwp | field-01-memo fields[0].memo 라이브 |
| FJ32 | 2 | field-01 작성자 guide 오라클 | field-01.hwp | fields[1].guide 라이브 |
| FJ35 | 1 | form-01 단칸 이름 오라클 | form-01.hwp | form-01 fields[0].name 라이브 |
| FJ43 | 1 | 원본 마케팅 재검색 오라클 | field-01.hwp | 원본 마케팅 matchCount 라이브 |
| FJ44 | 1 | 전화번호 칸 이름 오라클 | field-01.hwp | fields[3].name 라이브 |
| FJ51 | 1 | 전화번호 editableInForm 오라클 | field-01.hwp | fields[3].editableInForm 라이브 |
| FJ52 | 1 | 첫 칸 fieldType 오라클 | field-01.hwp | fields[0].fieldType 라이브 |
| FJ53 | 2 | 부서명 location.paragraph 오라클 | field-01.hwp | fields[2].location.paragraph 라이브 |

### FJ06+ — 본문 치환 후 재검색

| ID | 티어 | 질문 | 표본 | 판정 |
|----|------|------|------|------|
| FJ36 | 2 | 본문 치환 후 재검색 (운영전략) | field-01.hwp | 마케팅→운영전략, 옛0 새≥1 |

### FJ06+ — 채움 뒤 치환

| ID | 티어 | 질문 | 표본 | 판정 |
|----|------|------|------|------|
| FJ37 | 4 | 이메일 채움 뒤 본문 치환 | field-01.hwp | 이메일 채움 + 마케팅→여정연구 |
| FJ38 | 3 | 채운 값을 다시 치환 (form-01) | form-01.hwp | 치환전값→치환후값 재검색 |
| FJ39 | 3 | 제목 채움 뒤 제목 문구 치환 | field-01.hwp | 초안여정제목→확정여정제목, 회사명 공란 |

### FJ06+ — --verify 재파싱

| ID | 티어 | 질문 | 표본 | 판정 |
|----|------|------|------|------|
| FJ40 | 3 | 부서명 채움 --verify 재파싱 | field-01.hwp | 부서명 여정검증팀 + verify.identical |

### FJ06+ — 산출물 두 개

| ID | 티어 | 질문 | 표본 | 판정 |
|----|------|------|------|------|
| FJ45 | 4 | 작성자본과 이메일본을 따로 제출 | field-01.hwp | 두 파일 상이, 각각 한 칸만, 회사명 공란 |

### FJ06+ — field-01-memo

| ID | 티어 | 질문 | 표본 | 판정 |
|----|------|------|------|------|
| FJ46 | 2 | 메모 표본 작성자 채움 | field-01-memo.hwp | memo 표본 작성자, 회사명 공란 |
| FJ47 | 2 | 메모 표본 부서명 채움 | field-01-memo.hwp | memo 표본 부서, 회사명 공란 |
| FJ48 | 2 | 메모 표본 제목 채움 | field-01-memo.hwp | memo 표본 제목, 회사명 공란 |
| FJ49 | 2 | 메모 표본 이메일 채움 | field-01-memo.hwp | memo 표본 이메일, 회사명 공란 |

## 재현

```text
python gym/tools/audit.py
python -m unittest scripts/tests/test_gym_packs.py scripts/tests/test_gym_form_journeys_pack.py
```

바이너리 없이 스키마·정합·가드가 돈다. 라이브 채점은 로컬 `rhwp` 가 있을 때.

## 이 pack 이 하지 않는 일

- T07 복제 (`fields[0].value == 홍길동`)
- `batch fill` 메일머지 (레시피 05 축)
- `edit set-cell` 표 칸 서식 (table-editing)
- 새 연산자·새 표본·새 CLI
- PARK/README · 다른 프로파일 문구 수정

`gym/profiles/maintainer.json` 은 초안이 이미 `form-journeys` 를 정렬해
넣었다. 그대로 둔다.
