# #5511 Stage 2 기능군 배치 실행계획

- 이슈: [#5511](https://github.com/edwardkim/rhwp/issues/5511)
- 브랜치: `task_m100_5511`
- 재계측 기준: `cb337e70cd4febbd7028a28d4d56ec49aba23ea9`
- 통합 기준선: `upstream/devel` `1a6ce79fd56e3cdf5813c7938338fcb5b7d0a859`
- 작성일: 2026-08-19
- 상태: 실행 승인 — Q1·Q2·Q3 완료, Q4 inventory 중단 조건 발동·책임 분해안 승인 대기

## 1. 전환 이유

Stage 2 절편 24개 중 19개가 handler 이동, 4개가 선행 계약 보강, 1개가 PR 제출 구조 정정이다.
이 과정은 외부 동작 보존과 안전한 모듈 API를
입증했지만, 단일 handler 단위에서는 승인·전체 회귀·보고 비용이 실제 이동량보다 커졌다.

| 지표 | #5511 시작 | 현재 | 최종 기준 |
|---|---:|---:|---:|
| `src/main.rs` 줄 수 | 42,370 | 38,561 | 1,200 이하 |
| 최상위 함수 | 351 | 310 | entrypoint 조립에 필요한 최소 함수 |
| 새 query 경로의 최상위 명령 | 0 | 19 | 모든 query adapter 물리 분리 |
| 새 query 경로의 `inspect` 하위 명령 | 0 | 4 | 현재 전수 |
| 편집 handler | 92 | 92 | command 모듈로 전수 이동 |
| `wasm_api::HwpDocument` 직접 참조 | 42 | 42 | Stage 3에서 0 |
| `rhwp::service` 참조 | 0 | 0 | Stage 3에서 전환 |

1,200줄 목표까지 37,361줄이 남았다. 지금까지의 handler 이동 평균만 기계적으로 적용하면 약
190개 이동 절편이 더 필요하다. 이는 실제 예측이 아니라 현재 절차를 그대로 유지하면 생기는
비효율의 크기를 보여주는 지표다. 큰 책임을 기능군으로 묶되 보호 계약과 중단 조건은 유지한다.

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
기록했다. 책임 분해안을 승인하기 전에는 제품 코드를 변경하지 않는다.

### Wave M/P — metadata와 에이전트 protocol 분리

| 배치 | 범위 | 핵심 관문 | 예상 구현 커밋 |
|---|---|---|---:|
| M1 | MCP definitions·capabilities payload·help projection | catalog 정본 유지, byte/semantic 동등성 | 3~5 |
| P1 | replay·audit·lineage·anchor·gate·bundle·disclose·settle·harness | capsule 계보와 보안 경계별 모듈 분리 | 4~6 |

M1은 7,569줄을 하나의 새 God module로 옮기지 않는다. schema, capabilities projection, help
projection을 각각 1,200줄 이하 모듈로 나누고 catalog와 기계적으로 동형임을 유지한다. P1도
명령 이름이 아니라 capsule, anchor/gate, disclosure/settlement, harness 책임으로 나눈다.

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
