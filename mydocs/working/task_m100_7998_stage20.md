# #7998 다축 compound 직선 경계의 편집 소유

- 승인: stage19 `1669ba04b`의 시각·편집 구조 통과 뒤 다음 절편 진행 승인.
- 실제 native setter에서 여러 MoveTo/LineTo를 하나의 S로 그린 표의 오른쪽 변 지우기는
  source paint undo를 생성하지 않았다(`output/pr-review/issue7998/p4-compound-border/before.log`).
  같은 그림 전체를 한 변으로 태그하면 다른 변도 삭제되므로 bbox만으로 소유를 부여하지 않는다.
- 첫 경계는 서로 분리된 axial MoveTo/LineTo 서브경로의 compound stroke다. 같은 원본 PNG와
  실제 선분·dash/phase·alpha를 공유하고, 선택된 root 격자의 논리 변에 해당하는 면만 나눈다.
  교차점은 실제 잉크가 겹치는 변들의 집합으로 소유한다. 하나를 지운 뒤 다른 변의 잉크를
  유지하고 마지막 소유를 지운 때만 원본 조각을 제거한다. 가림/면·곡선·연결 join·임의 장식이
  섞인 source와 root 밖으로 연장된 선의 일반 closure는 보존/미확정으로 남긴다.
- source path → bounded PNG stroke witness → 실제 ink와 물리 셀/페이지 분할 → 공통
  CellBorderPaint의 변 집합 → native setter와 undo journal → 기존 Picture 공통 paint·양 저장을
  추적한다. 그림/배경/문단은 보존한다. 줄 측정·높이·pagination은 바꾸지 않는다.
- 같은 source의 서로 다른 축을 단순 source ID만으로 공유 변이라고 묶지 않는다. 실제 이웃
  셀의 논리 변을 확인한다. sidecar는 기존 V1–V3 읽기·4MiB 한도를 유지하고 필요한 다중 변만
  새 버전으로 보존한다. stale crop/배치/PNG·잘못된 mask가 삭제 권한을 얻지 않는지 확인한다.
- 독립 PDF 연산자의 solid·dash·위상/alpha·직선 그림/배경 대조, 한 변/전체 지우기와 연속 교체,
  실제 command undo/redo·문자 역편집·HWP/HWPX 저장, 독립 한컴 Print를 대조한다. Native/fresh
  Docker WASM 전쪽 TSV와 직접 판독 후 정식 회귀를 추가한다. 기존 runtime이 같으면 이전
  판정과 producer를 재사용하며 source가 바뀌면 영향 범위를 새로 검증한다.
- 증거는 ignored `output/pr-review/issue7998/p4-compound-border/`와 저장소 밖 대응 폴더다.
  전체 회귀/lint·Worker/package·최신 devel 통합은 P5다. 이 절편은 P4 전체 완료가 아니며
  `parser_allocation_bounds_proven=false`를 유지한다. 원격 반영은 하지 않는다.

## 실제 소비 경로와 보완

| 구현 주장 | 생산·실제 소비와 적용 경계 |
|---|---|
| 하나의 source 안의 여러 변을 구별한다 | `path_paint::lower`가 독립 axial MoveTo/LineTo의 PNG witness를 발행한다. `pdf_stroke::segment_rectangles`의 같은 dash/phase·방향 결과를 `table_paint::compound`의 실제 잉크 소유와 `renderer::pdf_stroke::project`가 소비한다. Fill·matrix·곡선·connected join은 이 편집 소유를 얻지 않는다. |
| 다른 축과 교차점의 잉크를 보존한다 | 선택된 단일 root의 단일 span 셀과 각 원본 선분의 전 구간을 확인한다. 물리 셀 조각의 교차점은 실제 잉크가 차지하는 변 집합을 보존한다. `border_paint_edit::plan`은 source ID만 비교하지 않고 실제 논리 이웃을 확인하고, 남은 mask 갱신/마지막 조각 삭제를 같은 journal에 넣는다. |
| 저장 후에도 실제 변 교체를 계속한다 | 공통 `CellBorderPaint.sides`와 bounded V4 sidecar를 양 adapter로 보존한다. 단일 변의 JSON/기존 V1–V3 기록은 그대로 읽고 필요한 다중 mask만 V4로 쓴다. Picture crop·현재 resource·owner의 검증을 유지한다. 한컴 자체에서 이 사설 편집 소유를 소비한다는 주장은 하지 않는다. |
| 측정 뒤 별도 소비에서 선이 잘리지 않는다 | `structured_lowering`의 물리 frame/crop → `ImageNode` → `PageRenderTree::clip_overlapping_same_bin_images` → 공통 Path paint를 추적했다. 기존 1px x/폭 근접 조건이 겹치지 않는 가는 조각을 잘랐다. 실제 가로 교집합과 세로 겹침을 확인하고 수치 roundoff의 접점은 제외한다. 실제 겹친 그림의 기존 정리 경로는 유지한다. |
| 지운 변이 native bitmap의 빈 조각에서 되살아나지 않는다 | Native는 빈 vector crop을 그리지 않지만 한컴의 PNG 소비에는 AA 경계 샘플이 남았다. 인증된 root 선의 소유 envelope를 HU 격자에 바깥 방향으로 양자화하고, PDF 잉크가 없다고 확인한 root/페이지 나머지는 Picture로 만들지 않는다. source vector 좌표·폭·alpha는 바꾸지 않는다. 인증되지 않은 그림/장식은 유지한다. 다른 target이 이 경계 잉크와 만나는 경우 편집 소유 인증을 보류한다. |

줄 높이·문단 측정·LineSeg·분할 컷/이어받기·pagination은 바꾸지 않았다. 실제 page/cell frame과
source crop의 소비를 검사한다. geometric frame 분할의 면적 보존은 잉크 없는 영역을 생략하기
전 분류에 적용한다. 빈 원본 padding을 보존해야 할 내용 그림과 혼동하지 않는다.

## 실행으로 확인한 결함과 보완 순서

1. `before.log`: 실제 native setter의 source undo 없음 FAIL → `after.log`/최종 `edits.log`의
   undo 생성·다른 축/자식 보존 PASS. 처음 fixture의 점선 교차점에도 반드시 다중 mask가
   있어야 한다는 진단 가정은 제거했다. 실제 dash gap에는 공동 잉크가 없을 수 있다.
2. `clip-guard-before.log`: 서로 인접한 0.3px crop의 높이 40→10 FAIL →
   `clip-guard-after.log`의 원래 높이 보존 PASS. 실제 겹침 대조군은 원래대로 잘린다.
   중간 `alpha-tree.json`에서 source 5,000HU 조각이 25HU로 줄고 crop도 잘리는 것을 확인했다.
3. `padding-guard-before.log`: 네 변 지우기 뒤에도 owner 없는 source crop이 남는 FAIL →
   `padding-guard-after.log`의 제거 PASS. 한컴 Print 확대에서 확인한 잔상이 수정 근거다.
   점수 90% 이상을 이 결함의 면제로 사용하지 않았다.
4. 여러 축 첫 보정과 빈 padding 보정 전 producer/저장본/Print/capture는
   `before-empty-padding/`에 보존한다. 이 자료를 최종 후보의 검증으로 합산하지 않는다.
   connected axial join·장식·연장·diagonal join은 별도 보존 반례다.

## 검증 기록 범위

최종 Native 원본 7개는 각 1쪽이며 최저 99.97209%다. 기존 실물·배경·가림·단일 축 등
21문서/32쪽의 Native를 다시 산출했고 90% 미달·누락이 없다. 기존의 5pt 배경 확장 수용은
stage19의 동일 범위로 유지한다. 새 source의 불투명 선에서 기존 raster와 공통 vector의
AA 경계 RGB가 달라지며, Native raw 최대 하락 0.00018%p를 별도 확대 화면에 표시한다.
같은 점수와 PNG 동일을 혼동하지 않는다. fresh WASM·편집 Print·직접 판정과 정식 검사
등록 결과는 최종 실행이 끝난 뒤 아래에 연결한다.

디스크 부족은 제품 오류와 구분한다. 첫 WASM 캡처의 기록이 ENOSPC로 중단됐으며, 기존
source/target/cache를 삭제하지 않았다. 해당 증적 폴더를 `/mnt/e/tmp/issue7998-stage20-evidence/`
아래로 옮기고 원래 경로에 symlink를 유지해 약 2GB를 확보했다. 최종 WASM 캡처는 같은
외부 디스크의 `*-final/`에 둔다. 완료되지 않은 TSV/JSON은 통과로 세지 않는다.

## 최종 검증과 직접 수용 — 2026-10-19

| 최종 source의 검증 | 결과 |
|---|---|
| Native / fresh Docker WASM | 각각 40개 입력·51쪽. 전체 최저 **96.41335% / 96.40682%**, 90% 미만·누락·쪽수 차이 0. `final-validation-summary.json`은 모든 TSV와 page count manifest를 확인한다. `--silhouette-only`의 자동 gate는 `not_evaluated`이며 이 수치만으로 승인했다고 보고하지 않는다. |
| 새 source 7개 | 각 1쪽. 불투명 실선·점선·alpha 및 connected axial join/장식/연장/diagonal join을 구분했다. alpha는 기준 Native 89.29272%→99.97209%, WASM 89.09203%→99.97270%. 불투명 source의 최대 raw 하락 0.00018%p와 AA RGB 차이는 확대 비교에 남겼다. |
| 같은 편집의 독립 한컴 Print 12개 | HWP·HWPX를 각각 한컴 2020 11.0.0.9136, method 0/one-up으로 인쇄. Native 최저 **98.59282%**, fresh WASM **96.94986%**. 두 형식 모두 정상 변환·1쪽이며, 네 변 지우기의 빈 crop 잔상도 사라졌다. |
| 기존 대조군 21문서·32쪽 | stage19의 동일 입력/기준과 비교해 **Native/fresh WASM 모두 PNG 변경 0쪽**. `final-existing-deltas.json`. 앞 절의 모든 쪽수에 이미 포함돼 있다. |
| 실제 Studio 명령 | 6동작, 문자 삽입·삭제 뒤 undo/redo와 양 저장 재열기 통과. page error 0. 12개 출력은 Native API 출력과 SHA-256이 같아 해당 입력의 Native/fresh WASM·한컴 비교를 재사용한다(`actual-identical-inputs.json`). 별도 12쪽으로 합산하지 않는다. |
| 신규 정식 회귀 | `regression_suite_022`의 실제 자동 수집 경로에서 **9 passed / 0 failed**. `formal-compound-tests.log`. 소유·이웃·내용/자식 보존, joint 순차 삭제, 양 저장/undo, stale/invalid mask, 비인증 반례, crop 최종 소비와 빈 bitmap 잔상을 검사한다. |
| 기존 집중 회귀 | 실제 기존 함수 **53 passed / 0 failed**, `existing-guards-final.log`. 9건과 합쳐 **62개 함수**이며 반복 진단 실행을 추가 검사 수로 세지 않는다. |

Native 후보 SHA-256은 `65d8003aa32b7ed230d7eba2bcbc13c4b4159bef4f741488faa7008b3e4ec47f`다.
Fresh Docker WASM은 **19,564,500바이트**, SHA-256
`eb2a741437f4ba3a2131012be95a1cb1893dc5f2be83ad1fdf38c3cb3c15972c`다.
JS glue SHA-256은 `5671802fbb89ed060c3bab7a6f458a78833a7e5ca3e7df8e1699d45098cddeb6`로
기존과 같고 추적 중인 Studio JS/types를 바꾸지 않았다. `source-runtime.json`의 runtime
8파일과 최종 작업 파일의 SHA가 같음을 확인했다. 문서/회귀 입력 추가 뒤 제품을 재빌드하지 않는다.

주요 재현 명령은 다음과 같다. Cargo target은 공통 host 경로를 재사용했다.

```bash
CARGO_BUILD_JOBS=14 cargo build --locked --profile release-test \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review --features native-skia --bin rhwp
docker compose -p rhwp --env-file .env.docker run --rm \
  -e CARGO_BUILD_JOBS=14 -e CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16 wasm
CARGO_BUILD_JOBS=14 cargo test --locked --profile release-test \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review --features native-skia \
  --test regression_suite_022 issue_7998_pdf_compound_border_editing -- --test-threads=4
```

전체 TSV 명령과 producer/기준 PDF/hash는 `final-validation-summary.json`·각 run manifest·
`print-sources.json`·`print-hwp-sources.json`에 있다. 독립 입력 7개와 한컴 Print 12개,
재생성 script/provenance는 `pdf/issue7998/compound-borders/`에 둔다. generator가 7개의
SHA-256을 그대로 재생성함을 확인했다. 새 rendering 회귀는 최종 Native/fresh WASM의
90%와 직접 확대 대조 뒤 등록했다. 기존 정상 검사의 기대값·기준값을 바꾸지 않았다.

직접 비교는 `/mnt/e/tmp/issue7998-compound-border-review/index.html`의 원본·편집·중간 잔상
보정 전후와 같은 원점 overlay/교차점 확대에서 수행했다. 실선/점선/alpha의 실제 변 교체,
지운 영역, 이웃 변·자식 표·앞뒤 문장 보존을 확인했다. 기존 Helvetica의 한컴 대체 글꼴과
PDF/Skia/CanvasKit의 가는 선 농도 차이는 별도로 표시했다. source SHA가 바뀐 앞선 중간
capture를 최종 증거로 사용하지 않았다. 기준 stage19 producer만 변경 전 비교에 재사용했다.

실제 사용자 CDP 새 탭 4개는 모두 위 새 WASM SHA와 1쪽 표시를 확인했다
(`user-cdp-tabs.json`). 메인테이너가 **2026-10-19 “시각·편집 구조 통과”**로 판정했다.
이 절편은 수용됐으며 다음 R1 경계는 인증된 source의 행·열 상속과 삭제/복원이다.
일반 compound·부분 병합/접한 중첩 이웃·R2·P5는 남는다. 전체 회귀/lint·Worker/package·
최신 devel 통합이나 PR 준비 완료를 주장하지 않으며 `parser_allocation_bounds_proven=false`를
유지한다. 원격 push·댓글·PR 상태 변경은 수행하지 않았다.
