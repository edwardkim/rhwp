# PR #7337 리뷰 — 셀 안 중첩 TAC 표의 저장 줄 소속

## 최종 판정

**메인터너 보정 후 수용 가능.** 원 head의 폭 초과 보정만으로는 둘째 표가 첫 표와 겹쳤습니다. 최신 devel에서 원 결함을 확인하고 기여자 이력을 보존한 통합 후보를 준비했습니다. Native/fresh WASM 전 6쪽, CDP, 전체 회귀와 Rust lint는 통과했습니다. Native Skia의 추가 integration과 신규 쪽수 원장 검사까지 통과했고, 통합 [#7585](https://github.com/edwardkim/rhwp/pull/7585)의 코드 후보 Full CI도 성공했습니다. 후행 문서 head의 최신 CI와 current-base 충돌 검증은 merge 전 조건입니다.

## 접수와 처리 경로

| 항목 | 확인값 |
| --- | --- |
| 원 PR·기여자 | [#7337](https://github.com/edwardkim/rhwp/pull/7337) / seongeun82 |
| 원 head | `371abaac7a00185c8818a16f69c3ec0296990577` |
| 최신 devel 대조군 | `e0f6b0942775b863e3bef292f133508b14a12e9e` |
| 원 기여 commit | `a77a25868912b3e3d54547980fdced6ff1097375` → author를 보존한 `c6f87c47bf0f431e1f988877e30449bc8b3bce8e` |
| 메인터너 보정 source | `64b3d366c3d6563b2d9ed1eba1eb869d99c0eb7a` |
| 기존 접수 metadata | reviewer postmelee / assignee seongeun82 / milestone v1.0.0 |
| 사용자 승인 | 2026-10-05 보정 후 PR 처리 진행 승인 |

원 PR의 base route는 `maintainer_general`, modifiers는 `intake_and_review`, `local_validation`, `visual_fixture_evidence`, `rework_and_exceptions`입니다. 첫 제출 PR이어서 `first_time_contributor`도 추가로 확인했습니다. 별도 통합 PR의 기록은 `collaborator_self_merge`의 번호 채번·self-review 경로, 원 PR 종료와 안내는 `post_merge`를 적용합니다. 최신 devel에서 원 기여만 cherry-pick했으며 CHANGELOG는 양쪽 내용을 보존했습니다. 원 contributor fork에는 devel 이력을 섞지 않습니다.

## 독립 입력과 최신 devel 선행 확인

- [원 PR의 공개 HWP](../../../samples/nested_tac_table_wrap.hwp): SHA-256 `7e1b32b9f57d3b13b6e32a9af872bbc64bc326bc815eaf6b6d4bfd566031f393`, 114688 bytes. 저장 제품 Hancom 2018 / 10.0.0.14515. 원바이트와 저장 LineSeg를 바꾸지 않았습니다.
- [한컴 기준 PDF](../../../samples/pr7337/nested_tac_table_wrap-2020.pdf): SHA-256 `0e0755d3f15532c5716e93fd0fa8109ead3d97f92f2cd96110449f443812c3e3`, 170038 bytes. 저장 제품에 따라 MCP engine 2020, Hancom 11.0.0.9136, input_preprocess=none으로 생성했습니다. PDF 1.4 / Creator Hwp 2020 0.0.0.0 / Producer Hancom PDF 1.3.0.550 / A4 / 6쪽입니다. 원 PR의 Hancom 2024 주장까지 검증한 것으로 표현하지 않습니다.
- 두 입력은 보정 source commit에 포함됐고, 실제 검증 바이트와 `git show HEAD:<path>`를 대조했습니다. LFS pointer가 아닌 실제 파일입니다.

최신 devel의 6쪽 첫 안내 표는 x97.9/y355.2, 둘째 실적표는 x718.4/y355.2였습니다. 둘째 표가 오른쪽 용지 밖으로 나가고 같은 세로 띠에 겹칩니다. [변경 전 비교](../assets/pr7337/before_native_review_006.png)의 2px 관용 실루엣 일치율은 57.6391%입니다. 선행 대조군 CLI는 source 1871ba72a의 동결 binary였고, e0f6b094의 src/crates/Cargo.toml/Cargo.lock object가 동일함을 [provenance](../assets/pr7337/validation.json)에 남겼습니다. fresh base build라고 표현하지 않습니다.

원 CI [37029757978](https://github.com/edwardkim/rhwp/actions/runs/37029757978)의 독립 실패는 B1의 text-overlap partition 5(새 sample 15건)와 D1의 둘째 표 아래줄 검사(안내 y515.8, 제목 y373.7)였습니다. fail-fast로 다른 검사가 미실행된 점을 수용 근거로 삼지 않았습니다.

## 원인과 공통 결과의 소비 경로

문제 셀의 한 문단은 가시 텍스트 없이 두 개의 8-unit 표 control을 담습니다. 저장 줄은 ts0/vpos0/lh14179와 ts8/vpos14839/lh19921입니다. 한컴 출력도 안내 표와 실적표를 서로 다른 줄에 놓습니다. 빈 텍스트는 줄 점유나 control의 줄 소속이 없다는 뜻이 아닙니다.

| 단계 | 실제 경로와 보정 |
| --- | --- |
| 생산 | `Paragraph::control_utf16_positions` → `float_placement::stored_control_line_indices`가 저장 UTF-16 축에서 줄 소속을 결정합니다. 합성/무효 저장 줄은 기존 수용 조건으로 제외합니다. |
| 측정 | `height_measurer.rs`의 `nested_table_groups`/`nested_group_occupied_height`가 각 그룹의 저장 vpos와 개체·바깥여백 높이를 소비합니다. 기존 측정 규칙과 높이 계산은 변경하지 않습니다. |
| 배치의 가로 | `table_layout.rs`의 `tac_line_widths`도 같은 줄 소속으로 폭을 합산합니다. 줄이 바뀔 때 해당 줄 전체 폭으로 정렬하고, 같은 줄의 표는 같은 cursor를 전진합니다. |
| 배치의 세로 | 각 표의 `table_seg`/`host_seg_lh`를 자기 줄에서 선택합니다. `stored_first_tac_line` 이후 첫 TextLine의 Y로 덮어쓰던 경로도 `TextLineNode.line_index`가 소유 줄과 일치하는 실제 줄 상자로 연결했습니다. |
| 최종 원점 | `table_anchor_y`에 해당 전용 줄의 바깥 위 여백을 반영하고, recursive `layout_table`의 inline 원점으로 전달합니다. 문서 ID·임의 좌표·새 clamp로 화면을 맞추지 않습니다. |

보정 후 두 표는 부모 셀 안에 포함되고, 둘째 표는 x111.7/y553.0으로 첫 안내 표의 하단(y540.5)보다 아래에 놓입니다. 원 기여의 X 누적 지점 진단과 실물 재현 문서·검사를 수용하면서 저장 줄과 실제 Y 앵커를 추가 보정한 것입니다.

새 너비 기반 줄 구성이나 저장 LineSeg 없는 편집 후 재조판은 이번 동작 확장 범위가 아닙니다. 원 PR의 포괄적인 자동 줄바꿈 주장을 검증 완료로 확장하지 않습니다. 페이지 스캐너의 컷·예약 예산·rowspan 유닛 소비 규칙은 변경하지 않았으며 기존 #7518 분할·물리 프레임 23개 검사도 통과했습니다.

## 검증 결과

검증 source는 `64b3d366c`, 초기 base 정책 비교는 `e0f6b094`, push 전 최신 base 비교는 `ea5ef5f6`입니다. 로그는 ignored `output/pr-review/pr7337-20261005/logs/`에 보존합니다. 공용 target `/home/edward/mygithub/rhwp/target/pr-review`을 재사용하고 삭제하지 않았습니다.

| 항목 | 실행과 결과 |
| --- | --- |
| 결함 검출 | 같은 HWP와 강화한 기존 검사 2개를 최신 devel에서 실행: 2 FAIL(본문 오른쪽 이탈, 앞 표와 프레임 겹침). 보정 후 2 PASS. |
| 정상 대조군 | 같은 줄의 중첩 표 #7008, 그림 줄바꿈 #6122, 그림+표 같은 줄 #6754, 앞 공백/줄 소속 #6737·#5601·#7150·#7049, 분할 #7518: 전체 회귀 안에서 42 PASS. |
| 전체 Rust | `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 8 --no-fail-fast`: 10355 PASS / 0 FAIL / 기존 50 skip, 456.298초. |
| 새 sample 보안 | `RHWP_SECURITY_SWEEP_SAMPLES_JSON=["samples/nested_tac_table_wrap.hwp"]`를 전체 회귀에 전달했고 3개 detector 검사를 실제 실행해 통과했습니다. |
| Rust lint | fmt check, native Clippy, WASM32 lib Clippy, workspace build, workspace all-target Clippy 모두 순차 통과했습니다. 파생 suite prepare 뒤 manifest를 고정 base와 비교해 통과했습니다. 파생 파일은 제출하지 않습니다. |
| fresh WASM | 루트의 locked wrapper로 최적화까지 빌드했고 pkg/public JS·WASM 해시가 일치합니다. WASM SHA-256 `03dc07efa2618604327866b5c36319bf24ab7c0ddb85212d8cc1980a41710eac`. |
| Visual Sweep | 같은 HWP/PDF/96dpi/print/검증용 폰트로 Native/fresh WASM 전 6쪽 비교. 양쪽 최저 99.30154%, 6쪽 99.9944%, gate passed, 누락/큰 위치·외곽선 차이 없음. PNG 6쌍이 바이트 동일합니다. |
| Windows CDP | 기존 Chrome 154의 19222에서 새 Studio 7719를 캐시 없이 열어 실제 package hash, portable/canvas 6쪽, Native/WASM geometry, 부모 포함·아래줄·내용·행 구조를 검사: 13/13 PASS. 실제 Studio 6쪽 화면도 직접 확인했습니다. |
| Native Skia | library 4109 PASS / 0 FAIL / 기존 13 ignored. placeholder integration 2 PASS, direct PDF integration 4 PASS입니다. |
| 새 fixture 래칫 | IR 1개 dump와 overflow-cell/off-canvas/text-overlap/body-overflow 각 16개 dump에서 새 sample의 양수 행은 없습니다. 기존 원장/공차를 완화하지 않았습니다. 한컴 6쪽과 Native/WASM 전쪽 증거로 쪽수 원장에 새 6/6 행만 추가했고 쪽수 원장 16개 partition 검사도 통과했습니다. |

대조군 빌드 직후 첫 후보 검사에서 Cargo가 0.15초 만에 대조군 산출물을 재사용했습니다. 공유 cache를 지우지 않고 후보 source의 mtime을 갱신해 fingerprint의 dirty 이유와 실제 재컴파일을 확인한 뒤 2 PASS를 얻었습니다. 이 최초 재사용 실행은 후보 회귀로 세지 않습니다.

## 시각 증적과 남은 차이

- [Native review](../assets/pr7337/native_review_006.png) · [Native standalone overlay](../assets/pr7337/native_overlay_006.png)
- [fresh WASM review](../assets/pr7337/wasm_review_006.png) · [fresh WASM standalone overlay](../assets/pr7337/wasm_overlay_006.png)
- [Native 전체 6쪽](../assets/pr7337/native_all_pages_review.png) · [fresh WASM 전체 6쪽](../assets/pr7337/wasm_all_pages_review.png) · [실제 Studio 6쪽](../assets/pr7337/studio_page6.png)
- [입력·source·PNG hash와 검사 결과](../assets/pr7337/validation.json)

review/standalone overlay를 직접 판독했습니다. 안내 상자 아래에 실적표의 전체 외곽과 내용이 보존되며 표 겹침·용지 이탈이 해소됐습니다. 99.99%는 2px 관용 내용 실루엣 지표이고, 6쪽 엄격 내용 픽셀 지표는 50.99281%로 별도 보존합니다. 글꼴 획·괘선 렌더링과 raster 차이를 배치 정확성과 혼동하지 않습니다. 1px 진단 공차에서 본문 프레임 신호 3건은 유지되지만 최대 이탈은 570.8px → 1.4px로 감소했습니다. 기본 2px body 래칫의 신규 발생은 0입니다. #7518의 85% 예외를 사용하지 않습니다.

## push 전 최신 base 반영

검증 중 devel에 #7584 Dependabot 통합이 추가되어 `ea5ef5f6`을 merge한 head `154a2314`에서 재확인했습니다. `src`, `crates`, `Cargo.toml`, `Cargo.lock`은 검증 source 64b3d366c와 동일하므로 Rust 전체 회귀·lint·Native/WASM build를 반복하지 않았습니다. 바뀐 Studio 의존성은 다른 작업의 공유 node_modules를 건드리지 않고 이 worktree에 `npm ci`로 설치했습니다. 실제 Vite 8.3.2에서 TypeScript와 CDP 13/13을 다시 통과했고, Native/fresh WASM 전 6쪽 review/overlay를 새로 산출해 양쪽 gate passed·최저 99.30154%를 확인했습니다. manifest 정책도 최신 base SHA로 통과했습니다.

## Merge 후 contributor PR comment 계획

한국어 존댓말로 실물 재현·검사와 정확한 X 누적 지점 진단에 감사드리고, 추가 저장 줄/vpos/여백 보정의 이유와 검증 범위를 구분해 설명합니다. 통합 PR·merge SHA·최신 CI·Visual Sweep 정본을 연결하고 대표 Native/fresh WASM review와 overlay 4개를 merge SHA 고정 raw URL의 Markdown 이미지로 표시합니다. UTF-8 without BOM body-file로 게시한 뒤 API에서 본문을 재조회합니다. 원 #7337은 통합 PR merge 성공 뒤 대체 완료로 닫고 contributor fork는 보존합니다. 관련 #7336은 이미 CLOSED이므로 이번 처리로 다시 종료하지 않습니다.

## 통합 PR 코드 후보 CI 완료

통합 [#7585](https://github.com/edwardkim/rhwp/pull/7585)의 정확한 code candidate `2f01f2788ee6dbf71eaa0157c3481d66048f6dd9`에서 Full CI·CodeQL·Render Diff·Adapter·Proptest가 모두 성공했습니다. archive 합계 10,161 PASS / 0 FAIL / 기존 50 skip, 필수 lint·Native Skia·Frontend·Build & Test도 성공했습니다. 원 실패 text-overlap partition 5 및 이 문서의 2개 최종 프레임 검사도 CI에서 통과했습니다. 같은 PR/source repository/branch/event/SHA와 실제 tested merge/tree를 대조했으며 [CI 정본](../assets/pr7337/ci_candidate_2f01f2788.json), [통합 self-review](pr_7585_review.md)에 연결합니다.

이 archive 이동과 오늘할일은 code candidate 뒤의 single-parent 문서-only 기록입니다. 최신 head의 실제 fast-pass·required checks·mergeability를 확인하고, merge 방식으로 원 author 이력을 보존한 뒤 원 #7337을 대체 완료로 닫습니다. 원 head 371abaac는 그대로이며 기여자 fork에는 push하지 않았습니다.
