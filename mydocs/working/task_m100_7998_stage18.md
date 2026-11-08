# #7998 단일 축 raster stroke의 셀 테두리 교체

- 승인: stage17 직접 시각·편집 구조 통과 후 다음 절편 진행. 시작 head `50580add6`.
- 원인: 실제 PDF 직선 dash stroke가 Picture로 내려오면 원본 paint는 보존되지만 셀 변의 편집 소유는 전달되지 않는다. BorderFill 변경만으로는 이전 그림 선을 제거할 수 없다.
- 이번 경계: 실제 PDF MoveTo/LineTo·단일 축·Butt stroke에 대한 source 증명, 물리 분할 closure와 셀/페이지 잔여 연결. 임의 그림·fill·곡선·여러 축 bitmap은 bbox만으로 인증하지 않는다. 기존 source 픽셀·crop·phase·alpha를 유지하며 명시적 변 교체만 실제 control을 제거한다. 새 행열·부분 병합/중첩 이웃·fill 일반화·R2/P5는 남는다.
- 생산 PathContent → table_paint의 소유 결과 → 실제 cell/page anchor → native sidecar와 setter preflight/undo를 연결한다. Picture는 현재 bitmap 바이트·crop·변환·앵커 지문이 맞아야 편집 권한을 유지한다. 저장 한도는 기존 4MiB를 유지한다.
- 독립 입력은 기존 공개 nested dash 원본 및 같은 PDF 연산자로 만든 phase/alpha/clip 대조군이다. 무편집은 PDF, 명시적 편집 양 저장은 독립 한컴 Print를 기준으로 확인한다. 증거는 ignored `output/pr-review/issue7998/p4-raster-border/`에 보존한다.

## 추가 판정과 선의 샘플링 원인

새 root alpha 0.4/phase 1.5 대조군은 수정 전·후 Native PNG가 바이트 동일하지만 raw 89.29272%(이전 WASM 89.09203%)다. 메인테이너는 선 출력 보정도 이번에 포함하도록 지시했다. 가는 선의 source raster는 0.5pt를 1.5개 샘플로 표현하고, 셀 분할 crop과 축소 필터에서 coverage가 다시 약해진다. 실제 96dpi 상단 선은 PDF의 회색153 한 줄에 비해 Native의 회색232 두 줄로 나타난다. alpha 자체를 바꾸거나 DPI/threshold를 바꾸지 않는다.

횡단 방향의 샘플 수만 늘린 시도는 실제 출력을 개선하지 못했고, 가로선이 사라지는 경계도 검출했다. 수정 코드는 제거했다. Native Skia의 Strict crop에서 mipmap 유무와 무관하게 source coverage가 빠지는 것을 별도 실제 Skia probe로 확인했다(sampling-probe.log). 중간 Native·Print와 중단된 WASM은 최종 성공으로 합산하지 않는다.

최종 보정 방향은 실제 단일 축 Butt stroke의 frame/끝점/폭/색/alpha/dash/phase를 원래 PNG에 bounded rhST witness로 보존하고, 편집용 ImageNode를 유지하면서 LayerBuilder와 legacy SVG/WebCanvas에서 같은 crop을 적용한 PathNode로 시각 투영하는 것이다. 모든 backend가 그 공통 결과를 소비한다. bitmap은 native 파일에서 그대로 보존한다. payload는 4096바이트·dash 64개·실제 분해 8192단계 상한이며 PNG 원문 SHA-256과 CRC를 확인한다. 일반 내용 그림·fill·곡선·비사각 clip·변환은 이 witness를 발행하지 않는다. 그림 효과·회전·HF 참조의 별도 경로는 기존 bitmap 소비를 유지한다.

기하의 생산은 PathContent → source PNG의 원본 witness와 crop → Image layout의 실제 frame → 공통 PathNode의 dash 채움이다. 가림은 저장 crop과 실제 셀 clip의 교집합이며, alpha나 선 폭을 바꾸거나 잘못된 원점을 clamp하지 않는다. 흐름 측정/줄 나눔/분할 컷은 바꾸지 않는다. vector_logic.log에 phase의 첫 짧은 run·opacity·바뀐 PNG의 witness 거절을 실제 코드로 실행했다. 최종 Native/fresh WASM과 독립 Print 검증 전에는 보정 완료로 판정하지 않는다.

## 구현·실제 소비와 반례

| 주장 | 생산 → 실제 소비 | 적용 경계·반례 |
|---|---|---|
| Picture인 원본 경계도 실제 셀 변으로 교체한다 | `parser/pdf/table_paint.rs`의 실제 PathContent 축/역할 → `structured_lowering.rs`의 최종 셀/페이지 앵커와 seal → `model/border_paint.rs`의 `PaintRef`/resource 지문 → `commands/border_paint_edit.rs`의 원자적 preflight·삭제·undo | 단일 축 Butt stroke와 같은 source의 물리 조각 전체가 확인될 때만 적용한다. 일반 source XObject·같은 bbox의 무태그 그림·곡선/fill은 삭제하지 않는다. 실제 확장선은 별도 page paint로 남기며, 투명 raster pad와 실제 Butt 끝점이 일치하는 경우만 외곽 논리 소유에 연결한다. |
| 저장본에서도 같은 편집 권한을 복원한다 | 기존 두 adapter → `border_paint::encode/decode` V3 Picture flag·지문 → 현재 PNG/resource/crop/변환과 결합 | V1/V2 읽기와 4MiB 한도를 유지한다. 바뀐 crop/크기/resource/reference/transform은 변 변경 전에 거절한다. 잘못된 record 종류·버전·source ID 0·잘린 metadata를 검사한다. |
| 가는 선의 alpha·위상을 필터 손실 없이 소비한다 | `path_paint.rs`의 원래 216dpi PNG + `model/pdf_stroke.rs`의 원본 stroke witness → ImageNode의 실제 frame/crop → `renderer/pdf_stroke.rs::project` → `paint/builder.rs`의 Path paint | 편집 RenderTree는 ImageNode를 유지한다. source stroke를 가로/세로 채움 구간으로 전개해 실제 crop과 교차한다. Native Skia·fresh WASM·Studio CanvasKit의 공통 layer와 legacy SVG/WebCanvas가 같은 결과를 소비한다. 별도 효과/변환/HF 참조는 bitmap fallback이며 해당 경로의 vector 승격을 주장하지 않는다. |

`model/bin_data.rs`의 공통 index-first/sparse-ID resolver를 기존 renderer와 지문 검사에서
함께 소비한다. 별도의 저장 ID 해석을 추가하지 않았다. 줄 높이·문단 측정·pagination의
컷/예약 높이는 변경하지 않아 분할·이어받기 변경 검증은 이번 경계에 비해당이다.

## 최종 빌드·시각·편집 검증

증거 기준 경로는 `output/pr-review/issue7998/p4-raster-border/`이며, 실제 자료는 저장소 밖
`/home/edward/mydev/rhwp-review-evidence/pdf-table-feedback-20261017/semantic-stage3/p4-raster-border/`에
보존한다. 중간 실패·중단 산출물은 `before-line-fix/` 등으로 구분하며 아래 결과에 합산하지 않았다.
제품 코드가 바뀐 최종 Native와 fresh Docker WASM만 아래 결과의 기준이다.

| 최종 범위 | Native 쪽수 / 최저 raw | fresh WASM 쪽수 / 최저 raw | 독립 기준 |
|---|---|---|---|
| 원본 14문서 | 25 / 97.57194% | 25 / 97.13390% | 같은 원본 PDF 전쪽 |
| Native API 실제 변 편집 3종 × HWP/HWPX | 6 / 98.89000% | 6 / 97.44269% | 같은 형식의 편집 저장본을 한컴에서 Print |
| 실제 Studio command 편집 3종 × HWP/HWPX | 6 / 98.89000% | 6 / 97.44269% | 위 Print 및 두 API의 저장 소유 지문 동일성 |
| 합계 | 37쪽, 90% 미만·누락 0 | 37쪽, 90% 미만·누락 0 | 쪽수 일치 |

- 원본 alpha 경계: Native **89.29272% → 99.97209%**, WASM **89.09203% → 99.97270%**.
  alpha와 source 폭·dash phase를 바꾸지 않고 공통 paint 소비를 보정했다.
- 기존 불투명 접점의 최대 하락은 Native **0.00028%p**, WASM **0.00029%p**다.
  무회귀/PNG 동일로 보고하지 않았으며 비교 화면에 변경 전후 확대를 표시했다.
  `native-source-deltas.json`·`wasm-source-deltas.json`과 판정을 함께 보존한다.
- source 원본에 대한 legacy SVG 실제 출력도 독립 PDF와 같은 96dpi PNG로 대조해
  raw **100%**를 확인했다. 별도의 `legacy-svg-visual/`이며 위 37쪽 수에 합산하지 않는다.
- 두 실물의 최종 CLI 재가져오기에서 보고서 **15표**, 견적서 **1표**를 유지한다.
  `instanceId`만 정규화한 page diagnostics와 실제 저장본의 재귀 Paragraph 원문/스타일·
  Table/cell 크기·병합·padding signature가 stage17과 동일하다.
  `source-semantic-summary.json`·`semantic-actual-readback.json`에 기록했다.
  견적서 원본·식별 정보·글꼴은 로컬 검증용이며 커밋하지 않는다.
- 실제 `SetCellBorderFillCommand`에서 첫 셀 외곽 교체·네 변 교체·지우기를 실행하고,
  X 삽입/삭제 뒤 undo·redo, 양 저장 재열기와 snapshot 해제를 확인했다.
  3건 모두 undo 원문/redo 바이트 일치, snapshot 0, page error 0이다.
  `studio-result.json`·`wasm-native-readback.log`에 실제 소비를 연결했다.
- 편집 저장본 6개 모두 한컴 2020 **11.0.0.9136**에서 열기·Print에 성공했다.
  `pdf_print_method=0`, `pdf_output_mode=hancom2020_pdf_driver_one_up`이며 입력/PDF SHA와
  작업 ID를 `print-sources.json`·`print-hwp-sources.json`에 보존했다. 기존 Helvetica의
  한컴 대체 글꼴과 점선 접점 농도 차이는 비교 화면에 별도로 명시했다.

최종 Native 빌드는 `CARGO_BUILD_JOBS=14 cargo build --locked --profile release-test
--target-dir /home/edward/mygithub/rhwp/target/pr-review --features native-skia --bin rhwp`,
WASM은 `docker compose -p rhwp --env-file .env.docker run --rm -e CARGO_BUILD_JOBS=14
-e CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16 wasm`이다. 명령/로그는
`native-build-final.log`·`docker-wasm-final.log`, 전쪽 실행 명령은 backend별
`results-source.json`·`results-edited.json`·`results-actual.json`, TSV는 각 `visual/`에 있다.
`validation-summary.json`에 26건/37쪽씩의 집계를 기록했다.

| 산출물 | SHA-256 |
|---|---|
| 최종 Native CLI | `6cb25dfa79f55ff2e678eb3b609efb87b9d3390a1777115c602fe798fdf69c81` |
| fresh WASM | `e65214a3448667d48f0ef41fdf07f9a6115b4f60f4dd43a0ac32b9551c14168d` |
| JS glue | `5671802fbb89ed060c3bab7a6f458a78833a7e5ca3e7df8e1699d45098cddeb6` |
| runtime 2,800경로 manifest | `57ec03dbc9dbda3005289b2c438d256e17389cadacc95946ecc6ae2a3708e96d` |

`source-runtime.json`의 현재 바이트 일치와 worktree Studio public의 WASM/JS 일치를 확인했다.
실제 CDP 19222의 Studio 7793 네 탭도 같은 WASM SHA를 로드했다(`user-cdp-tabs.json`).
기본 Studio 7700은 변경하지 않았다. 기존 제품 빌드·캡처는 반복하지 않았다.

## 직접 판정·정식 회귀와 다음 경계

2026-10-19 메인테이너는 `E:\tmp\issue7998-raster-border-review\index.html`과 Studio 7793의
`issue7998-raster-nested-dash-parent-left`, `issue7998-raster-raster-phase-alpha-all`,
`issue7998-raster-raster-clipped-none`, `issue7998-raster-alpha-source` 탭을 확인하고
**“시각·편집 구조 통과”**로 판정했다. 첫 셀의 변 교체/지우기, 다른 셀·자식 표 보존,
반투명 선 농도와 공개한 접점 차이를 포함한 판정이며 이전 다른 절편의 판정을 대체하지 않는다.

새 공개 입력 두 개와 source 주석은 `pdf/issue7998/raster-strokes/`에 넣었다.
`generate_sources.py --output <검증 디렉터리>`가 네 파일을 바이트 동일하게 재생성한다.
독립 원본 PDF 연산자에서 만든 입력이며 엔진 출력을 기대값으로 재사용하지 않는다.
전체 raw 조건·직접 판정 뒤에만 정식 `tests/cases/`에 회귀를 등록했다.

새 정식 회귀 **8건**, 기존 집중 회귀 **38건**, 합계 **46건이 통과**했다.
새 파일은 `tests/cases/issue_7998_pdf_raster_border_editing.rs`이며 기존 자동 수집이
`regression_suite_021`에 포함한다. `CARGO_BUILD_JOBS=14 cargo test --locked --profile
release-test --target-dir /home/edward/mygithub/rhwp/target/pr-review --features native-skia
--test regression_suite_021 issue_7998_pdf_raster_border_editing -- --test-threads=4`로
실제 Cargo suite 실행을 확인했다(`formal-raster-tests.log`). 기존 38건은 원래 정식
검사 함수들을 묶은 집중 harness로 실행했다(`existing-guards-final.log`). 전체 suite
실행으로 보고하지 않는다. 수정한 Rust의 rustfmt 검사와 `git diff --check`도 통과했다.
기존 ownership 검사 한 곳은 모든 Picture를 내용 그림으로 가정했으므로 실제 source
2×2 XObject와 새로 인증한 raster stroke를 구별하도록 바로잡았다. 내용 그림·가림 보존
조건은 유지하며 기대 픽셀/허용치를 완화하지 않았다. stage17의 실제 border API 재현은
원본 조각 4→4·undo 없음으로 FAIL, 보정 API는 4→3·undo 있음으로 PASS다.
`oracle-final-baseline.log`·`oracle-final-baseline_after.log`는 같은 원인의 실제 전후 실행이다.
모든 새 assertion이 이전 코드에서 실패한다고 주장하지 않는다.

이번 완료는 **R1 중 단일 축 raster stroke의 소유·교체·출력 절편**이다. fill/여러 축
compound의 역할/closure, 새 행열 상속·삭제/복원, 부분 병합·중첩 이웃을 이어서 처리한다.
확정 문단의 수평 메트릭 R2와 최신 devel 통합·전체 회귀/lint·Worker/package·최종 PR 판정
P5는 남는다. 기존 B19/B21 예외와 B22는 계획의 분류를 유지한다.
`parser_allocation_bounds_proven=false`이며 P4 전체 완료나 비신뢰 입력 배포 준비 완료로
표시하지 않는다. 원격 push·PR 생성/갱신·GitHub 댓글·merge·이슈 종료는 수행하지 않았다.
