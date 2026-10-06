---
kind: guide
status: active
canonical: gym/packs/serialization/README.md
last_verified: 2026-08-18
---

# serialization — 저장·변환 여정과 실패 모드

이 pack은 **형식을 넘나드는 왕복**을 채점한다. 변환물이 진짜 그 형식인지, 내용이
보존됐는지, 쪽 범위가 1 기준으로 잘렸는지, 검증 실패가 산출물을 삼키지 않는지를
각각 본다. 새 CLI는 없다. 기존 `convert` · `export-hwpx` · `export-pdf` ·
`extract-pages` · `export-doclang` · `export-markdown` · `export-hml` ·
`export-ir-schema` · `info` · `ir-diff` 와 `samples/` 만 쓴다.

채점은 라이브 오라클이다. 쪽수·차이 건수·백엔드 이름을 JSON에 박제하지 않는다.
`answer_eq` 가 채점 시점에 rhwp를 다시 돌려 기댓값을 계산한다. 골든 PDF/HWP
바이트를 저장소에 넣지 않는다.

## 왜 이 pack 인가

에이전트가 문서를 "저장했다"고 말하는 순간, 보통 네 가지 중 하나가 빠진다.

1. **컨테이너가 바뀌지 않았다.** 입력 `.hwpx` 를 그대로 `conv.hwp` 라는 이름만
   바꿔 제출한다. `differs_from_input` 이 이걸 거절한다.
2. **형식 표기를 추측했다.** `convert` 가 만든 파일이 HWP5인지 HWPX인지
   `info --json` 의 `format` 을 읽지 않고 `"hwp"` 라고 적는다. 실제 표기는
   `hwp5` 다.
3. **쪽 축을 0 기준으로 잘랐다.** `extract-pages --from/--to` 는 **1 기준**이다.
   `search` 의 `page: 1`(0 기준 둘째 쪽)을 그대로 `--from 1` 에 넣으면 한 쪽
   밀린 문서가 조용히 나온다.
4. **종료 코드만 보고 판정했다.** `ir-diff` 와 `--verify` 는 차이가 있으면
   exit 3, `--verify-pages` 는 exit 4 다. 산출물은 남고 봉투에 판정이 실린다.
   종료 코드만 보면 봉투를 읽지 않은 것이다.

이 pack의 과제는 위 네 구멍을 여정으로 나눈다.

## 여정 지도

과제는 명령 가족이 아니라 **사람이 하는 일**로 묶는다. 한 여정이 여러 과제를
가질 수 있다. 표본이 바뀌면 같은 명령도 다른 계약이 된다.

### J1. 편집 가능본으로 저장 (`convert`)

배포용 해제와 형식 변환을 한 명령으로 본다. 입력이 이미 편집 가능해도
`convert` 는 산출물을 쓰고 봉투를 낸다.

| ID | 하는 일 | 표본 | 지목 |
|---|---|---|---|
| SR09 | HWPX → 편집 가능 HWP5 | `hwpx_sample2.hwpx` | `format` |
| SR13 | 이미 편집 가능한 HWP5 를 다시 convert | `para-001.hwp` | `wasDistribution` |
| SR17 | convert `--verify` IR 자기검증 | `hwpx_sample2.hwpx` | `verify.identical` |
| SR21 | convert `--verify-pages` 쪽수 자기검증 | `para-001.hwp` | `verifyPages.identical` |
| SR25 | 다른 HWPX 표본도 같은 계약 | `hwpx/143E433F503322BD33.hwpx` | `format` |
| SR26 | 표 문서 HWP5 재변환 | `table-001.hwp` | `wasDistribution` |
| SR27 | 편집 가능본 `--verify` | `para-001.hwp` | `verify.identical` |
| SR48 | 변환 산출물이 실제로 열리는지 | `hwpx_sample2.hwpx` | `info.pageCount` |
| SR56 | 가로 문서 `--verify-pages` | `landscape-001.hwp` | `verifyPages.identical` |

**실패 모드**

- `wasDistribution` 을 `true` 로 박제한다. 편집 가능 입력은 보통 `false` 다.
  값은 봉투에서 읽어야 한다.
- `--verify` 와 `--verify-pages` 를 섞는다. 전자는 IR 차이(exit 3), 후자는
  쪽수 차이(exit 4).
- 산출 확장자를 `.hwpx` 로 준다. `convert` 는 `.hwp` 만 받는다.

### J2. 쪽 범위만 남긴다 (`extract-pages`)

대형 문서를 이분법으로 줄이는 진단 도구다. 정밀한 페이지 오려내기가 아니다.

| ID | 하는 일 | 표본 | 지목 |
|---|---|---|---|
| SR10 | 2쪽 문서에서 첫 쪽만 | `exam-kor-2p.hwp` | `pagesAfter` |
| SR14 | 2쪽 전체를 남긴 뒤 원본 쪽수 | `exam-kor-2p.hwp` | `pagesBefore` |
| SR18 | 둘째 쪽만 남긴 문단 수 | `exam-kor-2p.hwp` | `paragraphsKept` |
| SR22 | 중첩 셀 문서에서 첫 쪽만 | `issue2007_…42065.hwp` | `pagesBefore` |
| SR28 | 3쪽 문서에서 첫 쪽만 | `exam-kor-3p.hwp` | `pagesAfter` |
| SR29 | 3쪽 전체 추출 전 쪽수 | `exam-kor-3p.hwp` | `pagesBefore` |
| SR30 | 4쪽 문서에서 지운 문단 수 | `exam-kor-4p.hwp` | `paragraphsRemoved` |
| SR31 | 1쪽 문서에서 그 쪽만 | `exam-kor-1p.hwp` | `pagesAfter` |
| SR54 | 3쪽 문서에서 마지막 쪽만 | `exam-kor-3p.hwp` | `paragraphsKept` |

**실패 모드**

- `--from 0` 을 넣는다. 이 명령은 1 기준이라 사용법 오류이거나 한 쪽 밀린다.
- `pagesAfter` 가 요청 폭과 같다고 가정한다. 재조판으로 달라질 수 있다.
  라이브 오라클이 실측값을 채점한다.
- `pagesBefore` 와 `pagesAfter` 를 바꿔 적는다.
- 문단이 여러 쪽에 걸치면 한 쪽이라도 범위 안이면 남는다. 쪽 단위 가위로
  착각하면 `paragraphsKept` 가 어긋난다.

### J3. PDF로 보낸다 (`export-pdf`)

골든 PDF를 만들지 않는다. 형식·쪽수·백엔드만 봉투에서 읽는다.

| ID | 하는 일 | 표본 | 지목 |
|---|---|---|---|
| SR11 | 문단 문서 PDF 쪽수 | `para-001.hwp` | `pageCount` |
| SR15 | 표 문서 PDF 백엔드 | `table-001.hwp` | `backend` |
| SR19 | 그림 문서 PDF 쪽수 | `pic2.hwp` | `pageCount` |
| SR23 | HWPX 원본 렌더 쪽수 | `hwpx/143E…hwpx` | `renderedCount` |
| SR32 | 표 문서 PDF 형식 | `table-001.hwp` | `format` |
| SR33 | 2쪽 시험지 PDF 쪽수 | `exam-kor-2p.hwp` | `pageCount` |
| SR34 | 같은 시험지 렌더 쪽수 | `exam-kor-2p.hwp` | `renderedCount` |
| SR35 | HWPX 표본 백엔드 | `hwpx_sample2.hwpx` | `backend` |
| SR49 | 각주 문서 PDF 쪽수 | `footnote-01.hwp` | `pageCount` |
| SR50 | 가로 문서 백엔드 | `landscape-001.hwp` | `backend` |

**실패 모드**

- PDF 바이트 해시를 정답으로 삼는다. 폰트 환경이 바뀌면 깨진다.
- `pageCount` 와 `renderedCount` 를 혼동한다. 보통 같지만 지목 필드가 다르다.
- `--backend direct` 를 없는 feature에서 켠다. 기본 `svg` 경로를 읽고 적어라.
- `-p` 는 **0 기준** 단일 쪽이다. `extract-pages` 의 1 기준과 섞지 마라.

### J4. IR을 맞댄다 (`ir-diff`)

같은 바이트, 짝 파일, 다른 문서, 연도 쌍을 나눈다. 판정은 데이터다.

| ID | 하는 일 | 표본 | 지목 |
|---|---|---|---|
| SR01 | HWPX 변환 뒤 IR 동일 | `table-001.hwp` | `identical` |
| SR06 | 변환 뒤 차이 건수 | `issue2007_…` | `diffCount` |
| SR12 | 기존 pic2 HWP·HWPX 쌍 | `pic2.hwp` + `.hwpx` | `identical` |
| SR16 | 자기 자신과 대조 | `table-001.hwp` | `identical` |
| SR20 | pic2 쌍 차이 건수 | `pic2.hwp` + `.hwpx` | `diffCount` |
| SR24 | 서로 다른 두 문서 | `table-001` vs `para-001` | `identical` |
| SR36 | 문단 문서 자기대조 | `para-001.hwp` | `identical` |
| SR37 | hwpx_sample2 쌍 | `.hwp` + `.hwpx` | `identical` |
| SR38 | 표 vs 시험지 | `table-001` vs `exam-kor-2p` | `identical` |
| SR39 | pic2 vs pic2-2018 | 연도 쌍 | `diffCount` |
| SR53 | 각주 문서 자기대조 | `footnote-01.hwp` | `identical` |

**실패 모드**

- 차이가 있으면 실패라고 착각한다. exit 3 은 **차이 발견**이지 도구 고장
  이 아니다. `expect_exits` 에 0과 3이 들어 있다.
- `identical` 만 보고 `diffCount` 를 무시한다. 규모가 가려진다.
- 자기대조가 `false` 이면 파서 비결정성이다. 그 경우는 도구 버그로 올려라.

### J5. HWPX 컨테이너로 보낸다 (`export-hwpx`)

`convert`(배포용 해제·HWP5)와 반대 방향이다. 명령을 바꾸면 산출 확장자가
거부된다.

| ID | 하는 일 | 표본 | 지목 |
|---|---|---|---|
| SR01 | HWPX 변환 자기검증 | `table-001.hwp` | IR `identical` |
| SR06 | HWPX 왕복 차이 건수 | `issue2007_…` | `diffCount` |
| SR40 | 문단 문서 HWPX 형식 | `para-001.hwp` | `format` |
| SR41 | `--verify-pages` | `para-001.hwp` | `verifyPages.identical` |
| SR42 | `--verify` | `table-001.hwp` | `verify.identical` |
| SR51 | 서식 문서 HWPX 형식 | `form-01.hwp` | `format` |

**실패 모드**

- `export-hwpx` 자리에 `convert` 를 넣는다. 산출이 `.hwp` 가 된다.
- `--verify` 없이 IR 동일을 주장한다. 검증 객체는 옵션을 켠 때만 생긴다.

### J6. 의미 형식·스키마 (`export-doclang` · `export-markdown` · `export-hml` · `export-ir-schema`)

다운스트림이 읽는 형식이다. 변환 성공과 무손실은 다른 축이다.

| ID | 하는 일 | 표본 | 지목 |
|---|---|---|---|
| SR02 | HML 왕복 | `hml/formatting_table.hml` | 파일 + `info.pageCount` |
| SR03 | DocLang 손실 계수 | `issue2007_…` | `lossCount` |
| SR04 | 마크다운 쪽수 | `table-001.hwp` | `pageCount` |
| SR05 | IR 스키마 dialect | (문서 불요) | `dialect` |
| SR07 | HWPX 원본 조사 | `hwpx/143E…` | `pageCount` + `format` |
| SR08 | HWPX 마크다운 쪽수 | 같은 HWPX | `pageCount` |
| SR43 | DocLang 형식 | `para-001.hwp` | `format` |
| SR44 | DocLang 버전 | `para-001.hwp` | `doclangVersion` |
| SR45 | 표 문서 손실 계수 | `table-001.hwp` | `lossCount` |
| SR46 | 2쪽 시험지 마크다운 | `exam-kor-2p.hwp` | `pageCount` |
| SR47 | 그림 문서 마크다운 | `pic2.hwp` | `pageCount` |
| SR52 | 다중 표 DocLang 손실 | `multi-table-001.hwp` | `lossCount` |
| SR55 | 다른 HWPX 원본 조사 | `hwpx_sample2.hwpx` | `pageCount` |

**실패 모드**

- HWP/HWPX 를 `export-hml` 에 넣는다. 이 명령은 `.hml` 만 받는다.
- `lossCount == 0` 을 성공 조건으로 박제한다. 손실 보고는 정보이지 실패가
  아니다. 채점은 실측값을 맞추는 것이다.
- `doclangVersion` 을 `"1.0"` 으로 적는다. 스키마 버전과 DocLang 버전은
  다른 필드다.

## 실패 모드 카탈로그 (여정 공통)

아래는 과제 하나가 아니라 pack 전체가 반복해서 잡는 실수다. 예외·가장자리
상세는 [mydocs/working/gym_serialization_exceptions.md](../../../mydocs/working/archives/gym_serialization_exceptions.md)
에 적는다.

### F1. 무편집 복사

산출물 이름이 `conv.hwp` 여도 바이트가 입력과 같으면 변환이 아니다.
`differs_from_input` 이 거부한다. 이름만 바꾼 제출, `copy` 한 제출,
빈 파일을 확장자만 맞춘 제출이 여기 걸린다. `minBytes` 는 빈 껍데기를
한 번 더 거른다.

### F2. 형식 문자열 추측

`info --json` 의 `format` 은 `hwp5` · `hwpx` · `hml` · `hwp3` 같이 적힌다.
`"hwp"` · `"HWP"` · `"application/hwp"` 는 오답이다. PDF 봉투의 `format` 은
`"pdf"` 다. DocLang 봉투의 `format` 은 `"doclang"` 이다.

### F3. 쪽 축 혼동

| 명령 | 쪽 기준 | 필드 |
|---|---|---|
| `extract-pages --from/--to` | **1 기준** | `pagesBefore` / `pagesAfter` |
| `export-pdf -p` | **0 기준** | 단일 쪽 렌더 |
| `info` / `export-markdown` / `export-pdf` 전체 | 쪽 수 | `pageCount` |
| `export-pdf` 실제 렌더 | 쪽 수 | `renderedCount` |
| `search` / `export-text` 의 `page` | **0 기준** | 본문 주소 |

`search` 가 `page: 1` 을 주면 `extract-pages` 에서는 `--from 2 --to 2` 다.

### F4. 종료 코드로 판정

| 상황 | exit | 산출물 | 읽을 필드 |
|---|---|---|---|
| 변환·비교 성공, 차이 없음 | 0 | 남음 | `identical: true` |
| IR 차이 / `--verify` 실패 | 3 | **남음** | `identical` / `verify.identical` |
| `--verify-pages` 쪽수 불일치 | 4 | **남음** | `verifyPages.identical` |
| 읽기·파싱 실패 | 1 | 없을 수 있음 | stdout 비움 |
| 사용법 오류 | 2 | 없음 | 인자를 고쳐라 |

과제는 `expect_exits` / `allowExits` 에 0과 3(또는 4)을 같이 넣는다. 종료
코드 0만 허용하면 차이가 있는 정당한 왕복이 전부 실패한다.

### F5. 필드 이름 바꿔 읽기

같은 봉투 안에서 이름이 비슷한 필드가 있다.

- `pageCount` vs `renderedCount`
- `pagesBefore` vs `pagesAfter`
- `paragraphsKept` vs `paragraphsRemoved`
- `verify.identical` vs `verifyPages.identical`
- `format` vs `backend` vs `doclangVersion`
- `identical` vs `diffCount`

과제는 `answer` 키와 `path` 를 일부러 다르게 두기도 한다(예: answer
`pages` ← path `pageCount`). 힌트의 필드명을 읽지 않으면 틀린다.

### F6. 골든 파일

PDF·HWP·HWPX 바이트는 폰트·타임스탬프·직렬화 순서에 흔들린다. 이 pack은
해시를 정답으로 쓰지 않는다. `file_exists` + `differs_from_input` +
`value_eq(format)` + `answer_eq(실측 필드)` 만 본다.

### F7. 명령 가족 바꿔 치기

- HWPX가 필요하면 `export-hwpx`, 편집 가능 HWP5가 필요하면 `convert`.
- HML 원본만 `export-hml`.
- 쪽 범위 저장은 `extract-pages`, 단일 쪽 PDF는 `export-pdf -p`.
- IR 대조는 `ir-diff`. `info` 의 쪽수가 같다고 IR이 같은 것은 아니다.

## 정직한 경계 — 이 pack이 보지 않는 것

- 한컴 정본 PDF와의 픽셀 충실도. 그건 `render-diff` · fidelity harness 다.
- 암호 설정 왕복(`--output-password`). 보안 pack과 겹치지 않게 비웠다.
- HWP3 배포용 해제 특수 경로. 표본을 HWP5/HWPX/HML에 고정했다.
- 브라우저 저장 UX. gym 축은 CLI 다.
- `profiles/` · `PARK.md` · 다른 pack 수정. 이 확장은 serialization
  폴더와 계약 테스트·작업 노트만 건드린다.

## 재현

```bash
python gym/tools/audit.py
python -m unittest scripts.tests.test_gym_packs -v
python -m unittest scripts.tests.test_gym_serialization_pack -v
# 기준 풀이 왕복(바이너리 필요):
# python gym/tools/build_baseline.py --agent baseline --pack serialization --bin target/debug/rhwp
# python gym/score.py               --agent baseline --pack serialization --bin target/debug/rhwp
```

`runner` 블록은 기존 pack 신원을 유지한다. 이 확장은 과제·기준풀이·문서만
늘리고 바이너리 표면은 바꾸지 않았다.

## 과제 목록 (SR01–SR56)

기존 SR01–SR08은 유지한다. SR09–SR12는 이 PR의 첫 확장, SR13–SR24는
구조 왕복 구조대 과제, SR25–SR56은 같은 명령으로 표본·필드·실패 모드를
갈라 놓은 후속이다.

상세 설계·예외는
[mydocs/working/gym_serialization_pack.md](../../../mydocs/working/archives/gym_serialization_pack.md)
와
[mydocs/working/gym_serialization_exceptions.md](../../../mydocs/working/archives/gym_serialization_exceptions.md)
를 본다.
