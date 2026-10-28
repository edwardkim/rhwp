# #5511 Stage 2 기능군 배치 Q3 — data exchange adapter 경계

- 이슈: [#5511](https://github.com/edwardkim/rhwp/issues/5511)
- 브랜치: `task_m100_5511`
- 구현 시작 기준: `5114a9f95`
- 수행일: 2026-08-19
- 상태: 완료 — Q4 진입 승인 대기

## 1. 결과

메인테이너가 승인한 A안에 따라 고복잡도 handler를 먼저 제자리에서 분해하고, 검증된
여덟 data exchange handler를 CQRS 책임에 맞는 세 모듈로 이동했다.

| 모듈 | 소유 명령 | 최종 줄 수 |
|---|---|---:|
| `cli/outputs/text.rs` | `export-text`, `export-llm`, `export-markdown` | 765 |
| `cli/outputs/tabular.rs` | `export-tables`, `table-to-csv`, `chart-to-csv` | 547 |
| `cli/commands/tabular_import.rs` | `csv-to-table`, `csv-to-chart` | 745 |

`src/main.rs`는 Q3 시작의 36,453줄에서 34,495줄로 1,958줄 줄었다. 새 구현 모듈은 모두
1,200줄 상한 이하고, root에는 해당 명령의 최상위 dispatch만 남았다. `csv-to-table`과
`csv-to-chart`는 문서를 변경하고 직렬화하므로 output이 아닌 command 경계가 소유한다.
MCP·batch·다른 edit도 쓰는 `tables_json_value`, serialize·verify, cell resolve seam은 root에
한 번만 남겨 helper 복제를 피했다.

## 2. A안 책임 분해와 보호 계약

이동 전에 다음 책임을 `src/main.rs` 안에서 먼저 분리했다.

- `csv_to_table`: option parser와 행·열·병합·제어문자 선검증
- `export_markdown`: option parser, control 좌표 이미지 해석, BinData fallback

| handler | 분해 전 CC | 분해·이동 후 판정 |
|---|---:|---|
| `csv_to_table` | 37 | CC 경고 없음, 25 이하 |
| `export_markdown` | 33 | CC 경고 없음, 25 이하 |

CSV 크기 불일치·병합 셀·제어문자 거부 순서, 변경 적용, 저장·verify와 stdout/stderr·exit code는
바꾸지 않았다. Markdown도 control 이미지 우선, BinData fallback, 경고 문구, asset 이름과
상대 링크를 유지했다. parser·serializer·renderer 알고리즘은 변경하지 않았다.

기존 계약에 없던 Markdown 이미지 축은 공개
`samples/issue2817/paper_anchor_infront_pic.hwpx`로 먼저 고정했다. 1개 PNG, 876바이트,
`paper_anchor_infront_pic_assets/paper_anchor_infront_pic_p001_img001.png` 상대 링크와 SHA-256
`7f0977caf24233a6196c3b9898fb2f051bc24226ba14e1e680313a6699f95a33`가 분해·이동 뒤에도
동일함을 검증했다.

## 3. 커밋 계보

| 커밋 | 역할 |
|---|---|
| `4cbeec460` | Markdown 이미지 링크·자산 바이트 characterization |
| `5114a9f95` | Q3 inventory와 복잡도 중단·A안 선택지 기록 |
| `5a5c1442b` | 두 고복잡도 handler를 제자리에서 CC 25 이하로 분해 |
| `23d180332` | 여덟 handler를 text·tabular output과 tabular import command로 이동 |

분해 HEAD와 최종 이동 HEAD에서 같은 focused 범위를 실행했고 각각 1,044/1,044가 통과했다.
따라서 책임 분해와 물리 이동 전후를 같은 계약 모집단으로 비교했다.

## 4. 최종 검증

| 검증 | 결과 |
|---|---|
| 이동 전 focused nextest | 1,044/1,044 통과, 2 slow, 4 skipped |
| 이동 후 focused nextest | 1,044/1,044 통과, 2 slow, 4 skipped |
| release-test 전체 nextest | 7,776/7,776 통과, 3 slow, 38 skipped, 184.050초 |
| 대상 모듈 CC 25 상한 | 경고 없음 |
| `cargo fmt --all -- --check`·`git diff --check` | 통과 |
| `cargo check --all-targets` | 통과 |
| `cargo clippy --all-targets -- -D warnings` | 통과 |
| `cargo test --doc` | 8/8 통과, 3 ignored |
| integration manifest 정책 자체 계약 | 16/16 통과 |
| 최신 base manifest check | 754 sources / 3,727 static test attrs / 43 integration targets, 통과 |
| unit-tier 정책 자체 계약과 base check | 12/12, 4,225 tests / 298 modules, 통과 |
| CI impact Node·workflow 계약 | 62/62, 30/30 통과 |
| Markdown link check | 기존 capability 등록부 무결성 오류 16건, Q3 신규 오류 없음 |

검증 준비가 `Cargo.toml`에 만든 두 integration target과 Cargo가 재정렬한 lockfile package 순서는
추적 변경에서 제거했다. 새 integration source나 generated test target은 커밋하지 않았다.

이번 변경은 CLI adapter의 제어 흐름과 물리 위치만 바꾸며 renderer/layout/WASM 경계를 수정하지
않는다. exact Markdown 이미지 자산, CSV, JSON/NDJSON과 전체 회귀 계약이 직접 통과했으므로 시각
sweep과 WASM 빌드의 발생 조건에는 해당하지 않는다고 판정했다.

## 5. 원격 병합 위험

최종 재조회 시 `origin/devel`과 `upstream/devel`은 `161820019cfb`로 같고, 구현 HEAD
`23d180332`는 10커밋 뒤·69커밋 앞이다. 공통 조상 이후 원격 10개 커밋은 Q3 제품·test 경로와
겹치지 않았으며 최신 base와 구현 HEAD의 merge-tree는 충돌 없이 생성됐다.

열린 PR은 19개이고 `src/main.rs`, 새 CQRS 모듈, Q3 contract 파일과 교집합은 0개다. 이 증거는
시점 판정이므로 push 전에는 exact base SHA, PR head와 merge-tree를 다시 확인한다. remote push는
수행하지 않았다.

## 6. 다음 승인 단위

다음 기능군은 Q4 `scan·batch·batch query 봉투`다. NDJSON 입력 순서, 병렬 결정성, 건별 실패
격리와 공용 query 봉투 seam을 inventory하고, 미보호 계약이 있으면 독립 characterization
커밋을 먼저 만든다. Q4는 메인테이너의 Q3 배치 종료 승인과 진입 승인 전 시작하지 않는다.
