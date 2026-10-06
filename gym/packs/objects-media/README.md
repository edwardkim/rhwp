---
kind: guide
status: active
canonical: gym/packs/objects-media/README.md
last_verified: 2026-08-18
---

# objects-media — 누름틀·미리보기·개체 여정

이 pack은 **문서를 열지 않고 자리와 산출물을 지목**하는 축이다. 누름틀이
몇 개인지, 몇 번째 필드의 이름·유형이 무엇인지, 미리보기의 픽셀·MIME 이
무엇인지, 마크다운으로 내릴 때 이미지가 몇 장인지, 그리고 **지정한 필드에만**
값이 들어갔는지를 본다. 새 CLI는 없다. 기존 `fields` · `thumbnail` ·
`export-markdown` · `edit fill-fields` 와 `samples/` 만 쓴다.

채점은 라이브 오라클이다. 필드 수·이름·유형·픽셀·MIME·이미지 수를 JSON 에
박제하지 않는다. `answer_eq` 가 채점 시점에 rhwp를 다시 돌려 기댓값을
계산한다. 골든 PNG/HWP 바이트를 저장소에 넣지 않는다.

## 왜 이 pack 인가

에이전트가 "서식을 채웠다"거나 "미리보기를 만들었다"고 말하는 순간, 보통
네 가지 중 하나가 빠진다.

1. **자리를 세지 않았다.** `fields` 를 돌리지 않고 첫 필드만 채운다.
   두 번째·세 번째 자리, HWPX 서식의 영문 이름, 메모 판의 다른 구성이
   모두 다른 계약이다.
2. **이름과 유형을 섞었다.** `fields[0].name` 과 `fields[0].fieldType` 은
   다른 필드다. 유형을 이름 칸에 적으면 틀린다.
3. **미리보기를 눈으로 판정했다.** `thumbnail` 봉투의 `width` · `height` ·
   `mime` 을 읽지 않고 "대충 PNG겠지"라고 적는다.
4. **전역 훑기로 채움을 검사했다.** 편집 산출에 `deep_contains` 를 쓰면
   다른 필드에 흘러 들어간 값도 통과한다. 이 pack 의 채움 과제는
   `fields[n].value` 를 `value_eq` 로 지목한다. T07(`홍길동` + `filled.hwp` +
   첫 필드)을 복제하지 않는다.

이 pack의 과제는 위 네 구멍을 여정으로 나눈다.

## 여정 지도

과제는 명령 가족이 아니라 **사람이 하는 일**로 묶는다. 한 여정이 여러
과제를 가질 수 있다. 표본이 바뀌면 같은 명령도 다른 계약이 된다.

### J1. 누름틀을 센다 (`fields` · `fieldCount`)

서식이 같은 이름 축이어도 판이 바뀌면 개수가 달라질 수 있다. 숫자를
기억하지 말고 봉투를 읽어라.

| ID | 하는 일 | 표본 | 지목 |
|---|---|---|---|
| OM01 | 본편 서식 전수 | `field-01.hwp` | `fieldCount` |
| OM10 | 메모 판 전수 | `field-01-memo.hwp` | `fieldCount` |
| OM14 | HWPX 서식 전수 | `hwpx/form-01.hwpx` | `fieldCount` |
| OM26 | 둘째 HWPX 서식 전수 | `hwpx/form-02.hwpx` | `fieldCount` |
| OM40 | 서평 서식 전수 | `basic/BlogForm_BookReview.hwp` | `fieldCount` |
| OM43 | 요청 문서 전수 | `basic/request.hwp` | `fieldCount` |

**실패 모드**

- `field-01.hwp` 의 개수를 메모 판·HWPX 서식에 그대로 옮긴다.
- `fields` 배열 길이를 눈으로 세고 `fieldCount` 와 다른 값을 적는다.
- 누름틀이 없는 문서에 "없다"를 추측한다. 0 도 봉투 값이다.

### J2. 자리를 이름으로 지목한다 (`fields[n].name`)

인덱스가 같아도 서식마다 이름이 다르다. 한컴 서식은 한글 이름과 영문
이름이 섞인다.

| ID | 하는 일 | 표본 | 지목 |
|---|---|---|---|
| OM05 | 세 번째 이름 | `field-01.hwp` | `fields[2].name` |
| OM12 | 두 번째 이름 | `field-01.hwp` | `fields[1].name` |
| OM13 | 첫 이름 | `field-01.hwp` | `fields[0].name` |
| OM15 | HWPX 첫 이름 | `hwpx/form-01.hwpx` | `fields[0].name` |
| OM27 | 메모 판 첫 이름 | `field-01-memo.hwp` | `fields[0].name` |
| OM41 | 둘째 HWPX 첫 이름 | `hwpx/form-02.hwpx` | `fields[0].name` |

**실패 모드**

- 본편 서식의 `회사명`·`작성자`·`부서명` 을 HWPX 서식에 그대로 적는다.
- 인덱스를 0 기준이 아니라 1 기준으로 읽는다. `fields[1]` 은 두 번째다.
- 이름 자리에 유형 문자열을 넣는다.

### J3. 유형을 읽는다 (`fields[n].fieldType`)

이름과 유형은 한 객체의 다른 키다. 같은 인덱스를 두 번 읽는 과제가
있는 이유다.

| ID | 하는 일 | 표본 | 지목 |
|---|---|---|---|
| OM08 | 첫 유형 | `field-01.hwp` | `fields[0].fieldType` |
| OM11 | 두 번째 유형 | `field-01.hwp` | `fields[1].fieldType` |
| OM16 | HWPX 첫 유형 | `hwpx/form-01.hwpx` | `fields[0].fieldType` |
| OM28 | 메모 판 첫 유형 | `field-01-memo.hwp` | `fields[0].fieldType` |
| OM37 | 세 번째 유형 | `field-01.hwp` | `fields[2].fieldType` |

**실패 모드**

- `CLICKHERE` · `누름틀` · `field` 같은 추측 문자열을 적는다. 실제 표기는
  봉투에 있다.
- OM05(이름)의 답을 OM37(유형)에 재사용한다.

### J4. 미리보기를 숫자로 읽는다 (`thumbnail`)

미리보기는 눈이 아니라 `width` · `height` · `mime` 이다. 표본이 바뀌면
픽셀이 바뀐다. 방향이 바뀌면 가로·세로가 뒤집힐 수 있다.

| ID | 하는 일 | 표본 | 지목 |
|---|---|---|---|
| OM03 | 다쪽 문서 가로 | `issue2007_…42065.hwp` | `width` |
| OM09 | 다쪽 문서 MIME | 같은 다쪽 문서 | `mime` |
| OM17 | 표 문서 가로 | `table-001.hwp` | `width` |
| OM18 | 표 문서 세로 | `table-001.hwp` | `height` |
| OM19 | 표 문서 MIME | `table-001.hwp` | `mime` |
| OM20 | 문단 문서 가로 | `para-001.hwp` | `width` |
| OM21 | 실문서 세로 | `143E433F503322BD33.hwp` | `height` |
| OM22 | 가로 문서 MIME | `landscape-001.hwp` | `mime` |
| OM29 | 1쪽 시험지 가로 | `exam-kor-1p.hwp` | `width` |
| OM36 | HWPX 서식 세로 | `hwpx/form-01.hwpx` | `height` |
| OM38 | 2쪽 시험지 세로 | `exam-kor-2p.hwp` | `height` |
| OM42 | 실문서 MIME | `143E433F503322BD33.hwp` | `mime` |
| OM44 | 요청 문서 가로 | `basic/request.hwp` | `width` |

**실패 모드**

- MIME 을 `"png"` 로 적는다. 봉투는 `image/png` 같은 전체 표기를 준다.
- 한 표본의 가로를 다른 표본에 복사한다.
- `-o` 없이 `thumbnail` 을 돌려 산출을 버린다. 과제는 임시 PNG 경로를
  자리표로 준다.

### J5. 개체를 마크다운으로 센다 (`export-markdown` · `imageCount`)

이미지를 눈으로 세지 않는다. 마크다운 내보내기 봉투의 `imageCount` 가
정답이다. 표·문단·시험지·가로·실문서는 같은 필드, 다른 표본이다.

| ID | 하는 일 | 표본 | 지목 |
|---|---|---|---|
| OM04 | 다쪽 문서 이미지 | `issue2007_…42065.hwp` | `imageCount` |
| OM07 | 실문서 이미지 | `143E433F503322BD33.hwp` | `imageCount` |
| OM23 | 표 문서 이미지 | `table-001.hwp` | `imageCount` |
| OM24 | 문단 문서 이미지 | `para-001.hwp` | `imageCount` |
| OM25 | 2쪽 시험지 이미지 | `exam-kor-2p.hwp` | `imageCount` |
| OM30 | 가로 문서 이미지 | `landscape-001.hwp` | `imageCount` |
| OM39 | 1쪽 시험지 이미지 | `exam-kor-1p.hwp` | `imageCount` |

**실패 모드**

- 본문 `![](...)` 를 세어 봉투와 다른 값을 적는다. 채점은 봉투다.
- `pageCount` 를 `imageCount` 칸에 넣는다.

### J6. 지정한 자리에만 채운다 (`edit fill-fields` · `value_eq`)

채움은 조사 다음이다. 기준 풀이는 필드 **이름**으로 넣고, 채점은 필드
**인덱스**로 확인한다. 채우지 않은 자리는 빈 문자열이어야 한다.

| ID | 하는 일 | 표본 | 지목 |
|---|---|---|---|
| OM02 | 둘째만 '지목채움' | `field-01.hwp` | `fields[1]` + 첫째 공란 |
| OM06 | 첫째·셋째 | `field-01.hwp` | `fields[0]`·`[2]` + 둘째 공란 |
| OM31 | 셋째만 '진단부서' | `field-01.hwp` | `fields[2]` + 첫째·둘째 공란 |
| OM32 | 첫째만 '한컴점검' | `field-01.hwp` | `fields[0]` + 둘째 공란 |
| OM33 | 첫째·둘째 쌍 | `field-01.hwp` | `fields[0]`·`[1]` + 셋째 공란 |
| OM34 | HWPX `myMsg01` | `hwpx/form-01.hwpx` | `fields[0].value` |
| OM35 | 둘째·셋째 쌍 | `field-01.hwp` | `fields[1]`·`[2]` + 첫째 공란 |
| OM45 | 셋째만 '감사팀' | `field-01.hwp` | `fields[2]` + 첫째·둘째 공란 |

T07 과 겹치지 않는 것들: 값 `홍길동` 없음, 산출 `filled.hwp` 없음,
"첫 필드만 채우고 다른 자리는 보지 않는" 단일 검사 없음.

**실패 모드**

- 필드 이름을 추측한다. `회사명`·`작성자`·`부서명`·`myMsg01` 은 `fields`
  로 확인한 뒤에 넣는다.
- `deep_contains` 로 "값이 어딘가 있다"를 통과시킨다. 이 pack 은 그렇게
  채점하지 않는다.
- HWPX 서식 산출을 `.hwp` 로 낸다. OM34 는 `formout.hwpx` 다.
- 채우지 않은 자리가 이전 과제의 값을 들고 있으면 실패다.

## 실패 모드 카탈로그 (여정 공통)

### F1. 표본을 섞는다

`field-01.hwp` · `field-01-memo.hwp` · `form-01.hwpx` · `form-02.hwpx` 는
같은 "서식"이 아니다. 표 문서의 썸네일 가로를 다쪽 문서에 복사하면
틀린다.

### F2. 라이브 오라클을 박제한다

필드 수, 픽셀, MIME, 이미지 수는 환경·표본이 바뀌면 달라질 수 있다.
과제 JSON 에 숫자를 넣지 않는다. `answer_eq` 가 채점 때 다시 잰다.

### F3. 자리표를 섞는다

과제 검사의 산출 경로는 `{file:dept.hwp}` 다. 기준 풀이는 `{sub:dept.hwp}`
다. 기준 풀이에 `{file:}` 를 넣으면 러너가 제출 칸을 찾지 못한다.

### F4. 새 CLI 를 만든다

이 pack 은 `fields` · `thumbnail` · `export-markdown` · `edit fill-fields`
로 닫힌다. 새 하위명령, 새 pack, 새 연산자는 없다.

## 표본 지도

| 경로 | 쓰는 여정 |
|---|---|
| `samples/field-01.hwp` | J1–J3, J6 |
| `samples/field-01-memo.hwp` | J1–J3 |
| `samples/hwpx/form-01.hwpx` | J1–J4, J6 |
| `samples/hwpx/form-02.hwpx` | J1, J2 |
| `samples/basic/BlogForm_BookReview.hwp` | J1 |
| `samples/basic/request.hwp` | J1, J4 |
| `samples/table-001.hwp` | J4, J5 |
| `samples/para-001.hwp` | J4, J5 |
| `samples/landscape-001.hwp` | J4, J5 |
| `samples/exam-kor-1p.hwp` · `exam-kor-2p.hwp` | J4, J5 |
| `samples/143E433F503322BD33.hwp` | J4, J5 |
| `samples/basic/issue2007_nested_cell_pagination_42065.hwp` | J4, J5 |

입력은 기존 `samples/` 만 쓴다. 새 픽스처를 만들지 않는다.

## 연산자 계약

| 연산자 | 쓰는 곳 |
|---|---|
| `answer_eq` | 조사 과제 전부 (라이브 오라클) |
| `value_eq` | 채움 과제의 지정 자리·공란 |
| `file_exists` | 미리보기 산출이 있는 OM03 |

편집 축에 `deep_contains` · `not_contains` 를 쓰지 않는다.

## 관련 문서

- 작업 노트: [mydocs/working/gym_om_lr_cd.md](../../../mydocs/working/archives/gym_om_lr_cd.md)
- 조판 pack: [../layout-rendering/README.md](../layout-rendering/README.md)
- 코퍼스 pack: [../corpus-diagnostics/README.md](../corpus-diagnostics/README.md)
