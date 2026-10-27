---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/checkpoint-20261026-hidden-relative-contract.md
last_verified: 2026-10-27
---

> 해당 회차 당시 기록입니다. 현재 집계·완료 여부는 [현재 조사 보고서](<../../../../../working/task_m100_8232_stage1.md>)를 따릅니다. 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 source 추가 감사 체크포인트

- 목표 ACTIVE. 기존 지침 부적합 검사 전수 조사→이슈 공개→필요 출력/검사 보정→실제 보정 파일/함수 issue_8232 명명 및 호출/등록 갱신. 유효성이 없는 기존 기대값/검출 동작은 보존 전제가 아님. PR/push는 나중, subagent/예약/메모리 수정 없음.
- R `/Users/tsjang/rhwp`, branch `audit/8232-px-regressions-20261025`, HEAD `238de6de41a208648f4ade6b494fe433a81c1fce`. 제품/test/지침 변경·rename·commit·push 없음. 시작 status clean.
- 이전 체크포인트 `checkpoint-20261026-independent-contract.md`의 phase8/통합6·7/기존source21 증거 유지.
- 이번 source batch22=160, 158국소 비절대px 목적/2기존renderer전제 보류. `record_source_batch22.py`, `source-semantics-batch-22.json`.
- source batch23=160, 모두 직접 단언의 수치 의미상 model/byte/문자축/HU/데이터/paint op·input clip 보존 계약. 전체 helper closure/시각 수용은 아님. `record_source_batch23.py`, `source-semantics-batch-23.json`. 원장 안44789HU/인수300폭은 source와 재대조해 정정 완료. batch24 미준비.
- 최신 ledger9666그룹2103파일: unreviewed7228, function_px_policy_adjudicated1657, semantic_notes133, nested_helpers_pending77, function_evidence_pending359, file_context_only212. full_conformance_verified=false/조사 미완료. 후보 수를 위반 수로 쓰지 않음.
- 실제 새 발견: `src/document_core/queries/hidden_text.rs`의 `effective_size_ignores_relative_sizes_like_the_renderer_does`(1036), `default_relative_sizes_can_never_cause_a_zero_size_misjudgment`(1049)은 renderer가relative_sizes를 읽지 않는다는 전제를 고정. 현재 style_resolver623~627/ResolvedCharStyle::font_size_for_lang은 실제로 곱함. 진단classify_char→effective_pt255는base_size만 나누며 query는문자별언어를 전달하지 않음(834).
- `hidden-relative-size-observer.rs`를 현재 Native rlib SHA5768866158c10fc9e1336cc286e2daa3a8d548baaaf602a89359a2e8de217c27로 rustc 링크. session51366 정상 exit0 종료. build/log/producer JSON 저장.10pt+100%=resolved10/diagnostic10,10%=resolved1/diagnostic10,0%=resolved0/diagnostic10. threshold2에서셋다진단None. 최종paint/독립Print/0%malformed 계약은 미검증. 정상10%와0%입력 구분.
- 기존 root binary`target/pr-review/release-test/deps/rhwp-1dbea1fe72f5997a` SHA387b0011f6a5fce577156accb3851abfa0b8da7136f77f2b963c032270466408에서 두검사각exact 실행,각1PASS/3812filtered/exit0. `hidden-relative-existing-tests.log`/`...-result.json`. 전체/보정후PASS 아님. Cargo 재실행 없음.
- 원경로: 원 PR3809 CLOSED 미병합. 실제 통합3816 merge6ab503fe97b7abfd1839800c5c018da9f9abf4c5, 도입8a1db1933c719a9f8ccf3aa36bf51e9245e06b72.4171 merge23ff5b6f1acd67aaccdf1ecfb47a7dc1918a13ed는기본relative100으로바꾸면서old전제를유지. 실제relative읽기추가7eddd19da61247fcd291cb3c1912ab4037d8daae는commit/pulls API에서통합7505 연결. 후속7805의격자/자간도relative소비전제. 실제3commitancestor확인. `hidden-relative-origin-history.json` metadata 저장. fetch_pr diff 전체를shell인자로쓰려던1회E2BIG는파일쓰기전에실패하여metadata만추린명령으로정정.
- #8232 새댓글6401558974(phase9-source-predicate-report.md), exactremote검증, `phase9-published-receipt.json`. 두검사는literal페이지px핀은아니지만현재구현유래전제재인용문제로최종정상승인안함. 이를보안전체기능확대로쓰지않음.
- 동적 collector1064 후보/archived37 원장 미확정은 이전 체크포인트 그대로. 미해결 상태를 완료로 세지 않음. 임시관찰 파일은 모두ignored output. 공유 target/입력/PDF/기존증거 보존.
- 다음 source24·남은 integration/helper/상수/조건부/동적/E2E 감사와 독립 증거 연결 계속. 전체 감사 뒤 실제 잘못된 검사와 필요 구현 보정/rename/등록·before-after/정상대조/전쪽시각·필수게이트까지 완료. 원장상no product changes를완료로오인하지않음.
