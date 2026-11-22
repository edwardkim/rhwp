# PR #9280 버전별 한컴 본문 기호 변환

## 최종 판정

**메인터너 보정 후 수용 가능.** 원 기여의 15종 저장 결과는 한컴과 일치한다. 메인테이너가 구형 인접 기호 네 개, 현대 문자 네 개의 재열기·추출, 단독 결합 부호의 표시를 보완했다. 원 PR의 최신 이력을 포함한 로컬 후보가 아래 필수 검증을 통과했다. [판독 근거와 구현](pr_9280_review_impl.md) · [검증 자료](assets/pr9280/README.md)

## 접수와 독립 대조

- Raphael-KR 님의 rhwp 첫 기여다. OPEN 상태에서 reviewer가 없어 `edwardkim`을 지정했다. [원 PR](https://github.com/edwardkim/rhwp/pull/9280)
- 원 head `a0b40b2f937eee79413724928c79c603ce3d2433`의 CI와 CI Impact Policy가 성공했다. 검토 기준 `upstream/devel`은 `8a196ca7aea2aa3cf32d394f5b7da2bdf4520479`이다. 원 커밋 4개를 순서대로 적용한 로컬 후보 `42513825ea046c0a4813d6dd86b25d56d99ce3e6`의 tree는 실제 병합 시뮬레이션 `ba31f73fb59e7b0c72338e7bf038d87f5df3e863`과 같다. [원 head CI](https://github.com/edwardkim/rhwp/actions/runs/44868449596)
- 기존 #5140 본문·겹침 변환은 `f750ef0a1f`·`25c27ec5b1`, #5861 실측 보완은 `d5dada7c14`, #8306 검증 정규화는 `200cf5ecd5`에서 확인했다. 세 이슈는 이미 완료됐다.
- 5.0.1.5와 5.0.1.6 각각 한 쪽의 Windows 한컴 2020 PDF를 따로 인쇄했다. 합성 원본은 rhwp로 만들었으며 버전 차이는 한 바이트다. Windows 2020·2024 및 기여자의 macOS HWPX에서 15문단씩 문자와 굵은 서식이 일치한다. 원본에는 저장 줄이 없어 저장 줄 위치는 한컴의 저장 줄 정보를 사용한 검사와 구분한다. [구형 원본 Print](../../pdf/issue5140-version-boundary/legacy-symbols-v5.0.1.5-2020.pdf) · [현대 원본 Print](../../pdf/issue5140-version-boundary/legacy-symbols-v5.0.1.6-2020.pdf)
- 원 기여 후보로 내보낸 두 HWPX를 Windows 한컴 2020에서 직접 다시 인쇄한 결과는 각 원본 Print와 96 dpi 전체 RGB 픽셀이 같았다. Native에서 후보 HWPX를 다시 열어 얻은 raw는 100.00%·98.33%다. 현대 문자의 기존 표시 차이는 직접 판독했다. [문자·서식 대조](assets/pr9280/symbols.tsv) · [인쇄·저장 작업](assets/pr9280/hancom-jobs.tsv)

## 메인테이너 보완과 재검증

구형 HWP의 네 기호 `U+A805·U+A809·U+A847·U+A849`는 한컴이 저장한 보충 PUA 문자로 변환한다. 현대 HWP와 HWPX에서 `U+A807·U+A80E·U+A810·U+A832`는 본문 추출과 재열기에서도 원래 문자로 남는다. 원 기여자의 15종과 인접 대조군은 한컴 2020·2024 저장본과 본문 및 굵은 글자의 UTF-16 시작 위치를 대조했다. [저장·서식 비교](assets/pr9280/save-xml.tsv)

실로티 나그리 결합 부호 `U+A806`은 라틴 B 뒤에 단독으로 올 때 그리기 단계에서 점선 원을 받는다. 실로티 나그리 기본 글자 뒤에서는 그 글자에 결합하며, 사용자가 입력한 점선 원은 본문에 보존된다. HWP와 HWPX에서 30개 조합씩 총 60개가 독립 한컴 인쇄의 표시 순서와 일치했다. 편집·실행 취소·저장·재열기의 13개 상태에서 본문과 위치를 확인했다. [60개 표시 결과](assets/pr9280/syloti-display.tsv) · [편집 근거](assets/pr9280/syloti-edit.json) · [확대 비교](assets/pr9280/syloti-after.png)

함초롬 원본 글꼴에 없는 인도 숫자 기호는 한컴 인쇄에 쓰인 굴림의 미수록 글리프 윤곽과 전진폭을 확인했다. 짧은 줄에서 뒤의 B가 분리되고, 실제 #5800 현대 문서의 84개 기호가 한 줄에 남는다. 30개 문단의 인쇄 결과 55줄을 HWP·HWPX 양쪽과 비교했다. 일반 공백 U+00A0만 둔 대조군의 후속 C는 한컴보다 2.677px 오른쪽이며 devel과 보완본의 위치가 같다. [짧은 줄](assets/pr9280/indic-short-comparison.png) · [84개 연속 기호](assets/pr9280/indic-long-comparison.png) · [공백 대조](assets/pr9280/space-control.tsv)

원본·저장본·대조군 37쪽씩의 raw 최저는 Native 99.17%, fresh WASM 98.97%다. 원 PR의 HWP와 다시 연 HWPX는 구형 문서에서 100.00%, 현대 문서에서 99.97%로 두 렌더러가 일치했다. 최종 저장본 9개를 Windows 한컴 2020으로 다시 인쇄한 결과 각 원본 PDF와 전쪽 RGB 픽셀이 같았다. Studio에서는 로컬 Chromium의 두 글꼴 환경에서 화면·인쇄용 Canvas2D와 CanvasKit을 80회 실행하고 실제 그리기 결과를 확인했다. [쪽별 TSV와 화면 증거](assets/pr9280/README.md) · [Windows 한컴 재인쇄](assets/pr9280/hancom-reprint.tsv)

## 최신 기여 이력과 devel

검토 중 jangster77 님이 원 PR에 최신 devel `622af208ee`을 병합했다. 최신 원 head `38c3b5171c`의 독립 PDF 등록 21개와 쪽수 표의 모든 행을 검증 후보에서 대조했다. 원 기여 4개와 이 병합을 포함한 로컬 커밋은 `8d5774278a`다. 보완·회귀를 검증한 `3e5a553b87`과 병합 뒤 파일 트리는 `7a468c624b124c5d6c9e9c3f8b940bd957098890`으로 같다. devel의 #9318에서 병합한 표·WMF 대조군은 Native·fresh WASM 각각 167쪽의 모든 PNG가 devel과 바이트 단위로 일치했다. [원 PR의 최신 병합](https://github.com/edwardkim/rhwp/pull/9280/commits/38c3b5171c52a925b2f575e9ad17a61fa620ca28) · [334개 PNG 해시](assets/pr9280/devel-9318-pages.tsv)

원 PR의 최신 head에서 Build & Test와 CI Impact Policy가 성공했다. GitHub에서 이 PR이 자동으로 닫을 이슈는 0건이며, 관련 이슈 #5140·#5800·#5861·#8306은 모두 완료 상태다. [원 head의 Build & Test](https://github.com/edwardkim/rhwp/actions/runs/45027558091/job/137233658558)

## 필수 검증과 제출 경로

| 검사 | 결과 |
| --- | --- |
| 최소 글꼴 환경의 Rust 전체 회귀 | 13,971개 통과, 구성상 제외 45개. 집중 36개 통과 |
| Native Skia 공식 검사 | 라이브러리 4,033개, 그림 대체 표시 2개, 직접 PDF 내보내기 37개로 총 4,072개 통과 |
| Studio | 2,832개 통과, 구성상 제외 2개. 타입 검사·배포 빌드·렌더러 계약 통과 |
| Rust 품질 검사 | 포맷, diff, 기본·WASM·workspace 전체 타깃 Clippy와 workspace 빌드 통과. suite 계약 24개 통과 |
| 문서 검사 | 58개 입력에 세 검사 실행. 표시 대조에 넣은 폭 없는 연결·비연결 문자의 코드와 위치를 확인했다. 공식 보안 회귀 6개 통과 |
| 편람·특별 보호 문서 | 편람 HWPX Native 383쪽 통과. 보호 23종의 Native·fresh WASM 각 2,023쪽, 총 4,046개 쪽별 공식값 모두 일치 |

보호 TSV의 입력·독립 PDF·렌더러·글꼴 공급 해시를 쪽별 결과와 대조했다. 독립 Print에 등록한 38개 PDF 1,449쪽도 다시 읽었고, 기존 방식으로 본문을 읽을 수 있는 1,448쪽이 같은 텍스트를 반환했다. [로컬 검사 수치와 산출물 해시](assets/pr9280/local-validation.json) · [보호값 4,046행](assets/pr9280/protection/pages.tsv) · [문서별 검증 조건](assets/pr9280/protection/conditions.json)

보완 후보는 최신 devel과 리뷰어 미지정 상태인 메인테이너의 Draft #9301 `83fe8497e2` 양쪽에서 충돌 없이 병합된다. 원 fork의 push 권한은 `false`이므로 본 저장소에 수용 PR을 여는 경로다. 원격 제출 승인 뒤 수용 PR의 실제 head에서 필수 CI를 확인한다. [원 PR](https://github.com/edwardkim/rhwp/pull/9280)
