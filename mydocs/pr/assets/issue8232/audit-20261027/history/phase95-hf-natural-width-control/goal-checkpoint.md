---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase95-hf-natural-width-control/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase95-hf-natural-width-control/goal-checkpoint.md` · 원문 SHA-256: `3d84c23a3ea83880d8f8fc5be6ef27311757a4978b8f0bc9703051c5a04c4ad0`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 95차 체크포인트

전체 목표는 ACTIVE이며 미완료다. 전수 조사와 독립 판정을 마친 뒤 #8232를 최종 갱신하고, 현행 지침과 다른 기존 검사와 필요한 구현을 보정한다. 실제 보정한 파일·함수는 issue_8232로 명명하고 helper/module/suite/manifest/생성/스크립트/E2E 참조를 함께 갱신한다. 정상 검출 목적과 원본을 보존하며 이전 이름의 중복 wrapper는 만들지 않는다. 제품·검사 수정과 개명은 아직0건이다. PR은 나중이고 push/PR 승인은 없다. 다른 task/subagent/예약/메모리 쓰기는 하지 않는다.

작업장은 /Users/tsjang/rhwp, 브랜치는 audit/8232-px-regressions-20261025, HEAD는 01ced3d42724285c8f9765a453393ab6af326617다. 유일한 dirty rhwp-studio/public/rhwp.js는 감사 전부터 있던97줄 추가이며 SHA c1c8d1820cc4375ab668427fa0a42c87d71c3d0d83955f4d300880559023ddc6를 보존했다. 작성자를 추정하거나 감사 변경으로 표시하지 않는다. 사용자 원본/PDF/다른 작업장/공유 target/Studio 서버를 보존했다. 요청 없는 fetch/rebase, Cargo/prepare/추가 빌드는0회다.

정본은 active-audit.json → phase95-hf-natural-width-control/audit-ledger.json 및 scope-summary.json이다. 현재21,439그룹/2,501파일, 직접 점검10,166함수, 그중 Rust 검사6,506개/839파일이다. 미분류11,004그룹+의미 메모95그룹+파일 문맥165그룹=11,264그룹은 추가 판정이 필요하다. 근거 pending은511개다. 보정 필요로 명시된87개는 유지했다. 이것은87개 모두 실물 px 테스트라는 뜻이 아니며 fixture 부재의 무조건return, 검출 누락 및 helper/generator의 주소·단위 문제도 포함한다. 실행 PASS 수와 감사 함수 수를 구분한다.

80차의 초기 검색 누락8,517개 연결, workspace 명시 제외 vendor6,496개 분리와 최신 사용자 제외는 유지한다. 삭제 Rust3검사(원장4그룹), 신규 Rust95검사/48파일, 신규 JS callback과 관련 helper는 감사/PASS에 넣지 않았다. 매크로/동적 순회/생성 및 is_test=false callback·feature별 등록 대조는 아직 미완료다. 이전 실제 미달·잘못된 핀·검출 한계 및 #8266 관련 근거는94차 goal-checkpoint.md와 그 이전 원장/연결에 보존했다.

95차는94차의 newly_created_header_and_footer_use_body_default_justification 한 함수에 남긴200px 상한의 독립 근거를 보완했다. 신규 함수나 기존 unit 검사를 다시 추가/실행하지 않았고,94차의 현재7PASS는 당시 실행으로 유지한다. 동일 HEAD의 검증된 CLI SHA9d496496e8561ff04c7c64b7b7171581d29fa6babe1dcfcbe38467574250ecf5를 재사용해 인쇄용 대조군만 만들었다.

원 unit은 양쪽/홀수 머리말과 짝수 꼬리말을 만들고, 물리적으로 active가 아닐 수 있는 양쪽 머리말을 대표 페이지에 투영한다. 독립 Print 대조군은 같은 blank2010·작성 문구·기본0번 스타일을 쓰되 양쪽 머리말만 만들었다. CLI는 각 편집 뒤 저장/재파스를 하므로 원 unit의 in-memory 전체 상태/바이트가 같다는 주장은 하지 않는다. 실제 preview 생산 경로는 같은 page content를 복제한 뒤 active_header 참조만 지정 target으로 바꾸며 글자 속성/텍스트/페이지 폭을 다른 수치로 바꾸지 않는다. 원본 템플릿과 대조군의 첫 CHAR_SHAPE/PARA_SHAPE payload bytes가 동일함을 제품 parser 없이 확인했다.

대조군 natural-width-control.hwp SHA588c64426aaba9b6df6ddab09f4b698549f52d34254537055d3522590003fd51,1쪽,저장 제품Hancom2020을 확인했다. 기존 hwp2024-mcp-convert client의 비동기 start→status→download를 사용했고 engine2020/timeout1800을 명시했다. job3d9b6e94-08e0-47c9-8360-9d5923977b59는 실제 succeeded/terminal,true,한컴11.0.0.9136,Print method0,hancom2020_pdf_driver_one_up,10.880초다. 다운로드 PDF는9,735bytes/1쪽/SHA7c951253378d052a0f5570d75a44e972a5256decaae3e366a3096bf9e681d9d4로 확인했다. 환경 URL/인증값은 출력하지 않고 client 응답/로그를 정리했다. 같은 job을 재시작하거나 중복 Print하지 않았다. mcp-start/status/download.json과 각 실행 원문이 근거다.

PDFium으로 실제 문자 원점과 embedded HCRBatang glyph advance를 읽고 각 문자의 PDF text matrix를 적용했다. 마지막 글자 끝까지의 관측 폭은97.6286814800676 CSSpx다. 원본 HWP PAGE_DEF의 폭/여백/제본을 CFB 레코드로 읽은 본문 폭은566.9333333333333 CSSpx다. 같은 대조군에서 원점 전진과 마지막advance가 각 em 이내임을 확인한 분리 상한은119.45525845885277 CSSpx다. 이는 임의 문서/자간의 보편 상한이 아니다. glyph 자체 advance 합89.1932615와 실제 원점·간격을 포함한 전체 span97.6286815도 구분했다.

초기 observer 오류를 보존했다. PDF 선언 font size83과 glyph advance를 행렬 적용 전부터 page pt로 가정해 잘못된192.1007/743.6800px를 계산했다. 실제 text matrix는 약0.12배율이므로 이를 적용해 정정했다. 처음 분리 관계를 참으로 가정한 observer assert 실패는 제품/기존 회귀 FAIL이 아니다. measure-before-observation-fix.py,measure-initial-assumption-failure.log,measure-without-text-matrix-INVALID.py,independent-natural-width-without-matrix-INVALID.json은 사용 금지이며 measurement-correction.json이 이유와 정정 script SHA를 고정한다. 관측기를 수정한 뒤 같은 PDF를 다시 읽었고 Print/캡처/기존 unit/빌드를 다시 실행하지 않았다.

현재 Native의 별도 대조군1쪽을 canonical visual_sweep.py/192DPI/compat2022/PNG path/capture8 이하로 비교했다. 정확한 입력/PDF/CLI/script SHA를 실행 전후 확인했고 캡처와 TSV 명령은 모두 exit0이다. Native raw silhouette100.00000%,미달0이다. 대표 review_001.png와 overlay_001.png를 직접 열어 문구의 순서·공백·짧은 한 줄의 자연 폭을 확인했다. Native render tree의 텍스트 bbox 폭은 소수첫째자리97.8이며 PDF 측정과 구분한다. 미세한 획/AA 차이는 남고 raw100은 엄격 RGB pixel100과 다르다. 이 결과는 원 unit의 모든 대표 투영 상태 또는 fresh WASM/실물 문서 전체 자격이 아니다. 관련 PNG·TSV·direct-review.json·native-visual-producer.json에 남겼다.

200px은 원 커밋cfa0620c...가 명시한 “HF 스타일 해석 cache 부재로 공백이 영역 전체로 늘어남”을 구분하는 작성 입력의 상한이다. 독립 자연 폭97.629<200<본문 폭566.933으로 이 오류 부류를 구별하는 근거가 확인됐다. 특정 글자의 관측 x/y나 정확한폭을±오차로고정한검사로 분류하지 않는다. 원래 검출 목적을 삭제하거나 단순히 숫자가 있다는 이유로 보정 대상으로 확정하지 않았다. 반대로 이 guard가 모든 미세 자간/부분 폭 증가를 검출한다고도 주장하지 않는다. 입력/글자 속성/지면 크기가 바뀌면 이 분리 근거를 다시 확인한다.

따라서94차에서 추가한 해당 함수의 function_evidence_pending1건은 정책 판정에 한해 function_px_policy_adjudicated로 갱신했다. change_required_for_issue8232_px_policy는 null→false이며 원래 검사는 그대로다. 보류 전체는512→511,보정 필요87은 그대로다. previous-pending-ledger-row.json과94차 원장을 보존했고 분류 근거는 classification-basis.json,독립 수치는 independent-natural-width.json에 있다. full_conformance_verified 및 원 unit의 current_page_evidence_validated는false로 유지한다.

이번 turn의 신규 직접 함수0,기존 pending 재검토1/정책 보류 해소1,독립 한컴 Print1쪽,Native 진단1쪽,기존 unit 재실행0,추가 빌드0,제품/검사 수정0,개명0,원격 issue/PR 게시0이다. 소스나 검사를 바꾸지 않았으므로 전체 nextest를 다시 돌리지 않았다. 이전15PASS/124PASS 등은 각 생산 로그의 실행이며 이번 새 실행 수로 합산하지 않는다.

마지막 실제 프로세스 확인에서 Cargo/rustc/nextest/wasm-pack 및95차 준비/Print/시각/원장 driver는 없었다. 준비는exit0,Print start session86441 종료0,status/download 종료0,시각 session88424 종료0,원장 session11195 종료0을 확인했다. 환경 상태/파일만 보고 완료를 추정하지 않았다.

#8232 본문은80차 게시의 exact readback(updated_at2026-10-26T14:23:00Z,UTF16 65,534)이 마지막 확인이다. 현재 live 본문이 그 뒤 바뀌지 않았다는 주장은 아니다.81~95차는 로컬 정본만 추가했다. 전수 완료 뒤 최종 이슈 갱신 지시를 유지한다. 게시 시 freshGET/UTF16한도/다른 작업 및 실제 발견 근거 보존/exactreadback을 따른다. 전체 조사·독립 판정·실제 보정·issue_8232 개명/참조 갱신·필수 검증이 남아 있으므로 목표는 complete/paused/blocked로 바꾸지 않는다.

사용자가 현재까지 분석 결과를 요청했으므로 최신 원장/실행 상태를 다시 확인했다. 보고에서는 전체 수집21,439항목과 직접 점검10,166함수를 구분하고,확인된 보정 필요87건이 전부px테스트87개라는 오해를 피한다. 미달 쪽의 미세좌표핀·잘못된출력고정·fixture누락시무조건return22건·관계검사의검출누락 등 대표 결과를 구분한다. 전체 조사와 실제 수정/개명은 아직 미완료임을 명시한다.
