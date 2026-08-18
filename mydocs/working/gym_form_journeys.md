---
kind: working
status: active
canonical: mydocs/working/gym_form_journeys.md
last_verified: 2026-08-18
issue: 5209
pr: 5213
---

# gym form-journeys pack 확장 작업 노트 (PR #5213)

## 한 줄

`feat/gym-form-journeys` 가 FJ01–FJ05 (약 444줄)에서 멈춰 있던 서식 여정을,
기존 연산자·기존 표본·기존 명령만으로 FJ06–FJ56 과 README·가드 시험까지
늘린다. 새 PR 을 열지 않고 같은 브랜치에 얹는다. T07 의
`fields[0]==홍길동` 판정은 복제하지 않는다.

## 배경

이슈 #5209 · PR #5213 초안은 다섯 축만 과제화했다.

- FJ01 이름 지목 채움 (전화번호, field-01)
- FJ02 dry-run 조회 (form-01, myMsg01)
- FJ03 반복 순번 (목차1[2])
- FJ04 본문 치환 후 재검색 (마케팅→여정기획)
- FJ05 채움+sanitize (form-01, 배포전값)

pack 은 여전히 "5 과제" 이고, 다른 이름 칸, 다른 순번, 메모 표본, form-02,
notFound/ambiguous 오라클, 채움+치환, 두 산출, verify 가 비어 있었다.
집계 문서(gym/README, PARK, 다른 프로파일)는 초안이 손대지 않았고
이번에도 손대지 않는다. `gym/profiles/maintainer.json` 은 이미
`form-journeys` 를 정렬해 넣고 있어 그대로 둔다.

## 범위

포함한 것:

- `gym/packs/form-journeys/README.md` — pack 온램프·과제 지도·함정·재현
- `gym/packs/form-journeys/tasks/FJ06.json` … `FJ56.json`
- `gym/packs/form-journeys/reference/FJ06.json` … `FJ56.json`
- `scripts/tests/test_gym_form_journeys_pack.py` — 확장 계약 가드
- `mydocs/working/gym_form_journeys.md` — 이 노트

넣지 않은 것:

- 새 연산자 (`checks.py` 미변경)
- 새 CLI · 새 표본 · 새 pack
- T07 복제 (`fields[0].value == 홍길동`, 제목 "서식 채움")
- `batch fill` · `run` · `set-cell` · `export-tables`
- `gym/README.md` · `gym/PARK.md` · 다른 `profiles/*.json`
- `cargo fmt --all` (JSON·문서만)

## 설계 원칙

1. **라이브 오라클.** `answer_eq` / `len_answer_eq` 는 채점 시점 CLI 재계산.
   fieldCount·notFound·ambiguous·guide·memo 를 과제에 박제하지 않는다.
2. **축을 가른다.** FJ01 의 이름 지목을 작성자·부서·이메일·제목·쌍칸으로
   쪼갠다. FJ03 의 [2] 를 [0][1][3][4]·양끝·다섯 전부로 쪼갠다. FJ02 의
   dry-run 을 다른 표본·notFound·ambiguous·filled[] 로 쪼갠다. FJ04 의
   치환을 다른 문구·채움 후 치환으로 쪼갠다. FJ05 의 sanitize 를 다른
   표본·채움 없는 정리로 쪼갠다.
3. **표본을 흩는다.** field-01 한 파일에 질문을 몰지 않는다. memo·form-01·
   form-02 는 이미 `samples/` 에 있다.
4. **T07 금지.** 회사명에 홍길동을 쓰지 않는다. 채움 과제는 `fields[0].value`
   공란을 같이 검사한다. 조회 과제는 이름·타입만 읽는다.
5. **기존 연산자·기존 명령.** pack requires 밖 명령을 호출하지 않는다.

## 과제 묶음

| 구간 | 축 | 핵심 판정 |
|------|----|-----------|
| FJ01–FJ05 | 개장 코어 | 전화·dry-run·[2]·치환·sanitize |
| FJ06–FJ11 · FJ50 · FJ54 | 이름 지목 | 작성자·부서·이메일·제목·쌍·삼칸 |
| FJ12–FJ17 | 반복 순번 | [0][1][3][4]·양끝·다섯 |
| FJ18–FJ20 | 다른 표본 | form-02 채움·form-01 재검색·sanitize |
| FJ21–FJ24 · FJ33–FJ34 · FJ55–FJ56 | dry-run 봉투 | filledCount·notFound·ambiguous·occurrence |
| FJ25–FJ32 · FJ35 · FJ43–FJ44 · FJ51–FJ53 | 조회 | fieldCount·name·memo·guide·search |
| FJ36–FJ39 | 치환 | 운영전략·여정연구·채움 후 치환 |
| FJ40 | verify | identical |
| FJ41–FJ42 | sanitize 만 | 재실행 0 |
| FJ45 | 두 산출 | files_differ |
| FJ46–FJ49 | 메모 표본 | 작성자·부서·제목·이메일 |

## 개별 과제 (FJ06+)

| ID | 티어 | 제목 | 입력 | 판정 |
|----|------|------|------|------|
| FJ06 | 2 | 이름 지목 채움 (작성자) | samples/field-01.hwp | fields[1]==박서연기획, 회사명·부서 공란 |
| FJ07 | 2 | 이름 지목 채움 (부서명) | samples/field-01.hwp | fields[2]==여정운영팀, 회사명 공란 |
| FJ08 | 2 | 이름 지목 채움 (이메일) | samples/field-01.hwp | fields[4]==journey@example.go.kr, 회사명 공란 |
| FJ09 | 2 | 이름 지목 채움 (제목) | samples/field-01.hwp | fields[10]==서식여정점검표, 회사명 공란 |
| FJ10 | 3 | 두 칸 동시 지목 (작성자+부서) | samples/field-01.hwp | 작성자+부서 채움, 회사명·이메일 공란 |
| FJ11 | 3 | 두 칸 동시 지목 (이메일+전화) | samples/field-01.hwp | 이메일+전화(031-900-4411), 회사명 공란 |
| FJ12 | 3 | 반복 순번 지목 (목차1[0] 첫 칸) | samples/field-01.hwp | 목차1[0]==첫번째목차여정, 회사명 공란 |
| FJ13 | 3 | 반복 순번 지목 (목차1[1] 둘째 칸) | samples/field-01.hwp | 목차1[1]==두번째목차여정, 회사명 공란 |
| FJ14 | 3 | 반복 순번 지목 (목차1[3] 넷째 칸) | samples/field-01.hwp | 목차1[3]==네번째목차여정, 회사명 공란 |
| FJ15 | 3 | 반복 순번 지목 (목차1[4] 다섯째 칸) | samples/field-01.hwp | 목차1[4]==다섯째목차여정, 회사명 공란 |
| FJ16 | 4 | 반복 양끝만 채움 (목차1[0]·[4]) | samples/field-01.hwp | [0]·[4]만 채움, 가운데·회사명 공란 |
| FJ17 | 4 | 반복 다섯 칸 전부 순번 지목 | samples/field-01.hwp | 목차1 다섯 칸 서로 다른 값, 회사명 공란 |
| FJ18 | 2 | 다른 표본 단칸 채움 (form-02) | samples/form-02.hwp | form-02 myMsg01==이준호 귀하 |
| FJ19 | 3 | form-01 채움 후 본문 재검색 | samples/form-01.hwp | form-01 제출검토값 재독+재검색 |
| FJ20 | 3 | form-02 채움 뒤 sanitize 제출본 | samples/form-02.hwp | form-02 배포후확인 + 재sanitize 0 |
| FJ21 | 2 | field-01 작성자 dry-run 오라클 | samples/field-01.hwp | field-01 작성자 dryRun·filledCount 라이브 |
| FJ22 | 2 | form-02 dry-run 오라클 | samples/form-02.hwp | form-02 dryRun·filledCount 라이브 |
| FJ23 | 3 | 없는 필드 notFound 오라클 | samples/field-01.hwp | notFound[0] 라이브 (회사명칭) |
| FJ24 | 3 | 순번 없는 목차1 ambiguous 오라클 | samples/field-01.hwp | ambiguous[0].total 라이브 |
| FJ25 | 1 | field-01 fieldCount 오라클 | samples/field-01.hwp | samples/field-01.hwp fieldCount 라이브 |
| FJ26 | 1 | form-01 fieldCount 오라클 | samples/form-01.hwp | samples/form-01.hwp fieldCount 라이브 |
| FJ27 | 1 | form-02 fieldCount 오라클 | samples/form-02.hwp | samples/form-02.hwp fieldCount 라이브 |
| FJ28 | 1 | field-01-memo fieldCount 오라클 | samples/field-01-memo.hwp | samples/field-01-memo.hwp fieldCount 라이브 |
| FJ29 | 1 | 첫 칸 이름은 회사명 (읽기만) | samples/field-01.hwp | fields[0].name 라이브 (채움 없음) |
| FJ30 | 1 | textSecurity.status 오라클 | samples/field-01.hwp | textSecurity.status 라이브 |
| FJ31 | 2 | field-01-memo 첫 칸 memo 오라클 | samples/field-01-memo.hwp | field-01-memo fields[0].memo 라이브 |
| FJ32 | 2 | field-01 작성자 guide 오라클 | samples/field-01.hwp | fields[1].guide 라이브 |
| FJ33 | 3 | 세 칸 dry-run filledCount 오라클 | samples/field-01.hwp | 3키 dry-run filledCount 라이브 |
| FJ34 | 3 | 오타 두 키 notFound 길이 오라클 | samples/field-01.hwp | notFound 길이 라이브 |
| FJ35 | 1 | form-01 단칸 이름 오라클 | samples/form-01.hwp | form-01 fields[0].name 라이브 |
| FJ36 | 2 | 본문 치환 후 재검색 (운영전략) | samples/field-01.hwp | 마케팅→운영전략, 옛0 새≥1 |
| FJ37 | 4 | 이메일 채움 뒤 본문 치환 | samples/field-01.hwp | 이메일 채움 + 마케팅→여정연구 |
| FJ38 | 3 | 채운 값을 다시 치환 (form-01) | samples/form-01.hwp | 치환전값→치환후값 재검색 |
| FJ39 | 3 | 제목 채움 뒤 제목 문구 치환 | samples/field-01.hwp | 초안여정제목→확정여정제목, 회사명 공란 |
| FJ40 | 3 | 부서명 채움 --verify 재파싱 | samples/field-01.hwp | 부서명 여정검증팀 + verify.identical |
| FJ41 | 3 | form-01 sanitize 만 (채움 없음) | samples/form-01.hwp | form-01 sanitize만, 재실행 0 |
| FJ42 | 3 | form-02 sanitize 만 (채움 없음) | samples/form-02.hwp | form-02 sanitize만, 재실행 0 |
| FJ43 | 1 | 원본 마케팅 재검색 오라클 | samples/field-01.hwp | 원본 마케팅 matchCount 라이브 |
| FJ44 | 1 | 전화번호 칸 이름 오라클 | samples/field-01.hwp | fields[3].name 라이브 |
| FJ45 | 4 | 작성자본과 이메일본을 따로 제출 | samples/field-01.hwp | 두 파일 상이, 각각 한 칸만, 회사명 공란 |
| FJ46 | 2 | 메모 표본 작성자 채움 | samples/field-01-memo.hwp | memo 표본 작성자, 회사명 공란 |
| FJ47 | 2 | 메모 표본 부서명 채움 | samples/field-01-memo.hwp | memo 표본 부서, 회사명 공란 |
| FJ48 | 2 | 메모 표본 제목 채움 | samples/field-01-memo.hwp | memo 표본 제목, 회사명 공란 |
| FJ49 | 2 | 메모 표본 이메일 채움 | samples/field-01-memo.hwp | memo 표본 이메일, 회사명 공란 |
| FJ50 | 4 | 작성자·이메일·제목 삼칸, 회사명·목차 공란 | samples/field-01.hwp | 삼칸 채움, 회사명·목차1[0] 공란 |
| FJ51 | 1 | 전화번호 editableInForm 오라클 | samples/field-01.hwp | fields[3].editableInForm 라이브 |
| FJ52 | 1 | 첫 칸 fieldType 오라클 | samples/field-01.hwp | fields[0].fieldType 라이브 |
| FJ53 | 2 | 부서명 location.paragraph 오라클 | samples/field-01.hwp | fields[2].location.paragraph 라이브 |
| FJ54 | 3 | 이메일 채움 뒤 칸 이름 불변 | samples/field-01.hwp | 이메일 값+이름 불변, 회사명 공란 |
| FJ55 | 2 | dry-run filled[0].name 오라클 | samples/field-01.hwp | dry-run filled[0].name 라이브 |
| FJ56 | 3 | dry-run 목차1[2] occurrence 오라클 | samples/field-01.hwp | dry-run filled[0].occurrence 라이브 |

## 검증

- `python gym/tools/audit.py` — 전 pack 정합 (짝 기준풀이·ID 고유·스키마)
- `python -m unittest scripts/tests/test_gym_packs.py` — pack 구조 계약
- `python -m unittest scripts/tests/test_gym_form_journeys_pack.py` — 이 확장 가드
- gym JSON·문서만 변경. `cargo fmt --all` 생략

## 위험

- field-01 의 목록 순서(회사명=0 … 제목=10, 목차1 은 5부터)에 기대한다.
  순서가 바뀌면 라이브 채점이 깨진다. 근거는 explain 실측과
  `edit_fill_fields_contract` 주석이다.
- 로컬 `rhwp` 없이 기준 풀이를 라이브 채점하지 못한 환경이 있다.
  스키마·audit·가드만 통과한 상태로 올린다.
- form-01/02 의 첫 sanitize `removedCount` 는 메타데이터 존재에 기대한다.
  재실행 0 게이트(FJ05·FJ20·FJ41·FJ42)가 그 실측을 고정한다.

## 하지 않은 것 (재확인)

maintainer.json 은 이미 form-journeys 를 포함한다. 정렬을 깨지 않기 위해
손대지 않았다. PARK/README, 다른 프로파일, CLI, checks.py 도 그대로다.
