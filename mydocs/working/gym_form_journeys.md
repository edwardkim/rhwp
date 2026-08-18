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

## 2차 확장 FJ57–FJ72

1차(FJ06–FJ56) 위에 같은 규칙으로 16과제를 더한다. 새 표본·새 CLI·
새 연산자는 없다. 기존 표본 네 개만 쓴다. T07 은 복제하지 않는다.
`audit` 와 `test_gym_packs` 가 계속 통과해야 한다.

추가 좌표:

- FJ57 (artifact, field-01-memo, t2): 메모 전화 — FJ01 의 메모 판.
- FJ58 (artifact, field-01-memo, t3): 메모 목차1[0] — FJ12 의 메모 판.
- FJ59 (artifact, field-01-memo, t3): 메모 목차1[2] — FJ03 의 메모 판.
- FJ60 (artifact, field-01-memo, t3): 메모 전화+이메일 — FJ11 의 메모 판.
- FJ61 (artifact, form-01, t3): form-01 --verify — 최소 서식 재파싱.
- FJ62 (artifact, form-02, t3): form-02 --verify — 다른 최소 서식.
- FJ63 (artifact, field-01, t3): 제목+부서 — 새 쌍.
- FJ64 (artifact, field-01, t3): 작성자+이메일 — 새 쌍. 홍길동 금지.
- FJ65 (artifact, field-01, t4): 목차1[1]·[3] — 중간 짝. FJ16 의 반대.
- FJ66 (artifact, field-01, t3): 부서+sanitize — 이름 지목 제출 정리.
- FJ67 (artifact, field-01-memo, t3): 메모 제목+sanitize — 메모 제출본.
- FJ68 (answer, field-01, t2): 제목 dry-run name — 예고 칸 이름.
- FJ69 (answer, field-01-memo, t2): 메모 작성자 dry-run — 다른 표본 예고.
- FJ70 (answer, field-01, t1): 전화 guide — FJ32 와 다른 칸.
- FJ71 (answer, field-01, t3): 목차1[4] occurrence — FJ56 의 끝 순번.
- FJ72 (artifact, field-01, t4): 이메일·제목·부서 삼칸 — FJ50 과 다른 삼칸. 전화 공란.

## 과제 전수 한 줄 (FJ01–FJ72)

- FJ01: 전화번호 이름 지목 / field-01 / T07 은 첫 칸 홍길동. 이 과제는 네 번째 칸만.
- FJ02: dry-run 무기록 / form-01 / 파일을 만들지 않고 봉투만 읽는다.
- FJ03: 목차1[2] / field-01 / 순번 없이 채우면 첫 칸만 바뀐다.
- FJ04: 마케팅→여정기획 / field-01 / 누름틀이 아니라 본문 치환+재검색.
- FJ05: 채움+sanitize / form-01 / 값 실재와 재 sanitize 0.
- FJ06: 작성자 지목 / field-01 / 두 번째 칸. 회사명 공란.
- FJ07: 부서명 지목 / field-01 / 세 번째 칸.
- FJ08: 이메일 지목 / field-01 / 이름 키+재검색.
- FJ09: 제목 지목 / field-01 / 제목 칸만.
- FJ10: 작성자+부서 / field-01 / 두 이름 동시.
- FJ11: 이메일+전화 / field-01 / 다른 쌍.
- FJ12: 목차1[0] / field-01 / 첫 반복 칸.
- FJ13: 목차1[1] / field-01 / 둘째 반복 칸.
- FJ14: 목차1[3] / field-01 / 넷째 반복 칸.
- FJ15: 목차1[4] / field-01 / 마지막 반복 칸.
- FJ16: 목차1[0]·[4] / field-01 / 양끝만.
- FJ17: 목차1 다섯 칸 전부 / field-01 / 순번 전수.
- FJ18: 단칸 채움 / form-02 / form-01 이 아닌 최소 서식.
- FJ19: 채움 후 재검색 / form-01 / search 로 닫음.
- FJ20: 채움+sanitize / form-02 / 다른 표본의 제출 정리.
- FJ21: 작성자 dry-run / field-01 / 무기록.
- FJ22: form-02 dry-run / form-02 / 다른 표본 예고.
- FJ23: notFound / field-01 / 오타 키.
- FJ24: ambiguous / field-01 / 순번 없는 목차1.
- FJ25: fieldCount / field-01 / 라이브 개수.
- FJ26: fieldCount / form-01 / 1칸 서식.
- FJ27: fieldCount / form-02 / 다른 1칸.
- FJ28: fieldCount / field-01-memo / 메모 표본 개수.
- FJ29: 첫 칸 이름 / field-01 / 회사명 읽기만.
- FJ30: textSecurity / field-01 / clean 여부.
- FJ31: 첫 칸 memo / field-01-memo / 메모 축.
- FJ32: 작성자 guide / field-01 / 안내문 오라클.
- FJ33: 세 칸 dry-run / field-01 / filledCount.
- FJ34: 오타 두 키 / field-01 / notFound 길이.
- FJ35: 단칸 이름 / form-01 / myMsg01.
- FJ36: 운영전략 치환 / field-01 / 다른 찾기 문자열.
- FJ37: 이메일 후 치환 / field-01 / 두 축 연쇄.
- FJ38: 채운 값 재치환 / form-01 / 값→본문.
- FJ39: 제목 후 제목 치환 / field-01 / 채움+본문.
- FJ40: 부서 --verify / field-01 / 재파싱 identical.
- FJ41: sanitize 만 / form-01 / 채움 없음.
- FJ42: sanitize 만 / form-02 / 다른 표본.
- FJ43: 원본 마케팅 건수 / field-01 / 치환 전 기준선.
- FJ44: 전화 칸 이름 / field-01 / 읽기만.
- FJ45: 작성자본+이메일본 / field-01 / 두 산출.
- FJ46: 메모 작성자 / field-01-memo / 다른 표본.
- FJ47: 메모 부서 / field-01-memo / 다른 칸.
- FJ48: 메모 제목 / field-01-memo / 다른 칸.
- FJ49: 메모 이메일 / field-01-memo / 다른 칸.
- FJ50: 작성자·이메일·제목 / field-01 / 삼칸, 목차 공란.
- FJ51: editableInForm / field-01 / 엔진 값.
- FJ52: fieldType / field-01 / ClickHere.
- FJ53: 부서 paragraph / field-01 / location.
- FJ54: 이메일 후 이름 불변 / field-01 / 칸 이름 유지.
- FJ55: dry-run filled[0].name / field-01 / 예고 이름.
- FJ56: dry-run 목차1[2] occurrence / field-01 / 순번 2.
- FJ57: 메모 전화 / field-01-memo / FJ01 의 메모 판.
- FJ58: 메모 목차1[0] / field-01-memo / FJ12 의 메모 판.
- FJ59: 메모 목차1[2] / field-01-memo / FJ03 의 메모 판.
- FJ60: 메모 전화+이메일 / field-01-memo / FJ11 의 메모 판.
- FJ61: form-01 --verify / form-01 / 최소 서식 재파싱.
- FJ62: form-02 --verify / form-02 / 다른 최소 서식.
- FJ63: 제목+부서 / field-01 / 새 쌍.
- FJ64: 작성자+이메일 / field-01 / 새 쌍. 홍길동 금지.
- FJ65: 목차1[1]·[3] / field-01 / 중간 짝. FJ16 의 반대.
- FJ66: 부서+sanitize / field-01 / 이름 지목 제출 정리.
- FJ67: 메모 제목+sanitize / field-01-memo / 메모 제출본.
- FJ68: 제목 dry-run name / field-01 / 예고 칸 이름.
- FJ69: 메모 작성자 dry-run / field-01-memo / 다른 표본 예고.
- FJ70: 전화 guide / field-01 / FJ32 와 다른 칸.
- FJ71: 목차1[4] occurrence / field-01 / FJ56 의 끝 순번.
- FJ72: 이메일·제목·부서 삼칸 / field-01 / FJ50 과 다른 삼칸. 전화 공란.

## 2차에서 닫은 구멍

- 메모 표본에 전화번호·목차 순번이 없었다 → FJ57–FJ60
- form-01/02 의 --verify 가 없었다 → FJ61·FJ62
- 제목+부서, 작성자+이메일 쌍이 없었다 → FJ63·FJ64
- 목차 중간 짝 [1]·[3] 이 없었다 → FJ65
- field-01 부서 sanitize, 메모 제목 sanitize 가 없었다 → FJ66·FJ67
- 제목 dry-run name, 메모 작성자 dry-run 이 없었다 → FJ68·FJ69
- 전화 guide, 목차1[4] occurrence 가 없었다 → FJ70·FJ71
- FJ50 과 다른 삼칸(이메일·제목·부서, 전화 공란) → FJ72

## 2차 검산

각 새 과제는 기존 과제와 한 축만 다르게 만든다. 같은 점이 두 개
이상이고 다른 점이 없으면 중복이다.

| 새 과제 | 가까운 기존 | 다른 점 |
|---|---|---|
| FJ57 | FJ01 | 표본이 field-01-memo |
| FJ58 | FJ12 | 표본이 field-01-memo |
| FJ59 | FJ03 | 표본이 field-01-memo |
| FJ60 | FJ11 | 표본이 field-01-memo |
| FJ61 | FJ40 | 표본이 form-01 |
| FJ62 | FJ40 | 표본이 form-02 |
| FJ63 | FJ10 | 제목+부서 (작성자 아님) |
| FJ64 | FJ10 | 작성자+이메일 |
| FJ65 | FJ16 | [1]·[3] (양끝 아님) |
| FJ66 | FJ05 | field-01 부서 |
| FJ67 | FJ48 | sanitize 추가 |
| FJ68 | FJ55 | 제목 키 |
| FJ69 | FJ21 | 메모 표본 |
| FJ70 | FJ32 | 전화 guide |
| FJ71 | FJ56 | [4] occurrence |
| FJ72 | FJ50 | 이메일·제목·부서, 전화 공란 |

이 표가 2차의 좌표 증명이다. 기존 표본만 쓰고, 기존 명령을 쓰고,
T07 의 첫 칸 홍길동을 쓰지 않는다.

## 2차 재현

```bash
python -m unittest scripts.tests.test_gym_form_journeys_pack
python gym/tools/audit.py
```

바이너리 왕복은 `build_baseline.py --pack form-journeys` 가 닫는다.
JSON 가드는 rhwp 없이 돈다.

## 2차 지시문 규약

FJ57–FJ72 의 `instructions` 는 200자를 넘긴다. 테스트가 그 하한을
다시 본다. 끝맺음은 1차와 같다.

- 판정은 라이브 재실행 또는 산출물 재독이며, 파일 부재를 통과로 치지 않는다.
- T07 의 fields[0].value==홍길동 판정은 쓰지 마라.
- 새 CLI 는 없다.

answer 과제는 `submit.kind == answer` 이고 `answer.json` 만 낸다.
artifact 과제는 `-o {sub:...}` 로 원본을 보존하고
`differs_from_input` 으로 무편집 복사를 거부한다. field-01 계열
채움은 `fields[0].value == ""` 가드를 가진다.

## 2차 기준 풀이 규약

- fill 과제는 `write_json` + `edit fill-fields -o {sub:파일}`
- sanitize 과제는 그 다음 `edit sanitize`
- verify 과제는 `--verify` 플래그
- answer 과제는 `answer` 블록의 cmd/path 가 과제 checks 와 같다

`test_answer_reference_mirrors_check_cmd_path` 가 이 1:1 을 본다.

## 2차에서 여전히 닫지 않은 구멍

1. HWPX 형식 보존. 허용 표본에 hwpx 가 없어 넣지 않았다.
2. `누름틀-2024.hwp`. 같은 이유.
3. hongbo 12칸. 좌표 미실측.
4. batch fill. BO01 축.
5. inspect/memo 주입. security 축. FJ31 은 memo 문자열만 읽는다.
6. confusable 이름. 합성 픽스처 필요.
7. tier 5. expert-challenges 몫.
8. 바이너리 왕복. 이 환경에 rhwp.exe 가 없다. CI 가 닫는다.
9. leaderboard/baselines. 72과제를 다시 뛰지 않았다.
10. PARK.md. 1차가 손대지 않았고 2차도 손대지 않는다.

## 2차 커밋 계획

스테이징 경로만:

```
git add gym/packs/form-journeys
git add scripts/tests/test_gym_form_journeys_pack.py
git add mydocs/working/gym_form_journeys.md
git commit -m "feat(gym): form-journeys FJ57–FJ72 2차 좌표·지도"
git push origin HEAD:feat/gym-form-journeys
```

`git add -A` 금지. 새 PR 금지. force-push 금지.
다른 gym pack 금지. 새 CLI 금지.

## 점수 어림

1차 FJ01–FJ56 티어 합 + 2차 16과제 티어 합.
FJ57–FJ72: 2+3+3+3+3+3+3+3+4+3+3+2+2+1+3+4 = 45.
만점은 pack 합이며 총점은 편의값이다.

## 왜 2차를 또 넣는가

PR #5213 원격은 이미 FJ06–FJ56 과 가드 시험으로 insertions 를
3000 넘게 올려 두었다. 같은 브랜치에 늦게 합류한 확장은 그 위를
덮어쓰지 않는다. 좌표가 비어 있는 칸(메모 전화·중간 목차·form
verify·다른 삼칸)만 더한다. 1차 파일을 지우지 않는다.

## 샘플 화이트리스트 (재확인)

허용:

- samples/field-01.hwp
- samples/field-01-memo.hwp
- samples/form-01.hwp
- samples/form-02.hwp

금지 (테스트가 extra 로 잡음):

- samples/hwpx/form-01.hwpx
- samples/hwpx/form-02.hwpx
- samples/누름틀-2024.hwp
- samples/ 밖 신규 픽스처

## 명령 화이트리스트 (재확인)

허용: fields, edit fill-fields, edit replace-text, edit sanitize, search.
금지: batch, run, gate, export-tables, inspect, convert, harness.

## 연산자 화이트리스트 (재확인)

허용: value_eq, value_ge, value_in, answer_eq, len_answer_eq, len_ge,
differs_from_input, file_exists, files_differ, json_value_eq, same_hash.
금지: deep_contains, not_contains.

## 2차 과제별 실패 한 줄

- FJ57: 회사명이나 작성자를 채우면 실패. 전화를 비우면 실패.
- FJ58: 목차1[2] 를 채우면 FJ59 좌표. 회사명을 채우면 T07.
- FJ59: 목차1[0] 을 채우면 FJ58 좌표.
- FJ60: 한 칸만 채우면 실패. 회사명을 채우면 T07.
- FJ61: --verify 없이 저장하면 identical 검사를 라이브가 다시 돌린다.
- FJ62: form-01 을 열면 표본이 틀리다.
- FJ63: 작성자를 같이 채우면 공란 가드 실패.
- FJ64: 작성자에 홍길동을 넣으면 T07 문자열. 전화를 채우면 실패.
- FJ65: [0] 이나 [2] 를 채우면 실패. 양끝만 채우면 FJ16.
- FJ66: sanitize 생략 시 재실행 removedCount 가 남을 수 있다.
- FJ67: FJ48 산출물을 그대로 내면 재 sanitize 가 남을 수 있다.
- FJ68: 산출물을 내면 여정이 틀렸다.
- FJ69: field-01 을 열면 표본이 틀리다.
- FJ70: 작성자 guide 를 적으면 FJ32 복제.
- FJ71: [2] 의 occurrence 를 적으면 FJ56 복제.
- FJ72: 작성자를 채우면 FJ50 에 가깝고, 전화를 채우면 공란 가드 실패.

## 리뷰어 FAQ

**Q. 이미 56개인데 왜 72인가?**
A. 메모 표본의 전화·목차, form verify, 중간 순번, 다른 삼칸이
비어 있었다. 숫자 채우기가 아니라 빈 좌표를 채운 것이다.

**Q. README 를 통째로 다시 쓴 이유는?**
A. 1차 README 는 215줄이라 FJ57+ 를 끼워 넣으면 지도가 끊긴다.
전수 절을 한 파일에 모아 FJ01–FJ72 를 같은 형식으로 적었다.
필수 바늘(T07, 복제하지, 라이브 오라클, 기존 연산자, FJ06, FJ56)은
유지했다.

**Q. gym/README 와 PARK 는?**
A. 1차가 손대지 않았고 이번에도 손대지 않는다. maintainer 는
이미 form-journeys 를 포함한다.

**Q. 홍길동 search 0 가드를 왜 안 넣었나?**
A. 1차 테스트는 `value == 홍길동` 을 금지하고, 회사명 공란을
요구한다. 원문에 홍길동이 있는지는 실측하지 않았다. 문자열
금지는 값 필드에만 둔다.

## 작업 순서 (2차)

1. 고립 워크트리를 origin/feat/gym-form-journeys 로 hard reset.
2. 1차 FJ01–FJ56 과 가드 시험을 읽었다.
3. 허용 표본·MIN_TASKS·지시문 200자·회사명 공란 규칙을 지켰다.
4. FJ57–FJ72 와 기준 풀이를 추가했다.
5. README 를 전수 지도로 다시 쓰고 working 에 2차 절을 붙였다.
6. MIN_TASKS 를 72 로 올렸다.
7. unittest 가 통과한 뒤에만 커밋한다.
8. `git push origin HEAD:feat/gym-form-journeys` (non-ff 면 rebase).

## 종료

이 노트는 PR #5213 의 2차 확장 기록이다. 과제 파일이 지시서이고,
이 문서는 왜 그 지시서가 그렇게 생겼는가다. pack 이 다시 자라면
MIN_TASKS 와 README 전수 절과 이 행렬을 같이 고친다.

