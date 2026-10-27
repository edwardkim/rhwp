---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase15-goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase15-goal-checkpoint.md` · 원문 SHA-256: `435c348bddd7cec480df10da02ab6436a55746d0bbc1f20694f6dfad9121f9ba`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 15차 체크포인트

목표는 ACTIVE이다. 전수 조사와 실제 보정·issue_8232 새 명명·호출/등록 갱신·필수 검증이 모두 완료 조건이다. PR/push는 하지 않는다. subagent/다른 task 메시지·예약·메모리 쓰기도 하지 않는다. 이번 목표 턴은 실제 판정·독립 반례·캡처·실행·이슈 갱신을 했으므로 progress이다.

## 작업장

- R=/Users/tsjang/rhwp, branch=audit/8232-px-regressions-20261025.
- HEAD=238de6de41a208648f4ade6b494fe433a81c1fce, tracked worktree clean.
- 제품·테스트 source 수정과 개명은 여전히 0건이다. 모든 새 자료는 ignored output/issue8232-audit 아래에 있다.
- 이전 phase14의 원장/미완료는 phase14-goal-checkpoint.md에서 이어받는다. 전체 목표를 audit-only로 축소하지 않는다.

## 이번 직접 분류

- tests/boundary_integrity_contract.rs 1105줄 전체: test20/helper16.
- tests/mcp_server_contract.rs 1090줄 전체: test26/helper8.
- tests/table_csv_contract.rs 794줄 전체: test14/helper9.
- tests/batch_axes_contract.rs 736줄 전체: test17/helper11.
- tests/cases/issue_6950_paragraph_end_topbottom_anchor.rs 1317줄 전체: test27/helper9.
- 합계 test104/helper53=157함수. record_integration_batch23.py와 batch24.py, integration-semantics-batch-23/24.json, non-numeric-helper-semantics-14/15.json에 기록했다.
- 출력이 잘린 구간은 다시 읽었다: boundary475~635/450~485, mcp985~1030. source production 전체 호출 폐쇄를 모두 읽었다고 주장하지 않는다.
- CLI/MCP의 bytes·문자/매치 수·JSON-RPC code·논리 row/col·원본 HU·현재 단건/배치 봉투 동등성은 실물 px 핀과 구분했다.

## 확인된 새 미검출: MCP의 잘못된 응답 선택

파일 tests/mcp_server_contract.rs:279, boolean_false_does_not_inject_a_presence_flag.

Server::call_tool은 content[0].text를 JSON으로 파싱한 본문을 반환한다. 이 함수는 본문에 다시 result.content[0].text를 조회하고 unwrap_or("")로 빈 문자열을 받는다. !contains(dryRun:true)는 항상 통과한다.

실제 frozen Native 서버에 원본 samples/hwp3-sample.hwp와 dryRun false/true 요청을 각각 실행했다.
- false: 본문 dryRun=false, 실제 지정 output HWP 기록, replacedCount299.
- true: 본문 dryRun=true, output 무기록, replacedCount299.
- 기존 선택 문자열은 두 경우 모두 빈 문자열이고 기존 predicate는 둘 다 PASS.
- 직접 본문 boolean false 계약은 정상만 수용하고 true 반례를 거부한다.
- 현재 제품은 정상이다. 이 반례를 현재 제품 실패·과거 binary/mutation 실행·새 정식 test FAIL→PASS로 주장하지 않는다.

자료:
- observe_mcp_boolean_response_owner.py.
- mcp-boolean-response-owner/observation.json, server-stderr.log, false-output.hwp.
- observer exec35599는 terminal exit0, server도 exit0. 다시 시작하지 않는다.
- source SHA2237cac40269cc045b239f4cc6898922d77729af1db2c8396e6a78e7f87a227d.
- fixture SHA645525c8cd5ec11b1742ba7cfc759f68622861916233b5e982385cdb12f0ced2.
- Native frozen SHA5c9aed5b7779a9fffc102ebc662ad1f66faabc020012011dc1444a12c8222c9f.

계보:
- 원 PR3724 head200bc52a8e502f19f512fa6ef8efae77f6d0e4cf, CLOSED 미병합.
- 누적 e22e60dfcd8120ce3e873e35ee0531781a46eeb1에 해당 함수가 추가됐으며 현재까지 본문 동일.
- 실제 통합 PR3742 MERGED merge cc382911684c05c041a6b5ae86a873f7321f2153은 현재 head 조상.
- helper 최초 fef859a7b6012effb15abde3b18429bbb88b138a에서도 이미 파싱 본문 반환.
- 원 PR3725는 patch-equivalent duplicate라 미적용이었다.
- 읽은 자료: pr_3724_review.md, pr_3725_review.md, pr_3689_review_impl.md, report/archives/mcp_declared_args_wiring.md 및 원 도입 commit/git log -L. 원 PR/통합 PR 본문과 live metadata도 확인했다. 필요 시 실제 수정 전에 원 이슈/PR 전체 논의의 나머지를 더 읽는다.
- live metadata는 mcp-boolean-origin-pr-metadata.json. fetch_pr 전체는 diff가 클 수 있으므로 metadata만 골라 출력한다.

## 자료 누락 시 거짓 PASS

batch23에 boundary711/749/786, mcp308/323/369의 6개를 missing_fixture_silent_pass_requires_rewrite로 기록했다. mcp279에도 같은 return이 있지만 주 결함은 wrong_response_shape_contract_requires_rewrite다. 현재 원본 존재와 다른 환경의 미실행 PASS 가능성은 구분한다. 이전 insert_image_contract719 finding도 유지한다.

## #6950 현재 전쪽과 기본 모드 자격

입력 samples/hwpx/20260909-para-table.hwpx SHA cbf2ee7235861e93011d80834bfc49525349f6776c929b98ebd4f06003581d07.
독립 PDF pdf/hwpx/20260909-para-table-2024.pdf SHA c24244980c2c428b06575a1d948349a0971ba168898956a2bbfcccda816dc5ad, 3쪽.
한컴2024 저장13.0.0.3622. 원본의 saved n-up preference와 독립 Print one-up3쪽은 별개다. 근거는 report/task_m100_6950_report.md, working/task_m100_6950_stage3.md 및 pr7382 stage27_independent_geometry.json.

처음 저장 제품에 맞춰 compat2024로 캡처한 뒤, 테스트 생성자 from_bytes의 실제 기본값이 hangul2024_compat=false임을 commands/document.rs:379에서 확인했다. 기본 모드 compat2022도 별도로 캡처해 실제 test 설정을 입증했다. 과거 캡처를 기본 모드로 잘못 재인용하지 않는다.

- phase15-para-table-evidence: compat2024, Native raw[99.23139,99.13920,99.79882], WASM[99.22843,99.13740,99.79875].
- phase15-default-para-table-evidence: 실제 test 기본 모드, Native[99.23141,99.13921,99.79849], WASM[99.22845,99.13741,99.79842].
- 두 설정 모두 3/3쪽, 미달/누락/추가0. 설정 간 PNG가 완전히 동일하다고 주장하지 않는다.
- canonical PDFium5.13/96CSSdpi/2px raw, Native Skia PNG와 기존 fresh WASM PrintCanvas PNG, capture8 순차/배타 lock. Mac web/no-opt 대체 WASM이며 Docker 최적화가 아니다.
- run_phase15_para_table.py exec73792와 run_phase15_default_para_table.py exec93667 모두 terminal exit0. 중복 시작하지 않는다.
- 로그 phase15-para-table-driver.log / phase15-default-para-table-driver.log와 각 폴더 native.log/wasm.log/TSV 로그.
- 각 폴더의 state.json은 시도 종료 producer다. visual_acceptance_verified=false는 driver의 초기값으로 남아 있다. 실제 직접 판독/자격 결과는 별도의 existing-pins-qualification.json에 명시돼 있다. 이 초기값 때문에 캡처를 재시작하지 않는다.

직접 판독:
- 각 설정의 Native/WASM review 1/2/3쪽 모두 확인했다. compat2024는 별도 overlay3쪽도 읽었다.
- 1쪽 예산 마지막 줄, 3쪽 목표/투자 상자를 확대했다. 기본 모드에서는 p003-native-detail/p003-wasm-detail/p003-independent-pdf-detail.png를 각각1350×270으로 직접 읽어 실제2배 확대를 확인했다.
- 글리프 획/힌팅·점선 농도·화살표 이미지의 미세 차이가 남지만 문단·표·주요 내용 소속/순서·3쪽이 유지된다. pixel identity 주장이 아니다.
- make_phase15_detail.py / make_phase15_default_detail.py와 detail/producer.json. 첫 PDF PNG 이름을 pdf_001로 잘못 추정한 진단은 실패했고 실제 pdf-001 이름으로 정정해 최종 자료를 생성했다. 실패한 것은 캡처가 아니라 crop 진단이다.

독립 원본/좌표:
- qualify_phase15_source_oracles.py → phase15-para-table-evidence/independent-oracles.json.
- HWPX XML에서 top4251+header2834=body origin7085HU; body extent70018HU; 문단5 vpos68707/LH1200을 직접 읽었다.
- 외곽 cell54805HU, 안쪽 table52416+outMargin283+283=content52982HU, padding141+141=282HU, center 공간1541HU.
- pypdf로 독립 PDF3쪽 CTM·m/l/re·stroke를 직접 읽어 target top307.8226666667CSSpx를 재산출했다. /Im10은 Image뿐이고 Form을 무시하지 않는다.
- pypdfium2.PdfPath API를 추정한 초기 진단은 AttributeError였고 근거로 사용하지 않았다. 최종 좌표는 pypdf 원시 operator의 실제 성공 결과다.

판정:
- page_last_line_consumes_the_resolved_source_frame:1191의 상수는 독립 원본 HU/몸통 frame임을 확인해 이전 pending을 해소했다.
- centered_wrapper_preserves_its_cell_frame_and_nested_table_origin:1222: p3 양 raw98%·독립 좌표·직접 판독 자격은 충족했다. 기존307.823±0.6은 현행±2CSSpx로 보정할 대상이다. +8px 하한만으로 center 소유를 대신하지 말고1541HU 공간·외곽 최소/내부 포함/내용을 함께 보호해야 한다. 실제 source 변경은 아직 없다.
- caption:721은 expected를 product caption_height_px로 구하므로 독립 높이 근거 추가 확인 중이다. production helper 정의 stored_tac_display.inc.rs:215~281을 읽었다. 저장LineSeg와 compose 높이의 max다. 정확한 새 정답/변형 수용은 아직 미완료다.
- 작은 본문 범위2~4쪽:1092와 Top/Center/Bottom 변형의 고정3쪽:1277도 원본 전쪽 자격을 변형 전체에 확대하지 않고 pending으로 남겼다.

실제 기존 집중 검사:
- run_phase15_existing_tests.py가 현재 등록 목록에서 suite016을 계산해 기존27개를 실행했다.
- exec17547 terminal exit0; summary0.944초27PASS/0FAIL,217filtered. 컴파일 포함19.055초.
- phase15-existing-tests.log / producer.json. 전체 suite/새 검사/지침 적합 완료는 아니다.

## 최신 원장과 게시

- 10024그룹 / 2103파일.
- unreviewed4795; function_px_policy_adjudicated4449; function_evidence_pending438; semantic_notes_present_requires_adjudication130; file_context_only_requires_function_review212.
- full_conformance_verified는 여전히 false이며 판정 수는 적합·수정 완료 수가 아니다.
- paired-evidence-catalog는 93조건·각 backend2499쪽·입력/PDF183파일. 두 profile을 별도 조건으로 세며 같은 문서를2종으로 오인하지 않는다.
- build_paired_evidence_catalog.py에 두 설정의 입력/TSV/producer SHA를 별도 검증하도록 추가했다. 기본 모드 qualification은 phase15-default-para-table-evidence/existing-pins-qualification.json이다.
- #8232 본문과 댓글 https://github.com/edwardkim/rhwp/issues/8232#issuecomment-6403153257 갱신 및 정확한 원격/로컬 본문 일치 검증 완료.
- 최종 snapshot: phase15-final-issue-body.md, phase15-survey-report.md, phase15-published-receipt.json.
- 여전히 PR/push/source 보정/개명은 없다. 전수 조사 완료 뒤 실제 보정과 새 명명/등록/검증을 계속해야 한다.

## 다음 작업

남은 원장 직접 의미 판정과 독립 미검증·동적 helper/fixture 소비를 계속한다. 이번 다섯 파일을 다시 미분류로 읽기부터 시작하지 않는다. 다음 후보 tests/cases/issue_7390_prep_table_rowbreak.rs(646줄)는 길이만 확인했고 내용은 아직 읽지 않았다. issue_536_native_commands, issue_6963_hyperlink_edit 등도 미분류가 많다. 이전 phase13의 겹침 셀19/실제22·Recipe·Hyperlink·relative size 등 confirmed finding을 잊지 않는다.

Cargo는 종료했고 캡처/서버 observer도 모두 종료했다. 다음 긴 작업 전에 실제 ps/로그부터 확인한다. jobs8/tests8/capture8·Cargo 순차와 source/PDF/공유target 보존을 유지한다. 관련 memory 검색은 hits0이어서 메모리 기반 사실을 사용하지 않았다.
