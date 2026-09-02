# 85장 — 개발자 전용

`mydocs/manual/agent_codex/85_진단_프로브.md` 는 렌더·파서 개발용 저수준 진단이다.

생성 장 머리글:

> 문서 작업 에이전트는 이 장을 쓸 일이 거의 없다 — 레이아웃 버그 조사 시에만
> rhwp-cli 스킬의 디버깅 순서를 따라 진입하라.

## 여기 있는 것 (쓰지 마라)

`bench` · `core-pages` · `diag` · `dump*` · `export-png` · `export-render-tree` ·
`gen-pua` · `gen-table` · `hwp5-*` · `measure-width` · `test-caption` ·
`test-field` · `test-shape`

거의 전부 **계약만** 이다. 표본 실행이 부피가 크거나 개발 픽스처가 필요하다.

## 에이전트 규칙

1. 사용자 요청이 "문서 작업"이면 85장을 열지 않는다 (X07).
2. `capabilities --search` 가 dump/probe 만 돌려도 거절하고 상위 가족 장을 권한다.
3. 진짜 레이아웃 결함이면 `rhwp-cli` 로 인계한다. 이 스킬에서 프로브 레시피를 키우지 않는다.
4. 85장 생성 본문을 수기 수정하지 않는다.

## 왜 장이 존재하는가

커버리지 가드가 전 명령을 장에 묶어야 한다. 숨기면 가드가 뚫리거나
개발 명령이 조회 장에 섞인다. 존재하되 **입장 금지**가 계약이다.
