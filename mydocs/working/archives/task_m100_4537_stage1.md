# task_m100_4537 stage1 — rhwp harness: 검증 루프 단일 명령

- 이슈: #4537 / 브랜치: task_m100_4537 (base task_m100_4509 — 서명 필요 적층)
- 산출물: cmd_harness(init·wrap·status) + 7표면 + tests/harness_contract.rs
  + 선언 문서(mydocs/tech/agent_harness_no1.md — 완비도 10축, 정직 조항 선두)

## 설계 결정

- wrap = run(실산출) + 영수증 + 캡슐(연번 파일명 NNNN_계획해시8) + **직전
  캡슐 자동 부모 연결** + 서명(선택) 한 방 — replay 와 달리 output 을 실제로
  만든다(하네스는 노동 자체를 감싼다).
- status 는 wrap 구성상의 선형 체인을 자가 판정(파일명·해시 연쇄 + 서명 집계
  + --deep 재현) — 일반 그래프 검증은 lineage 의 몫으로 분리(축마다 한 일).
- 산출→입력 연쇄는 참고이지 강제 아님(독립 작업 혼적 허용 — 설계 주석 명시).

## 실측

- harness_contract 2/2: init(키·키링, 덮어쓰기 거부) → wrap×2(실산출 존재·
  연번·자동 부모=직전 캡슐·signed) → status --keyring --deep 전건 green
  (재현 2/2) → **후행 공백 1바이트 변조**를 자식 기록 해시가 폭로(brokenAt=
  자식, exit 3). 첫 변조 시도는 JSON 파괴라 파싱 검출에 먼저 잡힘 — 해시
  폭로를 검증하려 JSON 유효 변조로 교정한 경위 기록.
- 가드: cli_json·router·provenance(+harness 스윕 레시피)·사전(+5행, 203)·
  서명/영수증/감사/계보 회귀 전건 green · node 466(패리티 우산 표에 harness
  등록 — SUBCOMMAND_WRAPPERS 규약 실측) · 봉투 43 재생성.

## 함정 실록

- node 패리티는 우산 명령을 test/parity 의 SUBCOMMAND_WRAPPERS 표로 매핑
  한다(edit 선례) — 신설 우산은 이 표 등록이 7표면의 숨은 8번째 표면.
