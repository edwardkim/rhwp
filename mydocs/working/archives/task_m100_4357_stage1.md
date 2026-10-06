# Task M100 #4357 Stage 1 — W1 rhwp workspace v1 구현

- 이슈: [#4357](https://github.com/edwardkim/rhwp/issues/4357) · 브랜치 `task_m100_4357`
- 2026-08-09 KST · 구현·검증 완료 · **승인 매체 = 본 PR(작업지시자 결정: 머지=채택/클로즈=기각)**

## 구현 (설계 #4351/PR #4352 의 실물)

- `mcp-serve --workspace <dir>`: 기동 1회 스캔(.hwp/.hwpx/.hml, 숨김 제외,
  상한 10,000+truncated 표지), **경로 정렬 결정론 id(w1..)** 인벤토리.
- 신규 세션 도구 4종(전부 조회 축): `hwp_ws_list`(인벤토리) ·
  `hwp_ws_open`(id→기존 session_open 재사용) · `hwp_doc_tree`(안정 노드 ID 트리
  — 페이지 p0.. / 표 t0..(hwp_doc_tables 순서와 동일 — 셀 편집 좌표로 연결)) ·
  `hwp_ws_journal`(변이 저널 조회).
- **자기검증 루프**: 변이 4종(replace_text/set_cell/fill_fields/save)을
  `journal_wrap` 으로 감싸 매 호출의 본문 SHA-256 전/후·changed·isError 자동 기록.
- 등재: served_tools 정의 4종 + `ALL_SESSION_TOOLS`/`SESSION_READ_TOOLS`.
- 게이트 처리: R76 캐시 재설계와 **비중복**(열린 핸들 위 조회·저널만 — 재파싱
  회피는 기존 세션 담당), R28 동시성은 v1 단일 클라이언트 전제 명시.

## 검증

- `tests/mcp_workspace_contract.rs` 3본 **첫 실행 3/3**: 왕복(list w1→open→tree
  p0/tables→save→journal sha256 64자리·changed:false)·무워크스페이스 명시 실패
  (--workspace 안내)·tools/list 등재.
- 인접: mcp_server_contract 24/24 · agent_profile_router_contract 8/8 ·
  `cargo clippy --bin rhwp` 경고 0 · rustfmt 적용 후 재검 3/3.
