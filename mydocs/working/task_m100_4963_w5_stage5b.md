---
kind: working-note
status: completed
issue: 4963
stage: W5-5B
last_verified: 2026-08-22
---

# Task M100 #4963 W5 Stage 5B — rank 16 read-only feature disposition

- **이슈**: [#4963](https://github.com/edwardkim/rhwp/issues/4963)
- **계획**: [`task_m100_4963.md`](../plans/archives/task_m100_4963.md)
- **선행 단계**: [`task_m100_4963_w5_stage5a.md`](task_m100_4963_w5_stage5a.md)
- **단계 상태**: read-only 실행·복원·공개 disposition·queue 정정 완료

## 1. 목적과 실행 경계

rank 16 `한컴 윤고딕 230`에 대해 Stage W5-3의 selection probe만으로 exact-installed profile을 발행하지
않고, 복원된 기준선에서 실제 문서 open과 PDF export까지 기능 탐지했다. font 설치·제거,
`AddFontResourceEx`, private corpus 접근, 제품 metric DB·fallback·paint 변경은 수행하지 않았다.

실행 전후에 같은 기준선으로 복원해 다음 불변식을 확인했다.

- baseline font manifest와 unrelated font projection 동일
- 관리 설치 font와 일회성 font resource 0개
- HWP 잔류 프로세스와 일회성 작업 0개
- fixture·PDF·관측 JSON의 SHA-256 연결
- raw VM/checkpoint 이름, 절대 경로, font bytes, private 문서 식별자 비공개

## 2. 기능 탐지 결과

| 관측면 | 결과 | 판정 |
| --- | --- | --- |
| 문서 face 선택 | `한컴 윤고딕 230` → `함초롬바탕`, font type 5 | exact 아님 |
| 영문 SFNT alias 선택 | `Haan YGodic 230` → 동일 이름, TTF | exact |
| HWPX open | 1 page, 비어 있지 않은 text | 통과 |
| PDF font | embedded subset `HCRBatang-Bold` 한 종 | exact source bytes 미사용 |
| private corpus | 접근 안 함 | 통과 |

이 결과는 “같은 font의 영문 alias를 선택할 수 있다”와 “HWPX에 기록된 한글 face 이름이 exact source로
조판된다”가 서로 다른 기능임을 보여준다. Stage W5-3의 단발 selection probe는 exact를 기록했지만,
복원 직후 문서 open·PDF export를 포함한 현재 관측은 이를 재현하지 못했다. 빌드 번호 분기를 추가하지
않고 더 강한 현재 기능 탐지를 우선한다.

## 3. 판정

exact-installed profile은 발행하지 않는다. PDF가 exact SFNT의 PostScript subset을 사용하지 않았으므로
source `hmtx`·outline과 PDF advance를 identity로 연결할 수 없기 때문이다. 대신
[`oracle_stage5_rank16_read_only_disposition.json`](../tech/investigations/issue-4963/oracle_stage5_rank16_read_only_disposition.json)에
`blocked-document-face-name-resolution`을 기록했다.

나머지 missing/substitution 질문도 이 read-only lane에서 억지로 채우지 않는다. ambient 영문 alias는
관리 집합 밖에 있고, 이번 승인 범위에는 제거 또는 별도 font resource 주입이 없었다.

## 4. queue 정정과 다음 게이트

[`oracle_stage5_queue_projection.json`](../tech/investigations/issue-4963/oracle_stage5_queue_projection.json)은
W5-5B로 갱신했다. rank 16은 terminal read-only capability mismatch가 되었고 actionable rank는 rank 8
하나만 남았다.

rank 16을 다시 열 수 있는 조건은 별도 승인된 controlled state에서 다음 둘을 동시에 관찰하는 것이다.

1. 문서 face `한컴 윤고딕 230`의 exact readback
2. PDF subset이 준비된 동일 SFNT bytes를 사용했다는 glyph·name·hash 연결

다음 절편은 rank 8의 distinct fixture substitution 계약과 mutable three-state ladder다. 이는 별도 승인
전에는 시작하지 않는다.
