---
kind: guide
status: active
canonical: gym/packs/layout-rendering/README.md
last_verified: 2026-08-18
---

# layout-rendering — 쪽수·자기검증·렌더 여정

이 pack은 **눈이 아니라 봉투로 조판을 판정**하는 축이다. 쪽수와 문단 수,
형식·최소 쪽·최소 표 기대, 썸네일 픽셀, 자기 라운드트립 회귀, 첫 쪽 SVG
산출을 각각 본다. 새 CLI는 없다. 기존 `info` · `verify` · `thumbnail` ·
`render-diff` · `export-svg` 와 `samples/` 만 쓴다.

채점은 라이브 오라클이다. 쪽수·문단 수·판정 문자열·픽셀을 JSON 에
박제하지 않는다. `verify` 가 기대를 어기면 exit 3 이지만 산출은 남고
`verdict` 가 실린다. 종료 코드만 보고 실패로 위장하지 마라.

## 왜 이 pack 인가

에이전트가 "이 문서는 N쪽이다"거나 "검증을 통과했다"고 말하는 순간,
보통 다섯 가지 중 하나가 빠진다.

1. **파일 이름을 쪽수로 믿었다.** `exam-kor-2p.hwp` 는 이름에 2 가 있어도
   `info.pageCount` 를 읽어야 한다.
2. **형식 표기를 추측했다.** `verify --expect-format hwp` 는 틀린 기대다.
   실제 표기는 `hwp5` 또는 `hwpx` 다.
3. **통과만 데이터라고 생각했다.** 100쪽·99표처럼 일부러 틀린 기대는
   `verdict` 가 실패여야 정답이다. 실패도 데이터다.
4. **회귀를 눈으로 봤다.** `render-diff` 의 `status` 를 읽지 않고
   "같아 보인다"고 적는다. exit 3/4 는 차이가 있다는 뜻이지 도구 고장이
   아니다.
5. **SVG 자리에 원본 HWP 를 복사했다.** `differs_from_input` 과
   `xml_root_eq: svg` 가 거절한다.

이 pack의 과제는 위 구멍을 여정으로 나눈다.

## 여정 지도

### J1. 쪽수를 센다 (`info` · `pageCount`)

| ID | 하는 일 | 표본 | 지목 |
|---|---|---|---|
| LR01 | 다쪽 실측 | `issue2007_…42065.hwp` | `pageCount` |
| LR11 | 2쪽 시험지 | `exam-kor-2p.hwp` | `pageCount` |
| LR12 | 3쪽 시험지 | `exam-kor-3p.hwp` | `pageCount` |
| LR13 | 4쪽 시험지 | `exam-kor-4p.hwp` | `pageCount` |
| LR14 | 문단 문서 | `para-001.hwp` | `pageCount` |
| LR36 | 가로 문서 | `landscape-001.hwp` | `pageCount` |
| LR38 | 실문서 | `143E433F503322BD33.hwp` | `pageCount` |
| LR39 | 1쪽 시험지 | `exam-kor-1p.hwp` | `pageCount` |
| LR43 | 영문 표본 | `basic/english.hwp` | `pageCount` |

**실패 모드**

- 이름·폴더 설명의 숫자를 그대로 적는다.
- `dump-pages.pageCount` 와 `info.pageCount` 를 다른 pack 의 답으로
  섞는다. 이 pack 은 `info` 다.

### J2. 문단 수를 센다 (`info` · `paraCount`)

쪽수와 문단 수는 다른 축이다. 1쪽 문서도 문단은 여러 개일 수 있다.

| ID | 하는 일 | 표본 | 지목 |
|---|---|---|---|
| LR09 | 표 문서 문단 | `table-001.hwp` | `paraCount` |
| LR15 | 문단 문서 문단 | `para-001.hwp` | `paraCount` |
| LR16 | 1쪽 시험지 문단 | `exam-kor-1p.hwp` | `paraCount` |
| LR17 | 가로 문서 문단 | `landscape-001.hwp` | `paraCount` |
| LR44 | 영문 표본 문단 | `basic/english.hwp` | `paraCount` |

**실패 모드**

- `pageCount` 를 `paragraphs` 칸에 넣는다.
- 표 문서의 문단 수를 문단 문서에 복사한다.

### J3. 형식 기대를 건다 (`verify --expect-format`)

| ID | 하는 일 | 표본 | 기대 |
|---|---|---|---|
| LR10 | 표 문서 hwp5 | `table-001.hwp` | `hwp5` |
| LR18 | 문단 문서 hwp5 | `para-001.hwp` | `hwp5` |
| LR19 | 1쪽 시험지 hwp5 | `exam-kor-1p.hwp` | `hwp5` |
| LR20 | HWPX 서식 hwpx | `hwpx/form-01.hwpx` | `hwpx` |
| LR21 | HWPX 표 hwpx | `hwpx/basic-table-01.hwpx` | `hwpx` |
| LR37 | 가로 문서 hwp5 | `landscape-001.hwp` | `hwp5` |
| LR45 | 영문 표본 hwp5 | `basic/english.hwp` | `hwp5` |

**실패 모드**

- `"hwp"` · `"HWP"` · `"application/hwp"` 를 기대한다. 표기는 `hwp5` 다.
- HWPX 입력에 `hwp5` 를 건다. LR20·LR21 은 `hwpx` 다.
- exit 3 을 도구 실패로 보고 봉투를 버린다. `expect_exits` 는 0 과 3
  이다.

### J4. 쪽·표 최소 기대를 건다 (`verify --expect-min-pages` · `--expect-min-tables`)

통과하는 기대와 일부러 깨는 기대를 나눈다. 깨는 쪽의 정답은 실패
`verdict` 이거나 낮은 `passCount` 다.

| ID | 하는 일 | 표본 | 기대 |
|---|---|---|---|
| LR02 | 다쪽 17쪽 이상 | `issue2007_…` | min-pages 17 |
| LR06 | 표 문서 100쪽 (위반) | `table-001.hwp` | min-pages 100 |
| LR08 | 다쪽 10쪽+표 1 | `issue2007_…` | pages+tables |
| LR22 | 표 문서 1쪽 이상 | `table-001.hwp` | min-pages 1 |
| LR23 | 2쪽 시험지 2쪽 이상 | `exam-kor-2p.hwp` | min-pages 2 |
| LR24 | 1쪽 시험지 50쪽 (위반) | `exam-kor-1p.hwp` | min-pages 50 |
| LR25 | 표 문서 표 1개 | `table-001.hwp` | min-tables 1 |
| LR26 | 표 문서 표 99개 (위반) | `table-001.hwp` | min-tables 99 |
| LR27 | 표 문서 1쪽+표 1 | `table-001.hwp` | pages+tables |
| LR40 | 4쪽 시험지 3쪽 이상 | `exam-kor-4p.hwp` | min-pages 3 |
| LR46 | 3쪽 시험지 3쪽 이상 | `exam-kor-3p.hwp` | min-pages 3 |
| LR48 | 표 문서 100쪽 통과 수 | `table-001.hwp` | `passCount` |

**실패 모드**

- 위반 과제에 `"pass"` 를 박제한다. 판정 문자열은 라이브다.
- LR06(`verdict`) 의 답을 LR48(`passCount`) 에 넣는다. 필드가 다르다.
- `--expect-min-pages` 와 `--expect-min-tables` 를 한 과제에서 빼먹는다.
  LR08·LR27 은 둘 다 켠다.

### J5. 썸네일 픽셀 (`thumbnail`)

objects-media 의 미리보기 여정과 명령은 같다. 이 pack 은 조판 판정의
보조 축으로 픽셀을 읽는다.

| ID | 하는 일 | 표본 | 지목 |
|---|---|---|---|
| LR05 | 다쪽 문서 세로 | `issue2007_…` | `height` |
| LR28 | 표 문서 가로 | `table-001.hwp` | `width` |
| LR29 | 문단 문서 MIME | `para-001.hwp` | `mime` |
| LR41 | 가로 문서 가로 | `landscape-001.hwp` | `width` |

**실패 모드**

- 가로 문서의 가로 픽셀을 세로 문서에서 복사한다.
- MIME 을 확장자만 적는다.

### J6. 자기 라운드트립 (`render-diff`)

인자 하나면 자기 자신과의 렌더 비교다. 코퍼스 pack 의 `render-diff A B`
와 다르다.

| ID | 하는 일 | 표본 | 지목 |
|---|---|---|---|
| LR04 | 표 문서 회귀 | `table-001.hwp` | `status` |
| LR30 | 문단 문서 회귀 | `para-001.hwp` | `status` |
| LR31 | 2쪽 시험지 회귀 | `exam-kor-2p.hwp` | `status` |
| LR32 | 가로 문서 회귀 | `landscape-001.hwp` | `status` |
| LR47 | 실문서 회귀 | `143E433F503322BD33.hwp` | `status` |

**실패 모드**

- exit 3/4 를 실패로 위장한다. `expect_exits` 는 0, 3, 4 다.
- `maxDisp` 를 0 으로 박제한다. 이 pack 의 라운드트립은 `status` 만
  읽는다. 변위 0 고정은 corpus-diagnostics 의 자기비교 과제다.

### J7. 첫 쪽을 SVG 로 보낸다 (`export-svg -p 0`)

`-p` 는 **0 기준**이다. `extract-pages --from` 의 1 기준과 섞지 마라.

| ID | 하는 일 | 표본 | 산출 |
|---|---|---|---|
| LR03 | 표 문서 첫 쪽 | `table-001.hwp` | `svg/table-001.svg` |
| LR07 | 실문서 첫 쪽 | `143E433F503322BD33.hwp` | `svg/143E….svg` |
| LR33 | 문단 문서 첫 쪽 | `para-001.hwp` | `svg/para-001.svg` |
| LR34 | 1쪽 시험지 첫 쪽 | `exam-kor-1p.hwp` | `svg/exam-kor-1p.svg` |
| LR35 | 가로 문서 첫 쪽 | `landscape-001.hwp` | `svg/landscape-001.svg` |
| LR42 | 2쪽 시험지 첫 쪽 | `exam-kor-2p.hwp` | `svg/exam-kor-2p.svg` |

**실패 모드**

- `-p 1` 을 넣어 둘째 쪽을 렌더한다. 첫 쪽은 `-p 0` 이다.
- 원본 HWP 를 `svg/` 아래 복사한다. `xml_root_eq` 가 `svg` 루트를 요구한다.
- 빈 파일을 확장자만 `.svg` 로 낸다. `minBytes` 512 가 거절한다.

## 실패 모드 카탈로그 (여정 공통)

### F1. 쪽 축 혼동

| 명령 | 쪽 기준 | 비고 |
|---|---|---|
| `info` / `verify --expect-min-pages` | 쪽 **수** | 1, 2, 3… |
| `export-svg -p` | **0 기준** 단일 쪽 | 첫 쪽은 0 |
| `extract-pages --from/--to` (CD pack) | **1 기준** | 첫 쪽은 1 |

### F2. 종료 코드로 판정

| 상황 | exit | 읽을 필드 |
|---|---|---|
| 기대 충족 | 0 | `verdict` / `status` |
| 기대 위반 · IR/레이아웃 차이 | 3 | `verdict` / `status` |
| 쪽수 자기검증 불일치 | 4 | (이 pack 은 `--verify-pages` 없음) |

### F3. 새 CLI 금지

`info` · `verify` · `thumbnail` · `render-diff` · `export-svg` 로 닫힌다.
골든 PNG/SVG 바이트를 저장소에 넣지 않는다.

## 표본 지도

| 경로 | 쓰는 여정 |
|---|---|
| `samples/table-001.hwp` | J1–J7 |
| `samples/para-001.hwp` | J1, J2, J3, J5, J6, J7 |
| `samples/landscape-001.hwp` | J1, J2, J3, J5, J6, J7 |
| `samples/exam-kor-1p.hwp` … `4p.hwp` | J1, J2, J4, J6, J7 |
| `samples/143E433F503322BD33.hwp` | J1, J6, J7 |
| `samples/basic/issue2007_nested_cell_pagination_42065.hwp` | J1, J4, J5 |
| `samples/basic/english.hwp` | J1, J2, J3 |
| `samples/hwpx/form-01.hwpx` · `basic-table-01.hwpx` | J3 |

## 관련 문서

- 작업 노트: [mydocs/working/gym_om_lr_cd.md](../../../mydocs/working/gym_om_lr_cd.md)
- 개체 pack: [../objects-media/README.md](../objects-media/README.md)
- 코퍼스 pack: [../corpus-diagnostics/README.md](../corpus-diagnostics/README.md)
