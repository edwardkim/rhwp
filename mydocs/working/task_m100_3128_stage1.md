# #3128 Stage 1 — 기준선 재현과 원인 분리

- **Issue**: #3128
- **기록일**: 2026-08-18 KST
- **성격**: 소급 완료 기록

> 이 Stage는 계획 승인 전에 이미 수행됐다. 아래는 당시 산출물과 실패 가설을 재구성한 기록이며,
> 승인된 단계 완료보고서로 소급 간주하지 않는다.

## 1. 정답지와 기준선

- 입력: `samples/76076_regulatory_analysis.hwp`
- 정답지: `samples/issue1891/76076_regulatory_analysis-2024.pdf`
- 비교 페이지: 34쪽, 96dpi
- 페이지 수: rhwp 82쪽, PDF 82쪽

수정 전 render-tree에서 34쪽 외곽 continuation은 y=75.6px, h=451.8px였고 내부 1×1 child는
y=77.1px, h=426.9px였다. 후속 직접편익 표는 y=571.3px에서 시작했다. PDF raster의 첫 표 외곽은
대략 y=77..463px, 직접편익 표는 y=512px이므로 핵심 증상은 약 60px의 누적 하강이었다.

## 2. 분리한 원인

### 2.1 셀 fallback의 글자모양 소실

문제 문단에는 저장 `LINE_SEG`가 없고 둘 이상의 literal 공백으로 들여쓴다. `CharShapeRef` 구간은
같은 font metric에서 음수 tracking만 바꾸지만, 셀 fallback은 첫 style의 단일 run으로 재조판했다.
한컴은 이 문단의 literal 공백도 글꼴 고유 U+0020 폭이 아니라 반각 advance로 채웠다.

### 2.2 잘못된 saved cell margin 호환

문제 1×1 child는 `applyInnerMargin=false`, table left/right inMargin 0이다. 기존 #2308 회귀는
510HU saved cell margin을 보존한다고 기술했지만 PDF의 마지막 glyph paint edge는 table content box에
거의 닿았다. 저장 margin을 더하면 viewport가 좁아져 불필요한 한 줄이 생겼다.

### 2.3 terminal host tail 중복 예약

본문 wrap과 child 높이를 맞춘 뒤에도 후속 표가 19.2px 아래에 남았다. mixed nested fragment의
terminal child는 parent RowCut이 선택한 source cursor를 이미 소유하지만, generic terminal 경로가
첫 visible unit 높이를 다시 예약했다. 저장된 빈 host Enter는 가시 successor가 아니므로 중복이었다.

## 3. 기각한 넓은 해법

모든 no-lineseg 셀 문단에 CharShape tracking을 복원하자 80168 기준 문서의 페이지 수가 157쪽에서
156쪽으로 바뀌었다. tracking 복원 자체가 원인임을 진단 환경변수로 격리한 뒤 임시 환경변수는 제거하고,
적용 조건을 #3128의 구조 신호로 좁혔다.

## 4. Stage 결론

#3128은 PR #4763의 페이지 owner 문제가 아니라, 이미 올바른 p34 owner 안에서 재조판 폭·content-box·
terminal host tail이 누적된 독립 결함이다. 따라서 #4764의 전역 raster/font 작업을 기다리지 않고
issue-specific 구조 보정으로 처리할 수 있다.
