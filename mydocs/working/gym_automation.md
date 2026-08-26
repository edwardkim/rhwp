---
kind: working
status: active
canonical: mydocs/working/gym_automation.md
last_verified: 2026-08-18
issue: 5257
---

# gym automation pack 확장 작업 노트 (이슈 #5257)

## 한 줄

`feat/gym-automation-expand` 가 AU01–AU13 에서 멈춰 있던 검증 사다리
여정을, 기존 명령·기존 표본만으로 AU14–AU70 과 README·가드 시험까지
늘린다. 새 CLI 는 없다.

## 배경

사다리 pack 은 계획·캡슐·서명·앵커·적합·가림·관문·정산·보고·리콜·번들·
계보를 한 바퀴 돌리지만, 표본이 table-001 한 장에 몰려 있고 같은 봉투
필드를 한 과제에 묶어 두었다. 에이전트가 AU01–AU13 힌트를 외우면 축
전체를 통과한다. 이슈 #5257 은 기존 명령만으로 여정을 늘리라고 한다.

건드리지 않은 것:

- 새 연산자 (`checks.py` 미변경)
- 새 CLI · 새 표본 · 새 pack
- T07 서식 채움 복제 (`fill-fields`, 첫 필드 홍길동)
- XC01–XC05 복제 (L5, 오염 무영향 0, 정산 4관문, 3세대 --deep, L3 완주)
- WR01+ 복제 (일일 영수증 3해시 answer.json)
- `gym/README.md` · `gym/PARK.md` · `profiles/*.json` 집계 갱신
- 다른 pack 의 과제 파일
- `cargo fmt --all` (JSON·문서·테스트만)
- `pack.json` 의 `runner` 신원 (`rhwpVersion` / `rhwpCommit` /
  `capabilitiesSha256`). 요구 명령 목록도 기존 값을 유지했다.

## 설계 원칙

1. **한 축만 지목한다.** AU04 의 logged+logChainOk, AU06 의 restore+
   hash+differ, AU11 의 container+closure 를 쪼개 표본을 옮긴다. 한
   숫자가 나와도 추측으로 차이를 만들지 않고 봉투를 옮긴다.
2. **표본을 흩는다.** table-001 한 장에 질문을 몰지 않는다. table-004,
   multi-table-001/002, inner-table-01, hwp_table_test, table-complex,
   hwpx/basic-table-01. 전부 `samples/` 에 이미 있다.
3. **정확 건수.** AU02 는 `total≥1` 이라 두 장도 통과한다. AU19+ 는
   `total==1` 로 속임수를 거절한다.
4. **보스를 훔치지 않는다.** L5 · 3세대 --deep · 원장 4관문 · 무영향 0
   콤보는 XC 의 자이로드롭이다.
5. **T07 금지.** 누름틀을 채우지 않는다.

## 과제 묶음

| 구간 | 축 | 건수 | 핵심 필드 |
|------|----|------|-----------|
| AU14–AU18 | 계획·표 좌표 | 5 | `cell_text_eq` · `differs_from_input` |
| AU19–AU26 | 감사 정확 1건 | 8 | `reproducedRate` · `total==1` |
| AU27–AU32 | 서명 귀속 | 6 | `verify-signature.verdict` |
| AU33–AU38 | 앵커 | 6 | `logged` · `logChainOk` |
| AU39–AU45 | 적합성 L1/L2 | 7 | `conformance.verdict` |
| AU46–AU51 | 선택적 공개 | 6 | `byteIdentical` · `same_hash` · `files_differ` |
| AU52–AU56 | 관문 | 5 | `gate.verdict` · `--deep` / 얕음 |
| AU57–AU61 | 정산 청구 | 5 | `settle.verify.verdict` |
| AU62–AU64 | 감사 보고 | 3 | `audit-report` · `lineage.valid` |
| AU65–AU67 | 2링크 리콜 | 3 | `recall-scope.affected` |
| AU68–AU70 | 연합 번들 | 3 | `containerOk` · `closureOk` |

AU01–AU13 은 그대로 둔다. 번호 구멍 없음 (1…70).

## 사용한 기존 표본

- `samples/table-001.hwp`
- `samples/table-004.hwp`
- `samples/multi-table-001.hwp`
- `samples/multi-table-002.hwp`
- `samples/inner-table-01.hwp`
- `samples/hwp_table_test.hwp`
- `samples/table-complex.hwp`
- `samples/hwpx/basic-table-01.hwpx`

## 검증

```bash
python gym/tools/audit.py
python -m unittest scripts/tests/test_gym_packs.py
python -m unittest scripts/tests/test_gym_automation_pack.py
```

바이너리 없이 파일 계약만 닫는다. 기준 풀이 왕복(`build_baseline.py`)은
로컬에 `target/debug/rhwp` 가 있을 때 선택이다. 이 작업은 JSON·문서·
테스트만 바꾸므로 `cargo fmt --all` 을 돌리지 않는다.

## 산출물

- `gym/packs/automation/README.md`
- `gym/packs/automation/tasks/AU14.json` … `AU70.json`
- `gym/packs/automation/reference/AU14.json` … `AU70.json`
- `scripts/tests/test_gym_automation_pack.py`
- `mydocs/working/gym_automation.md` — 이 노트

`pack.json` 은 runner 복사가 이미 되어 있어 손대지 않았다.
