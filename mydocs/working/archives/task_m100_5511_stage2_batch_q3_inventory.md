# #5511 Stage 2 기능군 배치 Q3 — data exchange inventory와 복잡도 중단

- 이슈: [#5511](https://github.com/edwardkim/rhwp/issues/5511)
- 브랜치: `task_m100_5511`
- 시작 기준선: `ec2306eee`
- characterization 커밋: `4cbeec460`
- 수행일: 2026-08-19
- 상태: 중단 조건 발동 — 책임 분해안 승인 대기

## 1. 실제 명령과 CQRS 소유권

Q3의 실제 사용자 명령은 여덟 개다. 기존 계획의 대략적 함수 수 10은 전용 helper 둘을 포함한
작업량 좌표였으며, dispatch handler 수는 8이다.

| 소유권 | 명령 | 이동 후보 |
|---|---|---|
| text output | `export-text`, `export-llm`, `export-markdown` | `cli/outputs/text.rs` |
| tabular output | `export-tables`, `table-to-csv`, `chart-to-csv` | `cli/outputs/tabular.rs` |
| state-changing command | `csv-to-table`, `csv-to-chart` | `cli/commands/tabular_import.rs` |

`csv-to-table`과 `csv-to-chart`는 CSV를 읽어 문서를 변경·직렬화하므로 `outputs`에 둘 수 없다.
Stage 0 inventory도 두 명령을 `edit`로 분류한다. Q3에서 `cli/commands/mod.rs`를 처음 만들고
두 import handler만 그 경계에 두는 것이 CQRS 불변식과 현재 batch 범위를 함께 만족한다.

현재 함수 경계로 계산한 예상 본문은 text 약 753줄, tabular output 약 544줄, tabular import
약 676줄이다. 분해 helper와 import를 더해도 각 1,200줄 상한 안이다. `tables_json_value`,
edit serialize/verify 등 MCP·batch·다른 edit가 함께 쓰는 seam은 root에 남기고 복제하지 않는다.

## 2. 보호 계약 inventory

기존 계약은 다음 사용자-visible 축을 직접 보호한다.

- text: JSON/기본 파일, page 주소, 무제한 기본값, `--max-chars`, 옵션 순서, 실패 종료 코드
- tables: 병합 span·중첩·container, JSON/사람 요약, batch 동형성, 실패 stdout 순수성
- LLM: JSONL/JSON, 바이트 결정성, chunk·token·mode, 본문 coverage, 오류 종료 코드
- table/chart CSV export: 직사각 행렬, quoting, 다중 표/차트, JSON 봉투, 파일명, BOM 분리
- CSV import: dry-run, invalid 행렬, changed/wrote, 출력 형식 보존, verify, provenance
- Markdown: JSON 매니페스트, 옵션 순서, 쓰기 실패와 페이지 산출

Markdown의 이미지 자산 바이트와 상대 링크만 기존 계약에 없었다. 공개
`samples/issue2817/paper_anchor_infront_pic.hwpx`로 이미지 1개, 876바이트 PNG의 SHA-256,
`*_assets/...png` 상대 링크를 고정했다. 해당 output-axis 범위 8/8 통과 후 독립 커밋했으며,
새 integration source나 generated 산출물은 추가하지 않았다.

관련 9개 integration target을 묶은 이동 전 기준선은 938/938 통과, 1 slow, 3 skipped,
65.794초다. 테스트 준비 manifest도 최신 base에서 754 sources / 3,726 static test attrs /
43 integration targets로 통과했다. Markdown link 검사는 기존 capability 등록부 무결성 오류
16건만 재현했고 Q3 신규 오류는 없다.

## 3. 중단 조건 증거와 원인

Clippy `cognitive_complexity` 기본 feature 계측 결과 두 handler가 상한을 넘었다.

| handler | CC | 원인 | 판정 |
|---|---:|---|---|
| `csv_to_table` | 37 | option parser, CSV·문서 읽기, table 선택, 적용, 직렬화·verify가 한 함수 | 중단 |
| `export_markdown` | 33 | option parser, 페이지 loop, control/bin fallback, asset·MD write가 한 함수 | 중단 |

나머지 여섯 handler와 전용 helper는 CC>25 경고에 나타나지 않았다. `csv_to_table` 37은 Stage 0
정적 inventory에도 기록되어 있었다. 이는 누락된 발견이 아니라, 해당 기능군에 도달했을 때
분해안을 승인받도록 batch 중단 조건이 의도대로 작동한 것이다. `export_markdown`은 Stage 0의
상위 몇 개 예시에는 적지 않았지만 이번 기능군 전수 계측으로 확인했다.

두 함수를 그대로 새 파일로 옮기면 root의 줄 수만 줄이고 복잡도와 CQRS 혼합을 새 위치로
숨기게 된다. 따라서 제품 handler는 아직 이동하지 않았다.

## 4. 선택지

### A. 책임 분해 후 CQRS 경계로 이동 — 권장

같은 Q3 안에서 두 handler를 먼저 제자리에서 CC 25 이하로 분해한다.

- Markdown: option parser, 이미지 control/bin fallback 해석, asset write를 분리
- CSV→table: option parser, 입력·table 선택 준비, 적용 결과·저장/verify 보고를 분리

분해 HEAD에서 938개 focused 범위와 CC를 확인해 독립 커밋한 뒤, 여덟 handler를 위의
text output·tabular output·tabular import 세 모듈로 이동한다. 최종 HEAD에서 같은 focused
범위, 전체 release-test, 정적·정책 게이트를 수행한다. stdout/stderr·파일 바이트·BOM·JSON/
NDJSON·종료 코드가 달라지면 즉시 중단한다.

### B. 저복잡도 handler 여섯 개만 먼저 이동

`csv-to-table`과 `export-markdown`을 root에 남기고 나머지만 이동한다. 즉시 위험은 작지만
text 기능군과 CSV 왕복 쌍이 다시 갈라지고, Q3를 별도 승인 배치로 반복해야 한다.

### C. Q3 보류

Markdown characterization만 유지하고 Q4로 넘어간다. 복잡도 이동 위험은 없지만 data exchange
책임 약 1,973줄과 CQRS 혼합이 root에 남아 Stage 2 종료 조건을 달성할 수 없다.

## 5. 원격 위험

최종 재조회 기준 `origin/devel`과 `upstream/devel`은 `625758ee6`로 같고, Q3 시작 HEAD와 최신
base의 merge-tree는 충돌 없이 생성됐다. 열린 PR 16개의 head별 파일을 확인했으며 `src/main.rs`,
예상 신규 모듈, Q3 관련 contract 파일과 교집합은 0개다.

이 판정은 시점 증거이므로 구현 재개와 push 전 exact base SHA·PR head·merge-tree를 다시 확인한다.
remote push는 수행하지 않았다.

## 6. 승인 요청

권장안 A는 Q3의 물리 이동 전에 두 고복잡도 handler의 내부 책임 분해와 첫 command 모듈 경계를
추가한다. 제품 기능이나 출력 규약은 바꾸지 않는다. 승인되면 제자리 분해·focused 검증·독립
커밋을 먼저 수행하고, 세 CQRS 모듈 이동과 최종 배치 검증까지 이어간다.
