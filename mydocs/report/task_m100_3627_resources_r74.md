---
kind: report
status: active
canonical: mydocs/report/task_m100_3627_resources_r74.md
last_verified: 2026-08-06
---

# 처리 결과 — MCP resources 잔여: 스키마 3종 + 레시피 6편 (#3627, 로드맵 R74)

## 분석 — 지도가 낡아 있었고, 공백은 절반이었다

로드맵 R74 의 "지금"은 "resources/* 는 미구현"이라 적지만 **실물은 다르다** —
`src/mcp_serve.rs` 에 resources/list·resources/read 가 이미 있고(매니페스트 +
문서 3편: llms.txt·지식 지도·실패 사전), 계약 테스트도 있다(2026-08-06 실측).
남은 공백은 R74 설계가 지목한 노출 목록의 절반: **스키마(export-*-schema)와
레시피가 여전히 CLI·파일로만 닿는다.**

## 설계 판단

- **스키마 = 생성기, 레시피 = include_str!** — 두 부류의 본문 성질이 다르다.
  스키마는 코드에서 파생되므로 파일로 얼리면 첫 변경부터 낡는다 → lib 의
  `ir_schema()`·`plan_schema()`·`capabilities_schema()` 를 직접 부른다(단일 출처:
  CLI 의 export-*-schema 와 같은 함수). 레시피는 저장소 문서가 원본이므로 기존
  DOC_RESOURCES 방식(컴파일 시점 안기)을 그대로 따른다.
- **드리프트 가드는 왕복으로** — 목록을 하드코딩 재대조하지 않고
  `resources/list → resources/read` 왕복(광고한 모든 URI 가 읽히고 mimeType 이
  일치)으로 잡는다. 리소스가 늘어도 가드가 자동으로 따라온다.
- 프로필 무필터 정책·`rhwp://` 스킴 등 기존 결정은 그대로 승계(주석의 근거 유지).

## 변경

- `src/mcp_serve.rs` — `SchemaResource`(생성기 테이블) + 스키마 3종, DOC_RESOURCES
  에 레시피 6편, served_resources·read_resource 배선.
- `tests/mcp_resources_contract.rs` 신설 — ① 스키마·레시피 광고 ② **전 URI
  왕복 가드**(read-back·mimeType 일치·JSON 파싱·ir 의 irSchemaVersion) ③ 미지
  URI -32002 + data.uri.

## 실측

`cargo test --release --test mcp_resources_contract` — 결과는 PR 본문에 기록
(3본 green). 기존 `mcp_server_contract` 의 resources 검사(문서 3편·매니페스트)는
무변경 통과 대상 — 이 변경은 목록 **추가**만 한다.

## 남긴 판단

- capabilities 매니페스트 리소스의 프로필 렌더링(기존 동작)은 유지 — 스키마
  3종은 프로필과 무관한 전역 계약이라 필터하지 않는다.
- 레시피 07~10 이 생기면 같은 자리에 행만 추가된다(#4110·#4111 이 그 예약).
