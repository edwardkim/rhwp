# 가맹사업 118쪽 정렬 회귀의 변경 전·후 증거

변경 전 `7f04c48cd395086ed59c08565f343d3f9e1b013d`는99.99852%,
현재 `b4ff8c81b5d986baf47bad9a3eee83cbd9d8367b`는96.18473%다.
Native 전142쪽은최저93.16816%로자동지표를통과했지만, 118쪽 하단
중첩 표의3글줄은독립Print보다5.4px아래다. 현재 실제 배치 회귀로 판정한다.
표 외곽/내용 순서는같고 render tree와13,723개픽셀이달라졌다.
72쪽은96.90530%→99.95995%로개선됐으므로두경계를함께보호한다.

## 원인과 수정 경계

실제Debug호출에서72쪽은total/center73.6,118쪽은57.3333333333,
두경우모두full-width flag/ordinary owner/cell_was_split=true,closing=false다.
기존full-width flag는`stored_full_width_row_declared_height`와
`saved_nested_single_cell_frame_height`의다른출처를OR로합쳐정렬에사용했다.
원본전체행선택글줄과중첩단일셀의원본유닛높이는동일한계약이아니다.
출처별인증결과를분리해기존예약/정렬소유를유지하고정렬높이소비를바로잡는다.
문서ID/고정이동/clamp로맞추지않는다. 이파일은결함증거이며보정후승인이아니다.

동일입력/독립Print,새Native/fresh전쪽·확대판독과정식관계회귀검증이남는다.
입력: `samples/issue5966/1130000-202100008_franchise_review_report.hwp`.
Print: `pdf/pr_6088_6144/hancom2020/pr_6088_6144_issue5966_franchise_review_report_1130000-202100008_franchise_review_report-2020.pdf`.
원문해시/명령/로그/TSV/guard는ignoredv452의whole-native-plan/results,
franchise-native-current-prior-pixel-comparison,franchise-native-prior-raster-differences,
p118-and-p72-alignment-trace,p118-before-after에보존한다.

## PNG SHA-256

- `current-p118-review.png` — `9ec1bb37fdc6939c7c9571fd51ae1a5c7b344865c5d7450d7919278a240ba2a3`
- `current-p118-overlay.png` — `f3afc3d9bcd8bf90c32ffb9ff9174ef83263cf87fefa54bcce9409b2b374e42e`
- `previous-p118-review.png` — `fb6311c50ea956dfeb64c090efd67bd4a8f6593736c1076b84b0dd691126f04b`
- `previous-p118-overlay.png` — `26037f47e52a0fba9585f369bb661f69bd8786ff8e18248c3a49c45845bad3f1`
