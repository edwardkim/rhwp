# #7333 Stage 12 — PR 직접 증적 준비

## 목적

#7333의 꼬리말·도형 저장 좌표와 선택 페이지 소유권 보정은 실제 조판을 바꾸므로, 최종 head에서
Native, fresh WASM, RHWP PDF export를 한컴 2020 PDF와 비교해 PR 본문에서 직접 판독할 수 있게
준비한다. 이 문서는 PR 번호가 생기기 전의 증적 준비 기록이다. PR 번호 기반 self-review와
오늘할일은 원격 push·PR 생성 승인 뒤 실제 번호를 받은 뒤에만 작성한다.

## 기준과 보존 입력

| 역할 | 경로 | SHA-256 | 비고 |
| --- | --- | --- | --- |
| 원본 HWP | `samples/issue7333/aaaaaa.hwp` | `ab88150db2eb4652c945e6773ed40f61806782b6ca85694eb5f8c8ec390f96c5` | 50쪽, `lastSavedWith.product=hancom-office-2018` |
| 한컴 MCP 기준 PDF | `pdf/issue7333/aaaaaa-2020.pdf` | `fbc0ff34b909e8824e775872ff47ac7d6c904d2cb74504fbe9e7a8411b2c597b` | 50쪽, `Creator: Hwp 2020 0.0.0.0`, PDF 1.4 |
| 최종 RHWP PDF export | `pdf/issue7333/aaaaaa-rhwp-stage11.pdf` | `9278b2d9a65e291d42b96868200872fcddb3ea1f97461cb551dca5ddf088106b` | 50쪽, `Producer: rhwp`, PDF 1.7, 11.9MB |

기준 HWP와 MCP PDF는 이미 검토 branch에 추적되어 있다. 새 RHWP PDF export는 50MiB 미만이며,
현재 최종 head의 직접 PDF 대조 입력이므로 함께 보존한다. `provenance.sha256`과
`assets.sha256`은 같은 사실을 기계적으로 재확인할 수 있는 부가 기록이다.

## 분석과 보정

`upstream/devel`의 `d5cf695dfe2a8df11dc0a632224927bdb34818ae` 위에 rebase한 뒤, 전체 회귀에서
나온 14건을 원인별로 재검토했다. #7333의 빈 선행 문단/TAC 보정이 장식 도형만인 표라는 전제를
코드로 강제하지 않아 일반 TAC과 footer에까지 적용된 것이 원인이었다.

- 저장 대역 표 보정은 해당 control 앞에 InFront 장식 도형만 있을 때로 한정했다.
- 고정 줄간격 공제는 `TopAndBottom` wrap을 요구하도록 한정했다.
- footer leading은 Picture와 페이지/총쪽 AutoNumber가 함께 있을 때만 적용했다.
- `2025 행정업무운영 편람(최종).hwpx` 134쪽의 Table 오른쪽 확장은 한컴 PDF에서도 보이는
  의도된 범위다. Native visual sweep 134쪽과 fresh WASM 원본 SVG의 직접 rsvg/PDF overlay를
  대조해 clipping이 없음을 확인하고 off-canvas baseline을 19에서 20으로 바로잡았다.

## 실행한 검증

- fmt, native/wasm/workspace Clippy(`-D warnings`), workspace build, Rust test manifest: 통과.
- 전체 nextest: `--test-threads 8 --no-fail-fast`, **10,165 passed, 0 failed, 50 skipped**, 600.972초.
- Native Skia: lib **3,930 passed, 0 failed, 13 ignored**; missing-picture placeholder 2/2 통과;
  direct PDF export 4/4 통과.
- 최종 Native 전체 50쪽 visual sweep:
  `/tmp/rhwp-issue7333-pr-final-native-full-20260923/issue7333-pr-final-native-full`.
  completed 50/50, missing 0, structural flagged 0, pixel match 평균 95.74756%,
  ink match 평균 67.45244%.
- 최종 fresh WASM 전체 50쪽 visual sweep:
  `/tmp/rhwp-issue7333-pr-final-wasm-full-20260923/issue7333-pr-final-wasm-full`.
  completed 50/50, missing 0, structural flagged 0, Native와 동일한 지표.
- `rhwp export-pdf samples/issue7333/aaaaaa.hwp -o pdf/issue7333/aaaaaa-rhwp-stage11.pdf` 후
  한컴 MCP PDF와 96dpi 50쪽 전부 대조: 50/50, pixel match 평균 95.57073%, 최저 91.93506%(37쪽).
  최저 페이지도 overlay로 확인했으며 선/글꼴 rasterization 차이 외에 도형·표 프레임의 위치 이탈은
  확인하지 못했다.

자동 일치율은 사람의 판정을 대신하지 않는다. p14·p22·p23·p33·p40~p44·p47은 사용자가 지목한
도형/하단 문단/꼬리말 구간으로, 최종 Native와 WASM으로 각각 다시 overlay했다. p14는 저장 줄 뒤
하단 문단과 footer, p40은 빈 control stream 뒤 셀 스크린샷과 주석 도형을 PR 본문 대표로 둔다.

## PR 본문용 안정 asset

`mydocs/pr/assets/issue_7333_footer_shapes/`의 파일을 최종 PR head SHA에 고정한 raw URL로 PR 본문에
실제 Markdown 이미지로 표시한다.

| 출력 경로 | p14 하단 문단·footer | p40 셀 스크린샷·주석 | baseline 134쪽 |
| --- | --- | --- | --- |
| Native | `native_review_014.png`, `native_overlay_014.png` | `native_review_040.png`, `native_overlay_040.png` | `baseline_native_review_134.png`, `baseline_native_overlay_134.png` |
| fresh WASM | `wasm_review_014.png`, `wasm_overlay_014.png` | `wasm_review_040.png`, `wasm_overlay_040.png` | `baseline_wasm_review_134.png`, `baseline_wasm_overlay_134.png` |
| RHWP PDF ↔ MCP PDF | `pdf_export_review_014.png`, `pdf_export_overlay_014.png` | `pdf_export_review_040.png`, `pdf_export_overlay_040.png` | 해당 없음 |

## 다음 원격 단계

1. 이 후보 commit을 upstream의 임시 head branch로 push한다.
2. Open PR을 생성한 뒤 실제 PR 번호로 archive self-review와 필요한 오늘할일을 추가한다.
3. PR 본문에는 이 표의 각 실행 경로 대표 이미지를 정확한 head SHA raw URL로 표시한다.
4. 최신 head CI와 작업지시자 승인을 받은 뒤 merge 및 post-merge 절차를 수행한다.
