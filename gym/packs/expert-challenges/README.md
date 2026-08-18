---
kind: guide
status: active
canonical: gym/packs/expert-challenges/README.md
last_verified: 2026-08-18
---

# expert-challenges — 보스 어트랙션 (고난도 완주)

## 왜 이 pack 인가

놀이공원 한쪽 끝의 급류·자이로드롭이다. 검증 사다리의 여러 단을 **한 제출**에
묶고, 어느 한 단계만 빠지면 최종 판정이 막힌다. 부분 점수가 없다.

devel 에 이미 있던 XC01–XC05 는 개장 보스 다섯 대다.

| ID | 개장 보스 | 이 확장이 복제하지 않는 지문 |
|----|-----------|------------------------------|
| XC01 | 사다리 완주 L5 | `conformance --level L5 --deep` + 원장 |
| XC02 | 오염 리콜 드릴 | `unaffected==0` 과 `affected>=2` 동시 |
| XC03 | 정산 완주 4관문 | capsuleOk · gateOk · ledgerOk · workorderOk |
| XC04 | 계보 완주 3세대 | `lineage --deep` + `depth==3` |
| XC05 | 감사 표준 L3 | `L3 --deep` + `audit-report` |

이 확장(XC06–XC55)은 그 다섯 지문을 베끼지 않는다. 깊이 4·5, 중간 오염,
분기 형제, L4 게이트(원장 없음), 청구 verdict 만, 끝칸+계보 동시처럼
**다른 묶음**만 둔다. AU14+ 한 축 저티어 여정, WR 일일 3해시, T07 서식
채움(`fields[0]==홍길동`)도 복제하지 않는다.

권위 출처: `mydocs/manual/cli_commands.md` §계획 실행·증명·감사,
스킬 `rhwp-work-receipt`, 에이전트 작업 표준 AW-L1~AW-L5.

## 이 확장이 지키는 규칙

1. **기존 명령만.** `pack.json` requires 는 `keygen` · `replay` · `run` ·
   `anchor` · `settle` · `conformance` · `recall-scope` · `lineage` ·
   `audit-report` · `export-tables` 이다. 서식 채움·필드 조사·inspect·scan
   을 채점 명령으로 부르지 않는다.
2. **기존 연산자만.** `gym/core/checks.py` REGISTRY 에 있는 것만 고른다.
   `value_eq` · `value_ge` · `len_ge` · `file_exists` · `files_differ` ·
   `differs_from_input` · `cell_text_eq` 가 전부다. 새 연산자를 만들지 않는다.
3. **기존 표본만.** `samples/` 밖 파일을 만들지 않는다. table-001,
   table-004, multi-table-001/002, inner-table-01, hwp_table_test,
   table-complex, hwpx/basic-table-01 — 이미 있는 실문서를 보스 묶음으로
   다시 묻는다.
4. **라이브 오라클.** 해시·건수를 과제에 박제하지 않는다. 채점기가 같은
   명령을 다시 돌려 봉투 필드를 읽는다.
5. **T07 을 복제하지 않는다.** 서식 채움·첫 필드 홍길동은 core-cli 의
   일이다. 이 pack 은 누름틀을 채우지 않는다.
6. **XC01–XC05 를 복제하지 않는다.** 위 표의 지문을 그대로 옮기지 않는다.
7. **AU14+ · WR01+ 를 복제하지 않는다.** 한 축 저티어 여정과 일일 영수증
   3해시 answer.json 은 다른 pack 의 일이다.
8. **runner 를 복사만 한다.** `pack.json` 의 `rhwpVersion` · `rhwpCommit` ·
   `capabilitiesSha256` 을 바꾸지 않는다. 이 확장은 바이너리를 바꾸지 않는다.
9. **원본을 덮지 않는다.** 산출은 제출 폴더의 `-o` / `--capsule` 뿐이다.

## 명령 표면 (pack.json requires)

| 명령 | 하는 일 | 이 확장에서 읽는 봉투 |
|------|---------|------------------------|
| `lineage` | 머리→뿌리 계보 | `valid` · `depth` (4 또는 5, 3 금지) |
| `recall-scope` | 오염 후손 폐쇄 | `affected` (unaffected 와 묶지 않음) |
| `conformance` | L2 / L4 자가진단 | `verdict` (L5·L3 --deep 콤보 금지) |
| `settle verify` | 청구 3해시 | `verdict==ok` (4관문 path 금지) |
| `settle record` | 원장 기입 | `file_exists` ledger (관문 path 없음) |
| `audit-report` | 감사 보고서 | `lineage.valid` (L3 와 묶지 않음) |
| `anchor verify` | 등재 | `logged` |
| `export-tables` | 표 좌표 텍스트 | `cell_text_eq` |
| `keygen` · `replay` · `run` | 기준풀이 산출 | 채점 명령이 아님 |

`--deep` 은 4·5세대 계보와 L2/L4 재현에만 쓴다. 3세대 계보와 L3/L5
완주에는 붙이지 않는다.

## 함정 (실측, 과제에 녹여 둔 것)

- **3대에서 멈추면 깊이 4가 아니다.** XC06·XC18 은 네 대를 잇는다.
- **중간 세대를 오염으로 지목하면 뿌리와 건수가 다르다.** XC13·XC35·XC49.
- **형제는 직선이 아니다.** XC15·XC44·XC52 는 같은 부모에서 갈라진다.
- **L4 는 원장이 없다.** L5 로 올리면 `--ledger` 가 필수다 (XC16·XC50).
- **얕은 L4 정책은 lineageValid** 다. reproduced 를 요구하면 재료 미지정으로
  거부된다 (XC17).
- **서로 무관한 캡슐은 리콜 영향권이 1** 이다. 부모 링크가 필요하다.
- **비밀키를 제출하지 마라.** 키링에는 공개키만 넣는다.
- **T07 금지.** 누름틀 값을 넣지 마라.

## 과제 지도

난도 4=고급 · 5=보스. XC01–XC05 는 그대로 둔다. 번호 구멍 없음 (1…55).

### XC01–XC05 — 개장 보스 (devel 에 이미 있음)

| ID | 티어 | 질문 | 표본 |
|----|------|------|------|
| XC01 | 5 | L5 완주 | table-001 |
| XC02 | 5 | 오염 무영향 0 + 영향 2+ | table-001 |
| XC03 | 4 | 정산 4관문 + 원장 | table-001 |
| XC04 | 4 | 3세대 lineage --deep | table-001 |
| XC05 | 5 | L3 --deep + 감사 표준 | table-001 |

### XC06–XC10 — 깊은·얕은 계보 (깊이 4·5, 3 금지)

| ID | 티어 | 질문 | 표본 |
|----|------|------|------|
| XC06 | 5 | 4세대 깊은 계보 | table-001 |
| XC07 | 5 | 5세대 깊은 계보 | table-001 |
| XC08 | 4 | 4세대 얕은 계보 | table-004 |
| XC09 | 4 | 5세대 얕은 계보 | multi-table-001 |
| XC10 | 4 | 내부표 4세대 유효만 (깊이 숫자 고정 없음) | inner-table-01 |

### XC11–XC15 — 리콜 (unaffected 콤보 금지)

| ID | 티어 | 질문 | 표본 |
|----|------|------|------|
| XC11 | 5 | 4세대 뿌리 리콜 4건+ | table-001 |
| XC12 | 5 | 5세대 뿌리 리콜 5건+ | table-004 |
| XC13 | 5 | 중간 세대 오염 폐쇄 3건+ | multi-table-001 |
| XC14 | 4 | 잎 캡슐만 회수 | inner-table-01 |
| XC15 | 5 | 형제 분기 뿌리 리콜 3건+ | table-001 |

### XC16–XC20 — L4 / L2 (L5·L3 콤보 금지)

| ID | 티어 | 질문 | 표본 |
|----|------|------|------|
| XC16 | 5 | L4 --deep 4세대 게이트 | table-001 |
| XC17 | 4 | L4 얕은 4세대 (lineageValid 정책) | table-004 |
| XC18 | 5 | L4 와 4세대 계보 동시 | multi-table-001 |
| XC19 | 4 | L2 --deep 과 4세대 계보 | inner-table-01 |
| XC20 | 5 | L4 와 4세대 리콜 동시 | table-001 |

### XC21–XC25 — 서명·앵커·보고 (L3 콤보 금지)

| ID | 티어 | 질문 | 표본 |
|----|------|------|------|
| XC21 | 5 | 서명 4세대 깊은 계보 | hwp_table_test |
| XC22 | 4 | 쌍 캡슐 각각 앵커 | table-004 |
| XC23 | 5 | 4연속 앵커와 계보 | multi-table-001 |
| XC24 | 4 | 감사보고와 4세대 계보 | table-001 |
| XC25 | 4 | 감사보고와 4세대 리콜 | inner-table-01 |

### XC26–XC30 — 정산 (4관문 금지)

| ID | 티어 | 질문 | 표본 |
|----|------|------|------|
| XC26 | 5 | 청구 verdict 와 4세대 계보 | table-001 |
| XC27 | 4 | 원장 실재와 4세대 계보 | table-004 |
| XC28 | 5 | 청구 verdict 와 4세대 리콜 | multi-table-001 |
| XC29 | 5 | 청구 verdict 와 L4 | table-001 |
| XC30 | 4 | 청구 검증과 4세대 유효 | hwpx/basic-table-01 |

### XC31–XC40 — 표본을 옮긴 보스

| ID | 티어 | 질문 | 표본 |
|----|------|------|------|
| XC31 | 5 | 표사 4세대 깊은 계보 | table-004 |
| XC32 | 5 | 다중표 4세대 리콜 | multi-table-001 |
| XC33 | 5 | 내부표 L4 깊은 게이트 | inner-table-01 |
| XC34 | 5 | 표시험 5세대 깊은 계보 | hwp_table_test |
| XC35 | 5 | 복합표 중간 오염 폐쇄 | table-complex |
| XC36 | 5 | HWPX 4세대 깊은 계보 | hwpx/basic-table-01 |
| XC37 | 5 | 표사 L4 깊은 4세대 | table-004 |
| XC38 | 5 | 다중표2 5세대 리콜 | multi-table-002 |
| XC39 | 5 | 내부표 정산과 4세대 | inner-table-01 |
| XC40 | 4 | 표사 감사보고와 4세대 | table-004 |

### XC41–XC45 — 끝칸·분기 산출

| ID | 티어 | 질문 | 표본 |
|----|------|------|------|
| XC41 | 5 | 4세대 끝칸과 깊은 계보 | table-001 |
| XC42 | 5 | 5세대 끝칸과 리콜 | table-004 |
| XC43 | 5 | 표사 끝칸 서명 4세대 | table-004 |
| XC44 | 4 | 분기 두 잎 산출 상이 | table-001 |
| XC45 | 4 | 4세대 원본 차이와 계보 | multi-table-002 |

### XC46–XC55 — 혼합 보스

| ID | 티어 | 질문 | 표본 |
|----|------|------|------|
| XC46 | 4 | 독립 두 사슬 각각 유효 | table-001 |
| XC47 | 5 | 쌍 사슬 L4 깊은 게이트 | inner-table-01 |
| XC48 | 5 | 원장·보고·4세대 동시 | table-001 |
| XC49 | 5 | 5세대 중간 오염과 계보 | hwp_table_test |
| XC50 | 5 | L4 리콜 4세대 서명 완주 (L5 아님) | table-001 |
| XC51 | 5 | 끝칸과 L4 4세대 동시 | multi-table-001 |
| XC52 | 5 | 세 갈래 뿌리 리콜 | table-001 |
| XC53 | 4 | L2 재현 5세대 끝칸 | table-complex |
| XC54 | 5 | 5연속 앵커와 5세대 계보 | table-004 |
| XC55 | 5 | 표사 정산 L4 4세대 | table-004 |

## 재현

기준 풀이 왕복은 저장소 단독으로 돈다.

```text
python gym/tools/build_baseline.py --agent baseline --pack expert-challenges
python gym/score.py --agent baseline --pack expert-challenges
python gym/tools/audit.py
python scripts/tests/test_gym_packs.py
python scripts/tests/test_gym_expert_challenges_pack.py
```

바이너리 없이 구조 감사와 pack 가드는 파일만으로 통과한다. 기준풀이 왕복은
로컬 `rhwp` 가 있을 때 돌린다.

## 이 확장이 하지 않는 것

- 새 CLI · 새 연산자 · 새 표본 · 새 pack
- `pack.json` runner 신원 변경
- `gym/README.md` · `gym/PARK.md` · `profiles/*.json` 집계 갱신
- 다른 pack 의 과제 파일
- `cargo fmt --all` (JSON·문서·테스트만)
- T07 / AU14+ / WR / XC01–XC05 복제
