# PR #9444 리뷰 — 삽입 수식의 저장 크기와 기준선

## 최종 판정

**승인.** 한컴 2020·2024 인쇄에서 #8927의 수식과 뒤 본문이 분리됐고 중첩 분수는
글줄에 맞게 배치됐다. 정본 보호 4,046행의 최신 devel·병합 후보 PNG 및 측정값이
모두 같다. 사용자가 이번 PR에 한해 허용한 기존 차이의 범위는 아래에 기록했다.
원 커밋의 필수 CI와 병합 가능 여부를 재확인한 뒤 승인 리뷰를 게시하고 일반 병합했다.
실제 병합 트리는 검증 후보 트리와 같다. [승인 리뷰](https://github.com/edwardkim/rhwp/pull/9444#pullrequestreview-6087323040) · [병합 커밋](https://github.com/edwardkim/rhwp/commit/a7cb4208849f370d80f0f4d7dc7e0fbf2d4fb1cf)

## 접수 정보

| 항목 | 확인값 |
| --- | --- |
| 기여 PR·작성자 | [#9444](https://github.com/edwardkim/rhwp/pull/9444) · `johndoekim` |
| 연결 이슈 | [#8927](https://github.com/edwardkim/rhwp/issues/8927) · 원 PR은 `Refs`로 연결 |
| 대상 · 규모 | `devel` · 71개 파일 · 마지막 접수 상태 Ready, MERGEABLE, CI 성공 |
| 제출 커밋 | `584b2258d44cbcb119ebf0eb80c8a797712c9ab2` |
| 리뷰어 | 접수 시 `edwardkim` 지정 후 GitHub 리뷰 요청 목록에서 확인 |
| 최신 비교 기준 | `3e97980785dcee996646ed399f46e65fbaaee261` |
| 로컬 병합 검증 | `d5c4c39006d4ea00c110eba7e99a5dc4556c981f` · 병합 트리 `555e5ae6b97a38d468265f35645f78f037fb023f` |

제출 커밋에서 Build & Test·Native Skia·Frontend 패키지·Render Diff·CodeQL·Adapter·Proptest와 최종 정책 검사가 통과했다. [원 커밋 CI](https://github.com/edwardkim/rhwp/actions/runs/45943222525) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/45943222504) · [CI 정책](https://github.com/edwardkim/rhwp/actions/runs/45946464085)

## 변경과 조판 규칙

수식 편집과 삽입은 원본 수식 글꼴의 글줄 프레임과 한컴 조판 근거가 확인된 크기·구조에서 공통 배치 결과의 폭·높이·기준선을 저장한다. 수식 속성의 명시 축은 요청값을 사용한다. 삽입 문단을 재조판한 뒤 기존 문단 세로 위치 갱신 경로를 사용한다. [HWP5 수식 구조](../../tech/한글문서파일형식_5.0_revision1.3.md#표-105-수식-개체-속성)

| 확인 경로 | 비교 근거 |
| --- | --- |
| `produced_frame`의 측정 → `insert_equation_native`·`apply_equation_properties`의 저장 → 저장본 재파싱·실제 인쇄 | 이슈의 독립 한컴 재조판본 두 개의 저장 프레임과 새 한컴 2020·2024 Print |
| 큰 합·곱과 바닥·천장 괄호 | 루트 프레임과 표시 위치가 같은 공통 배치 결과를 사용하는지, 한계값과 인접 글자 위치 |
| 새 세로 줄 위치 → 뒤 문단 시작 | 삽입 전후 줄 소유와 본문 순서, 뒤 문단의 저장 위치와 첫 페이지 |
| 최신 `devel`의 HML 수식 | #9448에 포함된 원래 암시적 프레임·수식 소속·편집 및 재열기, 독립 Print. [앞선 검토](pr_9448_review.md) |

## 독립 인쇄와 반복 저장

`base-a4.hwp`에서 7개 수식을 CLI로 삽입했다. #9448을 반영한 기준의 결과 HWP는 이전 기준에서 인쇄한 HWP와 바이트 단위로 일치했다. 같은 한컴 2020에서 직접 인쇄해 보면 수정 전에는 닫는 바닥 괄호와 수식 뒤 글자가 겹치며 분수가 글줄에서 뜬다. 두 수식 글꼴의 제출 표본을 한컴 2020 11.0.0.9172와 한컴 2024 13.0.0.4225에서 A4·1-up 인쇄해 대조했다.

제출 커밋의 CI Archive C에 포함된 CLI로 수식 7개를 직접 삽입하고 두 번 저장했다. HWP 레코드와 HWPX의 XML에서 폭·높이·기준선 보존을 확인했다. 예를 들어 `1 over {2 over 3}`은 `1525×3525` HWPUNIT, 기준선 `42`다.

한컴 실측 CSV의 14pt `sum _{i = 1} ^{n} i = {{n ( n + 1 )} over {2}}`는 HancomEQN에서 `9526×3863`, 기준선 `63`이다. 최신 기준은 `10378×4200/85`, 제출 코드는 `9369×3863/63`을 저장했다. 한컴 2020 Print에서 기준선 개선과 함께 뒤 글자가 당겨졌고, 이 입력에서 분수 괘선과 뒤 글자 사이의 분리를 확인했다.

수식 원본 HWP 2개는 원 PR 커밋의 표본과 입력 SHA-256이 같다. 새 기준 PDF 4개는
[`pdf/issue-8927/`](../../../pdf/issue-8927/)에 보존했다. [입력·PDF 해시와 인쇄 영수증](../assets/pr9444/print/receipts.tsv) · [한컴 2020 수정 전·후 확대](../assets/pr9444/print/print-before-after.png) · [14pt 합·분수 확대](../assets/pr9444/print/sum-14pt-print-zoom.png)

병합 후보 CLI의 7개 수식이 든 HWP와 두 번 저장한 HWP가 제출 HYhwpEQ 표본과
바이트 단위로 같았다. CLI에서 HancomEQN으로 바꾼 두 번째 HWP도 원 PR 표본과
같다. 두 글꼴 모두 두 번 저장한 HWP 레코드와 HWPX XML에서 7개 프레임이 유지됐다.
[크기·기준선 비교와 두 형식 왕복](../assets/pr9444/checks/native-frame-roundtrip.tsv)
14pt 합·분수를 재현한 병합 후보 HWP도 기존에 한컴 2020으로 인쇄한 원 커밋의 HWP와 같다.

## 로컬 검사와 최신 devel의 기능

Native Skia는 `release-test`로 빌드했고, WASM은 저장소의
`scripts/wasm-pack-locked.sh --target web`로 바인딩과 `wasm-opt`까지 실행했다.
Chrome 156으로 로컬 WASM을 실행했다.
[두 소스의 Native·WASM 해시](../assets/pr9444/checks/build-digests.tsv)

- 공급 글꼴을 공개 테스트 글꼴 5종으로 제한해 삽입 수식 8건, 수식 API 5건,
  #9448의 HML 수식 5건이 통과했다. 마지막 5건에는 HML 수식의 글줄 배치, 편집 뒤
  저장 프레임, 수식의 클릭 소유 검사가 포함된다.
- 최신 devel과 병합 후보가 그린 HML 표본은 Native에서 한컴 2024 인쇄 대비
  각각 100.00%였으며 PNG 해시가 같다. 브라우저 WASM도 각각 100.00%였으며 같은 PNG를 그렸다.
- 로컬 병합에서 소스 충돌이 없고 원 head의 Full CI가 성공해 Rust 전체 회귀·Native 광범위
  회귀는 원 head CI 결과를 사용했다. 정식 로컬 검토는 집중 검사·브라우저·시각·저장 검증을
  사용한다. [중복 실행 생략 조건](../../manual/pr_review/local_validation.md#430-pr-검토의-github-full-ci-재사용)

Chrome의 최적화 WASM에서 같은 삽입·글꼴 변경 명령을 사용했다. HYhwpEQ는 7개 전체,
HancomEQN은 5개의 폭·높이·기준선이 한컴 2020 재조판본과 같다. HancomEQN의 나머지
두 수식은 폭이 75·225 HWPUNIT 더 넓고 높이·기준선은 같다. 양쪽 글꼴의 각 7개는
HWP와 HWPX를 각각 두 번 저장해도 세 값이 유지됐다.
[브라우저 왕복 기록](../assets/pr9444/checks/candidate-browser-roundtrip.json)

## Native와 브라우저 전쪽 대조

목표 2개 HWP의 1쪽을 각각 한컴 2020·2024 PDF에 비교했다. 인쇄 96 DPI,
`compat=2022`, 색상 경계 보정 전 2px 실루엣을 사용했다. 본문은 KoPubWorld바탕체 Light와
한컴바탕, 수식은 HYhwpEQ·HancomEQN을 같은 순서로 공급했다.

| 기준 인쇄 | 최신 devel Native | 병합 후보 Native | 최신 devel WASM | 병합 후보 WASM |
| --- | ---: | ---: | ---: | ---: |
| HYhwpEQ · 2020 | 99.13% | 99.95% | 99.26% | 99.98% |
| HYhwpEQ · 2024 | 99.13% | 99.95% | 99.26% | 99.98% |
| HancomEQN · 2020 | 99.08% | 99.91% | 99.21% | 99.97% |
| HancomEQN · 2024 | 99.11% | 99.91% | 99.23% | 99.97% |

한컴 2020의 확대 그림에서 중첩 분수가 본문 글줄에 놓이고, 분수 괘선·바닥 괄호와
뒤 한글 글자가 떨어져 있는 것을 확인했다. 곱의 아래 한계값과 연산자도 분리돼 있다.
글리프 윤곽과 일부 잔간격 차이가 남았다.
[Native HYhwpEQ 비교](../assets/pr9444/print/HYhwpEQ-native-review.png) · [Native HancomEQN 비교](../assets/pr9444/print/HancomEQN-native-review.png) · [브라우저 수식 확대](../assets/pr9444/print/wasm-equations-crop.png)

한컴 2020 Print의 1쪽 상세 비교에서 네 출력 모두 표시 필요 플래그가 0쪽이었다.
아래 수치는 자동 픽셀 비교의 보조값이며, 위 실루엣 수치와 확대 판독을 함께 사용했다.

| 출력 | 글꼴 | Pixel match | Visual accuracy proxy |
| --- | --- | ---: | ---: |
| Native | HYhwpEQ | 99.02342% | 42.76692% |
| Native | HancomEQN | 99.02836% | 43.01514% |
| WASM | HYhwpEQ | 99.14802% | 49.66510% |
| WASM | HancomEQN | 99.14870% | 49.71823% |

## 정본 보호

정본은 최신 devel의 `mydocs/pr/assets/visual-protected-baseline/final.tsv`이며,
SHA-256은 `8141acb297ce2f39148f9014e463986431d266120629e8f368665e5917138f9d`다.
23개 문서의 Native·WASM 각 2,023쪽에 대해 같은 원본·Print·96 DPI·조판 버전·
글꼴 프로그램 순서 및 렌더링 조건을 확인했다. Native 자동 웹폰트 공급까지 포함한
실제 실행 기록 92개와 글꼴 공급 슬롯 1,735개를 대조했다. 문서별 지정 해시와 기존
허용 해시를 적용했고 `_MAC.otf`는 공급 목록에서 제외했다.
[sample10의 연결 그림 SHA-256](../assets/pr9444/checks/sample10-linked-images.tsv) · [공급한 파일 순서와 해시](../assets/pr9444/checks/protected-font-supply.tsv)

| 출력 | 비교 쪽수 | PNG·점수 동일 | 공식 하한보다 낮은 기존 행 | 새 하락·누락 |
| --- | ---: | ---: | ---: | ---: |
| Native | 2,023 | 2,023 | 180 | 0 |
| 브라우저 WASM | 2,023 | 2,023 | 23 | 0 |

기존 하한 미달 203행에 이번 PR 한정 수용을 적용했다. Native에는 sample10 112행,
sample16 3종 44행, 화학물질 표시 기준 14행, 개요 번호 5행, 형식 사양서 3행,
종합부동산세법 2행이 있다. WASM에는 시행령 19행, sample16 3종과 중첩 표에서
각각 1행이 있다. 이 전쪽의 PNG 및 표시값은 최신 devel과 같다.
[Native 2,023쪽 대조](../assets/pr9444/checks/protected-native.tsv) · [WASM 2,023쪽 대조](../assets/pr9444/checks/protected-wasm.tsv) · [기존 미달 Native 180행](../assets/pr9444/checks/native-known-protected-below.tsv) · [기존 미달 WASM 23행](../assets/pr9444/checks/wasm-known-protected-below.tsv)

sample16 3종 및 개요 번호 201쪽은 #9448 이전 devel에서도 Native PNG가 같았다.
[이전·최신 devel의 201쪽 대조](../assets/pr9444/checks/protected-native-base-vs-current-base.tsv)

## 기존 수식 문서 대조

원 PR이 제시한 수식 문서 7개를 최신 devel과 후보에서 다시 출력했다. 기존 Print로
대조한 Native 110쪽 중 90% 미만인 27쪽은 양쪽에서 같다. 두 점검용 시험지의
기존 PDF는 A4, 원본은 771×1,116pt여서 용지 크기가 맞는 기존 한컴 2020 PDF로
추가 확인했다. 용지 크기를 맞춘 Native 결과는 수학 시험지 20쪽 최저 89.73%,
과학 시험지 4쪽 최저 94.12%로 최신 devel과 병합 후보가 같았다.
브라우저의 최저도 두 소스에서 각각 89.20%, 94.11%였다.
[정규 용지 크기 24쪽 Native 대조](../assets/pr9444/checks/source-size-controls-native-current-base-vs-candidate.tsv) · [WASM 대조](../assets/pr9444/checks/source-size-controls-wasm-current-base-vs-candidate.tsv)
#505 대조 문서의 Native는 후보와 devel이 4쪽 모두 픽셀까지 같았다.
한컴 Print와 비교하면 1·3·4쪽의 윗줄에서 수식 글리프와 중괄호 윗부분이 잘린다.
이 배치 차이는 이번 삽입 수식 변경 전부터 재현됐다. 올바른 용지로 비교한 수학 시험지의 90% 미달 쪽도 두 소스의 PNG와 점수가 같았다. 일반 수식 대조군의 쪽별 수치 차이는 가장 큰 하락이 0.04%p이고 새 90% 미달은 0쪽이다.
[기존 Print 110쪽 Native 비교](../assets/pr9444/checks/controls-native-current-base-vs-candidate.tsv) · [WASM 비교](../assets/pr9444/checks/controls-wasm-current-base-vs-candidate.tsv) · [#505 확대 대조](../assets/pr9444/print/issue505-reference-detail.png)

## 병합 기록과 후속 처리

원 PR은 2026년 11월 27일 20시 17분(KST)에 `devel`에 병합됐다. 독립 한컴 PDF와
비교 증적은 후속 기록 PR에 보존한다. 기여자 댓글에는 [Visual Sweep](../../manual/verification/visual_sweep_guide.md)과
`https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr9444/print/`
아래 대표 PNG를 연결한다. 게시 뒤 API에서 본문을 재확인한다. #8927은 코드 병합 직후 OPEN이었다.
기록을 반영한 뒤 원 재현 사례의 해결 근거를 게시하고 완료 처리한다.
