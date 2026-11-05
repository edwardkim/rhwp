# PR #8549 물리 행 조각 보정과 독립 Print 대조

제품 source는 `a314d7a24d3234305abff43149f575853db31105`다. 원본 HWP/HWPX와 기존 한컴 2020 Print PDF를 수정하지 않았다. 실제 빌드한 Native와 wasm-opt 최적화 fresh WASM에서 2026-11-06 KST, Print compat 2022, 96 DPI, PDFium, 색상 경계 보정 전 2px raw를 산출했다. 시각 측정에는 전체 시스템 글꼴을 공급하며 글꼴 파일을 배포하지 않는다. 회귀 실행의 최소 글꼴 환경은 이와 구분한다.

## 원인과 공통 수정

`moved-terminal-tab.hwp`의 마지막 문단은 같은 vpos의 FIRST..LAST 두 조각씩 두 물리 행을 선언한다. bit 31이 있다는 이유로 행 조각 소유를 버리던 공통 판정 때문에 오른쪽 조각이 별도 줄로 쌓였다. 수정 전 Native는 91.36%, 대응 HWPX는 99.92%였다.

공식 PARA_LINE_SEG의 첫/끝 조각·구현 편의 속성을 구분하고, 완결 FIRST..LAST·같은 vpos·양의 폭·비중첩 수평 구간을 공통 조각 소유 근거로 사용한다. `height_measurer::stored_seg_is_row_fragment`를 사용하는 높이 합산과 실제 배치 전진이 같은 결과를 소비한다. 일반 저장 줄의 기존 수용 조건은 유지한다. 문서 ID·임의 좌표·기대값·비교 문턱을 바꾸지 않는다.

## 같은 입력·PDF의 전쪽 측정

| 실제 입력 | 독립 Print | 전체 쪽수 | Native 전쪽 최저·TSV | fresh WASM 전쪽 최저·TSV |
| --- | --- | ---: | --- | --- |
| [rhwp-after.hwp](../../../line-seg-start-outside-control/rhwp-after.hwp) | [PDF](../../../../../../pdf/semanticist21-20261105/pr8549/rhwp-after-hwp-2020.pdf) | 1 | [99.92%](8549-rhwp-after-hwp-2022-native.tsv) | [99.92%](8549-rhwp-after-hwp-2022-wasm.tsv) |
| [rhwp-after.hwp](../../../line-seg-start-outside-control/rhwp-after.hwp) | [PDF](../../../../../../pdf/semanticist21-20261105/pr8549/moved-inline-picture-hwp-2020.pdf) | 1 | [99.92%](api-8549-moved-inline-picture-hwp-native.tsv) | [99.92%](api-8549-moved-inline-picture-hwp-wasm.tsv) |
| [moved-inline-picture.hwpx](../../../../../../tests/fixtures/semanticist21_integration_review/pr8549/moved-inline-picture.hwpx) | [PDF](../../../../../../pdf/semanticist21-20261105/pr8549/moved-inline-picture-hwpx-2020.pdf) | 1 | [99.92%](api-8549-moved-inline-picture-hwpx-native.tsv) | [99.92%](api-8549-moved-inline-picture-hwpx-wasm.tsv) |
| [moved-terminal-tab.hwp](../../../../../../tests/fixtures/semanticist21_integration_review/pr8549/moved-terminal-tab.hwp) | [PDF](../../../../../../pdf/semanticist21-20261105/pr8549/moved-terminal-tab-hwp-2020.pdf) | 1 | [99.92%](api-8549-moved-terminal-tab-hwp-native.tsv) | [99.92%](api-8549-moved-terminal-tab-hwp-wasm.tsv) |
| [moved-terminal-tab.hwpx](../../../../../../tests/fixtures/semanticist21_integration_review/pr8549/moved-terminal-tab.hwpx) | [PDF](../../../../../../pdf/semanticist21-20261105/pr8549/moved-terminal-tab-hwpx-2020.pdf) | 1 | [99.92%](api-8549-moved-terminal-tab-hwpx-native.tsv) | [99.92%](api-8549-moved-terminal-tab-hwpx-wasm.tsv) |

각 입력·두 경로 모두 독립 PDF와 1쪽으로 일치하고 누락·90% 미달은 없다. [입력 SHA·정식 경로](../../validation-r6/measurements.md), [실제 측정 기록](../../validation-r6/measurements.md), [Native producer](../../validation-r6/runtime-checks.md), [fresh WASM producer](../../validation-r6/runtime-checks.md)를 보존한다. 이 다섯 입력의 통과를 다른 통합 대상·보호 문서·전체 회귀의 완료로 확대하지 않는다.

## 주장한 페이지의 직접 판독

- 원본 HWP 1쪽 Native: [review](terminal-tab-hwp-native-review.png) · [overlay](terminal-tab-hwp-native-overlay.png).
- 같은 원본·1쪽 fresh WASM: [review](terminal-tab-hwp-wasm-review.png) · [overlay](terminal-tab-hwp-wasm-overlay.png).

같은 배율에서 본문·그림 앞뒤 텍스트·탭 문단·그림 양옆 마지막 본문의 줄을 대조했다. 원래 세 보이는 줄로 내려가던 마지막 문단이 독립 Print와 같은 두 물리 행이며, 오른쪽 조각과 다음 행의 소유·순서가 맞는다. 그림의 보간 가장자리 차이는 남지만 텍스트 줄바꿈·내용 누락·중복·표시 겹침은 확인되지 않았다. 점수와 직접 판독을 별개로 확인했다.

## 관계형 회귀와 반증

[회귀](../../../../../../tests/cases/physical_row_fragment_flow.rs)는 같은 행의 조각 y 관계·수평 비중첩, 다음 행 전진, 일반 행의 순서, HWP/HWPX 뒤 문단의 내용 보존을 검사한다. 절대 px 좌표나 전체 SVG 해시를 고정하지 않는다. 첫 줄부터 실제 측정 방법·쪽별 수치·정식 근거를 기록했다.

수정 전 `dc39658f812d22c961ddd0fbc8e71c7ec7a52e5b`의 고정 rlib와 수정 후 제품 rlib에 같은 검사 본문을 각각 연결했다. 저장소 허용 글꼴 5개만 노출하는 bwrap 격리에서 [수정 전 2 FAIL](../../validation-r6/runtime-checks.md) / [수정 후 2 PASS](../../validation-r6/runtime-checks.md), exit 101 / 0을 확인했다. [실행 결과](../../historical-validation.md)는 두 실행의 출처를 구분한다. 개인 OTF나 시스템 상용 글꼴 설치를 요구하지 않으며 검사 skip도 없다.

기존 정확한 줄 시작 경계 검사와 정상 행 검사는 유지했다. 기존 35모듈과 각주 관계 검사를 합친 최소 글꼴 집중 171건도 모두 통과했다. 원본·Print의 전체 시각 선행 조건을 충족한 뒤 이 두 관계 검사를 정식 suite 자동 수집 경로에 추가했다. 최종 전체 회귀·보호 문서의 검증은 통합 리뷰에서 별도로 완료해야 한다.
