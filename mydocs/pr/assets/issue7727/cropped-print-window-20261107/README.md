# 잘린 원본 이미지 창의 인쇄 축소 보정

부모 `07e07ed41c`, 고정 base `a7feb849e11642b1438b7e9ee460ff4c8a7c946e`에서 CBTA의 잘린 비트맵 인쇄 경로를 보정했다. Native/fresh WASM 각 전 204쪽을 다시 출력했다. **6쪽은 아직 98% 미만이며 이 문서·전체 브랜치·PR 준비 완료가 아니다.**

## 원본과 원인

- 원본: `samples/issue6764/1613000-202200037-air-traffic-controller-cbta.hwp`, SHA-256 `8ef9de3f35690bf9d7994527f77cb02d4a4fcff447c219a78fbc2855d64be6e7`.
- 독립 한컴 2020 Print: [기준 PDF](../../../../../pdf/issue7727/cbta-original-hancom2020-print-20261106.pdf), 204쪽, SHA-256 `0db1eb55eda0f1a4985eeaf187656c23499616b7dccddcbb7c874c0937e1b2e0`.
- PDF 세대와 원문 보존은 [선행 검증](../verification-20261106/README.md)을 따른다. 원본·PDF·비교 문턱을 바꾸지 않았다.

48쪽 FAA 양식은 큰 JPEG의 일부 창이다. 기존 전체 이미지의 장치 격자 축소와 달리 잘린 창은 backend의 좁은 선형 보간으로 넘어가 가는 글자와 선의 표본을 잃었다. 선행 `6097253468`의 공통 비트맵 격자와 `c97709b597`(#8517)의 최종 WMF 재표본화 제외를 확인하고 같은 경계를 유지했다.

원본의 소수 창 경계와 각 장치 픽셀에 겹치는 면적을 두 축에서 함께 소비한다. Native Skia와 WASM Canvas2D가 같은 준비된 PNG·사각형을 받고 원본 crop을 다시 적용하지 않는다. 캐시 신원에도 창 경계를 포함한다. 원본 좌표·줄 구성·이미지 bbox를 조판에서 고치지 않으며 문서/쪽 분기가 없다. 화면·회전/반전·투명 이미지·명시적 nearest·기존 WMF/JPEG 전달/재구성 경로는 기존 소유를 유지한다.

비교에는 독립 PDF에 실제 포함된 Gulim 프로그램과 일치하는 기존 로컬 프로그램을 표시용으로 선언했다. 101개 매핑의 윤곽·전진을 대조한 동일 조건을 부모와 후보에 적용했고 원본 HFT 메트릭은 유지했다. 상용 글꼴은 복사·커밋하지 않았으며, 이 시각 결과를 최소 글꼴 환경 수용으로 보고하지 않는다.

## 실제 결과

정본 PDFium 96DPI·compat2022·2px 색상 경계 보정 전 raw·capture 총 8 조건이다.

| 범위 | 이전 Native → 보정 | 이전 fresh WASM → 보정 |
| --- | ---: | ---: |
| 31쪽 | 97.80726 → 98.71775% | 97.80753 → 98.71785% |
| 48쪽 | 92.33573 → 99.92121% | 92.30842 → 99.92877% |
| 51쪽 | 97.54584 → 99.02125% | 97.54507 → 99.02681% |
| 전 204쪽 최저 | 92.33573 → 96.23471% | 92.30842 → 96.23239% |

Native 7쪽·WASM 10쪽의 PNG만 바뀌고 나머지 각각 197·194쪽은 부모와 바이트 동일하다. 부모 대비 0.1%p 초과 하락·90% 미만·누락은 없다. **96·118·162·166·176·182쪽의 기존 제목 줄바꿈·간격 등 차이는 남는다.** 98% 조건을 평균으로 상쇄하지 않는다.

[이전 Native TSV](before-native-silhouette.tsv) · [이전 WASM TSV](before-wasm-silhouette.tsv) · [보정 Native TSV](after-native-silhouette.tsv) · [보정 WASM TSV](after-wasm-silhouette.tsv)

## 회귀와 게이트

[정식 관계형 검사 4개](../../../../../tests/cases/issue_7727_cropped_print_window.rs)는 원본 면적의 독립 평균·평행 이동 불변식·동일 이미지의 창별 캐시 분리·잘못된 창 거절을 검사한다. 실제 공통 leaf를 포함하며 복제 구현이나 source 문자열 검색으로 검사하지 않는다. 자동 suite 경로와 macOS 최소 글꼴 sandbox에서 각각 4 PASS를 확인했다.

처음의 source-side 신규 cfg(test) 4개 배치는 정책 검사를 통과하지 못해 정식 integration으로 옮겼다. 상대 모듈 경로가 별도 target을 만들던 배치도 기존 절대 include 방식으로 보정했다. 정책 한도·기존 검사를 바꾸지 않았으며 48/48 target, source-side 4,099 tests/296 modules가 유지된다. 배치 변경 전후 평균 연산 본문이 같고, 새 빌드로 재출력한 Native/WASM 전 204쪽 PNG도 모두 같음을 확인했다.

fmt·Native/WASM/all-target/Skia Clippy·workspace build·manifest·unit tier가 통과했다. Native Skia 라이브러리 4,039 PASS/12 ignored, PNG 2 PASS, 직접 PDF 22 PASS다. threads 8·Cargo 순차를 사용했다. 최종 전체 nextest 통과를 뜻하지 않는다. Mac fresh WASM은 locked wrapper `--no-opt` 로컬 대체 빌드이며 Docker 최적화 결과가 아니다. root pkg/Studio public의 JS/WASM hash는 같지만 실제 Studio 초기화 응답·지원 CanvasKit의 최종 확인은 남아 있다.

빌드 후 최종 source guard도 모두 동일했다. 명령·해시·결과 (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue7727/cropped-print-window-20261107/validation.json`)를 보존하며 로그·임시 도구는 ignored output에만 둔다.

## 대표 48쪽

- 이전 Native: [review](before-native-p048-review.png) · [overlay](before-native-p048-overlay.png) · [raw](before-native-p048-raw.png)
- 보정 Native: [review](after-native-p048-review.png) · [overlay](after-native-p048-overlay.png) · [raw](after-native-p048-raw.png)
- 이전 fresh WASM: [review](before-wasm-p048-review.png) · [overlay](before-wasm-p048-overlay.png) · [raw](before-wasm-p048-raw.png)
- 보정 fresh WASM: [review](after-wasm-p048-review.png) · [overlay](after-wasm-p048-overlay.png) · [raw](after-wasm-p048-raw.png)
- 독립 한컴 Print: [raw](hancom-print-p048-raw.png)

보정 Native/WASM의 review와 standalone overlay를 직접 판독해 양식 안의 가는 글자·체크 상자·괘선 보존을 확인했다. 이미지 바깥 본문과 표의 소속·순서는 유지된다. 양식의 일부 괘선에 작은 장치 격자 차이가 남아 있어 raw 100%로 보고하지 않는다.
