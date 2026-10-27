---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase19-goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase19-goal-checkpoint.md` · 원문 SHA-256: `b1377db96fd783422bc247315efe5570907adcbe7cff8dca585b18b28d5a6229`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 19차 체크포인트

목표는 ACTIVE입니다. 전수 조사·판정·이슈 갱신 후 부적합 검사의 실제 보정, issue_8232 새 명명, helper/module/manifest/생성/스크립트/E2E 연결과 필수 검증까지 완료해야 합니다. PR/push, subagent/다른 task 메시지, 예약, 메모리 쓰기는 하지 않습니다. 이전 18차와 이번 19차 모두 실제 판정·실행·증적·원격 업데이트가 있어 progress입니다. blocker는 없습니다.

## 작업장

R=/Users/tsjang/rhwp, branch=audit/8232-px-regressions-20261025, HEAD=238de6de41a208648f4ade6b494fe433a81c1fce. 마지막 git status는 clean입니다. 제품·테스트 소스 수정/개명은 아직 0건입니다. 새 자료는 모두 ignored output/issue8232-audit 아래입니다. 이전 7445 worktree는 제거된 상태입니다. 원본·기준 PDF·다른 작업장·공유 target/pr-review는 보존했습니다. 전역 memory 검색은 hit 0이므로 관련 memory를 근거로 쓰지 않았습니다.

## 직접 분류

다섯 파일 전체와 중첩 helper/closure를 읽었습니다.

- rowbreak_short_row_whole_carry.rs: 842줄, test 24/helper 16.
- issue_6812_square_picture_tac_table.rs: 787줄, test 20/helper 14.
- issue_6639_cell_para_format_vpos.rs: 508줄, test 10/helper 8.
- layout_anomaly_contract.rs: 592줄, test 14/helper 13.
- issue_536_native_accessibility.rs: 518줄, test 12/helper 4.

합계 test 80/helper 55=135함수입니다. record_integration_batch34.py, integration-semantics-batch-34.json, non-numeric-helper-semantics-25.json을 사용했습니다. 같은 core API의 adapter 연결, 작성/저장 HU, 논리 주소, CLI 옵션 임계값을 독립 PDF 절대 px 핀과 구분했습니다. full conformance/call closure는 완료가 아닙니다.

## CFB/XML 원본과 실제 소유

inspect_phase19_raw_contracts.py가 rhwp 모델을 거치지 않고 CFB/XML을 읽습니다. 결과는 phase19-independent-raw-contracts.json입니다.

- survey: 직접 문단 109개, p76 vpos 64680 → p77 vpos 64462HU.
- CBTA: row17 LH1200HU, row28 두 번째 문단 첫 줄 vpos5760+LH1200=6960HU, padding141+141HU.
- CBTA 긴 셀: 왼쪽 51줄/오른쪽 74줄. 오른쪽 raw line36 vpos66020/LH1200, 다음 line37은 vpos0. reset 인덱스는 왼쪽5/40, 오른쪽5/37/70입니다.
- 1480000 row118c0: height1417/LH900HU. 면적조서 row29c0: height2362/LH800HU.
- 21761835 HWPX: cell(10,1) height7940/rowSpan2, cell(20,0) height18898/rowSpan5.

처음 CBTA raw ordinal에 모델 pi를 그대로 적용한 진단은 실패했습니다. 원시 ordinal1132/1142와 모델 pi1133/1143은 다릅니다. phase19-map-source-owners.rs를 현재 hash가 고정된 debug rlib와 정확한 serde identity로 컴파일했습니다. CTRL_HEADER 전체 42바이트·instance ID2061287189/2062292470·행열을 정확히 대조해 같은 표임을 확인했습니다. 모델은 소유 ID/raw bytes 연결에만 쓰고 기대 HU는 CFB에서 읽었습니다. compile exec87251/runtime31330은 모두 terminal exit0입니다. source-owner-producer와 raw/model JSON의 최종 SHA 일치도 확인했습니다.

inspect_phase19_row_owners.py는 frozen Native export-render-tree로 네 caller를 확인했습니다. CBTA126/127쪽은 pi1143, 1480000 43쪽은 pi263, 면적1쪽은 pi1이며 대상 row/col0 셀이 각각 유일했습니다. exec78548은 terminal exit0입니다. JSON bbox는 0.1px 반올림이라 허용폭 oracle로 쓰지 않습니다. 별도 CBTA123쪽 JSON에는 pi1133 표 하나/셀4개가 있습니다.

이로써 stored_restart_end_row_closes_at_its_declared_or_cut_content_box의 원본 크기 수치 의미 pending 1건을 해소했습니다. 작은 0.5px라는 이유로 정상 HU 계약을 ±2로 바꾸지 않습니다. 다른 cell_box caller 소유, 닫는 선100HU, 내용+4px/선언+30px 하한은 계속 확인합니다.

## 전쪽 비교와 실제 프로세스 종료

기본 compat2022, canonical PDFium/96CSSdpi/2px raw, Native Skia/fresh WASM PrintCanvas, capture 총8과 fcntl 배타 lock을 사용했습니다. 같은 코드의 기존 빌드 hash를 재사용하되 PNG/TSV는 새로 생성했습니다. Mac web/pkg/no-opt 대체 빌드이며 Docker 최적화가 아닙니다.

- run_phase19_current_layout.py: exec87479/PID45867, terminal exit0, 255.705534초. 드라이버 exit0은 모든 시도 수집 완료이며 시각 통과가 아닙니다.
- 어울림 그림 원본 11/11쪽: Native98.22063%, WASM98.22041%, 미달/누락0.
- 70833 기존 Print 18/18쪽: Native17.30667%, WASM17.08658%, 양쪽5/6/7/9/10/15쪽 미달.
- run_phase19_new_print_layout.py: exec95414/PID69684, terminal exit0.
- 면적조서 새 Print 10/10쪽: Native50.85095%, WASM50.84211%, 양쪽5/6쪽 미달.
- 70833 새 Print 18/18쪽: Native17.62349%, WASM17.42335%. Native5/6/7/9/10쪽, WASM은15쪽도 미달. 15쪽은 Native91.07744%, WASM89.25151%입니다.

4조건 각 backend57쪽/총114쪽, 누락·쪽수 불일치0입니다. 미달 조건의 capture/TSV exit1·re_review_required를 유지했습니다.

## 새 MCP Print

저장 제품2010/2020을 확인해 engine2020/timeout1800초를 명시했습니다. 실제 한컴11.0.0.9136/direct DLL host/Print method0/hancom2020_pdf_driver_one_up/scripts_disabled/session0 font verified입니다. 인증 정보는 출력하지 않았습니다. 기존 PDF/입력을 교체하지 않았습니다.

- 면적조서 job04a34b99-7525-452e-bb27-8fae01500e5b: 10쪽/126531bytes/SHA a4ba078927ab1839ff26270f8fc4e948960d72991417f40c6d84f8ff1e56aa99. exec16786 terminal exit0.
- 70833 job719ab169-b508-4407-ad2f-8d19d5db3f4b: 18쪽/587117bytes/SHA aa3a7ba5c3243b75fce7fe8e64c2a848b37e5b0a2ab4ba3c7d387a7c358e291a. exec63922 terminal exit0.
- 증적은 mcp-phase19와 mcp-phase19-control입니다. 기존70833 PDF의 과거 빌드는12.0.0.4605로 새11.0 대조군과 구분합니다.

## 직접 PNG 판독

qualify_phase19_existing_pins.py, 각 폴더 existing-pins-qualification.json, phase19-combined-pin-qualification.json에 직접 읽은 review/standalone overlay16개의 SHA가 있습니다.

어울림 그림 Native1/11쪽·WASM1쪽에서 상단 그림/TAC 표 순서와 본문·질문9를 확인했습니다. 부제 자간/글리프 잔차가 남으며 새 절대 px 자격을 승인하지 않았습니다. 기존70833 Native5쪽과 새10쪽은 표 조각/후속 본문 위치가 다릅니다. 독립 PDF9쪽에만 있는 특정 선행 bullet이 현재 Native10쪽에서는 II 앞에 있습니다. phase19-electrical-independent-page-content.json은 PDF 텍스트와 직접 PNG 판독을 연결합니다. 면적 Native5쪽은 같은 행 번호의 수직 띠 높이와 끝 괘선 차이입니다. 새70833 Native/WASM15쪽도 직접 읽고 미달 gate를 유지했습니다.

이는 소스 변경 전 고정 base 관측이며 후보의 새 회귀/보정 완료가 아닙니다.

## 기존 Native80개

run_phase19_existing_tests.py exec24994는 terminal exit0입니다. fresh nextest metadata의 suite006/019/022/025 경로와 test binary 실행 전후 SHA를 고정했습니다. 실제 80/80 PASS/0FAIL, Summary38.803초, 선택 target954 skipped, run elapsed39.326838초입니다. metadata 빌드16.62초는 별도입니다. 새 검사 FAIL→PASS·전체 회귀·시각 수용이 아닙니다.

nextest의 non-test CLI metadata 경로 release-test/rhwp의 실행 후 SHA와 mtime<=metadata를 별도 확인했습니다. phase19-cli-provenance-supplement.json입니다. 실행 중 env/CLI 호출을 별도로 가로챈 증거로 확대하지 않습니다.

## 최신 원장·게시·다음 작업

원장은10400그룹/2105파일, unreviewed4385/direct meaning5229/evidence pending447/semantic notes127/file context212입니다. 직접 판정5676개가 모두 연결됐고 누락0/현재 source SHA 일치입니다. paired catalog는100조건/각 backend2562쪽/입력·PDF196파일이며 미달을 제외하지 않았습니다. builder는 실제 capture gate와 보조 gate,90 문턱/쪽수/누락/hash를 함께 검증합니다.

#8232 본문과 댓글 https://github.com/edwardkim/rhwp/issues/8232#issuecomment-6404065215 를 갱신하고 remote/local 정확 일치를 확인했습니다. phase19-final-issue-body.md, phase19-survey-report.md, phase19-published-receipt.json(23증적 hash)을 참고합니다. 모든 driver/Cargo/rustc/observer의 실제 handle terminal과 마지막 git clean/예상 capture PID 종료를 확인했습니다.

4385개 미분류와447개 독립 pending/동적 호출 폐쇄를 계속합니다. 이전 phase18 이하의 확인된 부적합도 여전히 소스 미수정 상태입니다. 이번5파일/80개/4조건을 중복 시작하지 않습니다. 새 후보 issue_536_portable_glyph_geometry(969줄), layout_anomaly_glyph_band(455), issue_7028_partial_table_diagonal(281), issue_7061_stored_line_escape(343), issue_6936_pdf_synthetic_bold(245)는 아직 읽지 않았습니다. 전수 조사 후 실제 보정·개명·등록/호출 정리·필수 검증이 끝나야 완료입니다.
