# PR #9041 리뷰 — 어울림 띠 아래로 넘친 표의 쪽 소유와 줄 범위 보존

## 최종 판정

**승인 — 명시한 통합 head에서 원 PR 주장 범위의 로컬 검증을 완료했다.** 제품 메인터너 보정 없이 원 변경을 수용한다. 원격 조치는 별도이며, merge 전 통합 PR 최신 head의 필수 CI·mergeability를 확인한다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 원 PR·작성자 | [#9041](https://github.com/edwardkim/rhwp/pull/9041) · `planet6897` |
| 관련 해결 이슈 | [#9027](https://github.com/edwardkim/rhwp/issues/9027) — 아래 주장 범위 |
| 고정 원 head | `3bfcd54a2a6767b1d29962ae1845b9d8868d2abc` |
| 최종 제품 code | `c6e852a92b3661136552a432d649e168e27f88d4` |
| 시각 산출물 producer | `605740c4363e9e6bacd84895d93abbc7d7bf62c3`, 아래 동일성 근거로 재사용 |
| 최종 검증 base | `00d09f773a2f6c21604a4e6394df7152bbb26e8c` |
| 고정 base 시각 producer | `1003349996897c6ff097271e7296ae4d1a3d807f`, 새 devel의 제품 경로와 동일 |
| 통합 branch | `review/integration-20261113-planet6897-open5` |
| reviewer | `jangster77`, API 재확인. 원 source branch를 rewrite하지 않았다. |

#9050의 테스트·문서만 포함된 새 devel로 충돌 없이 리베이스했다. 20개 patch는 range-diff에서 모두 동일하며 제품 코드·빌드/출력 경로와 검증 입력 51개의 바이트가 변하지 않았다. 기존 PNG/TSV는 원 producer SHA를 유지해 재사용했다. 반영된 `issue_7620_cellbreak_float_row_carry`의 집중 3건 및 최신 전체 회귀를 실제 실행했다. [리베이스·재사용 근거](../assets/planet6897_open5_20261113/rebase-validation.json).

5개 원 PR의 고유 commit 20개를 누적 `cherry-pick -x`했다. 개별·모든 쌍·누적 merge와 실제 commit 적용의 tree를 대조했다. 충돌한 #8937·#8976은 제외하고 우리 reviewer 요청도 취소했다. [적용 원장](../assets/planet6897_open5_20261113/source-plan.json) · [시뮬레이션](../assets/planet6897_open5_20261113/simulation.json).

## 변경과 검토 범위

저장 host 위치와 실제 흐름 위치 중 아래쪽을 사용해 overflow를 판단한다. 뒤 저장 재시작이 표 바깥 상자 높이와 같은 경우에 표만 다음 쪽에 두며, 재시작이 문단 첫 줄일 때만 뒤 문단을 그 쪽에 연결한다. 어울림 띠의 바닥은 실제로 흡수한 줄 범위로 제한하고 host 끝 간격을 중복 소비하지 않는다. 측정/조판과 paint가 인증된 같은 successor 원점을 소비한다.

검토 소스: src/renderer/float_placement.rs · src/renderer/typeset/table/block/entry.rs · src/renderer/layout.rs. 원본 속성·소유·공통 조판 결과에서 조건을 계산하며, 문서명/쪽 번호에 맞춘 보정은 추가하지 않았다. 사용자 지시대로 원 PR의 주장이 충족된 범위에서 수용하며 범위 밖의 미세 모양까지 새 작업으로 확대하지 않는다.

## 검증 입력과 결과

- 원본: [1312000-201400081_2015_senior_policy_course.hwp](../../../samples/issue9027/1312000-201400081_2015_senior_policy_course.hwp), SHA-256 `93d3c39fec006a1a27deff66c5ce7c71bbc361b040c12583c83395dcf7ad120f`.
- 독립 Print: [1312000-201400081_2015_senior_policy_course-hwp-2020.pdf](../../../pdf/issue9027/1312000-201400081_2015_senior_policy_course-hwp-2020.pdf), SHA-256 `29c3e37689d5f2769f6db9824afe97c680197b53355c74160f12fa3d315ceafc`, 187,701바이트.
- 실제 PDF Creator `Hwp 2022 12.0.0.4547`, Producer `Hancom PDF 1.3.0.547`. 원 PR의 데스크톱 **인쇄(Print)** 근거를 재사용했다. 파일명에 있는 2020은 engine bucket이며 실제 출력 제품을 2020으로 바꾸어 기록하지 않는다. 웹한글기안기/Cairo PDF를 판정 기준으로 사용하지 않았다.
- 실제 실행 bytes와 검토 code의 Git blob이 같다. [입력 감사](../assets/planet6897_open5_20261113/committed-input-audit.json).

리베이스 후 최소 글꼴 macOS 프로세스 격리에서 전체 nextest **13,430 PASS / 0 FAIL / 45 skip**, 실제 테스트 632.595초를 확인했다. 리베이스 전 원 PR 5개 집중 **6 PASS**와 리베이스 후 전체 실행에서 동일 6건을 확인했으며 이 PR의 `issue_9027_float_table_band_overflow` **1 PASS**를 확인했다. 테스트에만 최소 글꼴 격리를 사용했다. **Visual Sweep은 원본 글꼴 프로그램을 공급한 일반 환경**에서 실행했다.

fmt·Native/WASM Clippy·workspace build·all-target Clippy, Native Skia lib 합계 **4,034 PASS / 12 ignore**, PNG **2 PASS**, PDF **36 PASS**, manifest contracts/check와 unit tiers를 통과했다. 실제 명령·로그 SHA·실행 결과는 [Rust 검증](../assets/planet6897_open5_20261113/rust-validation.json)에 연결한다.

fresh WASM은 Mac locked wrapper의 `--target web --out-dir pkg --no-opt` 로컬 대체 빌드다. Docker 최적화 통과라고 표시하지 않는다. source code·입력·PDF·조건이 같은 고정 산출물의 검증과 재사용 신원을 분리한다.

## 같은 입력·독립 Print의 전쪽 시각 비교

`2015년 고위정책과정 교육운영 개요`의 관련 전체 페이지를 96 DPI·compat2022·현행 canonical PDFium raster·색상 경계 보정 전 2px raw 실루엣으로 비교했다. strict RGB 일치율과 다르다.

| 경로 | 후보/PDF 쪽수 | 고정 devel 전쪽 최저 | 후보 전쪽 최저 | 페이지별 최악 변화 | 원본 TSV |
| --- | --- | --- | --- | --- | --- |
| native | 2 | 44.96250% | 99.99406% | +25.88768%p | [후보 TSV](../assets/planet6897_open5_20261113/claims/pr9041/native/silhouette.tsv) |
| wasm | 2 | 44.93477% | 99.99915% | +26.01971%p | [후보 TSV](../assets/planet6897_open5_20261113/claims/pr9041/wasm/silhouette.tsv) |

고정 devel에서 정상 출력된 동일 입력은 쪽별로 비교했고 2%p 초과 하락은 없었다. 열지 못한 문서의 0% TSV는 만들지 않았다. #9030·#9034의 Native 및 직접 로드한 WASM 파싱 오류를 구분했다. [전체 base/후보 대조](../assets/planet6897_open5_20261113/base-candidate-qualification.json) · [실제 base WASM 오류](../assets/planet6897_open5_20261113/base-wasm-parser-failures.json).

정본 보호 문서 18개 조건, 경로별 1,771쪽은 Native/fresh WASM 모두 90% 이상이며 최저 98.36252%, 공식 두 자리 표시값 하락 0쪽이다. 후보 3,562쪽의 PNG hash·쪽수·쪽 번호·누락/중복을 전수 확인했다. [보호 검증](../assets/planet6897_open5_20261113/protected-candidate-qualification.json) · [쪽별 신원 감사](../assets/planet6897_open5_20261113/candidate-page-identity-audit.json).

## 시각 증적과 남은 차이

Native와 fresh WASM에서 관련 모든 페이지의 raw/PDF·review·overlay를 직접 대조했다. 1쪽 어울림 띠 뒤 일반 본문과 하단 표, 2쪽 제목 표 바로 아래의 큰 비교 표를 확인했다. host와 빈 문단은 앞 쪽에 남는다. 도표 색상 차이는 raw 실루엣과 RGB 일치를 구분해 남겼으며 원 PR의 표 이월/간격 범위로 확대 보정하지 않았다.

[직접 판독](../assets/planet6897_open5_20261113/direct-reading.json) · [이미지별 hash](../assets/planet6897_open5_20261113/claim-rasters.json).

![원 PR #9041 주장 페이지 Native review](../assets/planet6897_open5_20261113/claims/pr9041/native/review-p001.png)

![같은 페이지 Native overlay](../assets/planet6897_open5_20261113/claims/pr9041/native/overlay-p001.png)

![같은 페이지 fresh WASM review](../assets/planet6897_open5_20261113/claims/pr9041/wasm/review-p001.png)

![같은 페이지 fresh WASM overlay](../assets/planet6897_open5_20261113/claims/pr9041/wasm/overlay-p001.png)

![같은 주장 2쪽 Native review](../assets/planet6897_open5_20261113/claims/pr9041/native/review-p002.png)

![같은 주장 2쪽 Native overlay](../assets/planet6897_open5_20261113/claims/pr9041/native/overlay-p002.png)

![같은 주장 2쪽 fresh WASM review](../assets/planet6897_open5_20261113/claims/pr9041/wasm/review-p002.png)

![같은 주장 2쪽 fresh WASM overlay](../assets/planet6897_open5_20261113/claims/pr9041/wasm/overlay-p002.png)

## 공통 원칙·실제 소비 경로

| 항목 | 판정과 근거 |
| --- | --- |
| 구현 일반성·원본 소유 | 위 원본 속성과 parser/layout 공통 소비를 대조했다. 문서/쪽 예외를 추가하지 않았다. |
| 줄·셀·개체 소속과 보존 | 원 PR 회귀와 독립 Print 전쪽 및 실제 render 결과로 주장한 소유/순서/겹침을 확인했다. |
| 회귀 기대값 | 원 contributor 검사를 그대로 실행했다. 문턱·PDF·기대값 완화나 검사 제거를 하지 않았다. 원 작성자의 수정 전 FAIL 근거는 [원 PR 고정 주장](../assets/planet6897_open5_20261113/original-pr-claims.json)에 분리했으며, 작성자의 전체 10k A/B를 검토자가 다시 실행한 것으로 표시하지 않는다. |
| Native PNG | 실제 native-skia CLI `export-png --profile print`. SVG raster를 대신 사용하지 않았다. |
| fresh WASM Print | 현행 실제 Canvas exporter의 PageLayerTree 소비. Studio 일반 화면/CK와는 별도 증거다. |
| Studio 실제 화면·지원 CK | 실제 36건: Canvas2D 32건(기존 글꼴 자원 guard의 CK→Canvas2D fallback 20건 포함), 지원 CK software 4건. fallback을 CK 성공으로 세지 않았다. 같은 주장 페이지의 실제 출력·초기 WASM 36개 응답 hash를 확인했다. [소비 경로 원장](../assets/planet6897_open5_20261113/actual-browser-qualification.json). |
| 쪽 이월·이어받기 | 셀 내부 조각을 자르는 변경이 아니라 저장 successor가 표 전체 바깥 높이임을 확인한 원자적 표 이월이다. host와 뒤 빈 문단은 이전 쪽에, 표와 인증된 첫 successor는 다음 쪽에 둔다. 측정·흐름과 paint가 같은 인증 원점을 소비한다. |
| px 기대값의 조건 | 본문 소유·순서·저장 사다리 상대 간격과 독립 Print 괘선 ±2px를 함께 검사한다. 고정한 두 쪽은 현재 Native/fresh WASM 모두 98% 이상이다. |
| 반복 저장 | parser 2종에 실제 CLI 편집→저장→재열기→재저장→재열기, HWP/HWPX 교차 경로 62회. 본문·표·머리말·sentinel 보존. HWP3 제어 slot은 기존 #4957 직렬화 계약으로 구분했다. |
| 조판 변경의 필수 편람 | Native 383쪽 `--handbook-gate` 실제 실행·결과를 보호 검증 원장으로 연결한다. 일반 contributor에게 비공개 OTF 보유를 요구하지 않는다. |
| 정본 보호 TSV | 통합 담당자 조건으로 원본 program bytes·순서·환경을 맞춘 18조건/경로의 공식 두 자리 표시값을 대조해 하락 0쪽을 확인했다. |

실제 Studio 초기화 응답의 WASM은 build와 동일하다. JS는 Vite 개발 변환 응답이므로 원 pkg hash와 다르며, 현재 pkg를 정본 Vite로 변환한 코드와 일치함을 따로 검증했다. [초기화·변환 근거](../assets/planet6897_open5_20261113/actual-initialization-qualification.json). CK 기본 screen의 스캔 보간 차이는 report-only RGB 진단에 남겼으며 인영서/직인 보존 주장과 구분했다. 이 PR에서 CK 자원 공급이나 bitmap 보간을 추가 보정하지 않았다.

## Merge 후 contributor PR comment 계획

통합 PR의 정확한 head CI 성공·merge 확인 뒤 원 PR에 통합 PR·merge SHA·이 PR의 주장/검증 결과를 한국어 존댓말로 안내하고 close한다. 관련 #9027도 실제 해결 범위를 확인해 처리한다. 원 contributor fork branch는 삭제하지 않는다.

댓글에는 해당 PR의 위 raw·PDF PNG·review·overlay를 최종 merge SHA 고정 `raw.githubusercontent.com` URL의 **Markdown 이미지**로 표시한다. 다른 PR/보호 문서 그림으로 대체하지 않는다. 데스크톱 한컴2022 Print 기준임을 명시하고 `--body-file` 게시 후 API로 실제 한글·이미지 URL을 재확인한다. 아직 수행하지 않은 원격 게시·CI·병합을 완료로 적지 않는다.
