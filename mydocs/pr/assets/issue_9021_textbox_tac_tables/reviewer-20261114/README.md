# PR #9057 독립 전쪽 검증과 쪽수 oracle 보완

2026-11-14 KST, macOS arm64. 대상은 `samples/issue9021/textbox_tac_tables_two_lines.hwp`의 전체 1쪽이다.

## 입력과 빌드

oracle 등록 후보 commit은 `75da9680e31b3179efb3a044f934dce4227f9182`다. 원 PR head 이후 제품 코드는 동일하며 PDF 이름·설명과 baseline 한 행만 바뀌었다.

- 기준 devel: `d7b927d073489b8c5ea95655224b04096c939534`.
- 후보 제품 코드: 원 PR head `079ec7bc893fd2e68bdb9d28c96c824149bdddc6`.
- HWP: 17,920바이트, SHA-256 `e0cd85dad3edbd8d00a4c37c2d3c78bc3912524f9527baac1cab6d23943df959`.
- 독립 Print PDF: 1,198바이트, SHA-256 `93b58ab9db87459cb699b1c3da773db0b013fcb4c6122c979881d5a21b07a018`.
- 실제 Print 제품: 한/글 2022 `12.0.0.535`, Microsoft Print to PDF. 파일명 보완 전후 바이트가 동일하다.
- Native는 각각의 source에서 `cargo build --locked --profile release-test --features native-skia --lib --bin rhwp`로 빌드했다.
- WASM은 각각 `scripts/wasm-pack-locked.sh --target web --out-dir <검증 출력>/pkg`로 새로 빌드했으며 `wasm-opt`까지 완료했다. 호스트 Chrome의 Canvas2D 직접 PNG를 사용했다.
- 입력·바이너리·WASM·측정 스크립트 해시와 결과는 [provenance.json](provenance.json)에 있다. 기준과 후보의 sweep 스크립트 차이는 비교 이미지의 라벨 글꼴 검사이며, 실루엣 계산 규칙은 동일하다.

## 전쪽 결과

96dpi, `--compat 2022`, 인쇄 프로필, PDFium. 판정값은 색상 경계 보정 전 2px 내용 실루엣 `silhouette_raw_match_percent`다.

| 경로 | 기준 전체 1쪽 | 후보 전체 1쪽 | 누락·쪽수 불일치 |
| --- | --- | --- | --- |
| Native Skia | 61.61% | 100.00% | 없음: 양쪽 출력 1쪽 = PDF 1쪽 |
| fresh WASM Canvas2D | 61.61% | 100.00% | 없음: 양쪽 출력 1쪽 = PDF 1쪽 |

기준 정밀값은 양쪽 61.61307%, 후보 정밀값은 양쪽 100.00000%다. 2%p 초과 하락 페이지는 없다. `base-native.tsv`, `base-wasm.tsv`, `head-native.tsv`, `head-wasm.tsv`는 전쪽 TSV 원본이다. 별도 일반 비교 실행의 후보 `pr_review_gate`도 두 경로 모두 `passed`다.

[같은 영역 2배 확대](table-detail-2x.png)와 네 경로의 `*-review-001.png`, 후보 `*-overlay-001.png`를 직접 판독했다. 기준에서는 셋째 표가 첫 줄 끝에서 글상자 밖으로 잘린다. 후보에서는 첫 두 표가 첫 줄, 셋째 표가 둘째 줄에 있고 모든 셀과 괘선이 글상자 안에 남는다. 낮은 둘째 표는 첫째 표의 기준선에 맞춰 내려온다. 중복·누락·겹침이나 불필요한 추가 페이지는 없다. 이 원본에는 글자·수식이 없다. 한컴 PDF의 2px 괘선과 rhwp의 1px 괘선 차이는 남으므로 raw 100%를 엄격한 픽셀 동일성으로 해석하지 않는다.

## oracle 보완

기존 `textbox_tac_tables_two_lines-2022.pdf`는 원본 형식과 canonical 엔진 표기가 없어 공식 생성기에서 제외됐다. 동일 PDF를 `pdf/issue9021/textbox_tac_tables_two_lines-hwp-2020.pdf`로 이동했다. `2020`은 `tools/oracle_pdf_selection.py`가 한/글 2022에도 지정하는 HOffice120 호환 엔진 구분이고, 실제 인쇄 제품을 바꿔 적은 것이 아니다.

공식 `tools/oracle_page_count/regenerate.py`의 `main()`을 사용하되 `sample_paths()`만 이번 원본 한 건으로 제한하고 별도 TSV에 출력했다. PDF 후보·canonical 선택·원본 정보·PDF 쪽수 측정 로직은 변경하지 않았다. 두 번 산출한 TSV가 바이트 단위로 동일했다. 모아 찍기 아님(`printMethodImpliesNup=false`)을 확인했다. 생성된 다음 행만 기존 원장에 추가했고 다른 기준값은 갱신하지 않았다.

```tsv
samples/issue9021/textbox_tac_tables_two_lines.hwp	1	1
```

기존 `issue_9021_textbox_control_only_tac_table_rows`의 표 소속·순서·포함·비겹침 검사 5개는 유지했다. 새 절대 좌표 핀이나 별도 회귀 파일은 추가하지 않았다.

## 필수 편람 Native gate

`--handbook-gate --page-workers 3` 실행 결과 383쪽 전체 `passed`, 정본 보호 TSV 대비 하락 0쪽이다. 최저 raw는 130쪽 99.96%이며 나머지 쪽의 공식 두 자리 표시값은 100.00%다. [handbook-native.tsv](handbook-native.tsv)에 전쪽 결과를 보존했다. 공개 웹폰트 7종의 공급 URL·해시와 정본 baseline 해시는 `provenance.json`에 있다.

canonical 이름과 oracle 행을 커밋한 `75da9680e3`에서도 실제 추적 경로로 두 경로의 전쪽 TSV를 다시 산출했다. 모두 100.00%이며 앞선 후보 TSV와 바이트 단위로 동일하다. 제품 코드·의존성·측정 스크립트의 변경이 없어 위에서 새로 빌드한 동일 바이너리와 최적화 WASM을 사용했다. 입력 해시와 최종 확인 SHA는 `provenance.json`에 있다.

## oracle 등록 후 회귀 검사

최소 글꼴 환경에서 Native Skia feature를 켜고 **25개 통과 / 실패 0개**를 확인했다. oracle 16개 partition, 이번 원본의 관계 회귀 5개, `issue_7524` 2개와 `issue_6465` 1개, 명시적 새 원본 보안 검사 1개다. 선택하지 않은 1,656개 suite 이웃 검사는 실행 대상으로 세지 않았다. [실행 결과](regression-results.txt)를 보존했다.

`RHWP_SECURITY_SWEEP_SAMPLES_JSON`에는 이번 HWP 한 건을 명시했다. `sandbox-exec`로 사용자·공유 글꼴과 개인 Font Registry, 시각 검증용 글꼴 캐시를 프로세스에 한정해 차단하고 `RHWP_FONT_PATH`를 해제했다. 실제 노출은 OS 글꼴 237개이며 일반 사용자 프로세스의 추가 글꼴 223개는 유지됐다. OS 글꼴의 실제 파일 해시는 [minimal-font-environment.json](minimal-font-environment.json)에 있다. 로컬 추가 글꼴의 존재에 의존한 통과가 아니다.

```sh
# 위 최소 글꼴 격리 환경에서 실행. target은 로컬 저장 정책의 공유 target을 사용한다.
cargo nextest run --locked --cargo-profile release-test --features native-skia \
  --test regression_suite_003 --test regression_suite_011 --test regression_suite_014 \
  --test regression_suite_002 --test regression_suite_015 \
  -E 'test(/(^|::)oracle_page_count_baseline::/) or test(/(^|::)issue_9021_textbox_control_only_tac_table_rows::/) or test(/(^|::)issue_7524_textbox_tac_picture_rows::/) or test(/(^|::)issue_6465_inline_object_owning_line::/) or test(/security_corpus_regression::new_sample_documents_are_clean_across_all_three_detectors/)' \
  --test-threads 2 --no-fail-fast
```

첫 빌드에서 sparse checkout의 기존 include fixture 8개가 누락되어 실패했으나, 원 커밋의 동일 파일을 복원한 뒤 위 검사를 통과했다. 테스트 구현·기존 기대값은 바꾸지 않았다.

## 범위

이 기록은 요청된 원본의 oracle 등록 및 독립 Native/fresh WASM 전쪽 검증 증적이다. 원격 push·GitHub review·merge는 수행하지 않았다. 전체 18문서 정본 보호 corpus를 새로 검증한 결과나 최종 PR 승인으로 대체하지 않는다.
