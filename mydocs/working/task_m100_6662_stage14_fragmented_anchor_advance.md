---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_6662_stage14_fragmented_anchor_advance.md
last_verified: 2026-09-06
---

# 열린 이슈 재검증 14단계: 가로 분할된 빈 그림 앵커

Issue: #6712. 시작 HEAD: `300bd354f`.

## 분석과 계획

- 13단계 전체 회귀는 9,086개 통과했지만 한국어 1쪽 마지막 예방 문단은 약 26.9px 위다.
  다른 관리 문단의 약 9.9px 오차와 구분하면 추가로 한 줄 약 17px가 빠져 있다.
- cell 3 p34의 두 LINE_SEG는 같은 vpos=54159, lh=800, ls=480이며 좌우 조각이다.
  다음 p35의 vpos=55559에는 이 1280 HU와 문단 앞 간격 120 HU가 포함된다.
  현재 렌더는 p34 y=902.3, p35 y=903.9로 문단 간격만 내고 빈 줄을 버린다.
- 단일 줄만 허용했던 빈 그림 앵커 판정을 한 시각 줄의 실제 가로 조각까지 확장한다.
  저장 사다리 차이가 줄 전진과 인접 문단 간격의 합으로 설명되는 경우만 허용한다.
  반환하는 전진에는 문단 간격을 포함하지 않아 caller와 중복 계산하지 않는다.
- 실제 fixture red/green 및 #6712, 관련 table/float, overlap/overflow/off-canvas 대조군을
  확인한다. 기존 baseline은 완화하지 않는다. 한국어 남은 약 9.9px와 중국어 테두리 문제는
  이 보정의 성공 여부와 별개로 남겨 둔다.

로그와 중간 SVG/PNG/JSON은 `/tmp`에만 두고, 결과 기록과 코드만 단계 커밋한다.

## 구현 및 재현 기록

- 공통 판정에 resolved 문단 스타일과 DPI를 전달한다. 모든 조각은 실제 저장 줄이고,
  같은 세로 좌표에 다른 가로 시작점, 같은 lh/ls를 가져야 한다.
- 다음 줄까지 저장 전진을 `lh + ls + spacing_after + next.spacing_before`와 비교한다.
  오차 허용은 0.02px 미만이며, 반환값은 lh+ls만 유지한다.
- 실물 계약에 한국어 p34를 추가했다. 최초 red 실행은 source ParaShape의 2배 간격
  스케일을 테스트에서 나누지 않아 1400 vs 1520 HU에서 실패했다. 이 실패는 구현 결함의
  red 증명으로 세지 않는다. 실제 수정 전 결함 근거는 13단계 트리의 p34/p35 간격 1.6px와
  저장 원장의 1400 HU(18.6667px) 차이다. 테스트에는 모델의 2배 스케일을 반영했다.

## 검증과 잔여 범위

- #6712 focused: 15 passed, 148 skipped, 0.245초, exit 0.
- #6712/#5862/#5863/#2007 및 text-overlap/off-canvas 대조군: 68 passed,
  1013 skipped, 69.758초, exit 0.
- standalone overflow target은 모듈명 test filter에 포함되지 않았으므로 별도로 실행했다.
  `--test overflow_cell_baseline`: 16 passed, 0 skipped, 50.351초, exit 0.
  합계 84개 서로 다른 대조군이며 focused 15개를 다시 더해 세지 않는다.
- 현재 CLI SHA-256:
  `9b83ff8593bf0eb0e8d9bab70a1ce506a51ac5441a7b4d785055accd026f6e4d`.
- 한국어 1~2쪽 visual sweep을 새로 실행해 exit 0 및 두 비교 PNG를 판독했다.
  p34/p35 간격은 1.6px에서 18.7px로 복구되었다. 마지막 예방 문단의 PDF 대비 y 차이는
  약 -26.9px에서 -9.9px로 줄었다. 한국어 p2와 중국어의 잔여 오차는 별개로 남는다.
- 두 입력 모두 2/2쪽이다. 단, 현재 단계 코드는 전체 9,086개를 다시 실행하지 않았으며
  13단계의 전체 통과를 현 단계 전체 통과로 인용하지 않는다.
- 사용자 범위 조정: #6712를 완료한 뒤 이미 수정한 #6714/#6699/#6708과 PR 준비로 전환한다.
  나머지 이슈의 추가 코드 수정에는 착수하지 않는다. PR 생성·merge 완료 상태가 아니다.
