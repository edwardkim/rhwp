# #7445 보정28 — Paper 표의 바깥여백과 저장 본문 원점 분리

Issue: #7445, 관련 #7390/#7406/#7437.

부모 `6e9ef166320b381c0342a1252fd0f8e017499fee`, base `2e3b4f880bd11d35398e6ea0100846611f13a72d`에서 PrEP 첫 쪽의 본문 원점 회귀를 보정했다. 원 해결 PR과 실제 호출 trace를 대조하니 typeset과 최종 layout이 각각 표 bottom margin을 본문 저장 줄 끝에서 빼고 있었다. 앞의 typeset만 고친1050은 최종 출력을 바꾸지 못했고 수용하지 않았다. 두 소비자가 인증된 같은 저장 줄 끝(vpos+LH)을 쓰도록 수정했다.

[입력·소스·build SHA, 전쪽 TSV, 전후 review/overlay와 회귀 결과](../pr/assets/issue7445/completed-20261009/stage28-20261021/README.md).

Native/fresh WASM 각250쪽(법령74/PrEP140/RowBreak18+18)은 전쪽90% 이상이며 Stage27 대비506쪽(별도 수도시설대조6포함)에서0.1%p 하락0이다. PrEP1쪽만양쪽PNG가변경됐고나머지504PNG동일이다.1쪽raw양쪽100%로제목/연구기관/표를직접판독했다. 문서명이아닌저장host/다음줄증명과원문소유에근거하며표자체바깥여백은유지했다.

같은새case의실제1FAIL→1PASS, --prepare후정식자동suite1PASS를확인했다. 독립PDF±2px와margin변형에서본문축불변/내용소유를함께검사한다. 기존10case30PASS/1기존FAIL이며#7751선택조건실패를이전바이너리로재현·분류했고삭제/완화하지않았다. Native/WASM Clippy·manifest/unit tier·fmt/diff는모두통과했다. Macfresh WASM은lockedweb/pkg/no-opt로컬대체빌드이며실제Studio초기응답hash도확인했다.

이는국소보정단계완료이며PR준비완료가아니다. 법령2/13쪽#8064 Native하한, 과학기준선, #7751검사의독립자격확인, 최신전체corpus/base TSV·전체회귀/필수gate·Actual36을계속한다. 수도시설기존3쪽미달은별도대조군으로유지한다. 지침·외부remote는변경하지않았다.
