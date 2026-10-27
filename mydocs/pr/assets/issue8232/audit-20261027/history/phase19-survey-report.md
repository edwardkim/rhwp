---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase19-survey-report.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase19-survey-report.md` · 원문 SHA-256: `02e83cbb8108c96a0d9098541dd4ef0ce1aa16aa054772e9e911ad1ecb47c09a`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 19차 — 원본 HU 계약과 실제 출력 미달의 분리

head `238de6de41a208648f4ade6b494fe433a81c1fce`, 브랜치 `audit/8232-px-regressions-20261025`입니다. 제품·테스트 수정/개명은 아직0건입니다. 전수 조사 후 실제 보정·issue_8232 명명/호출·등록 정리·필수 검증을 끝내는 목표를 유지합니다.

## 직접 분류와 원본 근거

다음5개 파일 전체의 검사80개/helper55개, 총135개 함수를 직접 읽었습니다.

- rowbreak_short_row_whole_carry: 행/컷/후속 문단/걸침 셀의 source HU와 독립 근거가 필요한 고정 하한을 구분했습니다.
- issue_6812_square_picture_tac_table: 교집합·여백·실제 본문 하단·앞뒤 텍스트·페이지별 회피 수명 관계입니다. 같은 물리 anchor 전환의0.01/0.02px는 실물 PDF 절대 위치 핀이 아닙니다.
- issue_6639_cell_para_format_vpos: batch/undo/snapshot/fragment reset의 작성·저장 HU 사다리입니다.
- layout_anomaly_contract: JSON/NDJSON·동적 쪽 선택·types/page 집계·strict/API 종료 코드 계약입니다.
- issue_536_native_accessibility: 구조 순서·scalar/UTF16·읽기 전용/예산·core geometry adapter 연결입니다. OS 접근성 등록이나 독립 한컴 배치 oracle가 아닙니다.

CFB 원시 레코드와 HWPX XML을 rhwp 모델과 별도로 읽었습니다. survey는109개 직접 문단과64680→64462HU, CBTA는1200/6960HU와 padding282HU, 1480000은 cell1417HU, 면적조서는2362HU, 전직 면제 표는7940/18898HU입니다.

CBTA의 원시 문단 ordinal1132/1142와 모델 pi1133/1143은 달랐습니다. 처음 ordinal을 그대로 쓴 진단은 실패했고 근거로 쓰지 않았습니다. 정확한 CTRL_HEADER42바이트·instance ID2061287189/2062292470·행열을 대조해 같은 표를 연결했습니다. 모델은 ID/raw bytes 연결에만 쓰고 기대 HU는 CFB에서 읽었습니다.

저장 끝 행 크기 검사의 네 caller는 현재 페이지에서 선택 셀이 각각 하나이고 기대 표 소유임을 확인했습니다. 해당 수치 의미 pending1건을 해소했습니다. 0.5px라는 숫자만 보고 정상 원본 크기 계약을 ±2로 바꾸지 않습니다. 다른 caller의 소유·100HU 닫는 선·+4/+30px 같은 임의 하한은 계속 확인합니다.

## 전쪽 Native/fresh WASM

canonical PDFium/96CSSdpi/2px raw입니다. 같은 코드의 hash 고정 Native와 기존 fresh WASM 빌드를 재사용해 PNG/TSV는 새로 생성했습니다. Mac web/pkg/no-opt 대체 빌드이며 Docker 최적화 결과가 아닙니다.

| 조건 | 쪽수 | Native 최저 | WASM 최저 | 90% 미달 |
| --- | ---: | ---: | ---: | --- |
| 어울림 그림 원본/기존 Print |11 |98.22063% |98.22041% |양쪽0 |
| 70833/기존 Print |18 |17.30667% |17.08658% |양쪽5·6·7·9·10·15쪽 |
| 면적조서/새2020 Print |10 |50.85095% |50.84211% |양쪽5·6쪽 |
| 70833/새2020 Print |18 |17.62349% |17.42335% |Native5·6·7·9·10쪽, WASM은15쪽도 미달 |

4조건 각backend57쪽, 총114쪽이며 쪽수 불일치·누락0입니다. 미달 조건의 canonical capture/TSV는 exit1·re_review_required로 남습니다. 예외·문턱 완화·평균 상쇄를 하지 않았습니다. 어울림 그림 원본의 수치98%를 수동 IR 변형이나 새 독립 좌표 핀의 자격으로 확대하지 않습니다.

새 Print는 두 입력의 저장 제품을 확인해 engine2020/timeout1800초로 요청했습니다. 실제한컴11.0.0.9136, direct DLL host, pdf_print_method0, hancom2020_pdf_driver_one_up, scripts_disabled와 session0 font 상태를 기록했습니다.

- 면적조서 job `04a34b99-7525-452e-bb27-8fae01500e5b`,10쪽,126531bytes, PDF SHA `a4ba078927ab1839ff26270f8fc4e948960d72991417f40c6d84f8ff1e56aa99`.
- 70833 job `719ab169-b508-4407-ad2f-8d19d5db3f4b`,18쪽,587117bytes, PDF SHA `aa3a7ba5c3243b75fce7fe8e64c2a848b37e5b0a2ab4ba3c7d387a7c358e291a`.

입력과 기존 기준 PDF는 교체하지 않았습니다. 기존70833 PDF의 실제 엔진 빌드는 과거12.0.0.4605였으며 새11.0 대조군과 구분했습니다.

## 직접 판독

review와 standalone overlay16개를 읽었습니다. 어울림 그림1쪽은 상단 그림과 TAC 표 순서·여백,11쪽은 질문9/후속 본문을 확인했습니다. 제목 자간·글리프 잔차는 남습니다.

70833의 기존5쪽과 새10쪽은 표 조각 높이·후속 본문 위치가 다릅니다. 새 독립 PDF에서 선행 bullet은 물리9쪽에만 있고 현재 Native10쪽은 그 bullet을 포함한 뒤 II를 시작합니다. 큰 차이를 폰트만의 문제로 돌리지 않습니다. 15쪽은 Native91.07744/WASM89.25151%이며 한 줄 제목의 글리프/미세 위치 차이가 보여도 canonical 보류를 그대로 유지합니다. 면적조서5쪽은 같은 행 번호들의 수직 띠 높이와 끝 괘선이 다릅니다.

이 관측은 source 변경 전 고정 base의 상태입니다. 후보 변경 전후 회귀나 새 보정 완료로 보고하지 않습니다.

## 실제 기존 Native80개

현재 nextest metadata의 suite006/019/022/025를 조회하고 정확한 binary 경로와 실행 전후 SHA를 고정했습니다. **80실행/80 PASS/0 FAIL**,38.803초, 선택 target의954개 skipped입니다. 실행 전후 binary hash는 같고 command/summary/exit0을 확인했습니다. 전체 회귀·새 검사 FAIL→PASS·시각 수용이 아닙니다.

## 최신 원장과 남은 작업

10,400그룹/2,105파일, 미분류4,385, 직접 의미 판정5,229, 독립 근거 대기447, 의미 메모127, 파일 문맥212입니다. 직접 판정5,676개가 원장에 모두 연결됐고 누락0입니다. 전쪽 목록은100조건/각backend2,562쪽/입력·PDF196파일이며 미달도 포함합니다.

새 절대 px 검사 자격은0건입니다. 조사·원본/출력 판정·실제 보정·새 명명·수집/호출 정리·필수 검증을 계속하며, 잘못된 기존 단언을 유지하는 조건은 적용하지 않습니다. PR/push는 하지 않았습니다.
