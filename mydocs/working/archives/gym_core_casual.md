---
kind: working
status: active
canonical: mydocs/working/gym_core_casual.md
last_verified: 2026-08-18
issue: 5258
---

# gym core-cli · casual-rides 확장 작업 노트 (#5258)

## 한 줄

입문존(`casual-rides`)과 코어 CLI(`core-cli`) 축이 얇아 새 에이전트
온램프가 약하다. 기존 연산자·기존 표본·기존 명령만으로 T15–T54 와
CR05–CR44 를 더하고, 각 pack README 와 가드 시험·이 노트를 남긴다.
T07 의 `fields[0]==홍길동` 은 복제하지 않는다. 새 pack / 새 CLI 는 없다.

## 배경

이슈 #5258 DoD: additions >= 3000, `audit.py` + `test_gym_packs.py`.

devel 기준 두께:

- `core-cli` T01–T14 (14과제). 표본이 issue2007 · field-01 · 국어원 ·
  수출입 · 143E 해시에 몰려 있다. 검색 질의는 '국어' 하나.
- `casual-rides` CR01–CR04 (4과제). 전부 `samples/table-001.hwp`.

건드리지 않은 것:

- 새 CLI, 새 pack, T07 복제 (`홍길동` + `fields[0].value` + 제목 "서식 채움")
- `gym/packs/automation/` 및 다른 pack 의 과제
- `profiles/` · `gym/README.md` · `gym/PARK.md` · `gym/core/checks.py`
- `cargo fmt --all` (JSON·문서·테스트만)
- `pack.json` 의 `runner` 신원. 요구 명령 목록도 기존 값을 유지했다.

## 설계 원칙

1. **라이브 오라클.** `answer_eq` / `len_answer_eq` 에 숫자를 박제하지 않는다.
2. **축을 가른다.** 같은 `info` 라도 pageCount · format · paraCount.
   같은 `search` 라도 질의와 표본이 다르다. 같은 `inspect` 라도
   hidden-text · unicode · injection.
3. **표본을 흩는다.** table-001 한 파일에 입문존을 가두지 않는다.
4. **T07 금지.** 채움 과제를 더하지 않았다. T32 는 name, T34 는 fieldType.
5. **기존 연산자·기존 명령.** 채점 cmd 는 각 pack requires 안이다.
   기준풀이의 `run` / `export-hwpx` 는 T06·T08·T12 와 같은 기존 패턴이다.
6. **runner 복사.** 신원을 새로 찍지 않는다.

## 검증

```
python gym/tools/audit.py
python -m unittest scripts.tests.test_gym_packs
python -m unittest scripts.tests.test_gym_core_casual_packs
```

`audit.py` 는 스키마·과제↔기준 짝·전역 고유 ID 를 본다.
`test_gym_packs.py` 는 전 pack 계약을 본다. 전용 가드는
`scripts/tests/test_gym_core_casual_packs.py` 다.

바이너리·네트워크 없이 돈다. 라이브 수치 재계산은 채점 러너가 한다.

## 신규 과제 — core-cli T15–T54

| ID | 제목 | 표본 |
|----|------|------|
| T15 | 표 표본 쪽수 | `samples/table-001.hwp` |
| T16 | 국어원 문서 형식 | `samples/2022년 국립국어원 업무계획.hwp` |
| T17 | 표 표본 문단 수 | `samples/table-001.hwp` |
| T18 | 서식 표본 쪽수 | `samples/form-01.hwp` |
| T19 | 누름틀 표본 형식 | `samples/field-01.hwp` |
| T20 | 한 쪽 시험지 쪽수 | `samples/exam-kor-1p.hwp` |
| T21 | 표 표본에서 '표' 검색 | `samples/table-001.hwp` |
| T22 | 국어원 문서에서 '국립' 검색 | `samples/2022년 국립국어원 업무계획.hwp` |
| T23 | 서식 본문에서 '서식' 검색 | `samples/form-01.hwp` |
| T24 | 누름틀 표본에서 '회사' 검색 | `samples/field-01.hwp` |
| T25 | 홍보 문서에서 '홍보' 검색 | `samples/20250130-hongbo.hwp` |
| T26 | 다중 표 문서에서 '표' 검색 | `samples/multi-table-001.hwp` |
| T27 | 문단 표본에서 '한글' 검색 | `samples/para-001.hwp` |
| T28 | 중첩표 표본에서 '규제' 검색 | `samples/basic/issue2007_nested_cell_pagination_42065.hwp` |
| T29 | form-01 누름틀 개수 | `samples/form-01.hwp` |
| T30 | form-02 누름틀 개수 | `samples/form-02.hwp` |
| T31 | 메모 서식 누름틀 개수 | `samples/field-01-memo.hwp` |
| T32 | 첫 칸 이름만 읽기 | `samples/field-01.hwp` |
| T33 | 둘째 칸 이름만 읽기 | `samples/field-01.hwp` |
| T34 | 첫 칸 타입만 읽기 | `samples/field-01.hwp` |
| T35 | 해외직구 안내 데이터 추출 | `samples/156457624_210622 7월부터 해외직구 구매대행업체 등록제 시행.hwp` |
| T36 | 국어원 계획 데이터 추출 | `samples/2022년 국립국어원 업무계획.hwp` |
| T37 | 표 표본 표 개수 | `samples/table-001.hwp` |
| T38 | 다중 표 표본 표 개수 | `samples/multi-table-001.hwp` |
| T39 | 서식 문서 표 개수 | `samples/form-01.hwp` |
| T40 | 홍보 문서 데이터 추출 | `samples/20250130-hongbo.hwp` |
| T41 | 조판 은닉 글자 수 | `samples/issue1892_hwp3_tab_roundtrip.hwp` |
| T42 | 유니코드 기만 소견 수 | `samples/unicode/각 항목에 명시되어 있는_유니코드.hwp` |
| T43 | 표 표본 주입 신호 | `samples/table-001.hwp` |
| T44 | 표 표본 은닉 스윕 | `samples/table-001.hwp` |
| T45 | 표 표본 유니코드 스윕 | `samples/table-001.hwp` |
| T46 | 누름틀 표본 주입 스윕 | `samples/field-01.hwp` |
| T47 | 서식 표본 HWPX 변환 자기검증 | `samples/form-01.hwp` |
| T48 | 표 표본 HWPX 변환 자기검증 | `samples/table-001.hwp` |
| T49 | 표 표본 첫 셀 온램프 표기 | `samples/table-001.hwp` |
| T50 | 다중 표 첫 셀 표기 | `samples/multi-table-001.hwp` |
| T51 | 내부표 첫 셀 표기 | `samples/inner-table-01.hwp` |
| T52 | 표 표본 두 칸 원자 계획 | `samples/table-001.hwp` |
| T53 | 표 표본 결정론 실증 | `samples/table-001.hwp` |
| T54 | 한 쪽 시험지 데이터 추출 | `samples/exam-kor-1p.hwp` |

묶음:

- T15–T20 신상 (`info` pageCount/format/paraCount, 다른 표본)
- T21–T28 검색 (`search` 질의·표본을 갈라 matches 길이)
- T29–T34 필드 조회 (개수·name·fieldType, 값 채움 없음)
- T35–T40 · T54 추출·표 (`extract-data` · `export-tables`)
- T41–T46 스윕 (`inspect` hidden-text/unicode/injection)
- T47–T48 변환 자기검증 (`export-hwpx` + `ir-diff`, T12 패턴)
- T49–T53 좌표 편집·계획·결정론 (T08–T10 패턴, 다른 표본·다른 문구)

## 신규 과제 — casual-rides CR05–CR44

| ID | 제목 | 표본 |
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

묶음:

- CR05–CR06 para-001 쪽수·문단
- CR07–CR08 · CR21–CR22 · CR35 · CR42 표 개수·'표' 검색
- CR09–CR13 · CR38–CR39 서식·누름틀 표본을 읽기만
- CR14–CR20 · CR26–CR28 · CR30–CR31 · CR33 · CR41 · CR43–CR44 쪽수 분산
- CR23–CR24 각주·미주
- CR29 · CR32 · CR34 · CR40 검색
- CR36–CR37 형식 (info vs explain)

## 실패하면 안 되는 것

- T07 클론: 제목 "서식 채움", `fields[0].value == 홍길동`, 제출 `filled.hwp` 단독
- 새 연산자 / 새 pack 디렉터리 / automation pack 수정
- runner 해시·커밋 교체
- answer_eq 에 pageCount 숫자를 박제

## 다음에 하지 않은 것

집계 문서(`gym/README.md`, `PARK.md`, tutorial)의 과제 수 표는 손대지 않았다.
프로파일은 이미 두 pack 을 가리키므로 유지했다. T13·T14 급 사다리 과제는
입문 온램프가 아니어서 늘리지 않았다.
