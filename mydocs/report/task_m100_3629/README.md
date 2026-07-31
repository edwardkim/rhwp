---
kind: report
status: active
canonical: mydocs/report/task_m100_3629/README.md
last_verified: 2026-08-01
---

# #3629 처리 기록 — 에이전트 역할 라우터 (프로필별 도구 세트·레시피)

단일 출처 `src/agent_profiles.rs`(7종)가 `capabilities --mcp --profile`(선언 필터+레시피)와
`mcp-serve --profile`(tools/list 필터+세션 게이트) 양쪽을 구동한다. 호스트 설정 한 줄로
직무 전용 서버 등록: `{"args":["mcp-serve","--profile","행정서식"]}`.

실측 프로필별 도구 수(profile_counts.txt): 경영보고 6 · 행정서식 8(+세션) · 데이터분석 5 ·
콘텐츠제작 6 · 아카이브검색 7(+세션) · 품질검증 6 · 개발통합 = 전체(필터 없음).

검증: agent_profile_router_contract 4건 green(필터·레시피·미지 프로필 exit 2+목록 안내·
서버 tools/list 및 세션 게이트), mcp_server_contract 6건·cli_json_contract 22건 무회귀, clippy 0.
