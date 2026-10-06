---
kind: investigation
status: active
canonical: mydocs/manual/codex/docs_and_git_workflow.md
last_verified: 2026-08-02
---

# Task #3738 Stage 1 — 정책연구용역 보고서 그림 23 RowBreak 이월 정합

- 이슈: [#3738](https://github.com/edwardkim/rhwp/issues/3738)
- 관련 기여 PR: [#3740](https://github.com/edwardkim/rhwp/pull/3740)
- 기준 입력: `samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp`
- 동등 비교 입력: 같은 stem의 `.hwpx`
- 한컴 기준 PDF: `pdf/pr3740/hwp/…-2020.pdf` 및 `pdf/pr3740/hwpx/…-2020.pdf`

## 문제 재현

한컴 2020 MCP로 HWP와 HWPX를 각각 PDF로 변환했다. 두 기준 PDF는 모두 215쪽이고,
검토 범위 p14–p16·p23의 raster는 서로 동일했다. 따라서 아래 관찰은 개인정보를 제거한
두 입력 형식 모두에 적용되는 한컴 기준이다.

수정 전 rhwp 출력은 그림 23을 PDF p24가 아닌 p23에 배치했다. 그 결과 p23의 그림 21·22
아래에 그림 23이 겹쳐 들어가고, p24에서는 그림 23이 사라진 채 뒤따르는 본문과 표 4가 먼저
시작했다. 이는 단순 raster 차이가 아니라 페이지 소유권이 한 쪽 앞당겨진 조판 결함이다.

| 페이지 | 한컴 PDF | 수정 전 rhwp |
| --- | --- | --- |
| p23 | 그림 21·22 뒤 본문으로 종료, 그림 23 없음 | 그림 23이 본문 위에 조기 배치됨 |
| p24 | 그림 23, EU 생존 장기기증 문단, 표 4 순서 | 그림 23 누락, 문단·표 4가 먼저 시작 |

## 원인

대상은 HWP 문단 `pi=344`다. 빈 host 문단의 1×1 비-TAC `TopAndBottom`·`Para`·`RowBreak`
표이며, 셀 내부에 `Picture`가 있다.

- host `LINE_SEG.vertical_pos=52230 HU`, 다음 빈 guide 문단 `pi=345`의 값은 `30689 HU`로
  되감긴다. 즉 표 다음에 새 물리 페이지가 시작된다는 저장 좌표 신호다.
- 남은 본문 높이는 259.8px인데, 실제 첫 행/그림 단위는 490.4px다.
- 기존 1×1 표 일반화 경로가 `remaining > 0`만으로 `force-split`을 허용해 현재 페이지에
  첫 조각을 남겼다. 표 선언 높이(약 243px)만으로 fit을 추정하면 이 결정이 가능해 보이지만,
  그림의 실제 paint bounds는 이를 크게 넘는다.
- layout은 그 조각의 내부 그림 음수 상대좌표를 p23 상단 그래프 영역으로 계산해, 그림 23을
  본문 흐름보다 위에 그렸다.

따라서 고칠 대상은 저장 좌표 전체를 raw page 좌표로 바꾸는 것이 아니라, 위의 좁은 형상에서
`force-split`을 금지하고 fresh page로 통째 이월시키는 paginator 결정이다.

## 보정 경계와 회귀 계약

다음 조건을 모두 만족할 때만 첫 행 강제 분할 대신 다음 페이지 이월을 선택한다.

1. native HWP5 layout, 빈 host, 비-TAC `TopAndBottom`·`Para` `RowBreak` 표
2. 1행 1열 1셀이고 셀 안에 `Picture`가 존재
3. 다음 문단의 실제 `LINE_SEG.vertical_pos`가 현재 host 값보다 작음
4. 표 전체가 fresh 페이지의 본문 높이에는 들어감

이 경계는 실제로 한 페이지보다 큰 1×1 표의 셀 내부 분할과, 텍스트 표·다행 표·HWPX의 기존
분할 계약을 변경하지 않는다. 시각 결과와 HWP/HWPX 동등성은 결과 보고서 및 visual sweep
기록에서 별도로 확인한다.
