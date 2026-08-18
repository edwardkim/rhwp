---
kind: guide
status: active
canonical: gym/packs/casual-rides/README.md
last_verified: 2026-08-18
---

# casual-rides — 입문 놀이기구 (누구나)

## 왜 이 pack 인가

놀이공원 입구의 키 제한 없는 존이다. 부모님·친구·처음 온 에이전트가
rhwp 를 **한 번** 실행하고 숫자 하나를 옮기면 통과한다. CR01–CR04 는
전부 `samples/table-001.hwp` 한 파일이라, 그 파일의 쪽수·문단·표·'표'
매치만 외우면 입문존이 끝이었다.

CR05+ 는 같은 네 명령(`info` · `explain` · `export-tables` · `search`)으로
**다른 표본·다른 키**를 묻는다. 새 CLI 는 없다. 새 연산자는 없다.
T07 의 홍길동 채움을 입문존에 들여오지 않는다.

## 이 확장이 지키는 규칙

1. **기존 명령만.** requires 는 `info` · `explain` · `export-tables` ·
   `search` 네 개다. `edit` · `fields` · `gate` 를 부르지 않는다.
2. **기존 연산자만.** 입문 과제는 `answer_eq` (일부는 길이). 전역 훑기는 없다.
3. **기존 표본만.** `samples/` 실파일만 가리킨다.
4. **라이브 오라클.** 쪽수·문단·표·매치·각주·미주·형식을 박제하지 않는다.
5. **T07 금지.** 누름틀에 홍길동을 쓰지 않는다. field-01 을 열어도 읽기만 한다.
6. **티어 1.** 입문존은 키 제한이 없다.
7. **runner 신원 복사.** `pack.json` runner 는 devel 값을 유지한다.
8. **새 pack / 새 CLI 없음.**

## 명령 표면

| 명령 | 묻는 것 | 봉투 키 |
|------|---------|---------|
| `info` | 쪽수·형식 | `pageCount` · `format` |
| `explain` | 문단·각주·미주·형식 | `paragraphCount` · `footnoteCount` · `endnoteCount` · `format` |
| `export-tables` | 표 개수 | `tableCount` |
| `search` | 글자 매치 | `matchCount` |

CR04 처럼 질의에 하이픈이 없더라도 `--` 뒤에 질의를 두는 습관을 들인다.

## 함정

- **한 파일의 답을 다른 파일에 옮기지 마라.** table-001 쪽수가 form-01 이 아니다.
- **같은 파일의 다른 키를 섞지 마라.** 쪽수와 문단 수, 표 개수와 '표' 매치는 다르다.
- **각주 개수 ≠ 쪽수.** CR23 과 CR43, CR24 와 CR44 를 바꿔 쓰지 마라.
- **서식을 채우지 마라.** 입문존은 읽기만 한다.
- **추측 금지.** 파일 이름에 1p 가 있어도 info 를 돌려라.

## 과제 지도

### CR01–CR04 — 개장 (table-001 한 파일)

| ID | 놀이기구 | 질문 |
|----|----------|------|
| CR01 | 회전목마 | 몇 쪽인가요? |
| CR02 | 관람차 | 문단이 몇 개인가요? |
| CR03 | 서커스 텐트 | 표가 몇 개인가요? |
| CR04 | 링 던지기 | '표' 글자가 몇 번? |

### CR05+ — 표본을 흩은 입문 여정

| ID | 질문 | 표본 |
|----|------|------|
| CR05 | 짧은 문단 문서는 몇 쪽? | `samples/para-001.hwp` |
| CR06 | 짧은 문단은 몇 개? | `samples/para-001.hwp` |
| CR07 | 표가 여러 개인 문서는 표가 몇 개? | `samples/multi-table-001.hwp` |
| CR08 | 다중 표 문서에 '표' 가 몇 번? | `samples/multi-table-001.hwp` |
| CR09 | 서식 문서는 몇 쪽? | `samples/form-01.hwp` |
| CR10 | 서식 문단은 몇 개? | `samples/form-01.hwp` |
| CR11 | 다른 서식은 몇 쪽? | `samples/form-02.hwp` |
| CR12 | 누름틀 표본은 몇 쪽? | `samples/field-01.hwp` |
| CR13 | 메모 서식은 몇 쪽? | `samples/field-01-memo.hwp` |
| CR14 | 2010년 표본은 몇 쪽? | `samples/2010-01-06.hwp` |
| CR15 | 2010년 표본 문단은 몇 개? | `samples/2010-01-06.hwp` |
| CR16 | 한 쪽 시험지는 몇 쪽? | `samples/exam-kor-1p.hwp` |
| CR17 | 두 쪽 시험지는 몇 쪽? | `samples/exam-kor-2p.hwp` |
| CR18 | 사회 시험지는 몇 쪽? | `samples/exam_social.hwp` |
| CR19 | 초등학교 표본은 몇 쪽? | `samples/el-school-001.hwp` |
| CR20 | 자동차 표본은 몇 쪽? | `samples/hcar-001.hwp` |
| CR21 | 내부 표 문서는 표가 몇 개? | `samples/inner-table-01.hwp` |
| CR22 | 계산 셀 문서는 표가 몇 개? | `samples/calc-cell.hwp` |
| CR23 | 각주 문서는 각주가 몇 개? | `samples/footnote-01.hwp` |
| CR24 | 미주 문서는 미주가 몇 개? | `samples/endnote-01.hwp` |
| CR25 | 줄 단위 표본 문단은 몇 개? | `samples/lseg-01-basic.hwp` |
| CR26 | 수식 표본은 몇 쪽? | `samples/math-001.hwp` |
| CR27 | 그림 표본은 몇 쪽? | `samples/pic2.hwp` |
| CR28 | 사업계획 표본은 몇 쪽? | `samples/biz_plan.hwp` |
| CR29 | KTX 문서에 'KTX' 가 몇 번? | `samples/KTX.hwp` |
| CR30 | aift 표본은 몇 쪽? | `samples/aift.hwp` |
| CR31 | 국어원 계획은 몇 쪽? | `samples/2022년 국립국어원 업무계획.hwp` |
| CR32 | 국어원 계획에 '국립' 이 몇 번? | `samples/2022년 국립국어원 업무계획.hwp` |
| CR33 | 홍보 문서는 몇 쪽? | `samples/20250130-hongbo.hwp` |
| CR34 | 홍보 문서에 '홍보' 가 몇 번? | `samples/20250130-hongbo.hwp` |
| CR35 | 두 번째 다중 표는 표가 몇 개? | `samples/multi-table-002.hwp` |
| CR36 | 표 표본의 형식은? | `samples/table-001.hwp` |
| CR37 | 표 표본을 explain 으로 보면 형식은? | `samples/table-001.hwp` |
| CR38 | 서식 문서에 '서식' 이 몇 번? | `samples/form-01.hwp` |
| CR39 | 누름틀 표본 문단은 몇 개? | `samples/field-01.hwp` |
| CR40 | 한 쪽 시험지에 '문제' 가 몇 번? | `samples/exam-kor-1p.hwp` |
| CR41 | 중첩표 다쪽 문서는 몇 쪽? | `samples/basic/issue2007_nested_cell_pagination_42065.hwp` |
| CR42 | 중첩표 다쪽 문서는 표가 몇 개? | `samples/basic/issue2007_nested_cell_pagination_42065.hwp` |
| CR43 | 각주 문서는 몇 쪽? | `samples/footnote-01.hwp` |
| CR44 | 미주 문서는 몇 쪽? | `samples/endnote-01.hwp` |

## 재현

기준풀이는 `reference/CR05.json` … `reference/CR44.json` 이다.

```
python gym/tools/audit.py
python -m unittest scripts.tests.test_gym_packs
python -m unittest scripts.tests.test_gym_core_casual_packs
```
