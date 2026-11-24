# #7774: 실물 회귀 입력과 검사 원문 이관

- 추적: [#7774](https://github.com/edwardkim/rhwp/issues/7774), 대표 [#7445](https://github.com/edwardkim/rhwp/issues/7445)
- 제거 전 source SHA: `d80593d15a3dba4c6c8066f8792b26fd6f02b6ff`
- 기준 upstream SHA: `463c48cc228170b576c832591b76826116e86a49`
- 사용자는 현재 전쪽90% 이상·완료 페이지 px 회귀 범위를 확인한 문서 이외의 실패 검사를 이관하도록 지정했다. 이번 이관을 새로운 base90% 미달 입증이나 실제 출력 결함 해결로 보고하지 않는다.
- 입력·독립 PDF를 아래 assets 경로로 이동했다. 제거 전 검사 원문은 자동 수집되지 않는 `.rs.txt`로 바이트 보존했다.
- parser·serializer·외부 이미지 등 범위 밖 정상 계약은 새 보존 경로를 직접 참조한다. 일반 samples 자동 수집 및 해당 render baseline 행에서는 분리한다.

## 보존 파일과 이전 경로

| 이전 경로 | 현재 파일 | SHA-256 |
| --- | --- | --- |
| `samples/hwp3-sample10-hwp5.hwp` | [hwp3-sample10-hwp5.hwp](hwp3-sample10-hwp5.hwp) | `a660a0d41898c8316479392c9687d81a15555431a581bdda988bf3598f6f0d51` |
| `samples/hwp3-sample10-hwpx.hwpx` | [hwp3-sample10-hwpx.hwpx](hwp3-sample10-hwpx.hwpx) | `3395e19bebea8b6689f383df1f4ea1ddb253dee91c4320392cc40e90e2e4f191` |
| `pdf/pr7268/hwp3-sample10-hwp5-p001-300-2024.pdf` | [hwp3-sample10-hwp5-p001-300-2024.pdf](hwp3-sample10-hwp5-p001-300-2024.pdf) | `650f210c1d4127b404beab774e445651d463b00ac774c84f68210f42773a391a` |
| `pdf/pr7268/hwp3-sample10-hwp5-p301-600-2024.pdf` | [hwp3-sample10-hwp5-p301-600-2024.pdf](hwp3-sample10-hwp5-p301-600-2024.pdf) | `ee223ef5a83e1349ed24af1a56fc3c786221df6c0f45c0100d65acd64d824e25` |
| `pdf/pr7268/hwp3-sample10-hwp5-p601-763-2024.pdf` | [hwp3-sample10-hwp5-p601-763-2024.pdf](hwp3-sample10-hwp5-p601-763-2024.pdf) | `52d42e6c96ce8c3631302ea9787e505f9639fd38ab7f01dc41f3140e4ee8954f` |

## 제거 전 검사 원문

- [issue_7051_hft_hangul_latin_halfwidth.rs.txt](issue_7051_hft_hangul_latin_halfwidth.rs.txt) / SHA-256 `16f7874c0f4e9b1dc04e46bf0a09e68c6885103773a6c37882d600f29cff50ca`
- [issue_7051_hwpx_twin_stored_ladder_witness.rs.txt](issue_7051_hwpx_twin_stored_ladder_witness.rs.txt) / SHA-256 `e77e445b44ee654bad8fd262ce3a8d907ec0d5d89461447254e97cf9b6c9dfff`

## 판정과 분리 범위

- HWP5 2건: `hft_hangul_ascii_run_fits_the_stored_line_width`, `hft_hangul_ascii_advance_matches_the_hancom_oracle`.
- HWPX 1건: `hwpx_twin_hft_ascii_run_fits_stored_line_and_matches_oracle_advance`.
- 독립 PDF 분할은300+300+163=763쪽이다. 중간 분할189쪽은 전체489쪽이며 TABLESPACE 문장이 존재한다. HWPX의 base·현재도763쪽, 같은 문장은489쪽·원본 문단17686이다.
- 이번 실패는 선행 공백이 별도 런으로 분리되면서 단일 런 문자열 검색이 깨진 것이다. 전체 쪽수가 틀렸다고 판정하지 않는다. 기존 HWP5 489쪽37.61557% 기록과 이번 HWPX 전쪽 미검증을 구별한다.
- [render baseline 제거 원문8행](render-corpus-baseline-rows.tsv.txt)을 보존했다. raw IR 동일 글자모양 관측 corpus와 순수 parser/serializer 계약은 유지한다.

## 복원 조건

같은 입력·독립 Print의 전쪽 Native/fresh WASM 최저90% 이상과 직접 확대 판독을 확보한 뒤 내용·줄/개체 소속·폭 비율·캐럿 경계 관계로 복원한다. ±2px 좌표를 고정하려면 해당 페이지 양쪽98% 이상과 독립 기대 좌표를 추가로 입증한다. 이관은 이슈 종료나 문서 피델리티 완료가 아니다.

## 2026-11-23~24 독립 Print 전쪽 대조

| 변환본 | 쪽수 | Native 최저 | fresh WASM 최저 | 489쪽 Native / fresh WASM |
| --- | ---: | ---: | ---: | ---: |
| HWP5 | 763 | [99.57%](sql-hwp5-native.tsv) | [99.34%](sql-hwp5-wasm.tsv) | 100.00% / 99.98% |
| HWPX | 763 | [99.57%](sql-hwpx-native.tsv) | [99.34%](sql-hwpx-wasm.tsv) | 100.00% / 99.98% |

측정 조건은 96 DPI, 한컴 2024 Print, 보정 전 2px raw 실루엣이다. HWP5는 위 세
Print 분할본을 순서대로 대조했다. 별도로 출력한 HWPX 1-up 기준 PDF는 96,144,250바이트다.
SHA-256은
`ad6c03db8a81c5a65ba064ef0933b8a37446f1b106fa4b09757bc8b4bb7f04a9`다.
한컴 엔진 `13.0.0.3901`, job `0aa8ae32-b51b-4b42-bb30-3a51dd8c43af`에서 출력했다.
원본 HWPX를 한컴 2024 직접 인쇄 경로로 전달했고 `pdf_print_method=0`과 763쪽을 확인했다.
전쪽 PDF는 `/home/tsjang/rhwp-local-pdf/issue7774/hwp3-sample10-hwpx-hancom-2024-one-up.pdf`에 보관한다.
아래 이미지는 독립 Print, Native, fresh WASM 순서다.

- [489쪽 SQL 문장과 소속 글줄](sql-hwpx-page-489.png)
- [554쪽 마지막 본문과 쪽번호](sql-hwpx-page-554.png)

## 16쪽 대괄호 원본 폭 확인

두 변환본의 16쪽에는 굵고 기울인 명조로 ‘[주의]’를 쓰는 원본 문단이 두 곳 있다.
독립 Print에서 각 대괄호는 450 HU로 전진한다. 다른 표시용 글꼴을 사용해도 네 글자의
원본 전진폭과 소속을 유지하는 회귀를 등록했다. 두 형식의 16쪽은 보정 후 Native와
fresh WASM 모두 99.99%이며 재측정한 여섯 쪽은 모두 99.98% 이상이다.

- HWP5 [Native](sql-hwp5-brackets-native.tsv) · [fresh WASM](sql-hwp5-brackets-wasm.tsv)
- HWPX [Native](sql-hwpx-brackets-native.tsv) · [fresh WASM](sql-hwpx-brackets-wasm.tsv)
- [16쪽 독립 Print와 양쪽 출력 확대 비교](sql-page-16-brackets.png)

대괄호 보정 후 각 변환본의 fresh WASM 763쪽을 다시 비교했다.
두 형식의 최저값은 757쪽 99.34%다. 100.00%인 쪽은 HWP5 464쪽과 HWPX
461쪽이다. HWPX 전쪽 TSV SHA-256은
`cfcac1a1d94fd117e3922054ccb24df3d14c69ff65db30f3447a5aeb3ddec36c`다.

## 리베이스 뒤 16쪽 대괄호 비교

devel `28a6afc1ebe3e9873605f2d306ca094b422abf24`과 후보 제품 `9a78578e17f46dbd53b19d1eb10445dbef7a2988`에
같은 입력·독립 Print·96 DPI와 119개 글꼴을 같은 순서로 공급했다. HWP5·HWPX
모두 16쪽의 fresh WASM raw 실루엣이 96.82%에서 99.99%로 올라갔다.
두 문단의 ‘[주의]’ 런은 원점과 줄 높이를 유지하며 폭이 49.2px에서 37.2px로 바뀌었다.
독립 Print를 확대해 대괄호 간격과 뒤 문장의 배치를 확인했다.

- [HWP5 독립 Print·devel·후보](sql-hwp5-page-16-rebase.png)
- [HWPX 독립 Print·devel·후보](sql-hwpx-page-16-rebase.png)
