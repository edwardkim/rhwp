# Task M100 #4391 Stage 1 — rhwp replay: 작업 영수증·제3자 재현 엔진

- 이슈: #4391 · 브랜치 task_m100_4391(upstream/devel 직분기) · 2026-08-10 KST

## 왜 1년 선행인가
관리형 하네스 GA·자기진화 하네스 연구·에이전트 실작업 확산의 수렴점은
"에이전트가 한 일을 제3자가 재현·증명"이다. 전제인 바이트 결정론을 선실측
(같은 계획 2회 = 동일 출력 해시)한 뒤 엔진화했다 — 이 저장소의 결정론 투자
(svg2pdf 결정화 벤더 패치 등)가 만든 성질이다.

## 구현
- cmd_replay: 계획을 **임시 산출로 재실행**(사용자 경로 무접촉 — 계획의 output
  은 확장자만 형식 결정에 쓰고 임시 경로로 대체) → (입력·계획·산출) SHA-256
  3종 영수증(attest). --expect-output-sha256 = verify(불일치 exit 3,
  reproduced:false — 판정은 봉투 데이터). run_plan_engine 무수정 재사용.
- 등재 4종: capabilities cmd_json · MCP hwp_replay(tool_with_optional_args —
  optionalArgs 규약, 품질검증 프로필) · help 행 · Node envelopes.ts 재생성
  (37→38, gen:check 최신).

## 검증
- 계약 4/4 첫 실행: 3해시+무접촉 · **결정론(2회 동일 outputSha)** · verify
  수락/기각 · 형식 오류. 가드: cli_json 31(미커버 2건이 red 로 잡아 등재를
  강제 — 설계대로) · router 8 · mcp 24 · provenance 17 · clippy 0 · fmt.
