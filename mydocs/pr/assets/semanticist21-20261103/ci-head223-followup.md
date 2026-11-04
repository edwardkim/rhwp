# 제출 head 223eb2361c의 실제 CI 보정

## Native CLI 계약의 누락 확인

Native CI의 `cli_exit_codes_native::png_json_metadata_and_render_preserve_feature_and_provenance_contracts`가 `untrustedContent=true`, 이전 기대값 false로 실패했다. 원 #4132 파일 게이트(`1ed97ba991`)와 #7903의 검사 추가(`116b2b6ae0`, 2026-10-14T22:12:10+09:00), #3787 실제 값 표지 계약과 #8565의 글꼴 보고를 대조했다. PNG metadata-only와 실제 render의 글꼴 보고는 문서 파생 값이므로 false 고정이 잘못됐다. 기존 종료 코드·feature·schema·쪽수·파일 생성 검사는 유지하고, 실제 실린 네 종류 글꼴 보고의 출처 경로와 true 표지를 검사한다. MAP을 검사에서 재인용하지 않으며 경로 순서를 계약으로 고정하지 않는다.

이 Native CLI target을 앞의 로컬 Native 3종 목록에서 빠뜨렸다. 원 검사의 FAIL을 확인했고, 이후 `.github/workflows/ci.yml`의 release-test Native 실행 목록 전체를 로컬에서 검증한다. Native 기존 라이브러리 4,040건·기본 출처 11건의 통과가 이 실행을 대신하지 않는다.

## CodeQL의 세 좌표 진단

새 Rust 분석 `2093009475`(tested merge b0165b43aa4f746bc4dc8fa0ea4d14d6ad8d4598)는 총 129개 결과이며 PR scope open alert 15개의 ID는 H4 분석과 같다. check는 이 중 이번 diff의 세 sink를 high로 판정했다. 새 alert ID가 없다는 이유로 성공이나 false positive로 분류하지 않는다.

| alert | 제출 head sink | code flow / 단계 합 | 실제 출력 |
| --- | --- | ---: | --- |
| #493 | paragraph_layout.rs:3672 | 4 / 274 | 인라인 표 문단 원점·너비·간격 |
| #494 | paragraph_layout.rs:6991 | 4 / 334 | 본문 overflow 좌표 |
| #496 | paragraph_layout.rs:11739 | 4 / 310 | raw fallback의 overflow 좌표 |

모두 NativeSession → from_bytes_with_password에서 열린 문서의 수치 배치가 진단 stderr로 흐른다. 기존 #8517 원 보정 `57ce496ef7`과 이번 앞선 보정 `3af2224e4e`의 정책을 적용한다. 인라인 표의 주 진단과 같은 블록의 저장 줄·표 진단, 본문/raw fallback의 overflow 출력만 제거한다. 계산·실제 y/높이·분할·배치·cell overflow 집계·paint·저장은 유지한다. 로그 전용 순수 조건·수치 계산만 함께 제거한다. 보안 query·언어·경로·분류·workflow·검증 기준은 바꾸지 않는다.

기존 실제 암호 HWPX CLI 회귀에 RHWP_LAYOUT_DEBUG와 제거한 채널을 추가한다. 정상 SVG 산출은 유지하며 보정 전/후를 실제 binary로 확인한다. 시각 증거는 조판 계산 동일성을 확인해 이미 검증한 Native/fresh WASM producer와 구분해 재사용한다.

## 최종 로컬 보정 검증

보정 code head `5e374d01f641745d20bc9424ce233021a327ed16`에서 아래 검사를 완료했다. 검증 source를 바꾸지 않은 문서-only 제출 commit은 별도로 확인한다. CI의 Native release-test 22개 명령을 target·profile·feature·인자 그대로 실행했고 Cargo의 lock 강제만 추가했다. private/system 글꼴을 가리고 저장소의 허용 글꼴 5개만 노출한 최소 환경을 유지했다.

- stock 전체: **Summary [ 604.964s] 12475 tests run: 12475 passed (1 slow), 45 skipped**.
- Native CI 목록 합계: **192 PASS**, 실패 0건. 아래에서 library filter와 각 integration target을 구분한다.
- Native 라이브러리 전체 재확인: **4,040 PASS / 12 ignored**, 실패 0건. CI의 skia filter 58건을 전체 4,040건으로 과장하지 않고 별도 실행했다.
- 추가 Native 실제 출처 계약: 11 PASS.
- 기존 실제 암호 HWPX CLI 회귀: 전체 실행 안에서 PASS했고 별도의 실제 export에서도 정상 SVG 생성·선택한 7개 진단 prefix 부재를 확인했다.

| Native CI target | PASS | ignored | 컴파일 포함 초 |
| --- | ---: | ---: | ---: |
| skia-lib | 58 | 0 | 334.92 |
| cli_exit_codes_native | 2 | 0 | 304.75 |
| issue_1144_native | 1 | 0 | 2.48 |
| issue_2225_missing_picture_placeholder | 2 | 0 | 15.41 |
| render_p37_direct_pdf_export | 22 | 0 | 19.45 |
| issue_7950_pdf_paint | 4 | 0 | 18.99 |
| issue_2083_hide_fill_page_background | 1 | 0 | 0.77 |
| issue_2292_chart_png_clip | 1 | 0 | 15.43 |
| issue_2293_chart_png_text | 2 | 0 | 1.66 |
| issue_7820_sixel_contract | 7 | 0 | 14.43 |
| issue_8150_kitty_contract | 8 | 0 | 14.37 |
| issue_4961_font_decision_trace | 12 | 0 | 18.28 |
| issue_7902_original_frame_and_content | 7 | 0 | 20.52 |
| issue_8160_hwp2022_paragraph_relations | 3 | 0 | 16.59 |
| native_fractional_glyph_origin | 4 | 0 | 11.94 |
| skia_native_glyph_replay | 10 | 0 | 18.10 |
| issue_8153_opaque_mixed_axis_image_sampling | 2 | 0 | 22.39 |
| issue_8153_print_glyph_geometry | 5 | 0 | 15.94 |
| issue_8153_symbol_fallback | 1 | 0 | 14.80 |
| issue_536_native_region_cache | 21 | 0 | 0.91 |
| skia_region_plan | 10 | 0 | 14.46 |
| skia_host_surface | 9 | 0 | 15.07 |

| 나머지 게이트 | 종료 | 컴파일 포함 초 |
| --- | ---: | ---: |
| fmt | 0 | 23.79 |
| manifest-prepare | 0 | 1.48 |
| manifest-check | 0 | 0.71 |
| unit-tiers | 0 | 1.58 |
| clippy-root | 0 | 50.04 |
| clippy-workspace | 0 | 70.78 |
| clippy-wasm | 0 | 44.44 |
| workspace-build | 0 | 46.29 |
| full-nextest | 0 | 1400.84 |
| native-provenance | 0 | 7.84 |

### 실제 반례와 검증 한계

출처 보정 전 ef6 Native binary는 table-vpos-01.hwp의 metadata-only 응답에 nativeFontSupply/fontEnvironment를 실제 싣고도 untrustedContent=false·빈 untrustedFields를 반환했다. 출처 선언을 보정한 실제 binary는 같은 요청에 true와 두 경로를 반환하며, 수정한 Native CLI 검사가 metadata-only/실제 PNG 양쪽에서 PASS했다. 기대값을 출력 품질·좌표에 맞춘 것이 아니다.

기존 실제 암호 fixture는 이전 binary에서도 이번 새 세 진단 그룹을 직접 실행하지 않았다. 따라서 해당 fixture를 새 세 경로의 runtime 보정 전 FAIL 증거로 보고하지 않는다. 원래 H4 세 채널의 과거 음성/양성 대조는 유지하고, 이번 새 세 sink의 독립 before/after 판정은 CodeQL 결과를 확인한다. 새 head CodeQL 성공 전에는 해당 보정을 최종 완료로 판정하지 않는다. 경고 분류·query·workflow·판정 기준은 바꾸지 않았다.

### 조판 증거의 재사용 근거

223 → 5e의 제품 diff는 paragraph_layout.rs의 진단 출력과 로그 전용 순수 계산 제거다. 실제 text_y/y/line_height·원점·높이·줄 소유·분할·본문 점유·cell overflow 집계·paint·저장 계산은 같다. blank_spacer_line과 equation_only_endnote_tail_line은 후속 내용 점유에도 쓰이므로 유지했다. 제거한 미주 log tolerance getter는 layout.rs:4454의 세 Cell.get 비교와 순수 허용치 조건뿐이며 상태를 바꾸지 않는다. raw fallback은 원래 y_clamped=y를 그대로 소비한다.

따라서 [Native/fresh WASM 32개 입력·72쪽, 실제 14개 역할·175쪽과 필수 Native 편람 383쪽](rebase-validation.md)의 renderer producer·SHA·TSV·직접 판독을 그대로 연결한다. 이 보정에서 새 픽셀을 측정했다고 바꾸어 적지 않는다. 새 레이아웃·baseline·px assertion을 추가하지 않았다. 실제 source와 계산 경로를 대조한 로그 보안 후속 보정으로 한정한다.
