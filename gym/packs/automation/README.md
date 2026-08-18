---
kind: guide
status: active
canonical: gym/packs/automation/README.md
last_verified: 2026-08-18
---

# automation — 자동화·검증 사다리

## 왜 이 pack 인가

제출이 곧 증명이다. 에이전트가 한 일을 말로 주장하지 않고, 사다리의 판정
명령 한 번으로 채점한다. 계획서 원자 실행 → 작업 캡슐 → 서명 귀속 →
앵커 등재 → 적합성 → 선택적 공개 → 관문 → 정산 청구 → 감사 보고 →
오염 범위 → 연합 번들 → 계보.

이 pack 은 그 축들을 **기존 CLI 표면**만으로 과제화한다. 새 pack 도, 새
명령도, 새 연산자도, 새 픽스처도 없다. `pack.json` 의 `runner` 신원은
그대로 둔다 — 이 확장은 바이너리를 바꾸지 않는다.

권위 출처: `mydocs/manual/cli_commands.md` §계획 실행·증명·감사,
스킬 `rhwp-work-receipt`, 에이전트 작업 표준 AW-L1~AW-L5.

## 이 확장이 지키는 규칙

1. **기존 명령만.** `pack.json` requires 는 `anchor` · `audit` ·
   `audit-report` · `bundle` · `conformance` · `disclose` · `export-tables` ·
   `gate` · `lineage` · `recall-scope` · `settle` · `verify-signature` 이다.
   서식 채움·필드 조사·inspect·scan 을 채점 명령으로 부르지 않는다.
   기준 풀이가 산출을 만들 때 쓰는 `run` · `replay` · `keygen` 은
   채점 명령이 아니다.
2. **기존 연산자만.** `gym/core/checks.py` REGISTRY 에 있는 것만 고른다.
   `cell_text_eq` · `differs_from_input` · `value_eq` · `value_ge` ·
   `len_ge` · `file_exists` · `files_differ` · `same_hash` 가 전부다.
3. **기존 표본만.** `samples/` 밖 파일을 만들지 않는다. table-001,
   table-004, multi-table-001/002, inner-table-01, hwp_table_test,
   table-complex, hwpx/basic-table-01 — 이미 있는 실문서를 축만 바꿔
   다시 묻는다.
4. **라이브 오라클.** 해시·건수를 과제에 박제하지 않는다. 채점기가 같은
   명령을 다시 돌려 봉투 필드를 읽는다.
5. **T07 을 복제하지 않는다.** 서식 채움·첫 필드 홍길동은 core-cli 의
   일이다. 이 pack 은 누름틀을 채우지 않는다.
6. **XC01–XC05 를 복제하지 않는다.** L5 완주, 오염 무영향 0, 정산 4관문
   + 원장, 3세대 lineage --deep, L3 서명+앵커 완주는 expert-challenges
   의 보스다. 여기서는 한 축만 지목한다.
7. **WR01+ 를 복제하지 않는다.** 일일 영수증 3해시 answer.json 여정은
   work-receipt pack 의 일이다. 이 pack 은 캡슐·관문·정산 사다리다.
8. **원본을 덮지 않는다.** 산출은 제출 폴더의 `-o` / `--capsule` 뿐이다.

## 명령 표면 (pack.json requires)

| 명령 | 하는 일 | 이 pack 에서 읽는 봉투 |
|------|---------|------------------------|
| `export-tables` | 표 좌표 텍스트 | `tables[i].cells` → `cell_text_eq` |
| `audit` | 폴더 직속 캡슐 전수 재현 | `reproducedRate` · `total` |
| `verify-signature` | 분리 서명 검증 | `verdict` |
| `anchor verify` | 등재·체인 무결 | `logged` · `logChainOk` |
| `conformance` | L1~L2 자가진단 | `verdict` |
| `disclose restore` | 가림 복원 | `byteIdentical` |
| `gate` | 반입 관문 | `verdict` |
| `settle verify` | 청구 3해시 | `verdict` |
| `audit-report` | 감사 보고서 | `lineage.valid` |
| `recall-scope` | 오염 후손 폐쇄 | `affected` |
| `bundle verify` | 오프라인 교환 | `containerOk` · `closureOk` |
| `lineage` | 머리→뿌리 계보 | `valid` · `depth` (기존 AU12) |

`--deep` 은 관문 일부(AU07, AU52, AU54, AU56)에만 쓴다. 적합성 L3/L5 와
3세대 계보에는 붙이지 않는다.

## 함정 (실측, 과제에 녹여 둔 것)

- **좌표가 바뀌면 다른 계약이다.** AU01 의 (0,0) 제출을 AU14 (0,1) 이나
  AU15 (1,0) 에 재사용하면 cell_text_eq 가 실패한다.
- **표본이 바뀌면 산출 해시가 다르다.** table-001 산출을 table-004 과제에
  복사하면 differs_from_input 은 통과할 수 있어도 표 내용이 틀리다.
- **audit 대상은 폴더 직속 `*.capsule.json`** (비재귀). 0개면 실패. AU02
  는 `total≥1` 이고 AU19+ 는 `total==1` 이다. 두 장을 넣으면 정확 건수가
  어긋난다.
- **비밀키를 제출하지 마라.** 키링에는 공개키만 넣는다.
- **앵커 로그를 손으로 편집하면 체인이 끊긴다.** `anchor add` 한 번이
  등재다.
- **L1 과 L2 는 누적이다.** L2 를 건너뛰어 L5 로 가지 마라. 그 완주는
  XC01 이다.
- **가림본이 원본과 같으면 가림이 일어나지 않은 것이다.**
- **관문 default 는 deny.** reproduced==true 규칙이 있어야 allow 다.
- **정산 원장은 이 확장의 범위가 아니다.** AU08/AU57+ 는 settle verify
  의 `verdict==ok` 만 본다. 4관문+ledger 는 XC03.
- **서로 무관한 두 캡슐은 리콜 영향권이 1 이다.** 부모 링크가 필요하다.
- **T07 금지.** 누름틀 값을 넣지 마라.

## 과제 지도

난도 1=입문 · 2=초급 · 3=중급 · 4=고급 · 5=보스. 보스 완주는 XC 의
일이다. 이 pack 의 확장은 2~3 에 머문다.

### AU01–AU13 — 개장 코어 (devel 에 이미 있음)

| ID | 티어 | 질문 | 표본 |
|----|------|------|------|
| AU01 | 2 | 계획서 원자 실행 (0,0)=계획실행 | table-001 |
| AU02 | 2 | 작업 영수증 · 재현율 1.0 + total≥1 | table-001 |
| AU03 | 3 | 서명 귀속 valid | table-001 |
| AU04 | 3 | 앵커 등재 logged + 체인 | table-001 |
| AU05 | 2 | 적합성 L1 | table-001 |
| AU06 | 3 | 선택적 공개 왕복 | table-001 |
| AU07 | 3 | 관문 allow --deep | table-001 |
| AU08 | 3 | 정산 청구 verdict ok | table-001 |
| AU09 | 3 | 감사 보고 lineage.valid=1 | table-001 |
| AU10 | 3 | 오염 리콜 affected≥2 | table-001 |
| AU11 | 3 | 연합 번들 container+closure | table-001 |
| AU12 | 3 | 계보 체인 depth 2 | table-001 |
| AU13 | 2 | 적합성 L2 | table-001 |

### AU14–AU18 — 계획서 좌표를 가른다

AU01 은 (0,0) 한 칸이다. 옆칸·둘째 행·다른 표 표본으로 좌표를 흩는다.

| ID | 질문 |
|----|------|
| AU14 | table-001 (0,1) = 옆칸확장 |
| AU15 | table-001 (1,0) = 둘째행확장 |
| AU16 | table-004 (0,0) = 표사다리 |
| AU17 | multi-table-001 첫 표 |
| AU18 | inner-table-01 첫 표 |

### AU19–AU26 — 감사 정확 1건

AU02 는 total≥1 이라 두 장을 넣어도 통과한다. 여기서는 **정확 1건** 과
다른 표본·다른 폴더 이름을 묻는다.

| ID | 질문 |
|----|------|
| AU19 | table-004 · capsules/ · total==1 |
| AU20 | multi-table-001 · total==1 |
| AU21 | inner-table-01 · work/ |
| AU22 | hwp_table_test |
| AU23 | table-complex · receipts/ |
| AU24 | hwpx/basic-table-01 |
| AU25 | multi-table-002 |
| AU26 | table-001 · ledger/ · total==1 (AU02 와 비교) |

### AU27–AU32 — 서명 귀속, 키 id 를 가른다

AU03 은 gym-agent. 표본과 키 id 만 바꾼다. L3 앵커 완주(XC05) 는 하지
않는다.

| ID | 키 id |
|----|-------|
| AU27 | au-sign-t004 |
| AU28 | au-sign-mt1 |
| AU29 | au-sign-inner |
| AU30 | au-sign-ttest |
| AU31 | au-sign-hwpx |
| AU32 | au-sign-complex |

### AU33–AU38 — 앵커 축을 가른다

AU04 는 logged + logChainOk 를 한 번에 본다. 여기서는 등재·체인·파일
실재를 가르고 표본을 옮긴다.

| ID | 축 |
|----|-----|
| AU33 | table-004 logged |
| AU34 | 다중표 logChainOk |
| AU35 | 내부표 logged |
| AU36 | 표시험 logChainOk |
| AU37 | HWPX logged |
| AU38 | 복합표 로그 파일 + logged |

### AU39–AU45 — 적합성 L1/L2 표본 확장

AU05=L1, AU13=L2, 둘 다 table-001. L3 --deep 과 L5 는 XC 다.

| ID | 등급 | 표본 |
|----|------|------|
| AU39 | L1 | table-004 |
| AU40 | L2 | multi-table-001 |
| AU41 | L1 | inner-table-01 · work/ |
| AU42 | L2 | hwp_table_test |
| AU43 | L1 | HWPX |
| AU44 | L2 | table-complex · ladder/ |
| AU45 | L1 | multi-table-002 |

### AU46–AU51 — 선택적 공개 축

AU06 은 restore + same_hash + files_differ 세 검사. 여기서는 축을 가르고
표본을 옮긴다.

| ID | 축 |
|----|-----|
| AU46 | table-004 byteIdentical |
| AU47 | 다중표 files_differ |
| AU48 | 내부표 same_hash |
| AU49 | 표시험 opening 실재 |
| AU50 | HWPX byteIdentical |
| AU51 | 복합표 files_differ |

### AU52–AU56 — 관문, 깊이와 규칙 id

AU07 은 --deep + R1-재현. 얕은 관문과 다른 규칙 id 를 섞는다.

| ID | 깊이 | 규칙 |
|----|------|------|
| AU52 | --deep | R-표사-재현 |
| AU53 | 얕음 | R-다중-재현 |
| AU54 | --deep | R-내부-재현 |
| AU55 | 얕음 | R-시험-재현 |
| AU56 | --deep | R-hwpx-재현 |

### AU57–AU61 — 정산 청구 (원장 없음)

AU08 은 gym-wo-1. 원장 4관문은 XC03.

| ID | workorderId |
|----|-------------|
| AU57 | gym-wo-t004 |
| AU58 | gym-wo-mt1 + claim 실재 |
| AU59 | gym-wo-inner |
| AU60 | gym-wo-ttest + claim 실재 |
| AU61 | gym-wo-hwpx |

### AU62–AU64 — 감사 보고

| ID | 질문 |
|----|------|
| AU62 | table-004 lineage.valid=1 |
| AU63 | 다중표 report.json 실재 |
| AU64 | 내부표 work/ 보고 |

### AU65–AU67 — 2링크 리콜 (무영향 0 을 묻지 않음)

| ID | 폴더 |
|----|------|
| AU65 | table-004 · capsules/ |
| AU66 | 다중표 · chain/ |
| AU67 | 내부표 · capsules/ |

### AU68–AU70 — 연합 번들 축

| ID | 축 |
|----|-----|
| AU68 | table-004 containerOk · domain=gym-t004 |
| AU69 | 다중표 closureOk · gym-mt1 |
| AU70 | 내부표 번들 실재 · gym-inner |

## 기준 풀이 왕복

```bash
python gym/tools/build_baseline.py --agent baseline --pack automation --bin target/debug/rhwp
python gym/score.py --agent baseline --pack automation --bin target/debug/rhwp
```

`reference/AU*.json` 은 정답 노출이다. 푸는 쪽은 보지 않는다. 채점 재현용이다.
runner 신원은 기존 `pack.json` 을 그대로 쓴다.

## 정합 가드

```bash
python gym/tools/audit.py
python -m unittest scripts/tests/test_gym_packs.py
python -m unittest scripts/tests/test_gym_automation_pack.py
```

`test_gym_automation_pack.py` 는 AU14+ 짝 기준풀이, 기존 연산자만, 기존
표본만, pack requires 밖 명령 금지, T07/XC/WR 복제 금지를 파일만으로
고정한다.

## 이 문서가 말하지 않는 것

- gym/README.md · PARK.md · profiles/*.json 의 과제 수 집계는 후속이다.
- `checks.py` · 새 연산자 · 새 CLI 는 이 pack 의 범위가 아니다.
- work-receipt pack 의 일일 3해시 여정, expert-challenges 의 보스 완주는
  각자 자리에서 다룬다.

## 재현 (사람용 한 줄)

계획서를 원자 실행하고, 캡슐을 남기고, 필요한 축(서명·앵커·적합·가림·
관문·청구·보고·리콜·번들)만 판정 명령으로 닫는다. 숫자가 아니라 봉투가
게이트다.
