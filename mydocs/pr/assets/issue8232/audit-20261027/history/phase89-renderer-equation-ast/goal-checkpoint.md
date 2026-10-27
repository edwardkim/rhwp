---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase89-renderer-equation-ast/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase89-renderer-equation-ast/goal-checkpoint.md` · 원문 SHA-256: `990b66d3b82af33eccf4956e3a21496c9c59aa57a662fce9e58b5b21ac63302c`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 89차 체크포인트

전체 목표는 ACTIVE이며 미완료다. 전수 조사와 독립 판정 완료 뒤 이슈를 최종 갱신하고, 현행 지침과 다른 기존 검사를 보정한다. 실제 보정한 파일·함수를 issue_8232로 명명하고 helper/module/suite/manifest/생성/스크립트/E2E 참조도 갱신한다. 정상 검출 목적은 보존한다. 제품·검사 수정 및 개명은 아직 0건이다. 원격 push/PR 승인은 없으며 다른 task/subagent/예약/메모리 쓰기는 하지 않는다.

작업장은 /Users/tsjang/rhwp, 브랜치는 audit/8232-px-regressions-20261025, HEAD는 01ced3d42724285c8f9765a453393ab6af326617다. 기존 dirty rhwp-studio/public/rhwp.js의 SHA는 c1c8d1820cc4375ab668427fa0a42c87d71c3d0d83955f4d300880559023ddc6이며 보존했다. 이 변경을 감사가 만든 것으로 취급하거나 revert/stash/commit하지 않는다. 요청 없이 fetch/rebase하지 않는다. 원본/PDF/공유 target/다른 작업장/기존 Studio 서버를 보존한다.

정본은 active-audit.json이 가리키는 phase89-renderer-equation-ast/audit-ledger.json과 scope-summary.json이다. 현재 21,369그룹 / 2,501파일, 직접 9,764함수 / Rust 6,469검사 / 836파일이다. 미분류 11,336그룹, 의미 메모 95그룹, 파일 문맥 165그룹이 추가 판정 대상이다. 직접 판정 안의 근거 pending 511개도 미완료다. change_required_for_issue8232_px_policy=true인 87개는 보정 필요 판정이며 구현 완료 수가 아니다. AST/원장 수를 실제 실행 testcase 수로 말하지 않는다.

80차의 초기 검색 누락 보정과 사용자 제외는 그대로 유지한다. Rust 2,887개·스크립트 5,630개, 총 8,517개의 기존 AST 후보를 미분류로 연결했고 workspace의 명시적 vendor 제외 6,496개는 별도 보존했다. 삭제 Rust 3검사(원장 4그룹), 신규 Rust 95검사/48파일, 새 JS callback 1(schema 2건)과 관련 helper는 감사/PASS에 넣지 않는다. is_test=false callback·매크로·동적 순회·생성 경로·feature별 실제 등록까지 확인하기 전 전수 완료가 아니다. 상세 근거는 phase80-root-inventory-backstop 및 phase86-eqedit-language-contracts/goal-checkpoint.md에 보존했다.

87차에는 높이 커서의 미분류 문자/도형 소유 검사 5개와 작성 helper 7개를 신규 판정했다. FFFC/CR/문자·실제 TAC 도형 소유, 작성 Paragraph/스타일/커서/미주 첫 문단 집합을 페이지 위치 핀과 구분했다. natural trailing root는 page base 무변경만 단언하고 반환 y는 버린다는 실제 검출 범위도 기록했다.

같은 파일의 기존 pending 12개를 재검토했다. 현재 함수 본문은 phase40 당시 본문과 12개 모두 같음을 직접 Git source로 대조했다. 원 도입 커밋과 실제 통합 PR 1194/1294/1743을 연결하고 현재 production의 +40/+18/+10/-4/41/30/16px 및 lazy anchor 분기를 코드 위치로 남겼다. 원본 samples/3-09월_교육_통합_2022.hwpx를 제품 모델 없이 ZIP/XML로 읽어 Contents/section0.xml의 endNotePr/noteSpacing betweenNotes=1984, belowLine=576, aboveLine=0을 확인했다. 입력 SHA는 b3ed3d5c4f0f95f9a8990b17720da3bef566b5a36174153aa3def28fd46e6029다. 96dpi의 1984HU는 정확히 1984/75px(약 26.45333px)이다. 이 선언은 추가 경험 완충값의 독립 정답을 입증하지 않으므로 pending 12개를 적합/완료로 승격하지 않았다. 총 pending 511개는 그대로다. phase40의 과거 39PASS를 현재 실행으로 재사용하지 않았다.

87차 근거는 pending-before.json, pending-current-body-provenance.json, pending-oracle-links.json, independent-note-spacing-source.json, producer.json과 validation.json이다. 원 보고서의 원래 간격 보존 문제와 후속 범위도 다시 읽었다. 오래된 한컴 비교/수동 측정 결과를 현재 양 backend 페이지 자격으로 세지 않았다.

88차에는 EqEdit lexer/parser/LaTeX의 선택 32함수 중 미분류 15검사와 ast/conv helper 2개를 신규 판정하고 기존 15검사는 보존했다. 숫자 토큰·행렬 2×2·분수·명령/장식 문자열과 깊이 제한을 실제 glyph px와 구분했다. lexer 전체 248줄, parser/latex 테스트 모듈, 재귀/연쇄 상한 분기와 관련 emitter dispatch/명령 표를 읽었다. MAX_EQ_DEPTH=64와 작성 반복 5,000/정상 중첩 30의 수량을 독립 대조했지만 실제 파서 실행·정확한 경계 64/65·모든 복합 AST의 스택 안전성을 입증한 것은 아니다. 문자열 매핑의 통과를 downstream TeX/glyph 지원으로 확대하지 않았다. 원 도입 77360ea71ec1 및 DoS 하드닝 c39304d49145의 조상 관계를 확인했다.

89차에는 src/renderer/equation/parser.rs의 미분류 AST 검사 20개를 신규 판정하고 기존 9검사는 보존했다. 해당 파일 안의 cfg(test) 29개 원문을 읽었으며 path로 분리된 자식 검사 파일 전체까지 완료했다고 세지 않는다. 분자/분모·근호·적분/합/극한·괄호/장식/글꼴 명령의 AST 소유와 Debug 부분 문자열 단언을 구분했다. 일부 root는 index/sub/sup의 존재나 노드 이름만 확인하므로 전체 내부 값/결합 순서/실제 paint의 증명은 아니다. 관련 parse entry/AST dispatch 문맥과 원 도입 9개 source 커밋을 연결했다. parser_atom.rs를 읽으려던 경로 오기는 실제 parser_atoms.rs로 고쳐 해당 문맥을 읽었다. 제품 파일 변경은 없다.

이번 목표 turn의 신규 판정은 49함수(검사 40개, helper 9개)다. 기존 판정 재독 24개와 pending 재검토 12개는 신규 수에 중복하지 않았다. 실제 Rust/Python 테스트 실행, 제품 CLI 렌더링, 실물 캡처, Cargo/prepare/추가 빌드, 제품·검사 수정, 개명은 모두 0건이다. 현재 HEAD의 보유 성공 metadata 8개에는 해당 library roots가 없어 정적 판정만 남겼다. 이것을 실제 CI 미등록이라고 단정하지 않으며, 이를 위해 새 빌드를 하지 않았다.

직전 turn의 143함수 신규 판정/Python 86PASS와 이번 수치는 구분한다. 기존 두 Python 도구 파일의 전체 158 AST함수/실제 138개 unittest 등록 및 성공 로그 대조는 phase85-fidelity-owner-clip-contracts/two-file-review-completeness.json에 있다. 이번에 그 검사를 다시 실행하거나 PASS 수에 더하지 않았다.

#8232 원격 본문의 마지막 확인/갱신은 80차(updated_at 2026-10-26T14:23:00Z, UTF16 65,534)다. 81~89차는 로컬 정본에 기록했고 이번 원격 게시 0건이다. 전수 완료 뒤 최종 이슈 갱신 지시를 유지한다. 다음 갱신도 fresh GET·UTF16 한도·다른 작업과 실제 실패 근거 보존·exact readback을 따른다. 실제 실패/부적합 핀/시각 미달/검출 한계를 삭제해 본문을 줄이지 않는다.

앞서 보정 필요로 분류한 87개와 근거 pending 511개의 자료는 유지했다. 특히 독립 source 단위와 구현의 경험 완충을 같은 정답으로 취급하지 않는다. 현재 source에서 일부 계산이 일치하거나 단위 검사가 통과해도 실제 위치 오라클/해당 Native·fresh WASM 페이지 자격을 대신하지 못한다. 전수 조사·독립 판정·실제 보정·issue_8232 개명/호출 갱신·필수 검증이 끝나지 않았으므로 complete/paused/blocked로 바꾸지 않는다.

다음은 나머지 source cfg(test)·/tests 및 scripts/tools/bindings 미분류와 간접 helper/동적 호출을 점검한다. 일반 문법/정수·주소·schema 계약을 실제 페이지 px pin으로 오분류하지 말고, empirical geometry/고정 selector/전체 SVG hash는 독립 출력 및 현행 지침과 대조한다. 원 해결 PR → 통합/보정 → 독립 기준을 코드/기대값 변경 전에 추적한다. 실제 보정은 전수/독립 판정 후 수행하며, 정상 검출 목적을 삭제하지 않는다. 같은 빌드나 이미 완료한 단위 실행을 반복하지 않는다.
