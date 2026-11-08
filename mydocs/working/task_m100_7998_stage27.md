# #7998 stage27 — 병합·나누기의 소유 보존과 HWPX 격자 왕복

2026-10-19 메인테이너가 최종 Studio의 **“시각 판정 통과 / 편집 구조 통과”**로
판정했다. 승인된 R1 절편을 마무리하며 전체 R1·R2·P5 완료로 확대하지 않는다.
기준은 `d69233b17c730f2f4d64a0ec8d6a3b6e21309625`다. 코드 검증 뒤 추가한 것은
정식 검사·공개 생성 입력/독립 PDF·기록이며 제품 소스와 런타임은 그대로다.

## 적용 규칙과 실제 소비

1. 원본 경계를 retained paint로 보존한 표에서 merge/split API가 성공한 뒤 소유 정보를
   잃는 결함을 재현했다. `ensure_table_cell_topology_supported`
   (`src/document_core/commands/border_paint_structure.rs`)가 대상 직접 셀의 paint와
   같은 Table ID의 표 밖 소유를 확인한다. `table_ops.rs`의 네 native 병합/나누기
   진입점에서 문단 이동·reflow·이벤트·저장 변경 전에 거절한다. 1×1 나누기 no-op은
   유지한다. 다른 표/자식 표의 소유와 단순 내용 그림을 대상의 소유로 간주하지 않는다.
   이 원자 거절은 미지원 source paint의 일반 재배분을 구현한 것이 아니다.
2. 정상 native 병합에서는 서로 다른 셀의 문단이 같은 저장 vpos를 유지해 겹쳤다.
   첫 후보의 독립 Print 대비 Native는 88.85449%, WASM은 87.47153%였다. 병합 후
   폭 reflow → 기존 `rebuild_table_cell_vpos_ladder_native` → 공통
   `apply_cell_vpos_ladder`의 줄 높이/줄간격/문단 간격 → 새 LineSeg의 vertical_pos로
   연결했다(`table_ops.rs:628`, `commands/text_editing.rs:87,2935`). 저장 좌표를
   옮기며 본문을 숨기거나 bbox를 clamp하지 않는다. 측정은 기존 height_measurer의
   셀 문단 LineSeg를, 실제 배치는 table_layout의 저장 줄 원점/점유를 소비한다.
   실제 RenderTree의 문단 순서·비겹침·셀 내부 포함을 두 저장 후에도 검사한다.
3. 정상 native 세로 병합의 HWPX가 한컴에서 읽기/저장 오류였다. live IR의 두 행은
   기존 split 역연산의 의미가 있으므로 모델 병합 결과를 축소하지 않는다.
   `src/model/table_row_grid.rs`가 완전한 비중복 셀 피복을 실제 셀에서 확인한 뒤,
   어느 셀/영역도 시작·끝나지 않는 중복 행 컷만 HWPX 표현에서 투영한다.
   Table/셀의 명시적 크기·본문·LineSeg는 유지하며 측정·paint·pagination 코드는
   변경하지 않는다. 물리 빈 영역·중복 소유·의미 있는 영역 경계는 제거하지 않는다.
4. 같은 writer는 원래 행 경계 맵을 `META-INF/rhwp-table-row-grids.json`에 기록한다.
   4MiB 직렬화 예산을 예약 전에 확인하며 projected Table ID/격자/크기 fingerprint와
   경계 맵을 저장한다. parser는 유일 ID·strict schema·fingerprint·단조 경계·기존
   4M grid 슬롯 제한을 확인하고 **paint 소유 복원 전**에 원래 격자를 복원한다.
   stale/malformed/중복 entry는 표준 셀을 덮어쓰지 않는다. 정상 행은 새 entry 없이
   그대로 기록한다. 한컴은 표준 투영 표를 편집하고 private 정보를 버릴 수 있음을
   `MetadataReduced` 손실 보고에 명시한다. 한컴의 latent-grid 보존은 주장하지 않는다.
5. 개별 Table 쓰기 오류를 문단 writer가 잡아 표를 누락할 수 있는 경로도 확인했다.
   새 Writer가 실패를 보존하고 package encode가 오류를 반환하도록 연결했다.
   표현 불가·중복 ID·한도를 표 없는 성공 HWPX로 바꾸지 않는다.

적용 경계는 셀 topology 명령과 HWPX 표 표현이다. 일반 행열 상속·명시적 테두리
교체는 기존 경로로 유지한다. 조각 컷/예약 높이·continuation을 바꾸는 구현은 없으므로
새 pagination 경계 전체 검증으로 보고하지 않는다. 현재 검사도 이 범위의 1쪽 표다.

## 독립 원인 분리와 수정 전후

- 빈 tr **표기**만 생략하거나 self-closing으로 변경한 진단, xmlVersion 1.5→1.4
  진단은 모두 한컴 오류였다. 해당 실험/패치는 ignored 증거에 보존하고 제품에서
  철회했다. 기존 xmlVersion 1.5를 전역 변경하지 않았다.
- 한컴이 HWP에서 직접 재저장한 `rowspan-merge-from-hancom.hwpx` 역시 다시 열면
  오류였다. 정상 생성본으로 인용하지 않는다. 별도 `rowspan-range-fromhancome-02.hwpx`는
  정상 열림 대조군이다. 실패/정상 구분과 사용자 판정을 원래 증거에 유지한다.
- 셀 내용·크기·글줄을 그대로 두고 중복 행 축만 정리한
  `rowspan-merge-grid-diagnostic.hwpx`는 사용자가 정상 열림으로 확인했다.
  실제 오류/정상 입력과 OWPML tr/tc 제약이 투영의 독립 근거다. 같은 grid 진단과
  최종 HWPX의 기존 ZIP payload는 같고, 원래 IR 복원 entry만 추가된다.
- 동일 정상 HWP를 보존한 구버전 CLI로 HWPX 재저장하면 2행/빈 tr 1개/sidecar 없음으로
  row 계약 FAIL, 최종 CLI는 1행/빈 tr 0개/sidecar 있음으로 PASS다
  (`final/before-after-export.json`). 보정 저장본의 새 한컴 Print도 성공했다.
  정식 새 검사 전체를 구버전 lib에서 다시 실행했다는 주장은 하지 않는다.
- 보정 HWPX·그 뒤 행 삽입 HWP/HWPX·HWP→HWPX 재저장 네 파일의 정확한 바이트를
  한컴 2020 Print로 검증했다. 원본 399×360pt 용지·one-up·PrintMethod 0이며
  engine 11.0.0.9136이다. 구버전 실패 입력을 재제출해 대기열을 늘리지 않았다.

## 제품 빌드·실제 API·전체 페이지

증거 루트는 ignored `output/pr-review/issue7998/p4-merged-border-inheritance/`이며
실제 안정 경로는 `/mnt/e/tmp/issue7998-stage27-evidence/`다. **`final/`만 최종 후보**다.
이전 후보·철회 진단의 빌드/점수를 최종 검증에 합산하지 않는다.

- 공용 `/home/edward/mygithub/rhwp/target/pr-review`에서 jobs 14로
  `cargo build --locked --profile release-test --features native-skia --bin rhwp --lib`를
  실행했다. 최종 Native SHA-256은
  `61cc3cd5a8acc2d9856266ff7297c4b375139b3551888017187cd9dedda2d8da`다.
- canonical Docker `scripts/wasm-pack-locked.sh --target web --out-dir pkg`를
  jobs 14·codegen-units 16으로 실행했다. fresh WASM은 19,691,452바이트,
  `73bc7e504539bea447489808782cffaf97aeb48e0a4f693656e2c0f41e38b0db`다.
  glue는 `5671802fbb89ed060c3bab7a6f458a78833a7e5ca3e7df8e1699d45098cddeb6`이다.
  최종 제품 8개 소스 해시와 frozen Native/pkg·실제 Studio의 런타임을
  `final/source-runtime.json`으로 연결했다. 이후 제품 빌드를 반복하지 않았다.
- 최종 **Native/fresh WASM 각 24입력·24쪽**, raw 최저는
  **99.93299% / 98.27024%**, 90% 미만·누락·전체 쪽수 차이는 0이다.
  원본 4·원문 보존 양 저장 4·실제 편집/왕복 16개다. 원본은 PDF 자체와,
  편집 저장본은 실제 해당 편집의 독립 Print와 비교했다. 실제 인쇄 원본 형식이 다른
  공통 대조를 별도 형식의 직접 Print로 세지 않는다. 기존 20개 Native PNG는
  앞선 같은 배치 후보와 바이트 동일하다(`final/native/compat-png-identities.json`).
- 전체 TSV/입력·기준 PDF·producer 명령·페이지 해시는
  `final/{native,wasm}/visual/<key>/silhouette.tsv`, `silhouette_manifest.json`과
  `results.json`에 있다. silhouette-only의 `not_evaluated`는 승인 판정이 아니며,
  다음 직접 판독/사용자 구조 판정과 따로 보존한다.
- 수동 A4 기준 중 colspan 병합·rowspan range/split은 원본 용지의 새 Print로 교체했다.
  제품 소스/입력/기존 PNG가 같은지 확인하고 PDF raster와 비교만 갱신했다
  (`final/reference-only-reuse.json`). 옛 수동 기준/TSV는 `final/manual-paper-reference/`에
  유지했다. 새 비교에서 같은 물리 좌표의 문단·아래 변을 다시 확대 판독했다.
- `final/reviews/`의 대표 compare·standalone overlay·review와 `final/zoom/`에서
  문단 비겹침·새 행·부모/자식·다른 셀·OUTSIDE 문구를 확인했다. 기존 가는 선 농도와
  backend 글꼴 smoothing 차이는 남으며 점수로 면제한 새 구조 결함은 없다.
  이동 가능한 화면은 `E:\tmp\issue7998-topology-border-review\index.html`이다.
- 실제 fresh Studio의 2개 원본 × 8개 public 경로(확장 API 포함), 실패 Snapshot rollback과
  1×1 no-op을 검증했다. 모델·이벤트·실제 SVG·양 저장은 무변경이다. 정상 6개 Snapshot
  편집과 undo/redo도 통과했고 16개 저장본은 Native와 바이트 같다
  (`final/studio-result.json`, `verify-studio.log`). Hwpctrl wrapper는 위임 경로를 코드로
  확인했으며 별도 독립 실행한 것으로 세지 않는다.
- 실제 HWPX 재열기 후 복원된 잠재 행으로 split하면 2→3셀, undo 2셀/redo 3셀과
  저장 바이트가 유지됐다(`final/reopened-grid-actual.json`). 이는 실제 모델/명령/저장
  계약 증거이며 이 후속 split 결과의 별도 한컴 Print 증거로 확대하지 않는다.
- 사용자 CDP의 기존 탭은 유지하고 최종 네 새 탭 `issue7998-topology-final-{retained,
  native-merge-h,native-merge-v,appended-row}`을 열었다. 로드한 실제 WASM 해시·1쪽 표시를
  `final/user-cdp-tabs.json`에 연결했고 메인테이너가 **시각·편집 구조 통과**로 판정했다.

## 추가 인쇄 차이의 승인된 경계 — B23

추가 자동 인쇄의 동일 `rowspan-merge.hwp`는 표 세로 변이 페이지 아래까지 늘어나
Native raw 56.38381%다. 사용자 직접 HWP 인쇄는 정상 높이이며 HWPX 자동 인쇄도
정상이다. 사용자 Print는 11.0.0.9166·A4, MCP는 11.0.0.9136·399×360pt라 버전/용지/
실행 조건을 아직 분리하지 못했다. 서비스의 성공 응답을 정상 출력으로 분류하지 않는다.

2026-10-19 메인테이너가 **“정상 PDF 기준·추가 차이는 백로그”**로 승인했다.
이번 정상 기준은 사용자 제공 HWP PDF를 유지한다. 같은 물리 내용 원점·배율로 비교하며
PDF를 사후 정합/확대하지 않았다. 원본 용지 Print 계약의 이 한 파일 제한도 명시한다.
실제 56.38% 미달과 두 출력은 [공개 추가 호환성 자료](../../pdf/issue7998/cell-regions/topology/compatibility/README.md)에
보존했다. 원인은 미확정이며 한컴 버전 문제나 rhwp 저장 결함 어느 쪽으로도 단정하지 않는다.
MCP는 잘못된 HWP/HWPX에서 30분 timeout 뒤 오류를 반환할 수 있다는 사용자 설명을
반영하고, 같은 실패 입력을 반복 제출하지 않는다.

## 정식 검사·공개 증거·남은 범위

- 기존 자동 수집 `tests/cases/issue_7998_pdf_retained_border_structure.rs`에 12개 검사를
  추가했다. Native 실제 RenderTree의 문단 소유·순서/비겹침·셀 포함, 양 저장·잠재 격자
  복원/나누기, 후속 행의 경계 맵, sidecar의 stale/변형/중복, 정상 행 무변경,
  물리 빈 영역/겹침/zone 경계·중복 ID의 전체 export 실패, 대상/다른 표/자식/내용 그림의
  소유 범위를 보호한다. 후자의 메모리 내 반례는 소유 계약 검사이며 한컴 생성/출력
  fixture로 주장하지 않는다. 새 절대 px나 SVG golden·완화된 baseline은 없다.
- 같은 기존 9모듈의 최종 집중 실행은 **80 passed / 0 failed / 0 filtered**다
  (`final/guards-registered.log`). `compile_tests.py guards`와 `guards --test-threads 14`의
  standalone harness는 실제 자동 수집 source를 그대로 사용하며 별도 ci.yml 호출은
  추가하지 않았다. 전체 suite를 실행한 결과가 아니다. 영향 없는 기존 통과 검사는
  테스트/문서 정리를 이유로 다시 실행하지 않는다.
- [공개 입력·생성기·정상 Print·provenance](../../pdf/issue7998/cell-regions/topology/README.md)를
  연결했다. 정상 API 생성 20개는 별도 폴더에서 바이트까지 재현했다
  (`final/generator-reproduction.json`). `provenance.json`에는 실제 인쇄 원본 형식과
  source/runtime·입력/PDF 해시·24개 쪽별 결과를 기록한다. 로그·임시 JSON·런타임은
  커밋하지 않는다. 기존 대상 검사도 내용/소유/관계 검사이며 교체할 절대 px 핀은 없다.
- 보존 paint 병합/나누기의 source closure와 일반 R1 상속, B23 추가 호환성 차이,
  R2 저장 수평 메트릭의 17줄 및 P5 최신 devel 통합·전쪽 최종 대조·전체 회귀/lint·
  Worker/package 검증은 남아 있다. `parser_allocation_bounds_proven=false`를 유지한다.
  이번에는 원격 push·PR 상태 변경·댓글·전체 suite를 수행하지 않는다.
