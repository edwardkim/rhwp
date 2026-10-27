---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase13-goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase13-goal-checkpoint.md` · 원문 SHA-256: `df1247323fca7363b951dfbf3c0b898018841ea43b9f7cafb2a956ff546922ee`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 계속 지점

목표는 ACTIVE이다. 전수 조사·판정 후 부적합한 기존 검사 실제 보정·issue_8232 새 명명·공용 helper/module/suite/생성/스크립트/E2E 호출 갱신·필수 검증까지 필요하다. PR/push는 아직 승인 범위가 아니다. 기존 검사의 잘못된 기대값/동작을 무조건 유지하지 않는다. 정상 보호는 유지한다. 원 이름 wrapper/중복 등록 금지. 다른 task/subagent/예약은 만들지 않는다.

## 현재 상태

- 저장소 `/Users/tsjang/rhwp`, 브랜치 `audit/8232-px-regressions-20261025`.
- HEAD `238de6de41a208648f4ade6b494fe433a81c1fce`, 마지막 확인 clean.
- 모든 새 기록은 ignored `output/issue8232-audit/` 안이다. 제품/테스트 source 수정·개명0건.
- 이 목표 turn은 실제 조사/독립 근거/실행·이슈 갱신이 있어 progress다. 전체 목표 완료가 아니다.
- phase13 capture/생성/검사/observer는 전부 명시 exit0로 종료했다. 같은 실행을 재시작하지 않는다. 다음 turn은 ps·HEAD/dirty·로그부터 확인한다.

## 원장

`audit-ledger-summary.json`: 9,899그룹/2,103파일, unreviewed4,985, 직접 의미 분류4,136,
evidence pending434, semantic notes132, file context212. full_conformance는 전건false,
call closure는 전건pending이다. 숫자는 결함 수나 실제 PASS 수가 아니다.

- batch15: 배치fill/CAS/CLI/대역batch-convert/agent bulk의87그룹 직접 단언.
- non-numeric08: #4968 authored text_run/page_tree/layer_tree3개 본문 확인.
- batch16/non-numeric09: #7490 edited indent와 table-vpos hit-test 파일 전체35test/33helper 판정. 고정 쪽수/px와 범위 근거 미완료를 구분했다.
- batch17/non-numeric10: #6520 SVG layer 전체1574줄,25test/16helper 직접 분류. 현재 호출 결정성/원본 font 단위/작성된 clip/paint/source 계약이며 frozen HWP golden이 아니다. native-skia 조건부 실제 실행은 아직 하지 않았다.
- batch18: 독립 근거를 확보한 #7490 px2개만 유지 자격으로 재판정.
- batch19: 잘못된 셀을 검사한 table-vpos marker245행을 실제 재작성 대상으로 확정.

## phase13 실제 독립 비교

`run_phase13_indent_hit.py` / `phase13-indent-hit-driver.log` / `phase13-indent-hit-evidence/state.json`.
Native와WASM 순차/동일capture.lock/8workers/PDFium5.13.0/96CSSdpi/2px raw.

| 입력 | 쪽 | Native최저 | fresh WASM최저 |
| --- | ---: | ---: | ---: |
| table-vpos-01.hwp | 5 | 99.93951 | 99.94159 |
| indent20444 현재 재생성 | 1 | 100 | 100 |
| fit-prefix-after 현재 재생성 | 1 | 100 | 100 |

전쪽 미달/누락/추가0. p5 별도TSV99.99539이므로 review 그림의 반올림100과 구분한다.
두 편집은 기존 공개 생성기를 현재 Native rlib로 재실행한 결과가 독립 Print 입력과 바이트 동일이다.
`phase13-edited-indent-inputs/producer.json`, `phase13-independent-indent-oracles.json`,
`phase13-existing-pins-qualification.json`, `phase13-detail/producer.json`에 hash/명령/확대 판독을 기록했다.
제목 PDFium 원점(413.535888671875,167.81656901041666), 표 두 의미 셀 clip 합집합
(326.064,630.3506666666667,159.91466666666665,132.17466666666672)이 기존±2px 기대값과 맞는다.
표 clip은 원시PDF의m/l/W*와CTM/q/Q를 읽고 총/인용 문자 원점이 든 가장 작은 셀을 선택했다.
기존 source의 내용/소유/본문 포함/단일출현/prefix 뒤 배치를 함께 확인했다. 다른 prefix/Enter 성장/전체 파일 수용이 아니다.
글리프 획/힌팅·table-vpos 제목/번호 모양 차이는 확대에서 남았으며 raw100을 pixel identity로 주장하지 않는다.

`build_paired_evidence_catalog.py`에 terminal phase13을 포함했다. 현재91조건/각backend2493쪽/입력PDF181파일 hash.
최종 px 자격은 목록만으로 승인하지 않는다.

`existing-indent-hit-tests` standalone: 기존 두 모듈 그대로35PASS/0FAIL/0ignored/5.89s, source/rlib/binary/명령hash `existing-indent-hit-producer.json`.
전체integration/CI 실행으로 주장하지 않는다. 로그의 다른 문서 overflow 진단을 보존했다.

## 확정된 잘못된 회귀: 실제 마커 대신 일반4를 검사

`tests/issue_table_vpos_01_page5_cell_hit_test.rs:245`
`page5_inner_11x3_char_overlap_marker_advances_one_box`.

- 경로는 parent34/outercell0/para1/innercell19/para0이다. 실제19번은 row9/col0의 일반텍스트4, CharOverlap제어0개, charshape79/base3100HU, cursor116.0→139.9 =23.9.
- 실제10번은 innercell22/para0, row10/col2이다. CharOverlap U+F02BA+U+F02C3,
  charshape68/base1400HU, bbox폭18.6666667, cursor184.4→203.1 =18.7이다.
- 독립 CFB/DocInfo CHAR_SHAPE21 offset42에서도 shape68=1400HU,79=3100HU를 확인했다.
- 현재 잘못된 검사도35PASS에 포함된다. 단순히허용폭을고칠문제가아니라 원본소유자/제어payload/논리offset을 바로잡아야 한다.
- 원 해결6c887fc79887d16eded0d8fadae174e7350bad1b, 보고서 `mydocs/report/archives/task_m100_1023_report.md`를 읽었다.
  최초 검사본문과현재본문은동일하며 HWP SHA2e6c5bb4...도동일하다. 원커밋과release merge는현재조상이다.
- associated PR1294는 v0.7.14 release집계이며 개별기능 해결PR이라고 부르지 않는다. strippedmetadata는 `table-vpos-marker-origin-release-metadata.json`.
- 후속f3f2d9bd(#3386)와84764fa2(#7941)의일반클릭점관계화에서도잘못된19경로는그대로남았다.
- `table-vpos-marker-audit-finding.json`, `observe_table_vpos_marker.rs`, producer/observation/raw-char-shapes에원본및실행증거.
- 추가현재같은run의일반scalar경로는 [0,20.5333333,41.0666667]이며실제한box18.6666667과다르지만기존16~30범위는둘다PASS한다.
  같은소유자paint끝203.08과실제cursor끝203.1관계는정상0.02오차를수용하고generic1.8666667오차를검출한다.
  0.051정밀도는cursor JSON소수첫째자리직렬화오차에서오며PDF절대좌표핀아니다.
  `observe_table_vpos_marker_counterexample.rs`/producer/observation/`table-vpos-marker-predicate-counterexample.json`.
  이것은현재public EmbeddedTextMeasurer의관계식반례이며과거전체바이너리나실제production mutation/newformaltest실행이아니다.
- 실제테스트재작성/issue_8232파일함수새명명/등록갱신/실제잘못된cursor분기에서FAIL→정상PASS검증은아직미완료.

## 도구 문제와 정정

Native rlib의직접serde_json은 `libserde_json-1dd0ed11f36f95bd.rlib`이다.
root `.fingerprint/rhwp-0f20a479faca3de2/lib-rhwp.json`의serde_json dep5840575122845678688과
`.fingerprint/serde_json-1dd0ed11f36f95bd/lib-serde_json`의little-endian fingerprint가정확히맞는다.
9466ccd...는동일version이지만serde_coreidentity가달라PageRenderTree직렬화에서컴파일실패했다.
실패로그는 `phase13-inputs-first-driver-serde-mismatch.log` 및 `fit-prefix-after-compile-serde-mismatch.log`에보존했다.
정확한의존성으로두생성기재실행성공/같은바이트확인.
counterexample처음private replay_positions_for호출도실패했으며로그보존후public TextMeasurer로수정,최종compile/run exit0.
PDF문자추출은첫학글자21회라실패한초안을전체제목학 력 사 항으로수정해유일성을검사했다.
실패초안을원검사/출력실패로세지않는다.

## 원격 이슈

#8232 body 현재9,899/4,985/91조건/2493쪽, 추가발견/px2개유지근거를반영했다.
현재댓글 https://github.com/edwardkim/rhwp/issues/8232#issuecomment-6402726546
`phase13-survey-report.md`와remote본문정확일치까지재확인했다. `phase13-published-receipt.json`에hash와receipt.
body snapshot `issue-body-phase13.md`. 기존보고서는과거snapshot으로보존한다.

## 다음 작업

실제상태확인후4,985미분류+434증거보류·간접helper/fixture/E2E호출폐쇄를계속한다.
`tests/cases/issue_7998_pdf_retained_border_structure.rs`2176줄은처음260줄만읽었고,
`issue_7998_pdf_table_inference.rs`2223줄은아직전건의미분류전이다. 읽지않은범위를승인하지않는다.
기존Recipe/Hyperlink/hidden-relative/1133등확정문제도여전히실제보정전이다.
전수완료를선언하거나기존PASS로모두적합처리하지않는다. 목표전체완료전update_goal complete/paused를호출하지않는다.
