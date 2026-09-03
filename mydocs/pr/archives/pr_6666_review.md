# PR #6666 검토 기록

- 원 PR: <https://github.com/edwardkim/rhwp/pull/6666>
- 제목: `ir: separate derived layout state`
- 기여자 head: `7408d6adb33bdab974bf52e5969e72b48f9d81b7`
- 통합 검토 branch: `review/humdrum00001010-green-20260903`
- 통합 적용: provenance-preserving `-x` 9개, 마지막 `82421ba63738b0ebc29bc8823c42ed2cd53ab1de`
- 연결 이슈: `Closes #4771` (병합 뒤 실제 auto-close 여부를 확인한다)

## 판정: 메인터너 보정 됨, 수용 가능

원 PR head의 CI는 green이었다. 최신 `upstream/devel`에는 그 뒤 `#6655`의 재분류 부동 그림 가로 오프셋 보정이 포함되어 있다. 따라서 원 PR이 새로 추가한 `issue_2004_projection_preserves_each_picture_identity_and_final_bounds`는 이전 x 좌표 `82.4`를 고정해 통합 검증에서만 실패했다.

메인터너 보정은 테스트 기대값 네 개만 현행 레이아웃 계약으로 갱신했다. HWP/HWPX 양쪽의 페이지 5~8 실제 x는 각각 `103.453`, `110.280`, `103.053`, `101.480`이며, `#6655`의 검증값 `103.5`, `110.3`, `103.1`, `101.5`와 일치한다. 렌더러, fixture, golden PDF는 변경하지 않았다.

## 검토 범위

- 원본 `Document`에서 renderer cache, table dirty state, local-resize 상태를 분리한다.
- 파생 상태 재생성 및 source document 불변성 경로를 검토했다.
- `#2004` 셀 그림 stack의 HWP/HWPX별 page 4~8 그림 identity와 최종 bounds를 검토했다.

## 로컬 검증

- `cargo fmt --all -- --check`: 성공
- `CARGO_TARGET_DIR=target/pr-review cargo nextest run --test regression_suite_022 -E 'test(issue_4771_derived_layout_state::issue_2004_projection_preserves_each_picture_identity_and_final_bounds)|test(issue_4771_derived_layout_state::every_hwp_lowering_api_preserves_the_live_document)'`: 2 passed
- HWP/HWPX `issue2004_cell_image_stack` page 4~8의 실제 image bounds를 `export-svg`로 재확인했다.

## 시각 증적

- 기준 PDF: `pdf/issue2004_cell_image_stack-2022.pdf`
- 비교 범위: HWP/HWPX 각각 page 4~8
- rasterizer: `rsvg` fallback. macOS Chrome WebFont 경로는 display service 오류로 완료되지 않아 Studio 동등성 증명으로 사용하지 않았다.
- 자동 분석: 두 형식 모두 5 page complete, `flagged_page_count=0`; frame overflow, text-flow collapse, square-wrap overlap 후보 없음.
- pixel/ink 일치율은 PDF 글꼴·안티앨리어싱 차이를 포함하므로 acceptance gate가 아니다. 그림 identity와 render-tree bounds 계약, 사람 검토를 우선했다.
- stable asset: `mydocs/pr/assets/pr_6666_6668_humdrum00001010_20260903/issue2004-hwp-page-004.png`부터 `issue2004-hwp-page-008.png`, `issue2004-hwpx-page-004.png`부터 `issue2004-hwpx-page-008.png`

## 병합 뒤 처리 계획

통합 PR의 최종 head CI와 devel post-merge CI가 모두 성공한 뒤에만 `#6666` 및 Issue `#4771`의 실제 close 상태를 확인하고, `post_merge.md`의 body-file comment/close 절차를 적용한다. 이 검토 시점에는 원 PR/issue에 comment, close, merge를 수행하지 않았다.
