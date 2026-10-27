---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase21-goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase21-goal-checkpoint.md` · 원문 SHA-256: `62c665bcd12d82b9d72343c1a68e0e5ebd7c4ad3e83b6a541e63832e44ff0766`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 21차 체크포인트

목표 ACTIVE, 완료 아님. 이전20차는 실제 실행·원장·이슈 갱신이 있어 progress였고 이번21차도 독립 근거·실행·판정·원격 본문 업데이트를 완료한 progress다. blocker/대기 중인 프로세스는 없다. 전수 조사 뒤 실제 부적합 검사 재작성·issue_8232 새 명명·공용 helper/module/suite/생성/스크립트/E2E 연결·출력 보정과 필수 재검증까지 해야 한다. 기존 단언/검출 방식의 무조건 보존은 요구하지 않는다. 정상 계약은 이름 변경만을 위해 훼손하지 않는다.

## 작업장

R=/Users/tsjang/rhwp, branch=audit/8232-px-regressions-20261025, HEAD238de6de41a208648f4ade6b494fe433a81c1fce. 시작/최종 Git clean, diff --check0. 제품·테스트 수정/개명0건. 새 자료는 모두 ignored output/issue8232-audit에 있다. 원본/PDF/다른 작업장/공유 target을 보존했다. PR/push는 아직 승인되지 않았으며 subagent/다른 task 메시지/예약/메모리 쓰기를 하지 않는다. 이전 전역 memory 검색0건이며 새 memory를 사용하지 않았다.

## 직접 판정

record_integration_batch36.py, integration-semantics-batch-36.json, non-numeric-helper-semantics-27.json:

- issue_7419_no_ls_tac_row_growth.rs 641줄,14검사/17helper.
- issue_493_cell_attrs.rs 654줄,12검사/9helper.
- skia_host_surface.rs 427줄,9검사/5helper.
- issue_7023_glyph_band_follows_baseline.rs 47줄,1검사/1helper.

네 파일을 전체 읽었다. 합계36검사/32helper=68함수이며 이전6개 기록과 중복돼 고유 직접 판정은62개 늘었다.35개 Native 집중 실행에는 마지막 chemical 진단1개가 포함되지 않는다. 전체 호출 폐쇄/full conformance는 미완료다.

원본/작성 HU·셀 포함/앞뒤 비겹침·원시/공통 속성 동기화·편집 비영향을 실물 절대 px와 구분했다. issue493의 resize 후 폭>90px는 편집5951→7151HU의 정확한 계약과 독립 편집 출력을 더 확인해야 하므로 pending이다. 같은 행 높이/이웃 행 비팽창의0.2 여유를 독립 페이지 핀으로 일괄 분류하지 않는다. chemical은 내용으로 쪽을 찾는 진단 음성 대조군이며 clean 쪽이 report에 없는 None 자체를 오류라고 단정하지 않는다.

## 독립 원본/합성 수학

inspect_phase21_source_contracts.py 및 phase21-independent-source-contracts.json은 CFB/XML을 rhwp 모델 없이 읽었다.

- 셀보호 HWP raw para0: common41950×17164HU/offset4,507. HWPX와 일치. 보호0/1/2, named cell2=name/editable1은 XML에서 확인했다.
- 셀보호2 HWP raw para1: common43190×17932HU/offset4,22133. HWPX와 일치.25셀 중 explicit margin은20 하나, 폭5951/높이3733HU, padding2834/2834/0/0HU, LineSeg text_start0/2/4.
- Skia host Surface의 명시 변환 T(5,7)S(2)T(3,4)S(1.5)을 독립 전개하면 사각형은 device[17,29]×[24,36], 상속 clip은[9,29]×[13,33]이다. pixel중심20.5/27.5는 내부,20.5/34.5와14.5/20.5는 외부다. 이는 작성 raster/합성 계약으로 실제 HWP 페이지 위치의 잠정 golden이 아니다.

## 원 해결·실제 통합

원 PR7433은 CLOSED/merged=false지만 미반영이 아니다. 실제 통합 PR7951은 MERGED/552514f423ec91f1b7171f6e6b0882a4efe034f2이며 HEAD의 조상이다. live 원 PR 본문/리뷰/종료 안내·통합 PR 본문과 현재 검사, Stage1/4/7·pr7433 review·fixture README를 대조했다. 원 기여ae67b089c9와 메인터너cd29e53235/45a46a0904를 구분했다. API 전체 diff 완독을 주장하지 않는다. phase21-original-pr-provenance.json을 참고한다.

PDF 세 개는 Stage7에서 다시 Print한 현재 추적본이다. 과거 Stage1 해시로 바꾸어 인용하지 않는다.

- page-height-28000 PDF42e48d3c85a14186dd2b9b7a7fada8cfcbdd88906552df9de998c798d31d31f6; job8543d52b-01b2-436b-9d47-513baa1cc020,3쪽.
- partial-nested-width-wrap PDFc159d38e6a2acadd70f295e1c02323168b71e9edd5722b548446d8e9c77557ff; job6b26616a-158c-4e25-98b0-598600a5146e,2쪽.
- partial-nested-wrapper PDFbe1e12d3736b35869804451be7a59b972547f40de6a204cb81fe2d5b4d78a6b8; job1b864fe4-9b5a-498f-8912-faa29482e38f,2쪽.

실제 과거 Hancom13.0.0.3901/PrintMethod0/frame_print_to_pdf_ex_one_up이며 이번에 새 MCP job을 시작하지 않았다. 다른10개 기존 Print도 Stage1/4/fixture README 대응을 확인했다. 수동 XML 입력에 lastSavedWith2024가 있다고 실제 한컴 저장본으로 바꾸어 부르지 않는다. 이번 프로파일은 test export와 같은 compat2024다.

## 현재 전쪽 실행

run_phase21_tac.py exec15711/PID6118 terminal exit0,259.711228초. capture fcntl lock/총8, canonical PDFium/96CSSdpi/2px raw·print PNG를 유지했다. frozen Native5c9aed5b...와 fresh WASM0a81d25f.../JSba97ed3f...의 동일 소스 빌드를 hash 확인 후 재사용했고 PNG/TSV는 새로 생성했다. Mac web/pkg/no-opt 대체 빌드이며 Docker 최적화가 아니다.

13조건·각 backend19쪽 모두100%, capture/TSV exit0·쪽수 일치·누락/미달0:
common-height-4838, nested-explicit-break, nested-outer-margins, nested-same-line, nested-surrounding-text, nested-width-wrap, page-height-28000(3쪽), tac_nested_two_line_cell_no_lineseg, tac_two_line_cell_no_lineseg, partial-nested-width-wrap(2쪽), partial-nested-wrapper(2쪽), mixed-child-storage(2쪽), no-child-storage(2쪽).

run_phase21_existing_tests.py exec74303/PID6936 terminal0. fresh metadata native-skia suite003/006/019 빌드3분09초. 실제35/35PASS/0FAIL, Summary3.932초, 선택 target769skipped, run elapsed17.477010초. 실제 test binary 경로·전후 SHA 동일. 전체 회귀나 새 검사FAIL→PASS가 아니다. process table에 Cargo/nextest/rustc/capture가 없음을 확인했다.

## 독립48핀과 선택 소유

inspect_phase21_title_origins.py exec77203 terminal0. PDFium FPDFText_GetCharOrigin으로 CSSx=x*4/3, CSSy=(pageHeight-y)*4/3을 직접 읽었다.13조건18쪽48핀과 기대값의 최대 차이는0.000040822917CSSpx, 현재 frozen CLI SVG는 모두±2px 이내이며 선택된 text/조상에 transform이 없다. Native/WASM render tree의 실제 제목 셀/후속 본문 소유를 대응했다. 원래 parser가 반환한 반올림 bbox를 독립 Print 기대값으로 재인용하지 않았다.

최초 진단은 모든 첫/둘 marker의 전체 출현 수가 핀 수와 같다고 잘못 가정해1조건에서 assertion 실패했다. 본문에도 첫/둘이 있어 틀린 진단 전제였다. phase21-title-origins-invalid-count와 first.log에 보존하고 최종 증거에서 제외했다. 기존 검사와 같은 nth 선택을 독립 PDF 원점·실제 제목/후속 소유에 연결하도록 수정한 새 실행만 최종이다. 정책 문턱·검사 소스는 바꾸지 않았다.

## 직접 판독과 동등 PNG 재사용

phase21-directly-read-pairs.json에28고유 backend/input/page 쌍이 있다. 각 review와 standalone overlay, 총56개를 실제 열었다.19쪽×2backend=38쌍 가운데10쌍은 Native PNG와 PDF PNG가 모두 바이트 동일하여 producer와 두 SHA로 판독을 연결했다. 다른 파일의 review 캡션까지 직접 읽었다고 표시하지 않는다. 각각 모두 실제 재출력했으며 캡처 자체를 생략한 재사용이 아니다. common-height↔tac_two_line, partial-width↔mixed/no-child의 같은 backend/page가 해당한다. Native와 WASM 사이에는 동일 PNG가0건이므로 backend 간 판독을 재사용하지 않았다.

두 본문 줄·중첩 자식·같은 줄의 좌우 배치·명시 개행/폭 부족·바깥여백·앞뒤 본문·작은 용지 제목/표의 쪽 분리·wrapper 통째 이월·뒤 본문의 순서를 확인했다. glyph baseline/윤곽의 미세한 raster 잔차는 보이며 엄격 픽셀 동일성은 주장하지 않는다. 현재 중요 배치에 잘림·새 겹침·누락·중복을 관측하지 않았다. 이 fixture에는 수식/각주가 없으므로 해당 조합을 검증했다고 하지 않는다.

phase21-tac-evidence/existing-pins-qualification.json으로 기존18쪽48핀의 수치·시각 자격을 연결해 assert_title_origins와 completed_pages...의 독립 수치 pending을 해소했다. full conformance/전체 production 호출 폐쇄는 별도다. 정상 핀을 삭제·완화하거나 이름만 바꾸기 위한 source 변경은 하지 않았다.

## 원장·게시·정정

10481그룹/2106파일. unreviewed4305, direct meaning5387, evidence pending451, semantic notes126, file context212. 직접5838개가 builder와 같은 sorted 우선순위로 모두 원장 연결·source SHA 일치·누락0이다. phase21-semantic-ledger-validation.json이다. paired catalog는115조건/각backend2588쪽/입력·PDF226파일. compat2024를2022로 잘못 표기하지 않도록 catalog profile을 별도 분기했다.

#8232 본문에21차 근거와 최신 수치를 갱신했다. 현재 OPEN, updated_at2026-10-25T22:38:14Z. phase21-final-issue-body.md와 API 재조회 본문을 실제 로컬 파일 말미 개행까지 정확히 맞춰 확인했다. phase21-remote-verification.json이다. 20차는 API 재조회와 게시 payload가 같았지만 저장 Markdown에는 말미 개행1개가 더 있었다. 20차의 '로컬까지 바이트 일치' 문구를 phase21-issue-body-verification-correction.json에서 정정했고 과거 파일/receipt는 당시 snapshot으로 보존했다. 본문 내용 차이는 아니다.

제품·테스트 수정/개명0건, PR 없음. 미분류4305와 pending451·동적/공용 호출 폐쇄 및 과거 체크포인트의 보정 후보를 계속 처리한다. 방금 끝낸13조건 캡처와35검사를 중복 실행하지 않는다. 남은 integration/helper 중 issue_1623_cellzone_diagonal, issue_7379_rowbreak_table_footnote_reservation 등 미검토 함수는 다음 직접 분류 후보이며 아직 이번에 전체 읽거나 실행하지 않았다. 전체 조사·독립 판정 이후 실제 부적합 검사 보정·개명·등록/호출 갱신과 필수 재검증을 마쳐야 목표 완료다.

## 핀 영역2배 확대 판독 보완

기존 canonical review/overlay에서48개 핀 주변을 각각 같은 CSS 좌표로 잘라 최근접2배 확대했고, review-detail/overlay-detail8쌍16개를 모두 직접 읽었다.이는 기존 출력의 분석용 배치이며 원본 PNG/PDF를 변경하거나 점수/문턱을 다시 계산하지 않았다.제목·후속 본문 소속/원점/정렬과 인접 줄을 확인했고 glyph 윤곽과 약1px raster 잔차는 구분했다.원점 이동·보간·정렬로 차이를 감추지 않았다. phase21-pin-magnification/manifest.json의 좌표·원본 이미지 SHA·확대 PNG SHA와 qualification에 연결했다. 직접 읽은 원본56개에 추가된 확대16개다.
