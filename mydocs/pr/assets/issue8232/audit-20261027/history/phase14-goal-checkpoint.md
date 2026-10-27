---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase14-goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase14-goal-checkpoint.md` · 원문 SHA-256: `3f38f8e9c6edcf36761eb53f062d082b7e5d4734aca1ff8e8266a91eb271f6d4`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 14차 체크포인트

목표는 ACTIVE이다. 전수 조사와 실제 검사 보정·issue_8232 새 명명·호출/등록 갱신·필수 검증 모두가 완료 조건이다. PR/push는 승인되지 않았다. 다른 task/subagent/예약/메모리 쓰기는 하지 않는다.

## 현재 기준

- R=/Users/tsjang/rhwp, branch=audit/8232-px-regressions-20261025.
- HEAD=238de6de41a208648f4ade6b494fe433a81c1fce. 실제 작업장은 clean이다.
- 제품·테스트 변경 및 이름 변경은 아직 0건이다. 새 자료는 ignored output/issue8232-audit 아래에만 있다.
- 목표 직전 턴은 문구 답변뿐인 no progress였다. 이번 턴은 실제 221개 함수/helper 직접 분류, 독립 원본 단위 대조, 65개 정식 집중 실행과 이슈 업데이트로 progress이다.
- 사용자가 지적한 모순은 기존 단언/검출 동작을 무조건 보존하지 않고 독립 검증된 유효한 목표를 보호하는 것으로 해결한다. 현재 #8232 본문은 이 방향을 명시한다.

## 이번 실제 진전

1. tests/cases/issue_7998_pdf_retained_border_structure.rs 전체 2176줄과 issue_7998_pdf_table_inference.rs 전체 2223줄의 검사 65개·helper 74개를 직접 확인했다.
2. record_integration_batch20.py → integration-semantics-batch-20.json / non-numeric-helper-semantics-11.json.
3. inspect_batch20_pdf_oracles.py → batch20-independent-pdf-operators.json. pypdf로 원본 PDF 16개를 직접 읽고 .50/.51/.70/.90pt→HU, Arial A667/1000em×12pt=8.004pt, rotation45 MediaBox240와 Tm(110,110)→top baseline(110,130)을 대조했다. 첫 임시 관찰의 잘못 추정한 parallel-vertical/horizontal 파일명은 실제 nonuniform-axis/horizontal로 정정했고 최종 원본 16개는 모두 존재한다. 생성기/원본 구문 근거와 현재 시각 수용은 구분한다.
4. inference:2121 rotated_text_hit_and_cursor_follow_the_painted_frame_through_both_saves는 절대 첫 baseline ±2CSSpx를 고정한다. 원본 행렬은 있으나 현재 Native/WASM 양쪽 98%·저장본별 직접 판독이 미연결이므로 independent_page_px_qualification_pending이다. 과거 >99% 주석을 현재 PASS 근거로 쓰지 않는다.
5. run_batch20_existing_tests.py로 정식 nextest 두 target을 현재 Cargo에서 검증/실행했다. exec session43080은 terminal exit0, 전체27.572초, 명시 summary3.522초 65run/65PASS/455filter skipped. 이는 전체 suite/시각/정책 적합 완료가 아니다. batch20-existing-tests.log와 producer.json에 현재 소스·명령·실제 바이너리 SHA·summary가 있다. 중복 실행하지 않는다.
6. tools/pdf-ir-poc/tests/table_detection.rs 338줄과 recognition.rs 327줄 전체를 읽었다. 검사38·helper13은 batch21 및 non-numeric12에 기록했다. 독립 자체 [workspace] crate라 root nextest와 다르며 RHWP_POC_REPO를 요구한다. 두 실제 PDF 위상/그림 소유 검사는 independent_source_topology_evidence_pending이다. production table_detection/recognition 전체 출력은 길어 잘렸으므로 이 두 production 파일의 전체 호출 폐쇄를 읽었다고 주장하지 않는다.
7. tests/insert_image_contract.rs 전체1168줄의 검사19·helper12를 읽고 batch22/nonnumeric13에 기록했다. paper_size_hu는 CLI overflow 자기조회이므로 별도 독립 CFB PAGE_DEF 확인을 했다.
8. inspect_insert_image_units.py는 parser-venv Python으로 실제 CFB FileHeader 압축 bit와 Section0 tag73(offset189)를 읽었다. field-01.hwp 원본 용지59528×84188HU, page_attr0과 frozen Native CLI dry-run 보고가 같다. 생성40×20px 그림은3000×1500HU다. 독립 HU/96dpi 계약이며 paint 수용/전체19검사 PASS가 아니다. insert-image-independent-units.json과 insert-image-unit-probe-40x20.png에 보존했다. 스크립트의 기존 stamp overwrite 방지 assert가 있으므로 무조건 재실행하지 않는다.
9. insert_image_contract:719 hwpx_input_keeps_hwpx_output_format은 원본이 없으면 eprintln+return하여 검사를 실행하지 않고 PASS한다. 현재 해당 HWPX는 존재한다. missing_fixture_silent_pass_requires_rewrite로 기록했으며 원 해결 이력/실제 보정은 아직 없다. 5000/20000 등의 명시 HU 값은 올바른 요청 단위다.

## 최신 원장

- 9981그룹 / 2103파일.
- unreviewed4870; function_px_policy_adjudicated4332; function_evidence_pending436; semantic_notes_present_requires_adjudication131; file_context_only_requires_function_review212.
- full_conformance_verified는 여전히 false이다. 직접 의미 판정은 PASS/적합/수정 완료 수가 아니다.
- prior phase13: 9899/4985에서 helper 발견으로 총수가82 늘고 미분류는115 줄었다. 신규 테스트221개를 추가한 것이 아니다.
- 기존 시각 catalog91조건·각 backend2493쪽은 그대로다. 새 시각 캡처는 이번 턴에 시작하지 않았다.

## 이슈 게시와 로컬 정확성

- #8232 본문은 최신9981/4870 및 새 판정을 포함한다.
- 새 댓글 https://github.com/edwardkim/rhwp/issues/8232#issuecomment-6402934431.
- 최종 정확한 본문 snapshot은 phase14-final-issue-body.md, 댓글은 phase14-survey-report.md이다. issue-body-phase14.md는 이 턴 중간9981 전의 snapshot이다.
- phase14-published-receipt.json은 원격 본문/댓글과 로컬 파일 내용의 정확한 일치 및 SHA를 재확인했다. apply_patch가 붙인 여분 마지막 newline 1개를 정리한 뒤 재검증/receipt SHA 갱신까지 마쳤다.
- connector fetch_issue_comments는 body_html 포함해 매우 크다. 전체 structuredContent를 출력하지 말고 필요한 id의 body/url만 선택한다. 이번 전체 출력은 잘렸지만 store에서 id6402934431의 body를 추출해 정확히 검증했다.

## 이어갈 작업

- 전수 조사와 독립 미검증 해소를 계속한다. 실제 보정/이름 변경이 끝났다고 표시하지 않는다.
- tests/boundary_integrity_contract.rs는 이번 턴 1–140줄만 읽었고 나머지는 아직 안 읽었으며 새 분류 원장에 기록하지 않았다. 초기 일부를 읽었다는 이유로 파일 전체를 적합 처리하지 않는다.
- 다음 미분류 많은 파일: boundary_integrity_contract17, issue_536_native_commands16, issue_6963_hyperlink_edit16, issue_536_native_editor15, issue_6788_mixed_char_format15, mcp_server_contract15, table_csv_contract15 등. 전체 통합 원장에서 현 상태를 다시 조회한다.
- 과거 phase13의 잘못된 겹침 셀19/실제22 finding과 반례, 기타 기대값 결함·독립 qualification은 phase13-goal-checkpoint.md를 이어받는다. 아직 모두 source 수정 전이다.
- 긴 실제 빌드/캡처를 시작하기 전 ps/로그 확인, Cargo 순차 jobs8/tests8/capture8을 지킨다. 이번 집중 실행은 종료됐고 새 장기 프로세스를 남기지 않았다.
