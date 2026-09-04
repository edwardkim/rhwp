---
kind: working
status: active
canonical: mydocs/working/gym_expert_challenges.md
last_verified: 2026-08-18
issue: 5261
---

# gym expert-challenges pack 확장 작업 노트 (이슈 #5261)

## 한 줄

`feat/gym-expert-challenges-expand` 가 XC01–XC05 에서 멈춰 있던 보스
어트랙션을, 기존 명령·기존 표본만으로 XC06–XC55 과 README·가드 시험까지
늘린다. 새 CLI 는 없다.

## 배경

보스 pack 은 사다리 여러 단을 한 제출에 묶는다. 개장 다섯 대(XC01–XC05)는
L5 완주 · 오염 무영향 0 · 정산 4관문 · 3세대 --deep · L3 서명+앵커 완주라는
지문이 강해, 에이전트가 그 힌트만 외우면 보스 존을 통과한다. 이슈 #5261 은
그 지문을 복제하지 말고 여정을 늘리라고 한다.

건드리지 않은 것:

- 새 연산자 (`checks.py` 미변경)
- 새 CLI · 새 표본 · 새 pack
- T07 서식 채움 복제 (`fill-fields`, 첫 필드 홍길동)
- XC01–XC05 복제 (L5, 오염 무영향 0, 정산 4관문, 3세대 --deep, L3 완주)
- AU14+ 복제 (한 축 저티어, tier≤3)
- WR01+ 복제 (일일 영수증 3해시 answer.json)
- `gym/README.md` · `gym/PARK.md` · `profiles/*.json` 집계 갱신
- 다른 pack 의 과제 파일
- `cargo fmt --all` (JSON·문서·테스트만)
- `pack.json` 의 `runner` 신원 (`rhwpVersion` / `rhwpCommit` /
  `capabilitiesSha256`). 요구 명령 목록도 기존 값을 유지했다.

## 설계 원칙

1. **묶음을 바꾼다.** XC04 가 3세대 --deep 이면 XC06 은 4세대, XC07 은
   5세대다. XC02 가 unaffected+affected 면 XC11+ 는 affected 만 본다.
   XC03 이 4관문이면 XC26+ 는 verdict 또는 원장 실재만 본다. XC01 이 L5
   면 XC16+ 는 L4 (원장 없음) 다. XC05 가 L3+보고면 XC24+ 는 보고만.
2. **표본을 흩는다.** table-001 한 장에 질문을 몰지 않는다. table-004,
   multi-table-001/002, inner-table-01, hwp_table_test, table-complex,
   hwpx/basic-table-01. 전부 `samples/` 에 이미 있다.
3. **티어는 4~5.** AU14+ 가 비운 보스 칸만 채운다.
4. **보스를 훔치지 않는다.** L5 · 3세대 --deep · 원장 4관문 · 무영향 0
   콤보는 개장 다섯 대의 것이다.
5. **T07 금지.** 누름틀을 채우지 않는다.

## 과제 묶음

| 구간 | 축 | 건수 | 핵심 필드 |
|------|----|------|-----------|
| XC06–XC10 | 계보 깊이 4·5 | 5 | `lineage.valid` · `depth`≠3 |
| XC11–XC15 | 리콜 | 5 | `affected` (unaffected 없음) |
| XC16–XC20 | L4 / L2 | 5 | `conformance.verdict` |
| XC21–XC25 | 서명·앵커·보고 | 5 | `logged` · `lineage.valid` |
| XC26–XC30 | 정산 | 5 | `settle.verify.verdict` · ledger 실재 |
| XC31–XC40 | 표본 이동 | 10 | 위 축을 다른 실문서에 |
| XC41–XC45 | 끝칸·분기 | 5 | `cell_text_eq` · `files_differ` |
| XC46–XC55 | 혼합 보스 | 10 | 두 축 이상 동시 |

XC01–XC05 는 그대로 둔다. 번호 구멍 없음 (1…55).

과제 ID 전체: XC06 XC07 XC08 XC09 XC10 XC11 XC12 XC13 XC14 XC15 XC16
XC17 XC18 XC19 XC20 XC21 XC22 XC23 XC24 XC25 XC26 XC27 XC28 XC29 XC30
XC31 XC32 XC33 XC34 XC35 XC36 XC37 XC38 XC39 XC40 XC41 XC42 XC43 XC44
XC45 XC46 XC47 XC48 XC49 XC50 XC51 XC52 XC53 XC54 XC55.

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

파일만으로 도는 가드:

```text
python gym/tools/audit.py
python scripts/tests/test_gym_packs.py
python scripts/tests/test_gym_expert_challenges_pack.py
```

`audit.py` 는 전 pack 정합(짝 기준풀이·ID 고유·스키마)을 본다.
`test_gym_packs.py` 는 전 pack 계약이다. pack 전용 가드는 XC06+ 50건,
runner 복사, T07/AU14+/WR/XC01-05 복제 금지, 기존 표본·기존 연산자,
문서에 모든 새 ID 가 있는지를 고정한다.

기준풀이 왕복(`build_baseline.py --pack expert-challenges`)은 로컬
바이너리가 있을 때 추가로 돈다. 이 확장은 바이너리를 바꾸지 않으므로
`cargo fmt --all` 을 돌리지 않았다.

## 크기

이슈 DoD 는 `upstream/devel` 대비 insertions ≥ 3000 이다. XC06–XC55
과제+기준풀이 50쌍, pack README, pack 가드, 이 작업 노트가 그 문을
채운다. `git add -A` 는 쓰지 않았고 다른 pack 은 편집하지 않았다.
