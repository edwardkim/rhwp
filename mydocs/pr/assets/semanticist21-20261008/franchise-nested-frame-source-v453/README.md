# 원본 전체 행·중첩 셀 프레임 정렬 보정의 Native 증거

생산 코드 `ddf17f86476293b114b6236bfa6cf1ebc9d9d474`.
Native CLI SHA-256 `bcf415c03b79c2e37627d558628136ca05e356e7cd0b54b0980426a9c292fd69`.
경계17쪽은exit0/source guard=true,최저68쪽93.16816%,미달/누락0이다.
72쪽99.95995%,118쪽99.99852%. 현재72/118 review와동일좌표확대를직접열었다.
72쪽선택글줄/오른쪽셀/외곽/각주·뒤내용을보존하고118쪽3줄5.4px하향회귀를복구했다.
16쪽은직전b4의통과/판독RGBA와동일하며118쪽은이전정상7f의RGBA와동일하다.
전142쪽새render tree도직전b4와118쪽만다르고이전7f와72쪽만다르다.
글자획·자간·불릿크기·괘선래스터차이는남는다.
fresh 경계17쪽도 아래와 같이 확인했다. 전쪽 이미지/회귀/lint/CI 완료로 확대하지 않는다.

## 구현과 소비

공통마지막행컷조건을유지하고`stored_full_width_row_declared_height`와
`saved_nested_single_cell_frame_height`의인증결과를각각생산한다.
`frame_owns_alignment`는두출처를모두소비하고,선택줄높이는원본전체행/
종료프레임에서사용한다.중첩셀은기존원본유닛높이소비로이어진다.
예약/물리상자/논리컷을바꾸거나고정좌표·문서조건으로이동하지않는다.

## 입력·독립 한컴 Print

- `samples/issue5966/1130000-202100008_franchise_review_report.hwp` — SHA-256 `a1b8299b2554a83873b780474defb02efa949b43c86c63bcc164967f96091e55`.
- `pdf/pr_6088_6144/hancom2020/pr_6088_6144_issue5966_franchise_review_report_1130000-202100008_franchise_review_report-2020.pdf` — SHA-256 `4cf22800e29baddb9b84023462c75252efd4efb04d22e9e64a8e3d9313076009`.

## 재현과 증거

ignored `output/pr-review/semanticist21-20261008/nested-frame-alignment-provenance-v453/`의
`native-boundary-plan.json`에명령·17쪽범위·실제공급글꼴·소스/입력해시를고정했다.
Native `--silhouette-only --compat 2022 --embed-fonts full`,검증한프랜차이즈글꼴환경을사용했다.
실제exit/guard는native-boundary-result.json,페이지TSV/manifest/PNG해시는
native-boundary/franchise5966/아래,판독은native-boundary-direct-reading.json,
출력동일성은native-boundary-prior-pixel-comparison.json과whole-franchise-before-after-tree-comparison.json에보존한다.
확대는왼쪽rhwp/오른쪽Print:72쪽 `(110,700)-(685,990)` 2배,118쪽 `(115,905)-(680,1006)` 3배다.

## PNG SHA-256

- `native-p072-review.png` — `c02bae23566838d416868d7c541c51229aa3dbe998e3e71dd738c72c8a2c8ed9`
- `native-p072-overlay.png` — `a65ec28491006e4579dde2d7ac19ab2695b44987c6399e31f79a2283e72e7ade`
- `native-p072-table-zoom.png` — `5bc572392922b4f00c9e44428a7f3d9697def77d660b8e018f1be3f594e034fb`
- `native-p118-review.png` — `3dbbfb769234b90ffebde86970dbadbe6e76efb3b3ff74e44c032b026da8ed48`
- `native-p118-overlay.png` — `1413247c532d009c745b02d75960a6f913c3923ecee13533ada1585374a90176`
- `native-p118-table-zoom.png` — `e1e3b38c35b1da8204c1f884dc8f2ed07f36683a6ec22ac4d95cf11d7a6a2477`

## 현재 fresh 경계

동일 생산 소스의 fresh17쪽 exit0·1517.642초/source guard=true, 최저68쪽93.16816%, 미달/누락0.
전체 문서의 쪽수142/142이며 선택17쪽 모두 현재 Native RGBA와 같다.
72쪽99.95995%,118쪽99.99852%. 현재 두 review와 같은 좌표 확대를 직접 열어
72쪽 개선/118쪽 중첩 표 세 줄 복구·셀 소유/외곽/각주 및 뒤 내용을 확인했다.
글자획·불릿 및 괘선 래스터 차이는 남는다.
WASM SHA-256 `4299eb204ec8283d75446b61f2183d3c4f3cd41acc0acabfd7be7dc065f01ae0`.
ignored v453/fresh-boundary-plan/result/current-proof/direct-reading에 명령/해시/실제 점수/판독을 고정했다.

- `fresh-p072-review.png` — `c3e3ca3dc5cce4f14d4b56e95dfe560167569d163a52f2f22191209da249ba2d`
- `fresh-p072-overlay.png` — `fda6399bf348072e93febc9f5f6ad9c4373dfe7e711324ade419bfb35d149f30`
- `fresh-p072-table-zoom.png` — `5bc572392922b4f00c9e44428a7f3d9697def77d660b8e018f1be3f594e034fb`
- `fresh-p118-review.png` — `fffe7db9ea2402d4f54aba6abef8a7ad1538971b1b84d805ce5ec5739840aec2`
- `fresh-p118-overlay.png` — `5e42cdb406d74fac000fe33c26bebd16c9a824d01c07ee65111a476697089b2e`
- `fresh-p118-table-zoom.png` — `2972898870b1380bcea91633eca02252826d1501f8070736ca650ca6a327be7e`
