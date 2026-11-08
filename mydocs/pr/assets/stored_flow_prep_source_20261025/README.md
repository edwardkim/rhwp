# PREP 표 뒤 본문의 원문 소유와 첫 실제 글리프

검사 대상은 이미 추적 중인 아래 원본과 독립 한컴 Print PDF다. 입력이나 PDF를
수정하거나 더 작은 대조군으로 바꾸지 않았다.

- [원본 HWPX](../../../../samples/issue2006/1790387_prep_final_report.hwpx)
  SHA-256: `c68baed24096386f9041930d24d39409b61ac99463bf04dfd242440dfdeb739f`
- [한컴 2024 Print PDF](../../../../pdf/issue2006/1790387_prep_final_report-hwpx-2024.pdf)
  SHA-256: `732baa118fbe380cf4c5e12e080eee5acb1eb8372070390fef47e53e2469f729`
- [정식 검사](../../../../tests/cases/issue_7445_stored_paragraph_flow_projection.rs)의
  `prep_post_table_body_preserves_independent_print_when_prior_source_axis_is_invalid`

## 전쪽 검증과 직접 판독

제품 runtime `8dc75860cec88ddf7c3f8aeee230b2ca7e2e5537`의 Native224와 최적화
fresh WASM063을 사용했다. `compat=2024`, 96 DPI, Print 프로필이며 PDF와 출력은
각각 전체 140쪽이다. 두 TSV 원본 모두 140행이며 누락·중복·90% 미만 쪽은 없다.

| 경로 | 전 140쪽 raw 최저 | 70쪽 raw | 전체 TSV | 대표 review / overlay |
| --- | ---: | ---: | --- | --- |
| Native224 | 96.01430% | 100.00000% | [Native TSV](tsv/native.tsv) | [review](png/native/review_070.png) / [overlay](png/native/overlay_070.png) |
| fresh WASM063 | 96.16717% | 100.00000% | [WASM TSV](tsv/wasm.tsv) | [review](png/wasm/review_070.png) / [overlay](png/wasm/overlay_070.png) |

70쪽의 표와 뒤 본문, 첫 어절의 원점·기준선·본문 순서 및 하단 내용을 같은 배율의
독립 PDF·Native·fresh WASM으로 직접 판독했다. 실루엣 100%는 엄격 RGB 일치가
아니며 글자와 선의 안티앨리어싱·농도 차이까지 없다는 뜻으로 사용하지 않는다.
이 검증은 해당 기존 검사 보정의 근거이며 전체 통합 PR이나 새 base 대비 무회귀
검증의 완료를 대신하지 않는다.

## 잘못된 run 선택과 유지한 독립 기대값

기존 검사는 `section:2/para:38/char:1`이 첫 어절로 시작하는 run이라고 가정했다.
실제 원본의 이 소유자는 선행 공백이며 본문 run은 그다음 원문 문자에서 시작한다.
텍스트 run이 나뉘거나 합쳐지는 방식은 첫 글자의 원문 소유와 다르다.

같은 문단 소유의 run을 순서대로 모으고 전체 원문과 대조해 누락·중복·순서를 검사한다.
그중 공백·제어 문자가 아닌 첫 실제 cluster를 찾아 그 run의 전체 `runToPage` 변환으로
쪽 좌표를 얻는다. `char:1`을 `char:2`로 바꾸는 다른 고정 핀을 추가하지 않았다.

독립 PDF 70쪽의 KoPubDotumLight, `cm=0.12`, `tm=(659,5118)`, MediaBox 위쪽
841pt에서 얻은 첫 글자 원점 `(105.44, 818.88)` CSS px와 ±2px 허용 오차는
변경하지 않았다. 양쪽 raw 100%인 이 완료 페이지에만 이 좌표 검사를 적용하며
140쪽 수와 원문 보존 검사도 유지한다. 원본 계약이나 baseline을 완화한 변경이 아니다.

같은 파일의 실제 네 검사를 동결 Native224로 실행해 **4 PASS / 0 FAIL**,
ignore·실행 누락 0건을 확인했다. 보정 전 실패는 실제 첫 어절을 찾지 못한 선택자다.
제품 코드는 바뀌지 않았으므로 이번 검사 보정을 새 렌더링 수정의 FAIL→PASS 증거로
주장하지 않는다. focused native Clippy와 rustfmt도 통과했다. 전체 Cargo lint와 전체
nextest 실행은 별도 게이트다.

## 재현과 출처

[provenance.json](provenance.json)에 실제 TSV 명령, Native producer와 fresh package
파일 SHA, 입력·PDF·TSV·PNG 및 검사 소스 해시를 보존했다. 전쪽 TSV는
`scripts/visual_sweep.py --silhouette-only`로 산출했고 대표 PNG는 같은 입력·PDF·
producer·글꼴·DPI에서 `--pages 70`으로 산출했다. 실루엣 도구와 지침은 변경하지 않았다.
