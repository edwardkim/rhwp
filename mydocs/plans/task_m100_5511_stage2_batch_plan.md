# #5511 Stage 2 기능군 배치 실행계획

- 이슈: [#5511](https://github.com/edwardkim/rhwp/issues/5511)
- 브랜치: `task_m100_5511`
- 재계측 기준: `cb337e70cd4febbd7028a28d4d56ec49aba23ea9`
- 통합 기준선: `upstream/devel` `1a6ce79fd56e3cdf5813c7938338fcb5b7d0a859`
- 작성일: 2026-08-19
- 상태: 실행 승인 — Wave Q(Q1~Q7)·M1·P1·C0·C1·C2 완료, C3 진입 승인 대기

## 1. 전환 이유

Stage 2 절편 24개 중 19개가 handler 이동, 4개가 선행 계약 보강, 1개가 PR 제출 구조 정정이다.
이 과정은 외부 동작 보존과 안전한 모듈 API를
입증했지만, 단일 handler 단위에서는 승인·전체 회귀·보고 비용이 실제 이동량보다 커졌다.

| 지표 | #5511 시작 | 현재 | 최종 기준 |
|---|---:|---:|---:|
| `src/main.rs` 줄 수 | 42,370 | 28,295 | 1,200 이하 |
| 최상위 함수 | 351 | 225 | entrypoint 조립에 필요한 최소 함수 |
| 새 query 경로의 최상위 명령 | 0 | 28 | 모든 query adapter 물리 분리 |
| 새 query 경로의 `inspect` 하위 명령 | 0 | 4 | 현재 전수 |
| 편집 handler | 92 | 92 | command 모듈로 전수 이동 |
| `wasm_api::HwpDocument` 직접 참조 | 42 | 22 | Stage 3에서 0 |
| `rhwp::service` 참조 | 0 | 0 | Stage 3에서 전환 |

1,200줄 목표까지 현재 27,095줄이 남았다. 배치 재기준화 당시에는 단일 handler 이동 평균을
기계적으로 적용하면 약 190개 이동 절편이 더 필요한 상태였다. 이는 실제 예측이 아니라 기존
절차를 그대로 유지하면 생기는 비효율의 크기를 보여준 지표다. 큰 책임을 기능군으로 묶되 보호
계약과 중단 조건은 유지한다.

## 2. 배치 실행 규약

하나의 기능군 배치는 다음 순서를 한 승인 단위로 수행한다.

1. 최신 `upstream/devel`, 열린 PR 경로, 현재 계약과 공유 seam을 확인한다.
2. 미보호 동작이 있으면 characterization test를 독립 커밋으로 먼저 고정한다.
3. 같은 기능군의 handler와 전용 helper를 새 모듈로 이동한다. 이동과 기능 변경은 섞지 않는다.
4. 이동 커밋마다 focused test와 format·diff 검사를 실행한다.
5. 배치 끝에서 전체 release-test, clippy, doc-test, manifest, unit-tier, CI 정책 검사를 한 번
   실행한다.
6. 지표·계약·원격 위험을 하나의 배치 보고서로 기록하고 로컬 커밋한다.
7. 메인테이너가 결과를 승인한 뒤 다음 기능군 배치로 넘어간다.

기본 배치 크기는 한 도메인, 3~12개 handler, 약 600~2,500 이동 줄, 2~4개 로컬 커밋이다.
characterization이 필요 없으면 테스트 커밋은 생략한다. 기능군이 2,500줄을 넘거나 새 모듈
하나가 1,200줄을 넘으면 같은 승인 단위 안에서 책임별 하위 모듈로 나눈다.

전체 회귀를 배치 종료에 모으는 것은 검증 축소가 아니다. 각 커밋에는 관련 focused 계약을
적용하고, 배치의 최종 HEAD가 기존의 전체·정적·정책 관문을 모두 통과해야 완료된다.

## 3. 현재 잔여 책임 지도

아래 줄 수와 함수 수는 재계측 기준 SHA의 함수 경계를 이용한 작업량 좌표다. 이후 원격 변경에
따라 줄 번호가 움직이므로 실제 이동은 함수 이름과 호출 계보를 기준으로 한다.

| 책임 구간 | 현재 줄 | 함수 | 배치 소유권 |
|---|---:|---:|---|
| MCP·capabilities·help metadata | 7,569 | 22 | M1 metadata projection |
| SVG·render tree·PNG·GPU·PDF | 1,950 | 19 | Q2 render output |
| text·table·LLM·CSV·Markdown | 1,973 | 10 | Q3 data exchange |
| scan·batch·공용 query 봉투 | 1,953 | 34 | Q4 batch pipeline |
| shape·form 초기 edit | 1,087 | 10 | C2 shape/form |
| info·dump-pages·dump-controls | 1,778 | 6 | Q5 diagnostics |
| convert·extract-pages·HWPX/HML·생성 | 1,650 | 20 | Q6 conversion |
| internal round-trip·IR·verify | 1,684 | 16 | Q7 verification |
| edit dispatch·serialize 공통부 | 256 | 5 | C0 command runtime |
| replay·audit·anchor·gate·bundle 등 | 5,686 | 54 | P1 agent protocol |
| field·text·redact·sanitize edit | 1,380 | 11 | C1 text/document |
| cell·table·equation edit | 3,639 | 33 | C3 table/objects |
| paragraph·page·note·bookmark edit | 3,181 | 26 | C4 document structure |
| char·para·cell style edit | 1,720 | 13 | C5 formatting |
| image·picture·shape edit | 978 | 5 | C2 shape/form |
| inspect 공유 seam·thumbnail | 440 | 5 | Q1 preview boundary |
| header/footer·note tail edit | 1,152 | 9 | C6 header/footer |

## 4. 기능군 순서

### Wave Q — 조회·출력 adapter 완결

| 배치 | 범위 | 핵심 관문 | 예상 구현 커밋 |
|---|---|---|---:|
| Q1 | `thumbnail`과 preview output 경계 | 파일 부작용 때문에 query가 아닌 output adapter인지 판정 | 1~2 |
| Q2 | SVG·render-tree·structure·PNG·GPU·PDF | renderer 동작 불변, 시각 게이트 발생 조건 분리 | 2~3 |
| Q3 | text·tables·LLM·CSV·Markdown | stdout·파일·BOM·JSON 봉투 동등성 | 2~3 |
| Q4 | scan·batch·batch query 봉투 | NDJSON 순서, 병렬 결정성, record 실패 격리 | 2~3 |
| Q5 | info·dump-pages·dump-controls | CC 68인 `dump_controls`를 그대로 옮기지 않고 책임 분해 | 2~3 |
| Q6 | convert·extract-pages·HWPX/HML·ingest·scaffold | parser/serializer 동작 변경 없이 adapter만 분리 | 2~3 |
| Q7 | internal round-trip·IR diff/sweep·verify | 진단·검증 exit code와 diff 계약 보존 | 2~3 |

Q1은 완료했다. 기존 `thumbnail` 테스트의 빈틈이었던 내장 이미지 바이트 동등성, 기본 출력
경로와 저장 실패를 보강하고, 파일 부작용을 명시하는 `cli/outputs/preview.rs`로 handler를
이동했다. 세부 증거는
[`task_m100_5511_stage2_batch_q1.md`](../working/task_m100_5511_stage2_batch_q1.md)에 기록했다.

Q2도 완료했다. 미보호 상태였던 GPU feature stub 계약을 먼저 고정하고, CC 25를 넘던
SVG·PNG·GPU·PDF handler를 parser와 준비 helper로 분해한 뒤 vector·raster·PDF output
모듈로 이동했다. 세 모듈은 모두 1,200줄 이하이고 renderer 알고리즘과 관찰 가능한 출력은
바뀌지 않았다. 세부 증거는
[`task_m100_5511_stage2_batch_q2.md`](../working/task_m100_5511_stage2_batch_q2.md)에 기록했다.

Q3도 완료했다. Markdown 이미지 상대 링크와 자산 바이트를 먼저 고정하고, CC 25를 넘던
`csv_to_table`과 `export_markdown`을 parser·검증·fallback helper로 분해했다. 읽기 전용
text·tabular output과 상태 변경 CSV import를 서로 다른 CQRS 모듈로 이동했으며, 세 구현
모듈은 모두 1,200줄 이하이다. 세부 증거는
[`task_m100_5511_stage2_batch_q3.md`](../working/task_m100_5511_stage2_batch_q3.md)에 기록했다.

Q4 inventory에서 `cmd_scan`의 CC 28을 확인해 중단 조건이 발동했다. batch의 ordered stream과
query/write 책임 경계, 기존 92개 인접 계약, scan characterization 공백과 세 선택지는
[`task_m100_5511_stage2_batch_q4_inventory.md`](../working/task_m100_5511_stage2_batch_q4_inventory.md)에
기록했다. 승인된 A안에 따라 scan의 사람 출력·symlink 비추적 계약을 먼저 고정하고, `cmd_scan`을
CC 25 이하로 분해한 뒤 scan query, ordered runtime, batch query, fill·convert command를 분리했다.
세부 증거는 [`task_m100_5511_stage2_batch_q4.md`](../working/task_m100_5511_stage2_batch_q4.md)에
기록했다.

Q5는 최신 `devel` `52d8bf8eb3`에서 시작했다. `show_info` CC 34와 `dump_controls` CC 68을
재현했고, 기존 1,096줄 `diagnostics.rs`에 합치지 않는 책임별 모듈 경계를 확정했다. 사람용
성공 stdout의 byte-level characterization과 분해 기준은
[`task_m100_5511_stage2_batch_q5_inventory.md`](../working/task_m100_5511_stage2_batch_q5_inventory.md)에
기록했다.

Q5도 완료했다. info·page·control 진단을 일곱 책임 모듈로 이동하고, `dump_controls`를 순회,
shape, table, story 출력으로 분해했다. 완료 직전 전진한 `devel`을 정상 merge한 뒤 #5542의
의도된 HWP3 첫 문단 `SectionDef` 출력에 characterization 기준을 정합화했으며, 최신 결합 HEAD의
전체·정적·정책 관문을 통과했다. 세부 증거는
[`task_m100_5511_stage2_batch_q5.md`](../working/task_m100_5511_stage2_batch_q5.md)에 기록했다.

Q6는 최신 `devel` `980bf59e4`를 정상 merge한 뒤 시작했다. 변환·생성 handler 전부 CC 25 이하이고,
기존 17개 계약 모듈 123/123이 JSON·exit·검증·원본 보호·생성물 재파싱을 이미 보호하므로 신규
characterization 없이 책임별 물리 이동으로 진행한다. 대상과 공유 seam 판정은
[`task_m100_5511_stage2_batch_q6_inventory.md`](../working/task_m100_5511_stage2_batch_q6_inventory.md)에
기록했다.

Q6도 완료했다. 변환 command, 문서 generation, DocLang output을 세 모듈로 이동했고 모두
1,200줄 이하이며 CC 25 초과 경고가 없다. 완료 직전 전진한 `devel`의 별도 q-pack 변경을 정상
merge하고, 결합 HEAD에서 Q6·q-pack focused 127/127과 전체 release-test 7,999/7,999를 포함한
정적·정책 관문을 다시 통과했다. 세부 증거는
[`task_m100_5511_stage2_batch_q6.md`](../working/task_m100_5511_stage2_batch_q6.md)에 기록했다.

Q7 inventory에서 `ir_diff_paragraph_fields` CC 28, `cmd_verify` CC 29, `ir_diff` CC 38과
`test-field` 성공·position diagnostics·`ir-sweep` 계약 공백을 확인해 중단 조건이 발동했다.
기존 직접 계약 104/104 기준선과 세 선택지는
[`task_m100_5511_stage2_batch_q7_inventory.md`](../working/task_m100_5511_stage2_batch_q7_inventory.md)에
기록했다. 권장안 A는 최소 characterization 뒤 세 함수를 책임 분해하고 Q7 전체를 이동한다.

Q7도 완료했다. 미보호였던 내부 저장·position diagnostics·IR sweep 계약 6개를 먼저 고정하고,
internal validation command와 position·verification·IR comparison query를 네 모듈로 이동했다.
세 고복잡도 함수는 scalar/control, parsing/evaluation, load/compare/output 책임으로 분해해 Q7
모듈의 CC 25 초과 경고를 0건으로 만들었다. 최종 focused 110/110과 전체 release-test
8,005/8,005를 포함한 정적·정책 관문을 통과했다. 세부 증거는
[`task_m100_5511_stage2_batch_q7.md`](../working/task_m100_5511_stage2_batch_q7.md)에 기록했다.

### Wave M/P — metadata와 에이전트 protocol 분리

| 배치 | 범위 | 핵심 관문 | 예상 구현 커밋 |
|---|---|---|---:|
| M1 | MCP definitions·capabilities payload·help projection | catalog 정본 유지, byte/semantic 동등성 | 3~5 |
| P1 | replay·audit·lineage·anchor·gate·bundle·disclose·settle·harness | capsule 계보와 보안 경계별 모듈 분리 | 4~6 |

M1도 완료했다. MCP 도구 정의 7개, capabilities projection 2개, help projection 3개 기능군과
각 조립 모듈로 나눴으며 전 파일이 1,200줄 이하이다. catalog 정본은 이동·복제하지 않았고 여섯
공개 출력의 byte hash와 8,005개 전체 계약이 유지됐다. 세부 증거는
[`task_m100_5511_stage2_batch_m1.md`](../working/task_m100_5511_stage2_batch_m1.md)에 기록했다.

P1도 완료했다. agent protocol 구현을 명령 이름이 아니라 capsule, trust, exchange, harness,
plan 책임으로 나눴고, 이동 전 CC 25를 넘던 6개 함수는 관찰 가능한 계약을 유지한 채 책임별
helper로 분해했다. 범용 CAS seam은 이후 C0도 사용하는 불변식이므로 root에 유지했다. 새 파일은
모두 1,200줄 이하이고 직접 계약 97/97과 전체 release-test 8,005/8,005가 통과했다. 세부 증거는
[`task_m100_5511_stage2_batch_p1.md`](../working/task_m100_5511_stage2_batch_p1.md)에 기록했다.

### Wave C — 상태 변경 command 분리

| 배치 | 범위 | 핵심 관문 | 예상 구현 커밋 |
|---|---|---|---:|
| C0 | `run_edit`, serialize·verify·write 공통 seam | command module이 공유할 최소 runtime API 확정 | 1~2 |
| C1 | field·text·replace·redact·sanitize | 입력 무훼손과 저장 검증 경계 | 2~3 |
| C2 | chart·shape·form·image·picture | binary 자산·anchor·target 선택 계약 | 2~3 |
| C3 | cell·row·column·table·equation | table 좌표와 `finish_edit_write` 수명주기 | 3~4 |
| C4 | paragraph·page·section·note·bookmark·control | 문서 구조와 위치 선택 계약 | 3~4 |
| C5 | char·para·cell style·formatting | 범위·상속·스타일 적용 계약 | 2~3 |
| C6 | header/footer·footnote/endnote tail | story 경계와 전용 위치 계약 | 2~3 |

C0에서 `EditContext` 또는 동등한 명시적 의존 묶음이 필요한지 결정한다. 이때 service 계층
전환까지 선행하지 않고, Stage 2에서는 기존 동작을 전달할 최소 CLI runtime seam만 만든다.
실제 `DocumentService`, typed error, 전역 인증 제거는 Stage 3 범위로 남긴다.

C0도 완료했다. 88개 edit 하위 명령 dispatch와 공통 output format·serialize·verify·write를
`cli/commands/edit/` 아래로 이동하고, edit와 protocol plan이 함께 쓰는 SHA-256·CAS path lock은
`cli/integrity.rs`의 범용 seam으로 분리했다. 기능군별 의존이 확인되기 전에 god object가 되는
광범위한 `EditContext`는 만들지 않았다. 직접 계약 101/101과 전체 release-test 8,005/8,005가
통과했으며 최신 `devel`의 별도 Studio 변경을 정상 merge했다. 세부 증거는
[`task_m100_5511_stage2_batch_c0.md`](../working/task_m100_5511_stage2_batch_c0.md)에 기록했다.

C1도 완료했다. 기존 1,380줄·11함수 계측에 field occurrence 정본 parser 20줄·1함수를 더해
실범위를 1,400줄·12함수로 보정하고, field·문서 전역 replace·privacy command를 세 모듈로
이동했다. `edit_replace_text`와 `edit_redact`는 option parsing과 실행을 나눠 CC 29·33을 모두
상한 이하로 낮췄다. 직접 계약 113/113과 전체 release-test 8,005/8,005가 통과했다. 세부 증거는
[`task_m100_5511_stage2_batch_c1.md`](../working/task_m100_5511_stage2_batch_c1.md)에 기록했다.

C2도 완료했다. 분리되어 있던 insert-image 전용 helper까지 실범위에 포함해 chart·number·form·
page border, shape lifecycle, image·picture command를 세 모듈로 이동했다. `edit_insert_image`는
argument parsing과 실행을 나눠 CC 27을 상한 이하로 낮췄다. 최신 #5647 chart B2 계약을 포함한
직접 계약 146/146과 전체 release-test 8,008/8,008이 통과했다. 세부 증거는
[`task_m100_5511_stage2_batch_c2.md`](../working/task_m100_5511_stage2_batch_c2.md)에 기록했다.

## 5. 중단 조건

다음 중 하나가 생기면 같은 승인 배치 안에서 다음 handler로 진행하지 않는다.

- help, exit code, stdout/stderr, JSON/NDJSON, MCP schema·annotation의 관찰 가능한 차이
- parser·serializer·renderer 알고리즘 변경 없이는 이동할 수 없는 경우
- 새 모듈 1,200줄 초과 또는 CC>25 함수를 분해 없이 다른 파일로 옮기게 되는 경우
- root와 새 모듈의 양방향 참조 또는 기능군 사이 helper 복제
- 최신 `devel`이나 열린 PR이 같은 handler·test·module 경계를 변경한 경우
- characterization이 현재 동작을 정상 규약으로 고정해도 되는지 판단이 필요한 경우

이 경우 현재 focused 결과와 선택지를 보고하고 메인테이너 결정을 기다린다. 단순한 테스트 추가나
동일 기능군 내부의 물리 이동만으로는 별도 승인 절편을 만들지 않는다.

## 6. Stage 2 종료 조건

Stage 3 진입 전 다음을 모두 만족한다.

1. root에는 프로세스 초기화, 전역 옵션 pre-scan, 최상위 dispatch와 exit 전파 및 Stage 3
   입력으로 명시한 공용 seam만 남고, command/query handler 구현은 남지 않는다.
2. query, output/export, command, metadata, protocol adapter가 물리적으로 분리된다.
3. 편집 handler 92개가 `cli/commands/`의 책임별 모듈에 위치한다.
4. 각 새 CLI 모듈은 1,200줄 이하이며 CC>25 함수를 새 위치로 숨기지 않는다.
5. catalog와 help·capabilities·MCP의 이름·가시성·참여 계약이 계속 동형이다.
6. 전체 release-test와 정적·정책 검증이 최종 Stage 2 HEAD에서 통과한다.
7. 남은 `HwpDocument`, parser/model/renderer/serializer, 전역 상태 직접 의존을 Stage 3 입력
   inventory로 고정한다.

Stage 2의 예상 잔여량은 16개 기능군 승인 배치와 약 35~53개 구현 커밋이다. 이는 현재 코드
기준 범위이며, 중단 조건이나 원격 통합에 따라 조정한다. Stage 3·4의 승인과 수행은 이 계획으로
자동 개시하지 않는다.
