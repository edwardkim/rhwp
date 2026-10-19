# 개인정보 표의 독립 일반 행 최소 보존 — C436

Issue: #7445. 관련 원 계약: #5846, #5714, #7095, #7642.

## 입력과 producer

- parent code: `3c7b279990cea4f6e1e1ad7164a0e50f0b0ebbdc`, 고정 devel base: `4a07d3ae4dd3255a57fb0166b2b2a2dc335a61fa`.
- 원본: `samples/task2097/75544_pii_bunseok.hwpx`, SHA-256 `f429768ec02e5a849d08c06678cbee029fdc8618ca9e2bdcdbba485087fe71f1`.
- 독립 한컴2020 Print: `pdf/task2097/75544_pii_bunseok-hwpx-2020.pdf`, SHA-256 `0fef38f44b3825f1c97869c2720112536504eaf30cec2a2d2af6d4399ffe8770`, 전66쪽.
- 선언 환경: `mydocs/pr/assets/issue7445/resolution-20261005/pii5846-reference-font-environment.json`, SHA-256 `21e5c5a0b081bf3101749e5b80c1b053fa3eeff4fcb1ba3f8aa33fd3e2a4d544`.
- Native108 SHA-256 `bc066a4518d57322c07a21603769e97b249ef052f6e7728c751295feabf19f85`.
- fresh WASM111 SHA-256 `8f850bdf8eb2bdbdcd8b39a1aaa4cc6ed0e3d8248a5c7273570607cad6757e84`; JS SHA-256 `23f085e36c78d952ddac0d91faed9752f33ee1124db78e290382687410917591`.
- Rust runtime source2199파일은 ignored `repair108-pii-independent-row-candidate/source.json`에 고정했다.
  test만 변경·정렬한 뒤에도 이 source는 모두 같다. Native/fresh WASM 전쪽은 새로 출력했다.
- WASM은 Mac locked wrapper `--target web --out-dir pkg --no-opt`, 실제 재컴파일210.334초/exit0이다.
  Docker 최적화 통과가 아니다. root pkg/Studio public 및 실제 Studio7736 초기화 응답을 `repair114-pii-browser-initial-response/result.json`으로 대조했다.
- font supply는 기존 네 경로와 동일하며 파일별 hash·실제 전체264쪽 PNG hash는 ignored `repair112-pii-whole/producer.json`, `repair122-pii-whole-audit/whole-png-hash-comparison.json`에 보존한다. 새 글꼴은 배포·커밋하지 않았다.

## 원인과 공통 소비

원 #5846 → 원 PR #5904 → 실제 통합 #5912의 `b9eb551`은 59→60쪽 중첩 행의 내용 소유를 보호했다.
이번 2→3쪽 독립 일반 칸의 빈 물리 공간과 같은 증상이라고 취급하지 않는다.
`363ef22d2c7`의 #7642 보정은 완료된 걸침 셀의 내용 유래 높이를 다시 열지 않게 했다. 이 억제는 유지했다.

section0/para13의 row9는 일반 칸(col2/6/10)의 저장 높이3758HU를 소유하지만 다른 열의 rowspan도 걸친다.
내용 컷 `[1,1,1]`을 모두 소비해도 같은 행의 저장 물리 최소가 끝난 것은 아니다.
기존 스캔은 첫 조각에31.826667CSSpx를 예약했으나 paint는26.2px로 줄였고 뒤쪽에서 row9를 건너뛰었다.
재래핑·dirty·편집 상태를 제외한 원본 일반 칸의 실제 최소만 scan 결과로 전달한다.

`stored_independent_row_minimum` → RSPAN_BAND의 실제 수용 row/minimum → refit 결과 전파 → emit의 같은 완료 컷·최종 수용 상자 차감 →
`end_row_owns_stored_frame` 및 `next_start_row_height_override`가 같은 잔여를 소비한다.
현재 첫 조각31.826667 + 뒤 조각18.28 = 원본50.106667CSSpx다. painted26.2px에서 잔여를 역산하지 않는다.
기존 first-frame 일반 칸 경로는 걸침 칸이 있는 행을 배제하므로 이번 독립 최소와 경쟁하지 않는다.
다른 column의 rowspan 물리 장부, 명시 저장 reset/frame 및 내용 유래 reflow tail 억제는 유지한다.

## 전체 결과와 미해결 범위

192DPI, compat2022, Print, canonical PDFium, raw2px90% 기준. 별도 `pdftoppm-audit` TSV는 canonical과 섞지 않는다.

| 환경/backend | 쪽수 | 전체 최저 | 미달 쪽 |
| --- | --- | --- | --- |
| 선언 Native | 66/66 | 85.44699% | 36 |
| 선언 fresh WASM | 66/66 | 85.44699% | 36 |
| 기본 Native | 66/66 | 81.66959% | 4,20,21,36 |
| 기본 fresh WASM | 66/66 | 81.93226% | 4,20,21,36 |

네 조건 모두 prior C434 대비 PNG byte가 바뀐 쪽은2·3쪽이며 나머지64쪽은 동일하다.
이것은 이번 변화 범위의 증거이며 최신 devel/전체 PR 수용을 대신하지 않는다.
Native 고정4a07 대비 남은0.1%p 이상 하락은 선언36쪽3.92801%p, 기본22쪽0.23111%p·36쪽3.92801%p다.
현재36쪽은 본문 없이 자동 쪽번호만 있는 쪽이다. Native snapshot은 실제 HCR Dotum을 선택하며 독립 PDF도 HCRDotum이다.
동일 family라는 사실만으로 파일·glyph 동일성을 주장하지 않는다. 번호 잉크가 약1.5CSSpx 위인 점과 base 대비 paint 차이를 추적 중이다.
**전66쪽 수용·전체 nextest/PR 준비 완료가 아니다.** 다른 전체 문서·actual36·필수 최종 게이트도 남아 있다.

## 완료 페이지와 회귀

| 쪽 | 수정 전 Native | 수정 후 Native | 수정 전 WASM97 | 수정 후 fresh WASM111 |
| --- | --- | --- | --- | --- |
| 2 | 97.57528% | 99.69840% | 97.17350% | 99.70746% |
| 3 | 35.07344% | 99.53257% | 35.59924% | 99.54130% |

두 쪽의 review/overlay를 Native/fresh WASM에서 직접 확대 판독했다. 해당없음 응답은 앞쪽에 한 번,
빈 행 띠·비용 본문은 뒤쪽에 한 번 남고 일몰제 표와 마지막 우선허용 행이 뒤따른다.
3쪽 예외기준 둘째 항목의 작은 자간 차이는 남아 있으며 해당 글리프를 새 golden으로 고정하지 않았다.
59쪽 Native/60쪽 fresh WASM도 원 중첩 행의 내용·순서를 직접 대조했다. 원 범위와 새 범위를 혼동하지 않는다.

완료2·3쪽은 양쪽 raw99%대이며 독립 Print 벡터 선에서 읽은96dpi CSS 좌표를 기존 #5846 검사에 ±2px로 추가했다.
2쪽 마지막 행 하단1042.698667, 3쪽 표 상단77.514667, 빈 행 뒤 비용 행 시작95.734667이다.
independent 추출은 PDF의 원래 transform을 적용했으며 helper·현재 칸 높이로 기대값을 만들지 않았다.
기존 소유·순서·누락·중복·본문 포함·후속 표 검사를 유지한다.

- 기존 #5846 미수정 검사: before96 1FAIL → after108 1PASS (`repair109-pii-native-probe`).
- 강화한 최종 동일 검사: before96 실제1FAIL(행 하단1038.1 vs 독립1042.698667) → after108 1PASS (`repair118-pii-qualified-px/formatted-probe`).
- 기존7case 정식 nextest:21PASS/1675선택skip,3.978초,전체339.080초 (`repair113-pii-focus-contracts`).
- 강화 뒤 실제 자동 target 재계산: suite026, 정식1PASS (`repair119-pii-qualified-gates/qualified-nextest.log`), 전체56.919초.
- Native Skia lib Clippy `-D warnings`:94.678초/exit0. fmt 최종/manifest-check도exit0.
- 초기 nextest 호출은 `-j8`과 `--test-threads8` 중복으로 테스트 전에 거절됐다. invalid 로그를 별도 보존하고 CARGO_BUILD_JOBS8 + test-threads8 한 번으로 실제21건을 재실행했다.
  첫 px tuple 포맷 검사 실패도 고친 뒤 최종 fmt exit0을 확인했다.

## 대표 이미지

| 범위 | review | overlay |
| --- | --- | --- |
| 3쪽 Native 이전 | [review](p003-before-native-review.png) | [overlay](p003-before-native-overlay.png) |
| 3쪽 WASM 이전 | [review](p003-before-wasm-review.png) | [overlay](p003-before-wasm-overlay.png) |
| 2쪽 Native 이후 | [review](p002-after-native-review.png) | [overlay](p002-after-native-overlay.png) |
| 2쪽 fresh WASM 이후 | [review](p002-after-wasm-review.png) | [overlay](p002-after-wasm-overlay.png) |
| 3쪽 Native 이후 | [review](p003-after-native-review.png) | [overlay](p003-after-native-overlay.png) |
| 3쪽 fresh WASM 이후 | [review](p003-after-wasm-review.png) | [overlay](p003-after-wasm-overlay.png) |

모든 최종 TSV는 원본 바이트를 보존했다. 생성 SVG만 batch 종료 후 hash manifest를 남기고 정리했으며
PNG·TSV·로그·원본·PDF·공유 target을 유지했다. 삭제 SVG를 재사용 증거로 주장하지 않는다.
