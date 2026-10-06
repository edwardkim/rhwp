---
kind: guide
status: active
canonical: gym/packs/corpus-diagnostics/README.md
last_verified: 2026-08-18
---

# corpus-diagnostics — 폴더 스윕·쪽 덤프·비교·추출 여정

이 pack은 **문서 한 건이 아니라 무더기와 이상 좁히기**를 채점한다.
폴더를 깊이·탐침으로 스캔하고, 쪽 덤프를 세고, 같은 파일·짝 파일·다른
파일을 IR/렌더로 맞대고, 쪽 범위를 잘라 내고, 형식 변환 계약을 확인한다.
새 CLI는 없다. 기존 `scan` · `dump-pages` · `ir-diff` · `render-diff` ·
`extract-pages` · `convert` · `info` 와 `samples/` 만 쓴다.

채점은 라이브 오라클이다. 폴더 파일 수·차이 건수·동일 여부를 JSON 에
박제하지 않는다. `samples/hwpx` 에 문서가 더해지거나 빠지면 CD08·CD15
정답도 같이 바뀐다. 그게 의도다.

## 왜 이 pack 인가

에이전트가 "폴더를 훑었다"거나 "쪽을 잘랐다"고 말하는 순간, 보통 다섯
가지 중 하나가 빠진다.

1. **깊이를 무시했다.** `scan` 기본은 재귀다. `--max-depth 1` 은 한 단만
   센다. `samples/hwpx` 아래 `hancom-hwp` · `opengov` · `ref` 가 깊이 2
   에서 더 잡힌다.
2. **탐침을 생략했다.** 확장자 불일치는 `--probe` 없이는 안 보인다.
3. **차이를 실패로 위장했다.** `ir-diff` 의 exit 3 은 차이 발견이다.
   산출은 남고 `identical` · `diffCount` 가 실린다.
4. **쪽을 0 기준으로 잘랐다.** `extract-pages --from/--to` 는 **1 기준**
   이다. `export-svg -p 0` 과 섞으면 한 쪽 밀린다.
5. **추출 쪽수가 요청 폭과 같다고 가정했다.** 추출물은 다시 조판된다.
   단일 쪽 추출의 결과 쪽수는 1 이어야 한다는 계약만 고정한다.

이 pack의 과제는 위 구멍을 여정으로 나눈다.

## 여정 지도

### J1. 폴더를 센다 (`scan` · `--max-depth`)

입력 파일은 자리만 차지한다. 실제 대상은 `samples/` 아래 폴더다.

| ID | 하는 일 | 대상 | 깊이 |
|---|---|---|---|
| CD01 | hml 전수 | `samples/hml` | 기본(재귀) |
| CD08 | hwpx 1단 | `samples/hwpx` | 1 |
| CD11 | basic 1단 | `samples/basic` | 1 |
| CD12 | chart 1단 | `samples/chart` | 1 |
| CD13 | unicode 1단 | `samples/unicode` | 1 |
| CD15 | hwpx 2단 | `samples/hwpx` | 2 |
| CD35 | images 1단 | `samples/images` | 1 |
| CD39 | issues 1단 | `samples/issues` | 1 |
| CD43 | hml 1단 | `samples/hml` | 1 |

채점은 `len_answer_eq` 로 `files` 배열 길이를 센다. 숫자를 박제하지
않는다.

**실패 모드**

- CD08 의 답을 CD15 에 복사한다. 깊이가 다르다.
- CD01 의 답을 CD43 에 복사한다. 재귀와 1단이 다르다.
- 폴더 안의 PDF·TXT 를 손으로 세고 확장자 필터를 추측한다. `scan` 이
  세는 집합이 정답이다.

### J2. 확장자를 탐침한다 (`scan --probe`)

| ID | 하는 일 | 대상 | 지목 |
|---|---|---|---|
| CD02 | hml 첫 문서 | `samples/hml` | `files[0].extMismatch` |
| CD14 | hwpx 첫 문서 | `samples/hwpx` | `files[0].extMismatch` |
| CD48 | basic 첫 문서 | `samples/basic` | `files[0].extMismatch` |

**실패 모드**

- `--probe` 없이 `extMismatch` 를 읽으려 한다.
- `files[0]` 이 정렬 불안정이라고 추측해 값을 박제한다. 라이브 오라클이
  같은 명령을 다시 돌린다.

### J3. 쪽 덤프를 센다 (`dump-pages` · `pageCount`)

`info.pageCount` 와 같은 숫자일 때가 많지만, 이 여정은 **덤프가 잡은
쪽 수**다. 레이아웃 pack 의 `info` 과제와 명령을 바꾸지 마라.

| ID | 하는 일 | 표본 |
|---|---|---|
| CD03 | 다쪽 실측 | `issue2007_…42065.hwp` |
| CD16 | 표 문서 | `table-001.hwp` |
| CD17 | 2쪽 시험지 | `exam-kor-2p.hwp` |
| CD18 | 3쪽 시험지 | `exam-kor-3p.hwp` |
| CD19 | 문단 문서 | `para-001.hwp` |
| CD37 | 가로 문서 | `landscape-001.hwp` |
| CD44 | 4쪽 시험지 | `exam-kor-4p.hwp` |

**실패 모드**

- 파일 이름의 `2p`·`3p`·`4p` 를 그대로 적는다.
- `info` 로 잰 값을 `dump-pages` 과제에 넣는다. 명령이 다르다.

### J4. IR 을 맞댄다 (`ir-diff`)

자기 자신, 같은 문서의 HWP·HWPX 쌍, 서로 다른 두 문서를 나눈다.
`identical` 과 `diffCount` 는 다른 필드다.

| ID | 하는 일 | 대상 | 지목 |
|---|---|---|---|
| CD04 | 143E HWP·HWPX | `143E…hwp` + `.hwpx` | `identical` |
| CD09 | 표 문서 자기대조 | `table-001.hwp` A=A | `identical` |
| CD20 | 문단 문서 자기대조 | `para-001.hwp` | `identical` |
| CD21 | 2쪽 시험지 자기대조 | `exam-kor-2p.hwp` | `identical` |
| CD22 | 2쪽 시험지 쌍 | `.hwp` + `.hwpx` | `identical` |
| CD23 | OSS 문서 쌍 | `2026_oss_rst` 쌍 | `identical` |
| CD36 | 가로 문서 자기대조 | `landscape-001.hwp` | `identical` |
| CD40 | 143E 차이 건수 | 같은 143E 쌍 | `diffCount` |
| CD45 | 1쪽 시험지 자기대조 | `exam-kor-1p.hwp` | `identical` |
| CD47 | 표 vs 문단 | `table-001` vs `para-001` | `identical` |

자기대조 과제는 `answer_eq` 와 `value_eq true` 를 같이 둔다. 자기
자신이 `false` 이면 파서 비결정성이다.

**실패 모드**

- exit 3 을 실패로 위장한다. `expect_exits` 는 0 과 3 이다.
- CD04 의 `identical` 을 CD40 의 `diffCount` 칸에 넣는다.
- 서로 다른 두 문서가 같다고 박제한다. 라이브 값이 정답이다.

### J5. 렌더를 맞댄다 (`render-diff A B`)

레이아웃 pack 의 인자 하나(자기 라운드트립)와 다르다. 이 여정은
**두 경로**를 준다. 같은 파일을 두 번 넣으면 변위 0 이어야 정상이다.

| ID | 하는 일 | 표본 | 지목 |
|---|---|---|---|
| CD05 | 표 문서 A=A | `table-001.hwp` | `status` + `maxDisp` 0 |
| CD24 | 문단 문서 A=A | `para-001.hwp` | `status` |
| CD25 | 1쪽 시험지 A=A | `exam-kor-1p.hwp` | `status` + `maxDisp` 0 |

**실패 모드**

- 인자 하나짜리 `render-diff` 를 돌린다. 이 여정은 A B 다.
- `maxDisp` 를 박제하지 말아야 할 과제에 숫자를 넣는다. CD05·CD25 만
  자기비교 계약으로 0 을 고정한다.

### J6. 쪽 범위를 자른다 (`extract-pages` · 1 기준)

단일 쪽 추출의 결과 쪽수는 1, 원본과 바이트가 달라야 한다.

| ID | 하는 일 | 표본 | 범위 |
|---|---|---|---|
| CD06 | 다쪽 첫 쪽 | `issue2007_…` | 1–1 |
| CD10 | 다쪽 둘째 쪽 | `issue2007_…` | 2–2 |
| CD26 | 2쪽 시험지 첫 쪽 | `exam-kor-2p.hwp` | 1–1 |
| CD27 | 2쪽 시험지 둘째 쪽 | `exam-kor-2p.hwp` | 2–2 |
| CD28 | 3쪽 시험지 첫 쪽 | `exam-kor-3p.hwp` | 1–1 |
| CD29 | 3쪽 시험지 마지막 | `exam-kor-3p.hwp` | 3–3 |
| CD30 | 4쪽 시험지 첫 쪽 | `exam-kor-4p.hwp` | 1–1 |
| CD41 | 4쪽 시험지 둘째 쪽 | `exam-kor-4p.hwp` | 2–2 |
| CD46 | 4쪽 시험지 마지막 | `exam-kor-4p.hwp` | 4–4 |

**실패 모드**

- `--from 0` 을 넣는다. 사용법 오류이거나 한 쪽 밀린다.
- `--from 1` 을 "둘째 쪽"으로 읽는다. 1 이 첫 쪽이다.
- 원본을 이름만 바꿔 제출한다. `differs_from_input` 이 거절한다.
- 빈 파일을 확장자만 `.hwp` 로 낸다. `minBytes` 1024 가 거절한다.

### J7. 형식 변환 계약 (`convert` + `info.format`)

변환물이 실제로 열리고, 형식 표기를 봉투에서 읽는다. 골든 HWP 바이트는
없다.

| ID | 하는 일 | 표본 |
|---|---|---|
| CD07 | 표 문서 | `table-001.hwp` |
| CD31 | 문단 문서 | `para-001.hwp` |
| CD32 | 가로 문서 | `landscape-001.hwp` |
| CD33 | 1쪽 시험지 | `exam-kor-1p.hwp` |
| CD34 | HWPX 표 | `hwpx/basic-table-01.hwpx` |
| CD38 | 실문서 | `143E433F503322BD33.hwp` |
| CD42 | aift | `aift.hwp` |

**실패 모드**

- `"hwp"` 를 형식 칸에 적는다. 표기는 보통 `hwp5` 다.
- 입력 `.hwpx` 를 이름만 `.hwp` 로 바꾼다. `convert` 가 컨테이너를
  다시 써야 한다.
- 변환 산출 없이 `answer.json` 만 낸다. `file_exists` 가 거절한다.

## 실패 모드 카탈로그 (여정 공통)

### F1. 폴더 입력을 파일로 착각

과제 `input` 은 스키마가 요구하는 자리일 뿐, 스윕 대상이 아니다.
`scan` 의 첫 인자는 `samples/hml` 같은 폴더다.

### F2. 쪽 축 혼동

| 명령 | 기준 |
|---|---|
| `extract-pages --from/--to` | **1 기준** |
| `export-svg -p` (LR pack) | **0 기준** |
| `dump-pages` / `info` | 쪽 **수** |

### F3. 종료 코드로 판정

| 상황 | exit | 읽을 필드 |
|---|---|---|
| 동일 · 변환 성공 | 0 | `identical` / `format` |
| IR 차이 | 3 | `identical` / `diffCount` |
| 렌더 차이 | 3 또는 4 | `status` / `maxDisp` |

### F4. 새 CLI · 새 표본 금지

기존 명령과 `samples/` 만 쓴다. 스윕 정답이 폴더 구성에 묶여 있다는
사실을 숨기지 않는다.

## 표본·폴더 지도

| 경로 | 쓰는 여정 |
|---|---|
| `samples/hml` | J1, J2 |
| `samples/hwpx` | J1, J2 |
| `samples/basic` · `chart` · `unicode` · `images` · `issues` | J1, J2 |
| `samples/table-001.hwp` · `para-001.hwp` · `landscape-001.hwp` | J3–J7 |
| `samples/exam-kor-1p.hwp` … `4p.hwp` | J3, J4, J6, J7 |
| `samples/basic/issue2007_nested_cell_pagination_42065.hwp` | J3, J6 |
| `samples/143E433F503322BD33.hwp` + `hwpx/` 짝 | J4, J7 |
| `samples/exam-kor-2p.hwp` + `hwpx/exam-kor-2p.hwpx` | J4 |
| `samples/2026_oss_rst.hwp` + `hwpx/` 짝 | J4 |
| `samples/hwpx/basic-table-01.hwpx` · `samples/aift.hwp` | J7 |

## 관련 문서

- 작업 노트: [mydocs/working/gym_om_lr_cd.md](../../../mydocs/working/archives/gym_om_lr_cd.md)
- 개체 pack: [../objects-media/README.md](../objects-media/README.md)
- 조판 pack: [../layout-rendering/README.md](../layout-rendering/README.md)
