---
kind: guide
status: active
canonical: gym/packs/work-receipt/README.md
last_verified: 2026-08-18
---

# work-receipt — 일일 작업 영수증 여정

## 왜 이 pack 인가

에이전트가 한 일을 **말이 아니라 재실행으로** 증명한다. 매일 밟는 길은 짧다.

1. 영수증 한 장 (`replay`) — 입력·계획·산출 SHA-256 3종
2. 뿌리 계보 (`lineage`) — 부모 없는 첫 캡슐, `parentOk==null`
3. 부모 링크 (`lineage` 둘째 자리) — 어제 산출이 오늘 입력, `lineageOk`
4. 폴더 회계 (`audit`) — 오늘 폴더의 **정확 건수**, 비율이 아님

automation·expert-challenges 가 다루는 사다리 완주(AU02 재현율 1.0+total≥1,
AU12 depth 2, AU07 gate --deep, XC01–XC05)는 복제하지 않는다. T07 서식
채움도 이 pack 의 일이 아니다. 새 CLI 는 없다.

권위 출처: 스킬 `rhwp-work-receipt`, `mydocs/manual/cli_commands.md` 의
replay·audit·lineage, 에이전트 작업 표준 AW-L1.

## 이 확장이 지키는 규칙

1. **기존 명령만.** `pack.json` requires 는 `audit` · `lineage` · `replay`
   뿐이다. `gate` · `conformance` · `settle` · `recall-scope` · `keygen` ·
   `fill-fields` · `fields` 를 부르지 않는다.
2. **기존 연산자만.** `gym/core/checks.py` REGISTRY 에 있는 것만 고른다.
   `answer_eq` · `value_eq` · `value_in` · `file_exists` · `json_value_eq` ·
   `files_differ` 가 전부다. 전역 훑기(`deep_contains`)는 쓰지 않는다.
3. **기존 표본만.** `samples/` 밖 파일을 만들지 않는다. field-01, field-01-memo,
   form-01, form-02, table-001, 중첩 표 규제 표본만 축을 바꿔 다시 묻는다.
4. **라이브 오라클.** 해시·버전 숫자를 과제에 박제하지 않는다. 채점기가 같은
   명령을 다시 돌려 봉투 필드를 읽는다.
5. **T07 을 복제하지 않는다.** 누름틀을 채우지 않는다. 첫 필드 홍길동을 묻지
   않는다.
6. **사다리를 완주하지 않는다.** `--deep` 재현, 적합성 L5/L3, 정산 4관문,
   오염 리콜, 3세대 depth 를 넣지 않는다.
7. **원본을 덮지 않는다.** 산출은 제출 폴더의 `-o` / `--capsule` 뿐이다.

## 명령 표면 (pack.json requires)

| 명령 | 하는 일 | 이 pack 에서 읽는 봉투 |
|------|---------|------------------------|
| `replay` | 단건 영수증 발급·제3자 검증 | `mode` · `steps` · `input` · `inputSha256` · `planSha256` · `outputSha256` · `toolVersion` · `schemaVersion` · `reproduced` · `expectedOutputSha256` |
| `lineage` | 머리부터 뿌리까지 계보 | `valid` · `depth` · `brokenAt` · `links[].parentOk` · `links[].lineageOk` · `links[].reproduced` |
| `audit` | 폴더 직속 캡슐 전수 재현 | `total` · `reproduced` · `failed` · `schemaVersion` |

`--deep` 은 이 pack 이 쓰지 않는다. 얕은 계보에서 `reproduced` 는 null 이다
(WR27). 거짓 주장 해시는 `reproduced==false` 이고 exit 3 이다 (WR14).

플래그 에코 필드(`mode`, `input`, `expectedOutputSha256`)는 값이 명령에서 온
것이라 박제가 아니라 계약 확인이다.

## 함정 (실측, 과제에 녹여 둔 것)

- **영수증 발급 중 `output` 경로의 실파일은 생기지 않는다.** 임시 재실행이다.
  실산출이 필요하면 `rhwp run` 을 따로 실행하라 (WR03·WR32 기준풀이).
- **`--expect-output-sha256` 이 없으면 `reproduced` 와 `expectedOutputSha256`
  은 null** 이다. false 가 아니다 (WR09·WR10).
- **캡슐은 발급 후 불변이다.** 에디터로 열어 저장하면 부모 해시 대조가 깨진다.
- **`--parent` 상대 경로는 캡슐 파일 기준**이다. 같은 폴더에 두는 것이 가장
  단순하다 (WR33).
- **audit 대상은 폴더 직속 `*.capsule.json`** (비재귀). 0개면 exit 2.
- **비율만 맞추는 제출을 거부한다.** AU02 는 `reproducedRate==1.0` 과
  `total≥1` 이다. 이 pack 은 `total==N` 과 `reproduced==N` 을 정확히 묻는다.
- **형제는 체인이 아니다.** 부모 없이 나란히 둔 두·세 장(WR40–WR42·WR53·WR56)
  은 depth 를 묻지 않는다.
- **T07 금지.** 누름틀 값을 넣지 마라. 영수증은 치환 계획의 해시만 본다.

## 과제 지도

난도 1=입문 · 2=초급 · 3=중급 · 4=고급. 보스(5) 사다리 완주는 XC 의 일이다.

### WR01–WR04 — 개장 코어 (초안)

| ID | 티어 | 질문 | 표본 |
|----|------|------|------|
| WR01 | 2 | 일일 영수증 3해시 라이브 | field-01.hwp |
| WR02 | 2 | 첫 영수증은 뿌리 (depth 1 · parentOk null) | field-01.hwp |
| WR03 | 3 | 어제 산출 = 오늘 입력 (lineageOk · parentOk) | field-01.hwp |
| WR04 | 2 | 오늘 폴더 회계 정확 1건 | field-01.hwp |

### WR05–WR56 — 일일 여정을 가른다

WR01–WR04 가 네 축의 입구다. 확장은 **같은 명령 가족**으로 모드·검증 기각·
다른 표본·캡슐 내부 자리·형제 회계를 가른다.

| ID | 티어 | 질문 | 표본 |
|----|------|------|------|
| WR05 | 1 | 발급 모드 attest 상수 | `samples/field-01.hwp` |
| WR06 | 2 | 도구 버전 라이브 대조 | `samples/field-01.hwp` |
| WR07 | 1 | 영수증 스키마 버전 라이브 | `samples/field-01.hwp` |
| WR08 | 1 | 영수증 입력 경로 에코 | `samples/field-01.hwp` |
| WR09 | 2 | 발급 영수증 reproduced 는 null | `samples/field-01.hwp` |
| WR10 | 2 | 발급 영수증 expectedOutputSha256 는 null | `samples/field-01.hwp` |
| WR11 | 2 | 출근장 치환 3해시 | `samples/field-01.hwp` |
| WR12 | 2 | 규제 표본 3해시 | `samples/basic/issue2007_nested_cell_pagination_42065.hwp` |
| WR13 | 3 | 두 걸음 계획 step 수 | `samples/field-01.hwp` |
| WR14 | 3 | 거짓 산출 주장 기각 | `samples/field-01.hwp` |
| WR15 | 2 | 검증 모드 verify 상수 | `samples/field-01.hwp` |
| WR16 | 2 | 거짓 주장 해시 에코 | `samples/field-01.hwp` |
| WR17 | 2 | 마감장 치환 3해시 | `samples/field-01.hwp` |
| WR18 | 2 | 메모 표본 3해시 | `samples/field-01-memo.hwp` |
| WR19 | 2 | 서식1 뿌리는 parentOk null | `samples/form-01.hwp` |
| WR20 | 2 | 규제 표본 뿌리 깊이 1 | `samples/basic/issue2007_nested_cell_pagination_42065.hwp` |
| WR21 | 1 | 캡슐 kind 는 workCapsule | `samples/field-01.hwp` |
| WR22 | 1 | 뿌리 캡슐 parent 필드는 null | `samples/field-01.hwp` |
| WR23 | 2 | 캡슐 속 영수증 모드 attest | `samples/field-01.hwp` |
| WR24 | 2 | 캡슐 속 영수증 steps 1 | `samples/field-01.hwp` |
| WR25 | 1 | 캡슐 속 계획 planVersion 1.0 | `samples/field-01.hwp` |
| WR26 | 2 | 서식2 뿌리 계보 유효 | `samples/form-02.hwp` |
| WR27 | 3 | 얕은 계보 reproduced 는 null | `samples/field-01.hwp` |
| WR28 | 3 | 뿌리 링크 lineageOk 는 null | `samples/field-01.hwp` |
| WR29 | 2 | 메모 표본 뿌리 깨진 지점 없음 | `samples/field-01-memo.hwp` |
| WR30 | 2 | 캡슐 속 입력 경로 에코 | `samples/field-01.hwp` |
| WR31 | 2 | 표 표본 뿌리 parentOk null | `samples/table-001.hwp` |
| WR32 | 3 | 메모 표본 어제 산출이 오늘 입력 | `samples/field-01-memo.hwp` |
| WR33 | 3 | 오늘 캡슐이 어제 파일을 부모로 기록 | `samples/field-01.hwp` |
| WR34 | 2 | 오늘 캡슐 속 영수증도 attest | `samples/field-01.hwp` |
| WR35 | 3 | 어제는 뿌리·오늘은 부모 객체 | `samples/field-01.hwp` |
| WR36 | 3 | 규제 표본 어제 파일 무결 | `samples/basic/issue2007_nested_cell_pagination_42065.hwp` |
| WR37 | 3 | 어제-오늘 계보는 유효하고 안 깨졌다 | `samples/field-01.hwp` |
| WR38 | 2 | 규제 표본 오늘 폴더 1건 | `samples/basic/issue2007_nested_cell_pagination_42065.hwp` |
| WR39 | 2 | 오늘 폴더 실패 목록은 빈 배열 | `samples/field-01.hwp` |
| WR40 | 3 | 형제 두 장 폴더 회계는 정확히 2건 | `samples/field-01.hwp` |
| WR41 | 3 | 형제 두 장 실패 목록은 비었다 | `samples/field-01.hwp` |
| WR42 | 3 | 형제 세 장 폴더 회계는 정확히 3건 | `samples/field-01.hwp` |
| WR43 | 2 | 메모 표본 오늘 폴더 1건 | `samples/field-01-memo.hwp` |
| WR44 | 1 | 감사 봉투 root 에코 | `samples/field-01.hwp` |
| WR45 | 2 | 계보 봉투 스키마 버전 라이브 | `samples/field-01.hwp` |
| WR46 | 2 | 감사 봉투 스키마 버전 라이브 | `samples/field-01.hwp` |
| WR47 | 2 | 표 표본 오늘 폴더 1건 | `samples/table-001.hwp` |
| WR48 | 2 | 서식1 오늘 폴더 실패 없음 | `samples/form-01.hwp` |
| WR49 | 2 | 발급 영수증 steps 라이브 | `samples/field-01.hwp` |
| WR50 | 1 | 발급 모드 허용 집합 | `samples/field-01.hwp` |
| WR51 | 2 | 캡슐 속 계획 input 에코 | `samples/field-01.hwp` |
| WR52 | 2 | 캡슐 속 계획 output 에코 | `samples/field-01.hwp` |
| WR53 | 3 | 형제 두 장은 서로 다른 바이트 | `samples/field-01.hwp` |
| WR54 | 3 | 어제와 오늘 캡슐은 서로 다르다 | `samples/field-01.hwp` |
| WR55 | 2 | 서식2 오늘 폴더 1건 | `samples/form-02.hwp` |
| WR56 | 4 | 형제 세 장 모두 서로 다르다 | `samples/field-01.hwp` |

## 축을 가르는 방법

- **발급 vs 검증.** attest (WR05) 와 verify (WR15) 는 같은 replay 의 다른
  모드다. 거짓 주장(WR14)은 기각이지 도구 고장이 아니다.
- **null vs false.** 주장 없는 발급의 `reproduced` 는 null (WR09). 거짓
  주장의 `reproduced` 는 false (WR14).
- **뿌리 vs 부모 링크.** 뿌리의 `parentOk`·`lineageOk` 는 null (WR02·WR28).
  둘째 링크만 true 를 갖는다 (WR03·WR32·WR36).
- **형제 vs 체인.** 형제(WR40–WR42)는 부모를 붙이지 않는다. 체인(WR03·WR32+)
  은 `--parent` 로 잇되 depth 숫자를 묻지 않는다.
- **정확 건수 vs 비율.** `total==N` 과 `reproduced==N` 만 쓴다.
  `reproducedRate` 와 `total≥1` 을 짝으로 두지 않는다.

## 기준 풀이 왕복

답 과제(WR01·WR05–WR18·WR45·WR46·WR49·WR50)의 기준풀이는 검사와 같은
`cmd`/`path` 를 가진다. 산출 과제는 `replay --capsule {sub:…}` 로 제출
파일을 만들고, 어제-오늘은 `run` 으로 실산출을 남긴 뒤 replay --parent 로
잇는다.

```bash
python gym/tools/audit.py
python -m unittest scripts/tests/test_gym_packs.py scripts/tests/test_gym_work_receipt_pack.py
```

바이너리 없이 스키마·짝 기준풀이·ID 고유만 검사한다. `cargo fmt --all` 은
JSON·문서만 고친  sparseness 때문에 생략한다.

## 이 pack 이 하지 않는 것

- 새 CLI · 새 연산자 · 새 표본 · 새 pack
- XC01 적합성 L5, XC02 오염 리콜, XC03 정산 4관문, XC04 3세대 --deep,
  XC05 감사표준 L3
- AU02 재현율+하한, AU07 gate --deep, AU12 depth==2
- T07 fill-fields · 첫 필드 홍길동
- `gym/README.md` · `gym/PARK.md` · 다른 프로파일 · `checks.py` 집계 갱신
