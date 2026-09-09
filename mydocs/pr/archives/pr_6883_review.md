# PR #6883 검토 기록

## 대상

- 원 PR: #6883 `fix(renderer): 편집 세션에서 자란 표의 재조판`
- 기여자 head: `0e862edb0418668bc51b283565b2af3343881922`
- 검토 기준: `upstream/devel` `ad84192839eb7b8534715ab085dd91d69a5c4a38`
- 누적 검토 head: `a6d4bdcf10a834cacfed0ba59e52714c168a4dd4`
- 누적 적용 commit: `00248e68d` (원 PR #6883), 충돌 없음
- GitHub 상태 재확인: head 유지, `CLEAN`, 원 PR CI 전체 성공

## 원 PR 변경 링크

- [원 PR의 고정 변경 commit](https://github.com/edwardkim/rhwp/pull/6883/changes/0e862edb0418668bc51b283565b2af3343881922)
- [셀 분할/병합 뒤 측정 캐시 무효화](https://github.com/edwardkim/rhwp/pull/6883/changes/0e862edb0418668bc51b283565b2af3343881922#diff-4458d5afd7a9f2e3e76618f579d47c9bc422c845e5480efc2c9c9c028b703a61)
- [렌더링 profile의 편집 세션 전달](https://github.com/edwardkim/rhwp/pull/6883/changes/0e862edb0418668bc51b283565b2af3343881922#diff-b2609c4c4f162eb26aaaecc69735e3aa404ac304fa5f6f0f81a29534193cf67a)
- [HWP5 편집 세션 판정](https://github.com/edwardkim/rhwp/pull/6883/changes/0e862edb0418668bc51b283565b2af3343881922#diff-07d5df0bc3883ca464eb05496d8d0b8f83cc72223fda5450323bb5a16489eb8e)
- [자란 표의 실측 높이와 페이지 분할](https://github.com/edwardkim/rhwp/pull/6883/changes/0e862edb0418668bc51b283565b2af3343881922#diff-91be5e3d19af44c06d74858496117beb257f94c7a5bafe40d32508831198fe24)
- [Enter/병합 회귀 시험](https://github.com/edwardkim/rhwp/pull/6883/changes/0e862edb0418668bc51b283565b2af3343881922#diff-8587017bfc7f4c3c8cdc426e742e97d0723a84c01cfb9dc22a40c02286f1f521)

## 변경 검토

- 셀 분할과 병합 뒤 해당 셀 control의 측정 캐시를 무효화한다.
- HWP5 편집 세션에서 저장된 행 높이와 span을 그대로 재사용하지 않고, 자란 표는 실측 높이로 페이지 분할한다.
- synthetic 문서에는 `raw_provenance`와 `raw_stream` 조건을 함께 적용해 새 `Document::default()` 문서가 편집 세션으로 오인되지 않게 한다.
- 새 회귀는 Enter 8회 후 3쪽, Enter 20회 후 4쪽, 병합 후 2쪽 복귀를 확인한다.

## 검증

- Rust lint 묶음: manifest prepare/check, fmt, native/WASM/workspace Clippy, workspace build 성공.
- sampler 4종과 Oracle page count: 68/68 성공.
- security corpus에 새 fixture 지정: 6/6 성공.
- focused: `issue_2164_cell_enter_overlap` 5/5, `issue_2299_edit_vpos_reset_preserve` 8/8, `tac_group_page_bottom_overflow` 5/5 성공.
- 전체 release-test: 9,334/9,334 성공, 46 skipped.
- Native Skia: lib 3,930 성공/13 ignored, placeholder 2/2, direct PDF export 4/4 성공.
- Docker WASM: 이 호스트에 `docker` 실행 파일이 없어 compose wrapper는 실행하지 않았다. 다만 lint 묶음의 `wasm32-unknown-unknown` Clippy는 통과했으며, Docker 부재는 로컬 수용 판단의 차단 사유가 아니다. 통합 PR CI가 동일 범위를 다시 확인한다.
- 한컴 2020 직접 변환과 원본 fixture 스윕: 2쪽 완료, 구조 이상 플래그 0, review PNG 두 장 직접 판독에서 표 경계, 본문 프레임, 페이지 흐름의 잘림을 발견하지 못했다.
- 최종 기준 PDF: [`pdf/synth_cell_enter_table_growth-2020.pdf`](../../../pdf/synth_cell_enter_table_growth-2020.pdf), 263,932 bytes, SHA-256 `c59a49aeb564aa2aae981728dc1572698bd1434aef2bd981a02536c44cb48668`, Hancom 2020 직접 변환 2쪽이다.
- 최종 시각 증적: [page 1 review](../assets/pr_6883_6909_20260909/pr6883-synth-cell-enter-table-growth-p001-review.png), [page 2 review](../assets/pr_6883_6909_20260909/pr6883-synth-cell-enter-table-growth-p002-review.png). 이는 원본 fixture와 Hancom 기준 PDF의 1-2쪽 비교이며, Enter/병합 동작 자체의 페이지 수는 focused 회귀 시험으로 확인했다.

## 자산 정책

- 최종 기준 PDF와 위의 두 최종 review PNG는 장기 재현과 PR 판독에 필요한 증적이므로 포함한다.
- 원시 raster, 중복 compare·overlay·review, contact sheet, export SVG, render-tree·metric JSON, MCP 응답 JSON은 중간 산출물로 제외한다.

## 판정

코드 수준의 차단 결함은 발견하지 못했다. 누적 통합 PR 생성은 가능하며, 최종 GitHub 승인과 병합은 통합 head의 CI가 녹색인 상태에서 진행한다.

## Merge 후 확정 기록

- 원 PR #6883의 변경은 단독 merge가 아니라 통합 PR [#6944](https://github.com/edwardkim/rhwp/pull/6944)로 수용되었고, merge commit은 [`74d0a68b74919761cc30343f7511dbc5d0fe32d3`](https://github.com/edwardkim/rhwp/commit/74d0a68b74919761cc30343f7511dbc5d0fe32d3)이다.
- 통합 head `44b2c1def8dc43ff4bd4dfcce7264dfa725eb8a5`의 필수 CI는 `Build & Test`, `CI Impact Policy`, `Lint (fmt, clippy, WASM check)`, `Native Skia`, `Canvas visual diff`, archive shards, CodeQL, adapters, Proptest까지 성공했다.
- `synth_cell_enter_table_growth`의 Hancom 2020 직접 PDF 기준본과 대표 비교 PNG 두 장은 `devel`에 보존했다. 원시 raster, overlay, SVG, metric JSON은 검토용 중간 산출물이므로 포함하지 않았다.
- 이슈 #6882는 #6944로 해결되어 close하고, 원 contributor PR #6883은 수용 경로를 설명한 뒤 superseded로 close한다.

## Merge 후 contributor PR comment 계획

- 게시 대상: PR #6883과 이슈 #6882.
- 게시 순서: 이 아카이브 기록을 담은 문서 전용 후속 PR이 `devel`에 merge된 뒤, #6883에 통합 merge SHA, CI 결론, Hancom 2020 기준 PDF와 대표 PNG 링크를 남긴다. 이어서 #6882에 같은 해결 근거를 남기고 close한다.
- 코멘트에는 #6883 원 head가 직접 merge된 것이 아니라 #6944에 cherry-pick 수용되었다는 점과, 증적의 비교 범위가 `synth_cell_enter_table_growth` fixture임을 명시한다.
