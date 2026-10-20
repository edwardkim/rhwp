# #7419 Stage 1 — 체리픽과 메인터너 조판 보정

Issue: #7419, 원 PR: #7433. 기준 devel `375aff29498166260a22796d54e67f508c84aada`.
체리픽 `617e41864`(원52a51c0a97), `89b9014bf`(원4905a31d07), 입력 보존 `6c920596a`.
이 문서와 함께 커밋하는 Rust 코드가 메인터너 보정이다. 검사 추가·전체 회귀·lint는 다음 단계다.

## 위반 규칙과 실제 소비 경로

1. 저장 줄 없는 셀은 가용 폭으로 다시 줄을 구성한다. 표·행 선언 높이 합의0.5px 일치를 생성 출처로 추정하던 조건을 제거했다. `HeightMeasurer::measure_table_impl`의 `row_heights`/`total_height`를 현재 내용의 하한으로 보존한다. HWP5와 실제 저장 줄을 쓰는 분기는 기존 계약을 유지한다.
2. `composer::recompose_cell_lines_with_nested_tables`는 중첩 표의 물리 본체 높이를 받아 기존 `recompose_stored_lines_in_frame`을 호출한다. 측정은 `measure_table_impl`의 본체(캡션 제외), 배치는 `nested_table_body_height`의 행합·셀 간격을 공급한다. 바깥여백은 `inline_control_occupied_size_hwp`에서 한 번만 더한다. `FrameGeometry`가 실측 개체 높이·줄 기록을 함께 발행하고, `paragraph_with_frame_geometry`를 통해 측정·셀 높이·실제 cell paint가 같은 UTF-16 줄 소속과 원점을 소비한다.
3. `inline_frame_input::project`의 글자 없는 문단은 모든 제어문이 가시 문자 축의 빈 경계에 속한다. 원시 UTF-16의0·8 슬롯 순서는 유지한다. 순차 문자 인덱스 폴백 때문에 두 번째 표가 입력에서 제외되던 결함을 수정했다. 같은 줄/폭 부족/명시적 개행은 공통 프레임의 실제 줄 나눔으로 결정한다.
4. `layout_horizontal_cell_paragraphs`는 파생 프레임을 적용한 문단과 `control_slot_line_index`의 소유 줄을 사용한다. 높이의 우연한 일치나 같은 TAC 속성으로 소속을 추정하지 않는다. 프레임 이후 별도 앵커 선택까지 확인했다. 아래 실제 반례가 같은 줄 합산·개행 유실·후행 텍스트 이동을 검출한다.
5. 페이지 예산: `tac_fit::prepare`의 `measured_tac_floor` → `height_for_fit` → pre-flush/이월 → `flow_table::place`의 `format_table`/`effective_height`와 `typeset_tac_table` → `tac_reconcile::effective_cap` → 실제 `resolve_row_heights_trusting_declared`·paint를 대조했다. 최신 devel의 기존 실측 하한이 체리픽한 행 성장분을 이미 소비하므로 pagination 코드는 중복 수정하지 않았다. 용지28000HU는 원 devel2쪽, 체리픽부터3쪽, 독립 Print3쪽이다. 1쪽 제목 / 2쪽 첫 표 / 3쪽 뒤 표를 실제 출력에서 확인했다.

이 Stage1 시점에는 분할 continuation의 컷·rowspan 소유·예약 알고리즘을 변경하지 않았다. 후속 분할 경로의 추가 보정과 실제 반례는 [Stage3](task_m100_7419_stage3.md)에 기록한다. 이 작업의 TAC 전체 표는 한 쪽에 들어가지 않으면 통째로 이월한다. 작은 용지의 제목 뒤 예산은 선언64px만 fit하던 경계이며 내용70.45px가 들어가지 않아 다음 쪽으로 간다. 각각 마지막 행과 뒤 표 보존을 확인했다. 배경·비TAC·저장 줄·HWP5는 새 중첩 프레임 분기의 비적용 경로다. 정상 대조군과 전체 회귀는 다음 단계에서 별도로 확인한다.

## 독립 Print와 입력 생성

원본2개는 원 PR의 수동 XML HWPX다. 실제 한컴 저장본으로 오인하지 않는다. 선언4838/용지28000은 과거 reviewer의 정확한 반례 바이트를 재사용했다. 나머지는 원 중첩 입력의 자식 표를 복제해 만든 유효 XML 반례다. 같은 줄·개행·앞뒤 본문은 자식 폭15500HU, 폭 부족은24000HU 두 개, 여백은 상300/하600/좌200/우400HU다. `hp:t/hp:lineBreak`가 유효한 명시적 개행이며 초기 잘못된 run 직계 개행 출력은 제외·별도 보존했다.

모든9개 입력을 한컴2024 `13.0.0.3901`, `PrintMethod=0`, `frame_print_to_pdf_ex_one_up`으로 새로 출력했다. 비동기 start/status/download 영수증은 아래 ignored 경로의 `print-jobs.json`이다. 비밀 endpoint/token은 기록하지 않았다. 기준은 `pdf/issue7419/*-print-2024.pdf`다. 원 PR의 이전 PDF2개는 `pdf/tac_*-2024.pdf`로 별도 보존한다.

## 전체 페이지 비교와 직접 판독

DPI96·compat2024·Print profile·실제 HBATANG/HDOTUM을 동일하게 공급했다. Native는 Skia PNG 직접 내보내기, WASM은 이 소스로 새 빌드한 CanvasKit PNG다. SVG 중간 파일을 기본으로 만들지 않았다.

| 입력 | devel Native | devel fresh WASM | 보정 Native | 보정 fresh WASM |
| --- | ---: | ---: | ---: | ---: |
| `common-height-4838` | 88.24520% | 87.71732% | 100.00000% | 100.00000% |
| `nested-explicit-break` | 47.86180% | 47.04528% | 100.00000% | 100.00000% |
| `nested-outer-margins` | 54.89478% | 55.00565% | 100.00000% | 100.00000% |
| `nested-same-line` | 52.18348% | 52.09133% | 100.00000% | 100.00000% |
| `nested-surrounding-text` | 44.02413% | 45.53546% | 100.00000% | 100.00000% |
| `nested-width-wrap` | 48.91545% | 47.96036% | 100.00000% | 100.00000% |
| `page-height-28000` | 2/3쪽 불일치 | 2/3쪽 불일치 | 100.00000% | 100.00000% |
| `tac_nested_two_line_cell_no_lineseg` | 72.80910% | 71.48351% | 100.00000% | 100.00000% |
| `tac_two_line_cell_no_lineseg` | 88.24520% | 87.71732% | 100.00000% | 100.00000% |

보정9문서·11쪽의 모든 행은 양쪽100%다. 쪽수는 독립 Print와 전부 일치하며 미달·누락0이다. `direct-crops/`에서 같은 영역의 Native/Print/WASM을 원본 해상도로 직접 대조했다. 두 줄의 기준선·들여쓰기·각 좁은 표의 마지막 줄, 명시적 개행 앞뒤, 같은 줄 최대 높이와 앞뒤 본문, 여백과 작은 용지3쪽의 내용·누락·중복을 확인했다. 이 입력에는 수식이 없다. 실제 glyph 차이는2px 관용 내에 있으며 점수와 별도로 줄 순서·표 제목 간격·내용 포함을 확인했다.

**devel 대비 페이지 gate는 별도다.** 작은 용지는 devel2쪽/Print3쪽으로 기준 TSV의3쪽이 없으므로 전체 일대일 무회귀를 입증하지 못했다. 수정 후 Print 일치·100%를 이 gate의 통과로 바꾸지 않는다. 사용자가 선택한 현행 엄격 기준에 따라 이 차이는 PR 생성 전 보류 조건으로 남긴다. 다른8문서는 페이지별0.01%p 하락 없이 전부 개선됐다.

## 재현 명령과 증적

ignored 기준 경로: `/home/tsjang/rhwp/output/pr-review/pr7433-current-20261016/`.

```bash
CARGO_BUILD_JOBS=8 cargo build --locked --features native-skia \
  --target-dir /home/tsjang/rhwp/target/pr-review --bin rhwp
CARGO_BUILD_JOBS=8 CARGO_PROFILE_DEV_CODEGEN_UNITS=16 \
  CARGO_TARGET_DIR=/home/tsjang/rhwp/target/pr-review \
  scripts/wasm-pack-locked.sh --target web --out-dir <증적>/candidate-wasm --dev --no-opt
/home/tsjang/rhwp/venv/bin/python scripts/visual_sweep.py --silhouette-only \
  --file-target <stem> <입력> pdf/issue7419/<stem>-print-2024.pdf \
  --rhwp-bin <증적>/candidate-native-rhwp --rasterizer png --compat 2024 --dpi 96 \
  --embed-fonts full --font-path <증적>/fonts --page-workers 2 --out <증적>/candidate-native-sweep
# WASM: 같은 명령에 --wasm-pkg <증적>/candidate-wasm, 출력은 candidate-wasm-sweep.
# 대표 PNG: --silhouette-only를 빼고 별도의 candidate-{native,wasm}-review에 저장.
```

전체 실제 argv·로그는 `run_sweeps.py`, `candidate-{native,wasm}-result.json`, 각 `run_manifest.json`/`silhouette_manifest.json`과 `silhouette.tsv`에 있다. 빌드 로그와 `candidate-v2-source-fingerprint.json`은 커밋 전 source SHA-256 및 Native 실행 파일을 고정한다. `maintainer-v2-result.json`은 셀/본문/표 bbox 진단이다. 과거 `verify.py`의 buildCommand/scope 텍스트는 초기 base용 고정 설명이므로 후속 phase의 실제 argv·해시·빌드 로그를 우선한다. TSV·로그·중간 JSON은 Git에 추가하지 않는다.

| 입력 파일 | 입력 SHA-256 | 독립 Print PDF SHA-256 |
| --- | --- | --- |
| `common-height-4838.hwpx` | `bbfb52b87f03ea5e2a73abbcf3f2e17fbe76a7566d92f62c357d67d4479ccf42` | `13e52d61463c4725cb52e4e55d7e51651bf88e97136152d4ecf163f39cfc9099` |
| `nested-explicit-break.hwpx` | `605cadab0527822e11afd3370e1546040e73f0f1798ce646e6ce41cb68625a09` | `723e2a09c5b883edbd3fb8a055eb3aa187da2c01eb3131fd1d72888de1d26088` |
| `nested-outer-margins.hwpx` | `79c872d0a949b13d5d2191d3c39cff4fecbcee0aa3802648a810d9435324dcc7` | `c01bdbcfe939328d8ede43958689a61c5382e7cbe85b1e8f2f9cf2bfcd6f9703` |
| `nested-same-line.hwpx` | `0340236f0d79a68a343bd771266544781f9313db63ded04168f62b8fb2770d7c` | `86facc427943289a3854da58614813ca1d391210e8bb6092c6f7e0fa5a9db91c` |
| `nested-surrounding-text.hwpx` | `052a2460132202b24dc392bcd74cb2e3957d229026eacc88767b59c55cc8fe1c` | `efdb4dea26c76eb6ecc2f0245470e78a67a84a0df0397a38d35a55e8be213e96` |
| `nested-width-wrap.hwpx` | `8f102ecbd644d720f3766c794fa6b3ba55d7f93e8441f07d0004f519ec0d0f3c` | `8e1164da675863c542c2d9873f104b6485e294bcbf6bcd9087d66c9fe5afb1d1` |
| `page-height-28000.hwpx` | `8d3a298e85cf65d64e06da76c3911b927b6d1d1b056714f1fd1cf3c27be37d71` | `73149057ed3a89dfaf8a1ebc4ea620efc8bab3e034618df318ab2c46d3a2b5af` |
| `tac_nested_two_line_cell_no_lineseg.hwpx` | `59f8fa47e3766124d7e267e4b95e97f9a7050ca8b963261c6bd24c46f517dd76` | `80bce03f48471797f672f904eb1c06931e59243f7d3b6143373efe6d96ced7af` |
| `tac_two_line_cell_no_lineseg.hwpx` | `08a6740ac106b24fbe8dd6d632137ba5748ab8fb2a95187882b6be734f5c36c2` | `8521966a066ecff427158a1d092379c8a9b06b546c8cf6b44d97358d800c2bd7` |

## 다음 단계

90% 시각 선행 조건과 직접 판독을 충족한 범위에 한해 원본2개 회귀를 관계 검사로 보완한다. 양쪽100% 완료 페이지의 독립 Print glyph 기준선을±2px로 보호하며 관계 검사도 유지한다. 수정 전 FAIL/후 PASS, 정상 대조군, 필수 전체 회귀·Native Skia·lint와 PR 전 head 재확인은 아직 미완료다.
