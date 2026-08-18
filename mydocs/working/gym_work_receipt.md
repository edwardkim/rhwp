---
kind: working
status: active
canonical: mydocs/working/gym_work_receipt.md
last_verified: 2026-08-18
issue: 5231
pr: 5238
---

# gym work-receipt pack 확장 작업 노트 (PR #5238)

## 한 줄

`feat/gym-work-receipt-pack` 가 WR01–WR04 (약 401줄)에서 멈춰 있던 일일
영수증 여정을, 기존 연산자·기존 표본·기존 명령만으로 WR05–WR56 과 README·
가드 시험까지 늘린다. 새 PR 을 열지 않고 같은 브랜치에 얹는다.

## 배경

이슈 #5231 · PR #5238 초안은 네 축만 과제화했다.

- WR01 일일 영수증 3해시 (replay 라이브 오라클)
- WR02 첫 영수증은 뿌리 (depth 1 · parentOk null)
- WR03 어제 산출 = 오늘 입력 (lineageOk · parentOk · brokenAt null)
- WR04 오늘 폴더 회계 1건 (total==1 · reproduced==1)

pack 은 여전히 "4 과제" 이고, 발급/검증 모드, 거짓 주장 기각, 다른 실문서
반복, 캡슐 내부 자리, 형제 회계가 비어 있었다. 집계 문서(gym/README, PARK,
다른 프로파일)는 초안이 손대지 않았고 이번에도 손대지 않는다.
`gym/profiles/maintainer.json` 은 이미 `work-receipt` 를 정렬해 넣고 있어
그대로 둔다.

## 범위

포함한 것:

- `gym/packs/work-receipt/README.md` — pack 온램프·과제 지도·함정·재현
- `gym/packs/work-receipt/tasks/WR05.json` … `WR56.json`
- `gym/packs/work-receipt/reference/WR05.json` … `WR56.json`
- `scripts/tests/test_gym_work_receipt_pack.py` — 확장 계약 가드
- `mydocs/working/gym_work_receipt.md` — 이 노트

넣지 않은 것:

- 새 연산자 (`checks.py` 미변경)
- 새 CLI · 새 표본 · 새 pack
- XC01–XC05 복제 (`conformance` · `recall-scope` · `settle` · lineage `--deep`
  depth 3 · `audit-report`)
- T07 서식 채움 복제 (`fill-fields`, 첫 필드 홍길동)
- AU02 (`reproducedRate==1.0` + `total≥1`), AU07 (`gate --deep`),
  AU12 (`depth==2`)
- `gym/README.md` · `gym/PARK.md` · 다른 `profiles/*.json`
- `cargo fmt --all` (JSON·문서만)
- 프로파일 과제 수 문구

## 설계 원칙

1. **라이브 오라클.** `answer_eq` 는 채점 시점 CLI 재계산. 박제 숫자는 모드
   상수(`attest`/`verify`), null 자리, 정확 건수, 플래그 에코뿐이다.
2. **축을 가른다.** WR01 의 3해시를 모드·버전·스키마·입력 에코·검증 기각·
   두 걸음 steps 로 쪼갠다. WR02 의 뿌리를 다른 표본·캡슐 내부 kind/parent/
   receipt.* 로 쪼갠다. WR03 의 부모 링크를 다른 표본·상대 경로·파일 상이로
   쪼갠다. WR04 의 1건 회계를 실패 목록·형제 2/3건으로 쪼갠다.
3. **표본을 흩는다.** field-01 한 파일에 질문을 몰지 않는다. 메모·서식1·
   서식2·표·중첩 표 규제 표본은 이미 `samples/` 에 있다.
4. **정확 건수가 판정한다.** 비율 1.0 에 하한 1건을 짝으로 두지 않는다.
5. **T07 금지.** 이 pack 은 채우지 않는다. 누름틀은 본문 치환의 재료일 뿐이다.
6. **사다리 금지.** `--deep`, 서명, 앵커, 정산, 적합성, 리콜을 넣지 않는다.

## 과제 묶음

| 구간 | 축 | 건수 | 핵심 필드 |
|------|----|------|-----------|
| WR01–WR04 | 개장 코어 | 4 | 3해시 · 뿌리 · 부모 링크 · 1건 회계 |
| WR05–WR18 | 단건 영수증 | 14 | mode · toolVersion · schemaVersion · reproduced null/false · 다른 표본 |
| WR19–WR31 | 뿌리 캡슐 | 13 | parentOk null · kind · parent null · receipt.* · 얕은 reproduced |
| WR32–WR37 | 어제-오늘 | 6 | lineageOk · parent.capsule · files_differ · valid |
| WR38–WR48 | 폴더 회계 | 11 | total==N · reproduced==N · failed==[] |
| WR49–WR56 | 라이브·상이 | 8 | steps 라이브 · value_in · plan.input/output · 3형제 상이 |

WR01–WR04 는 그대로 둔다. 번호 구멍 없음 (1…56).

## 사용한 기존 표본

- `samples/field-01.hwp` — 본문 `마케팅` (FJ04 와 같은 근거)
- `samples/field-01-memo.hwp` — 누름틀 메모. 채우지 않고 영수증만
- `samples/form-01.hwp` · `samples/form-02.hwp` — 서식. T07 복제 아님
- `samples/table-001.hwp` — 자동화 pack 표본이지만 표 좌표를 묻지 않음
- `samples/basic/issue2007_nested_cell_pagination_42065.hwp` — 본문 `규제`

## 과제 목록 (WR05+)

- **WR05** (tier 1) 발급 모드 attest 상수 — `samples/field-01.hwp` · 연산자 value_eq · 명령 replay.
- **WR06** (tier 2) 도구 버전 라이브 대조 — `samples/field-01.hwp` · 연산자 answer_eq · 명령 replay.
- **WR07** (tier 1) 영수증 스키마 버전 라이브 — `samples/field-01.hwp` · 연산자 answer_eq · 명령 replay.
- **WR08** (tier 1) 영수증 입력 경로 에코 — `samples/field-01.hwp` · 연산자 value_eq · 명령 replay.
- **WR09** (tier 2) 발급 영수증 reproduced 는 null — `samples/field-01.hwp` · 연산자 value_eq · 명령 replay.
- **WR10** (tier 2) 발급 영수증 expectedOutputSha256 는 null — `samples/field-01.hwp` · 연산자 value_eq · 명령 replay.
- **WR11** (tier 2) 출근장 치환 3해시 — `samples/field-01.hwp` · 연산자 answer_eq · 명령 replay.
- **WR12** (tier 2) 규제 표본 3해시 — `samples/basic/issue2007_nested_cell_pagination_42065.hwp` · 연산자 answer_eq · 명령 replay.
- **WR13** (tier 3) 두 걸음 계획 step 수 — `samples/field-01.hwp` · 연산자 value_eq · 명령 replay.
- **WR14** (tier 3) 거짓 산출 주장 기각 — `samples/field-01.hwp` · 연산자 value_eq · 명령 replay.
- **WR15** (tier 2) 검증 모드 verify 상수 — `samples/field-01.hwp` · 연산자 value_eq · 명령 replay.
- **WR16** (tier 2) 거짓 주장 해시 에코 — `samples/field-01.hwp` · 연산자 value_eq · 명령 replay.
- **WR17** (tier 2) 마감장 치환 3해시 — `samples/field-01.hwp` · 연산자 answer_eq · 명령 replay.
- **WR18** (tier 2) 메모 표본 3해시 — `samples/field-01-memo.hwp` · 연산자 answer_eq · 명령 replay.
- **WR19** (tier 2) 서식1 뿌리는 parentOk null — `samples/form-01.hwp` · 연산자 file_exists, value_eq · 명령 lineage.
- **WR20** (tier 2) 규제 표본 뿌리 깊이 1 — `samples/basic/issue2007_nested_cell_pagination_42065.hwp` · 연산자 file_exists, value_eq · 명령 lineage.
- **WR21** (tier 1) 캡슐 kind 는 workCapsule — `samples/field-01.hwp` · 연산자 file_exists, json_value_eq · 명령 (파일).
- **WR22** (tier 1) 뿌리 캡슐 parent 필드는 null — `samples/field-01.hwp` · 연산자 file_exists, json_value_eq · 명령 (파일).
- **WR23** (tier 2) 캡슐 속 영수증 모드 attest — `samples/field-01.hwp` · 연산자 file_exists, json_value_eq · 명령 (파일).
- **WR24** (tier 2) 캡슐 속 영수증 steps 1 — `samples/field-01.hwp` · 연산자 file_exists, json_value_eq · 명령 (파일).
- **WR25** (tier 1) 캡슐 속 계획 planVersion 1.0 — `samples/field-01.hwp` · 연산자 file_exists, json_value_eq · 명령 (파일).
- **WR26** (tier 2) 서식2 뿌리 계보 유효 — `samples/form-02.hwp` · 연산자 file_exists, value_eq · 명령 lineage.
- **WR27** (tier 3) 얕은 계보 reproduced 는 null — `samples/field-01.hwp` · 연산자 file_exists, value_eq · 명령 lineage.
- **WR28** (tier 3) 뿌리 링크 lineageOk 는 null — `samples/field-01.hwp` · 연산자 file_exists, value_eq · 명령 lineage.
- **WR29** (tier 2) 메모 표본 뿌리 깨진 지점 없음 — `samples/field-01-memo.hwp` · 연산자 file_exists, value_eq · 명령 lineage.
- **WR30** (tier 2) 캡슐 속 입력 경로 에코 — `samples/field-01.hwp` · 연산자 file_exists, json_value_eq · 명령 (파일).
- **WR31** (tier 2) 표 표본 뿌리 parentOk null — `samples/table-001.hwp` · 연산자 file_exists, value_eq · 명령 lineage.
- **WR32** (tier 3) 메모 표본 어제 산출이 오늘 입력 — `samples/field-01-memo.hwp` · 연산자 file_exists, value_eq · 명령 lineage.
- **WR33** (tier 3) 오늘 캡슐이 어제 파일을 부모로 기록 — `samples/field-01.hwp` · 연산자 file_exists, json_value_eq · 명령 (파일).
- **WR34** (tier 2) 오늘 캡슐 속 영수증도 attest — `samples/field-01.hwp` · 연산자 file_exists, json_value_eq · 명령 (파일).
- **WR35** (tier 3) 어제는 뿌리·오늘은 부모 객체 — `samples/field-01.hwp` · 연산자 file_exists, json_value_eq · 명령 (파일).
- **WR36** (tier 3) 규제 표본 어제 파일 무결 — `samples/basic/issue2007_nested_cell_pagination_42065.hwp` · 연산자 file_exists, value_eq · 명령 lineage.
- **WR37** (tier 3) 어제-오늘 계보는 유효하고 안 깨졌다 — `samples/field-01.hwp` · 연산자 file_exists, value_eq · 명령 lineage.
- **WR38** (tier 2) 규제 표본 오늘 폴더 1건 — `samples/basic/issue2007_nested_cell_pagination_42065.hwp` · 연산자 file_exists, value_eq · 명령 audit.
- **WR39** (tier 2) 오늘 폴더 실패 목록은 빈 배열 — `samples/field-01.hwp` · 연산자 file_exists, value_eq · 명령 audit.
- **WR40** (tier 3) 형제 두 장 폴더 회계는 정확히 2건 — `samples/field-01.hwp` · 연산자 file_exists, value_eq · 명령 audit.
- **WR41** (tier 3) 형제 두 장 실패 목록은 비었다 — `samples/field-01.hwp` · 연산자 file_exists, value_eq · 명령 audit.
- **WR42** (tier 3) 형제 세 장 폴더 회계는 정확히 3건 — `samples/field-01.hwp` · 연산자 file_exists, value_eq · 명령 audit.
- **WR43** (tier 2) 메모 표본 오늘 폴더 1건 — `samples/field-01-memo.hwp` · 연산자 file_exists, value_eq · 명령 audit.
- **WR44** (tier 1) 감사 봉투 root 에코 — `samples/field-01.hwp` · 연산자 file_exists, value_eq · 명령 audit.
- **WR45** (tier 2) 계보 봉투 스키마 버전 라이브 — `samples/field-01.hwp` · 연산자 answer_eq, file_exists · 명령 lineage.
- **WR46** (tier 2) 감사 봉투 스키마 버전 라이브 — `samples/field-01.hwp` · 연산자 answer_eq, file_exists · 명령 audit.
- **WR47** (tier 2) 표 표본 오늘 폴더 1건 — `samples/table-001.hwp` · 연산자 file_exists, value_eq · 명령 audit.
- **WR48** (tier 2) 서식1 오늘 폴더 실패 없음 — `samples/form-01.hwp` · 연산자 file_exists, value_eq · 명령 audit.
- **WR49** (tier 2) 발급 영수증 steps 라이브 — `samples/field-01.hwp` · 연산자 answer_eq · 명령 replay.
- **WR50** (tier 1) 발급 모드 허용 집합 — `samples/field-01.hwp` · 연산자 value_in · 명령 replay.
- **WR51** (tier 2) 캡슐 속 계획 input 에코 — `samples/field-01.hwp` · 연산자 file_exists, json_value_eq · 명령 (파일).
- **WR52** (tier 2) 캡슐 속 계획 output 에코 — `samples/field-01.hwp` · 연산자 file_exists, json_value_eq · 명령 (파일).
- **WR53** (tier 3) 형제 두 장은 서로 다른 바이트 — `samples/field-01.hwp` · 연산자 file_exists, files_differ · 명령 (파일).
- **WR54** (tier 3) 어제와 오늘 캡슐은 서로 다르다 — `samples/field-01.hwp` · 연산자 file_exists, files_differ · 명령 (파일).
- **WR55** (tier 2) 서식2 오늘 폴더 1건 — `samples/form-02.hwp` · 연산자 file_exists, value_eq · 명령 audit.
- **WR56** (tier 4) 형제 세 장 모두 서로 다르다 — `samples/field-01.hwp` · 연산자 file_exists, files_differ · 명령 (파일).

## 검증

저장소 루트에서:

```
python gym/tools/audit.py
python -m unittest scripts/tests/test_gym_packs.py
python -m unittest scripts/tests/test_gym_work_receipt_pack.py
```

기대: audit 전 pack 통과, test_gym_packs 기존 건수 유지, work-receipt 가드
green. 바이너리·네트워크 없이 돈다. `cargo fmt --all` 생략 (sparse, Rust
변경 없음).

## 위험

- WR12·WR20·WR31·WR36·WR38·WR47 는 규제 표본의 `규제` 문자열에 기대한다.
  TE01 과 같은 근거다. 문구가 빠지면 빈 치환이 되거나 실패할 수 있다.
- WR03 계열의 `links[1].lineageOk` 는 `tests/lineage_contract.rs` 의 2링크
  봉투 좌표다. 링크 순서가 바뀌면 라이브 채점이 깨진다. 그래도 depth==2 를
  묻지는 않는다.
- WR14–WR16 의 거짓 해시는 64자리 0 이다. `expect_exits: [3]` 이 필수다.
- 이 세션은 스키마·audit·가드 시험만 돌린다. 로컬 `rhwp` 바이너리로 기준
  풀이를 라이브 채점하지 못할 수 있다.

## 커밋·푸시

같은 브랜치 `feat/gym-work-receipt-pack`. 새 PR 없음. `git add -A` 금지.
한글 커밋. `upstream/devel` 대비 insertions ≥ 3000.
