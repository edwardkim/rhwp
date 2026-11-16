# PR #9057 devel 동기화와 충돌 해결 검증

2026-11-14 KST. 원 PR source의 충돌 해결 요청에 따른 기록이다. 최종 승인·병합 기록은 아니다.

## 커밋과 충돌 해결

- 기존 원격 head: `511dba954a05c36a3d14c462e743e787402a6e28`.
- 검증 기준 devel: `dc3d0ff15e6e7cdb39138c72a5466994dea0e0d4`.
- 병합 커밋: `b565d9c54ae36751958e79834471017b35ebd0da`. 기여자 커밋을 재작성하지 않고 최신 devel을 병합했다.
- 검증·push한 code head: `52fdce92f65581058e49ff2e9985f32c8478d4bb`.

충돌은 `tests/fixtures/oracle_page_count_baseline.tsv`의 끝에 양쪽이 행을 추가하면서 발생했다. devel의 PR #9042 원본 7행과 #9021의 `1 → 1쪽` 행을 모두 보존했다. 후속 append와의 충돌을 줄이도록 #9021 행을 PR #9042 묶음 앞에 놓았으며 기준 쪽수·원본·PDF 바이트·제품 코드는 바꾸지 않았다. 최종 devel 대비 oracle diff는 한 행 추가뿐이다.

Native/WASM은 병합 커밋의 제품 코드에서 각각 새로 빌드했다. 이어진 `52fdce92f6`은 baseline 행 위치만 바꾸며 `src`, `crates`, `vendor`, `Cargo.toml`, `Cargo.lock`이 빌드 source와 동일함을 확인했다. 기준 devel도 별도로 Native와 최적화 WASM을 빌드했다. [후보 빌드 해시](head-build.json)와 [기준 빌드 해시](base-build.json)를 보존했다.

## 전쪽 Native/fresh WASM 비교

독립 한컴 Print PDF, 96 DPI, `--compat 2022`, 인쇄 프로필, PDFium, 색상 경계 보정 전 2px 내용 실루엣 `silhouette_raw_match_percent`다. Native Skia와 호스트 Chrome Canvas2D를 각각 실행했다. WASM은 `scripts/wasm-pack-locked.sh --target web --out-dir <출력>/pkg`로 `wasm-opt`까지 완료했다. Native 명령은 `cargo build --locked --profile release-test --features native-skia --lib --bin rhwp`다. 두 경로에 같은 공개 HCR Dotum/Batang 글꼴 프로그램을 공급했다([URL·해시](webfonts.json)).

| 문서 | 전체 쪽수 | 기준 Native / WASM 최저 | 후보 Native / WASM 최저 |
| --- | ---: | --- | --- |
| issue9021 | 1 | 61.61% / 61.61% | 100.00% / 100.00% |
| metadata-empty-footer | 2 | 100.00% / 100.00% | 100.00% / 100.00% |
| metadata-empty-header | 2 | 100.00% / 100.00% | 100.00% / 100.00% |
| metadata-unchanged | 2 | 100.00% / 100.00% | 100.00% / 100.00% |
| nested-tail-margins-hancom | 2 | 100.00% / 100.00% | 100.00% / 100.00% |
| nested-tail-single-line-hancom | 2 | 100.00% / 100.00% | 100.00% / 100.00% |
| nested-tail-two-paragraphs-hancom | 2 | 100.00% / 100.00% | 100.00% / 100.00% |
| pr9042-hancom-saved | 2 | 100.00% / 100.00% | 100.00% / 100.00% |

8개 문서 15쪽 모두 원본 출력과 독립 PDF의 쪽수가 일치하며 모아 찍기가 아니다([입력 해시·쪽수](page-counts.json)). [전쪽 TSV 원본](tsv), [페이지별 기준·후보 비교](comparison.json), [요약](summary.json)을 보존했다. 후보의 일반 시각 검증 gate도 두 경로의 8개 문서 모두 `passed`다. 기준 실행의 종료 코드 1은 #9021의 기존 61.61% 결함을 재현한 결과이며, 다른 7개 문서의 실패가 아니다.

[대표 PNG](png)의 전쪽 비교와 같은 영역 2배 확대를 직접 판독했다. #9021의 셋째 표가 둘째 줄에 복구되고 낮은 둘째 표의 기준선 정렬, 글상자 내부 포함·순서·비겹침이 유지된다. 기존 7문서는 제목과 본문 표 사이 간격, 중첩 표 안의 두 문단, 뒤 본문과 둘째 쪽 내용을 보존한다. 괘선 두께·래스터 차이는 남으므로 독립 PDF 대비 raw 100%를 엄격한 픽셀 동일성으로 해석하지 않는다. 이 대상들에는 수식이 없다.

기존 7문서 14쪽의 기준·후보 이미지 자체는 각 backend 안에서 픽셀 단위로 동일했다([Native](native-preservation.json), [WASM](wasm-preservation.json)). 새 devel의 표 페이지 분할·저장·재열기 동작과 그림·글꼴·PDF CMap 처리는 아래 집중 회귀로 함께 보호했다.

## 집중 회귀와 편람

최소 글꼴 환경에서 Native Skia를 켜고 72개 검사 모두 통과했다. 실행하지 않은 suite 이웃 2,011개는 통과 수에 포함하지 않았다([실행 결과](regression-results.txt)).

| 모듈 | 통과 |
| --- | ---: |
| oracle_page_count_baseline | 16 |
| issue_9021_textbox_control_only_tac_table_rows | 5 |
| issue_7524_textbox_tac_picture_rows | 2 |
| issue_6465_inline_object_owning_line | 1 |
| coanchored_title_float_pagination | 9 |
| picture_cell_flow_frame | 7 |
| issue_8919_pdf_cmap_range_entries | 10 |
| hft_outline_font_contract | 22 |

`regression_suite_005/011/012/014/022/025`에서 위 모듈을 nextest filter로 선택했다. `sandbox-exec`로 사용자·공유 글꼴, Font Registry와 시각 검증용 글꼴 캐시를 차단하고 `RHWP_FONT_PATH`를 해제했다. 실제 노출된 OS 글꼴 237개의 파일 해시는 [최소 글꼴 환경](minimal-font-environment.json)에 있다. 초기 실행은 sparse checkout의 기존 compile-time fixture 한 건과 runtime HWP/HWPX 두 건 누락으로 실패했다. 원 커밋의 파일을 복원한 뒤 72개를 모두 다시 실행했으며 테스트 구현·기대값은 변경하지 않았다. #9021의 기존 관계형 검사 5개도 유지했다.

편람 Native `--handbook-gate`는 383쪽 전체 통과, 정본 보호 TSV 대비 하락 0쪽이다. 최저는 130쪽 99.96%, 나머지 382쪽의 표시값은 100.00%다([전쪽 TSV](handbook-native.tsv), [gate·공개 글꼴 근거](handbook-gate.json)). 전체 정본 보호 corpus의 신규 검증이나 최종 PR 승인으로 대신하지 않는다.

## 원격 반영과 Full CI

`52fdce92f6`을 원 PR source 브랜치 `johndoekim/rhwp:fix/issue-9021-textbox-control-only-tac-lines`에 일반 push했다. 같은 SHA의 Full CI에서 35개 성공, 실패·대기 0개, 조건부 생략 6개를 확인했다. 테스트 A–E, Native Skia, lint, 프런트엔드가 실제 실행되어 통과했다. GitHub는 `MERGEABLE`, `CLEAN`으로 판정했다.

- [CI / Build & Test](https://github.com/edwardkim/rhwp/actions/runs/43555425361)
- [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/43555425234)
- [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/43555425211)
- [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/43555425240)
- [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/43555425260)

[전체 check 원본](code-ci.json)을 보존했다. 이 기록은 위 code head의 Full CI 성공 뒤에 추가하는 trailing 증적이며, 기록 커밋의 최신 head CI는 별도로 확인한다. GitHub review·승인·병합은 수행하지 않았다.
