---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase86-eqedit-language-contracts/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase86-eqedit-language-contracts/goal-checkpoint.md` · 원문 SHA-256: `415f4293b575010ff80c1348123b7faa4c0fecf1ba063148e68d077458258a4f`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 86차 체크포인트

전체 목표는 ACTIVE/미완료다. 전수 조사 및 독립 판정 완료 뒤 이슈를 최종 갱신하고, 현행 지침과 다른 기존 검사를 실제 출력 보정/필수 시각 선행 조건에 맞게 변경한다. 실제 보정한 파일·함수는 issue_8232로 명명하고 helper/module/suite/manifest/생성/스크립트/E2E 참조도 갱신한다. 정상 검출 목적을 보존하며 예전 이름 wrapper를 중복 등록하지 않는다. 제품·검사 수정 0, 개명 0이다. 원격 push/PR 승인은 없으며 다른 task/subagent/예약/메모리 쓰기는 하지 않는다.

현재 R=/Users/tsjang/rhwp, branch=audit/8232-px-regressions-20261025, HEAD=01ced3d42724285c8f9765a453393ab6af326617다. 기존 dirty rhwp-studio/public/rhwp.js는 SHA c1c8d1820cc4375ab668427fa0a42c87d71c3d0d83955f4d300880559023ddc6로 보존했다. 이번 감사가 만든 변경이 아니므로 revert/stash/commit하지 않는다. 요청 없이 fetch/rebase하지 않는다. 원본/PDF/공유 target/다른 작업장/기존 Studio 서버를 보존한다. 최근 반복 빌드 지적을 반영해 이번84~86차 Cargo/prepare/추가 빌드는 모두0이다.

최신 정본은 active-audit.json → phase86-eqedit-language-contracts/audit-ledger.json 및 scope-summary.json이다. 현재21360그룹/2501파일, 직접9715함수/Rust6429검사/836파일이다. 미분류11376+의미메모95+파일문맥165=11636그룹은 추가 판정이 필요하다. 직접 판정 안의 근거 pending511도 해결 전이다. change_required_for_issue8232_px_policy=true 87개는 보정 판정이며 구현 완료 수가 아니다. AST/원장 수를 실제 실행 testcase 수로 부르지 않는다.

80차 초기 검색 누락 Rust2887·스크립트5630=8517 기존 AST 후보 연결과 workspace 명시 제외 vendor6496 분리는 유지한다. 삭제 Rust3검사(원장4그룹), 신규 Rust95검사/48파일, 새 JS callback1(schema2건) 및 관련 helper 사용자 제외도 유지한다. 전체 최초 범위와 실제 신규/삭제 구분 근거는 phase80-root-inventory-backstop의 scope/validation을 따른다. is_test=false callback·매크로·동적 순회·생성 경로·feature별 실제 등록을 놓친 채 완료로 판정하지 않는다. 원본/정상 검사를 일괄 제거하지 않는다.

84차는 test_visual_sweep.py 826..1563줄의37함수/32 Python roots/5 helpers를 신규 판정했다. 기존32검사 PASS, 실패0, skip0,1.193초,빌드0이다. 직접 작성한 Square/Tight/Through 겹침/접촉·상단 이월·표 옆 잉크·옛자모/PUA·frame/tail·본문/각주 흐름·단 흐름 detector를 읽었다. x/y·14/50 등은 작성 반례/heuristic의 값이며 실물 renderer의 잠정 배치를 고정하는 회귀가 아니다. displayText 후보 억제를 glyph paint 성공으로, .977 frame fallback/2px pad를 실제 문서의 독립 PDF 정답으로 확대하지 않았다. root 이름과 달리 한 검사에서 Square만 실행하는 범위도 기록했다. 초기 driver는 의존 파일을 scripts/fidelity_compare.py로 잘못 지정해 source hash 사전 확인에서 종료했다. 테스트 실행0이며 phase84-sweep-layout-detectors-invalid-preflight/INVALID.txt로 보존했다. 실제 tools/fidelity_compare/fidelity_compare.py로 수정한 새 실행만 최종 증거다.

85차는 test_fidelity_compare.py 전체1464줄의58함수/54 Python roots/4 helpers를 신규 판정했다. 기존54검사 PASS, 실패0, skip0,.037초,빌드0이다. 0/1기준 쪽 번호·Counter/sequence의 인접 owner 이동·표/셀 소유·중첩 셀 분리·clip/자연폭/공백·그림 감싸기·자료 등급·argv 계약을 읽었다. 14px=300−286 겹침,5px=305−300 넘침,.8px=300−299.2 여백,48px=128−80 이월은 작성 상자에서 설명되는 산술이다. 실제 문서의 기존 출력 위치 핀이 아니다. stage96/p65·saved-margin 형태의 진단 표본도 원본 페이지의 현행 시각 자격으로 세지 않는다. Chrome runner는 mock이고 가짜 png 존재는 실제 PNG 무결성을 증명하지 않는다. SVG glyph band는 .8/.2em 근사이며 transform은 건너뛰므로 실제 글꼴 윤곽/모든 clip 안전으로 확대하지 않았다.

추가 범위 검증은 phase85-fidelity-owner-clip-contracts/two-file-review-completeness.json이다. 현재 source hash에서 test_visual_sweep.py의 전체100 AST함수/실제 unittest84roots와 test_fidelity_compare.py의58함수/54roots를 대조했다. 82~85차 각 원본 성공 로그의 class.method ID 합집합이 실제 unittest 등록 목록과 정확히 같고 중복/누락은0이다. 총158함수/서로 다른138 Python PASS의 두 파일 범위만 입증한다. 범위 대조 호출은 등록 수집만 했으며 테스트를 다시 실행하지 않았다. 첫 로그 parser는 unittest docstring의 두 줄 출력을 놓쳐 assertion이 났고, 해당 record를 다음 test 경계 안에서 읽도록 고쳤다. 제품/테스트 변경 없이 최종 범위 대조가 통과했다. 이 오류를 실제 검사 실패로 세지 않는다.

86차는 src/doclang/eqedit/mod.rs 전체434줄과 변환 wrapper를 읽고 cfg(test)의47검사+check helper=48함수를 신규 판정했다. 작성 EqEdit→LaTeX 문법/중괄호 오류/미지 명령 보존/손실 목록을 수식 glyph/위치/높이와 구분했다. 수식1/2·근차수3·행렬2×2·지수2·다중token3은 화면 px가 아니다. eq-01에서 유래한 작성 문자열도 실제 원본 문서의 paint 검증이 아니다. 원 source77360ea71ec1의 조상 관계를 확인했고 lexer→parser→emitter 호출을 읽었으나 각 외부 구현 전수를 이번48함수에 포함하지 않았다. 같은 HEAD의 보유 성공 metadata에서 해당 doclang::eqedit::tests:: prefix 실행 파일을 찾지 못해 Rust 실행0·빌드0이다. 등록되지 않은 CI 검사라고 단정하지 않는다. registration-search.json에 검색 범위/한계를 남겼다.

이번 목표 turn의 신규 판정은143함수(신규 Python roots86·Rust roots47·helpers10)다. 실제 실행은 기존 Python86PASS/0FAIL/0skip, 합계1.230초다. Rust 신규 실행/제품 CLI 렌더링/실물 캡처/빌드/제품·검사 수정/개명은 모두0이다. 직전 turn은92함수 판정/Python52PASS였으며 이번 합계와 중복하지 않는다. 82~85차의 누적 두 파일138PASS를 이번 새 실행86PASS와 구분한다.

#8232 원격 본문의 마지막 확인/갱신은80차(updated_at2026-10-26T14:23:00Z,UTF16 65534)다. 81~86차는 로컬 정본에 기록했고 이번 원격 게시0이다. 전수 완료 뒤 최종 이슈 갱신 지시를 유지한다. 다음 갱신은 fresh GET·UTF16 한도·다른 작업/실패 근거 보존·exact readback을 따른다. 실제 잘못된 핀/시각 미달/검출 한계를 삭제하고 정상 요약만 남겨서는 안 된다.

80~83차 및 이전의 실제 발견사항과511 pending을 계속 보존한다. 83차 체크포인트와 이전 정본에 독립 PDF/현재 Native/fresh WASM/page 자격·기존 오라클/새 회귀 구분·실패/보정 필요 증거가 있다. 전수/독립 판정·필요한 실제 보정·issue_8232 개명/호출 갱신·필수 검증이 끝나지 않았으므로 complete/paused/blocked로 바꾸지 않는다.

다음 작업은 source cfg(test)·/tests와 scripts/tools/bindings의 미분류 및 동적/helper 경로를 계속 점검하는 것이다. 큰 script 파일은 함수 원본을 읽고 작성 input의 단위/반례와 실제 product 출력 기대값을 구분한다. 이미 완료한 두 파일/동일 빌드를 반복하지 않는다. 나머지 EqEdit lexer/parser/latex의 cfg(test), src/renderer/equation/parser.rs, Gym gate/score/manifest unit들은 후보지만 신규 제외와 원 의미를 먼저 확인한다. 실제 page px 오라클/영역 selector가 있는 경우 독립 source/Print·현재 양 backend의 해당 쪽 자격을 확보해 판정한다. 전수 범위를 줄이거나 정적 PASS로 미달 화면을 수용하지 않는다.
