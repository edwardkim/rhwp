# PR #9435 검토 — 한컴이 저장한 HWPX 글상자의 인쇄 높이

## 최종 판정

**승인.** #9422에서 보고한 KAF 부록 HWPX 36쪽의 글상자는 원 기여로 해결됐다. 최신 `devel` 대비 Native 95.50%와 브라우저 WASM 95.43%가 두 경로 모두 100.00%가 됐다. 원본 54쪽 중 다른 쪽은 바뀌지 않았다. [36쪽 비교](../assets/pr9435/page36-native.png) · [브라우저 비교](../assets/pr9435/page36-wasm.png)

보호 문서 4,046행의 PNG가 기준과 후보에서 모두 같지만, 두 소스 모두 193행의 공식 하한을 재현하지 못했다. HWPX를 HWP로 변환해 한컴으로 인쇄하면 39·52쪽에 기존 그림 잘림도 재현된다. 두 소스에서 재현한 기존 보호값 차이는 작업지시자가 이번 PR 한정으로 수용했고, 변환 인쇄 차이는 별도 이슈에서 추적하기로 했다. [하한 미달 193행](../assets/pr9435/protected-below.tsv) · [변환본의 그림 잘림](../assets/pr9435/exported-hwp-p39-p52.png)

제품 수정은 기여자의 원 커밋을 수용한다. 최신 head의 필수 검사와 병합 가능 상태를 다시 확인했다. 원 PR에 승인 리뷰를 게시하고 2026-11-27 10:51:56 KST에 일반 merge commit으로 병합했다. [승인 리뷰](https://github.com/edwardkim/rhwp/pull/9435#pullrequestreview-6082447830) · [병합 커밋](https://github.com/edwardkim/rhwp/commit/c6fcb1a61211d3d6eb8698660074849431b63e34)

## 접수와 소스

| 항목 | 확인 결과 |
| --- | --- |
| 원 PR·이슈 | [#9435](https://github.com/edwardkim/rhwp/pull/9435) · [#9422](https://github.com/edwardkim/rhwp/issues/9422) |
| 기여자·리뷰어 | `planet6897` · `edwardkim` |
| 원 기여 | `task/9422-hwpx-saved-affine`, `18eb0184f1d2de005fd5bf25890f4e2bd0ccd733`, 대상 `devel` |
| 범위 | 13개 파일, 465줄 추가·1줄 삭제. 제품 코드 1개 파일, 회귀 검사 1개 파일, 나머지는 비교 자료다. |
| 통합 대조 | PR 분기점 `ba4f9b8d50` 이후 #9306·#9436·#9437을 반영한 `fef912a6fb`과 실제 병합했다. 제품 트리는 `f93a1878ab9890a09626f588f57555c5f933732d`이다. |

## 변경과 선행 이력

원 HWPX에서 저장된 변환 행렬은 파서와 렌더러까지 전달됐다. #9422의 초기 추정과 달리 높이가 거절된 직접 원인은 `current_width_was_zero` 검사였다. HWPX 파서는 현재 폭 0을 원래 폭으로 복원하고 재저장용 표지를 남긴다. 글상자 높이 계산은 원래·현재·선언 폭과 저장 글줄을 이미 비교한다. 원 PR은 이중으로 확인하던 폭 0 표지를 높이의 거절 조건에서 제거한다. 저장 높이 0, 크기 보호, 회전 및 기울임 조건과 줄 점유·세로 배율의 일치는 계속 검사한다. 이 결과는 외부 글상자의 사각형과 인쇄 클립에 사용된다. [선행 HWP 보정 #9415](pr_9415_review.md) · [KAF 부록과 두 인쇄본의 차이](../assets/planet6897_20261125/kaf-print-repair/README.md)

입력 `curSz.width=0`을 파서가 원래 폭으로 복원한 뒤, `saved_line_box_height`가 글줄·여백이 차지한 높이와 저장 세로 배율을 대조한다. 도형 배치가 이 높이를 사각형의 높이에 반영하고 본문 직배치 경로도 같은 결과를 사용한다. 크기 보호가 켜졌거나 저장 높이가 0이면 이 경로에서 높이를 늘리지 않는다. 추가된 문서별 상수는 없다. [높이 수용 조건](../../../src/renderer/layout/shape_layout/overlay_textbox.rs) · [도형 배치의 두 소비 지점](../../../src/renderer/layout/shape_layout.rs)

## 독립 한컴 대조

같은 한컴오피스 2020 빌드 11.0.0.9172, 96 DPI에서 36쪽을 재인쇄했다. 현재 폭만 47,685로 명시한 HWPX와 rhwp의 HWPX 재저장본은 원래 인쇄와 36쪽 픽셀이 모두 같다. 크기 보호만 켠 대조본은 글상자 아래 테두리가 올라가 셋째 줄 일부가 잘린다. 보호 여부를 바꾼 대조본은 입력 XML 한 속성의 범위와 인쇄 결과를 확인했다.

## 소스와 CI

2026-11-27 09:19 KST에 메인테이너 리뷰어 지정을 확인했다. 원 커밋과 최신 devel을 병합한 로컬 검증 커밋은 `d19864f66c2e613f18659f4ce2c5cb03d348693d`이다. 원본 이력과 로컬 병합 양쪽의 `git diff --check`가 통과했다. 원 head의 GitHub 검사 31개와 별도 상태인 CI Impact Policy가 성공했다. 조건에 따라 건너뛴 검사 6개와 CodeQL 요약 neutral 1개도 확인했다. 기여자가 PR 본문에 붙인 수정 전후 Native와 WASM 비교 그림 네 장은 공개 URL에서 받은 PNG와 해당 커밋의 파일이 바이트 단위로 같았다. [Build & Test](https://github.com/edwardkim/rhwp/actions/runs/45862945942/job/140073438997) · [Native Skia](https://github.com/edwardkim/rhwp/actions/runs/45862945942/job/140058653283) · [CI 정책](https://github.com/edwardkim/rhwp/actions/runs/45867729519)

새 `devel`에서 병합된 #9306의 중첩 표 편집·저장, #9436의 글상자 편집, 기존 #9379·#9415와 #9419 및 이번 #9422를 최소 글꼴 환경에서 검증했다. 다섯 회귀 실행 파일에서 집중 검사 25건이 통과했고, 같은 실행 파일의 나머지 1,832건은 필터 범위 밖이었다. `cargo fmt --all -- --check`, Rust 회귀 목록과 단위 검사 등급의 일관성 검사도 통과했다. 전체 검사는 제품 보정이 없는 외부 PR에 관한 로컬 검증 4.3.9에 따라 원 커밋 CI를 사용했다. [로컬 검증 절차](../../manual/pr_review/local_validation.md) · [각 소스·빌드·PDFium의 SHA-256](../assets/pr9435/builds.tsv)

원 HWP·HWPX, 한컴 인쇄본과 추가 입력은 파일별 출처, 크기, SHA-256, Git blob으로 기록했다. KAF 검증에는 글꼴 19개를 같은 순서로 두 소스에 공급했다. [입력과 독립 인쇄본 목록](../assets/pr9435/inputs.tsv) · [KAF 글꼴과 공급 순서](../assets/pr9435/kaf-fonts.tsv)

## 36쪽과 인접 쪽 검증

최신 devel 대비 후보의 54쪽 Native HWPX 출력 중 RGB 픽셀이 바뀐 쪽은 36쪽 하나다. 원 HWP는 54쪽의 PNG가 바이트 단위로 같다. 36쪽은 작성자 인쇄(11.0.0.1623)에서 95.96% → 100.00%, 두 번째 인쇄(11.0.0.9172)에서 95.50% → 100.00%이다. 한컴 인쇄 경계의 상단은 131.089pt이다. 아래 경계는 원본과 폭을 명시한 파일에서 218.522pt, 크기 보호만 켜면 209.647pt이다. 세 줄의 텍스트와 아래 테두리를 확대 판독했다. [Native 확대 비교](../assets/pr9435/page36-native.png) · [대조본의 XML 변경 및 독립 인쇄 해시](../assets/pr9435/hancom-controls.json)

새 Rust 검사는 첫째 줄과 셋째 줄의 글상자 소속 및 셋째 줄 아래 여백을 단언한다. 가운데 줄의 소속은 Native 렌더 트리와 독립 인쇄에서 별도로 대조했다.

## 브라우저 인쇄와 저장 대조

후보와 기준에서 잠금 도구로 WASM을 각각 새로 빌드하고 `wasm-opt` 최적화를 마친 뒤 Chrome Canvas에서 출력했다. 96 DPI, 호환 조판 2022, 색상 경계 보정 전 2px 실루엣 기준이다. [두 인쇄본에 대한 쪽별 측정과 RGB 해시](../assets/pr9435/kaf-pages.tsv) · [WASM 36쪽 확대](../assets/pr9435/page36-wasm.png)

| 한컴 2020 인쇄본 | 36쪽 Native | 36쪽 브라우저 WASM |
| --- | ---: | ---: |
| 11.0.0.1623 | 95.96% → 100.00% | 95.92% → 100.00% |
| 11.0.0.9172 | 95.50% → 100.00% | 95.43% → 100.00% |

HWPX는 Native와 WASM 모두 54쪽 중 36쪽만 바뀌었다. 원 HWP의 54쪽은 각 렌더러에서 기준과 후보의 RGB가 모두 같다. 11.0.0.9172 인쇄본에서 31쪽은 양쪽 소스 모두 Native 95.72%, WASM 95.70%다. 이 HWPX 인쇄본에서 직접 확인한 `HCI Poppy` 객체 31의 하위 객체 347은 Type1을 선언하고 기술자 343은 `INPILL+Gulim`을 가리킨다. MuPDF는 이 쪽을 그릴 때 `unknown cid font type`을 기록했다. 이전 독립 인쇄본과 같은 후보 출력을 대조한 31쪽은 Native 98.86%, WASM 98.71%다. 이전 인쇄본을 기준으로 하면 HWPX 전 54쪽의 최저는 Native와 WASM 모두 98.23%다. 11.0.0.9172 기준으로 98% 미만인 쪽은 두 형식 모두 31쪽 하나이며 모든 원본 쪽이 90% 이상이다. [31쪽의 두 PDF 글꼴 참조](../assets/pr9435/page31-reference-fonts.json)

현재 폭만 명시한 대조본은 양쪽 렌더러 모두 36쪽 100.00%를 유지했다. 크기를 보호한 대조본은 한컴과 같이 선언 높이에서 잘렸고, 보정 전후 Native 99.96496%, WASM 99.96395%로 같았다. 36쪽에서 HWPX 재저장은 원 이슈와 같은 개선을 보였고, HWP 변환본은 두 렌더러에서 100.00%였다. 기준과 후보가 저장한 HWP·HWPX의 파일 전체 SHA-256도 형식별로 일치했다. [대조군 측정](../assets/pr9435/page36-controls.tsv) · [재열기 검사와 파일 해시](../assets/pr9435/roundtrip.json) · [검증 입력](../../../tests/fixtures/pr9435-saved-frame) · [독립 인쇄 PDF](../../../pdf/issue9422)

기여자 공통 보호 검사의 공개 문서 4종을 같은 후보의 Native와 브라우저 WASM으로 검증했다. 8개 조건 모두 통과했다. [공통 검사 결과](../assets/pr9435/contributor-gate.tsv)

추가 대조본 6개를 렌더러별로 54쪽씩 전부 검사했다. 648개 비교 중 픽셀이 바뀐 것은 HWPX 재저장본 36쪽의 Native와 WASM 두 건이다. 한컴이 직접 재저장한 기존 #9415 HWP는 일반·크기 보호본 모두 두 렌더러에서 54쪽 전체가 같았다. 크기 보호본 36쪽은 Native 99.97%, WASM 99.96%로 한컴의 잘림을 보존했다. [추가 대조본 전쪽의 점수와 해시](../assets/pr9435/control-pages.tsv)

## HWP 변환본에서 확인한 기존 그림 잘림

원 HWPX를 rhwp로 HWP로 변환하면 독립 한컴 인쇄의 39쪽과 52쪽에서 도식 아래가 잘린다. 기준과 후보가 내보낸 HWP의 SHA-256은 모두 `02d1052b656c1ba8c3dc4e3b148ae2060a3d3f7c5414c0344cb5b91dd24bb3e8`이다. 이 HWP를 두 소스의 rhwp로 각각 그리면 렌더러별 54쪽의 PNG 파일이 바이트 단위로 같다. 그 출력을 해당 파일의 한컴 인쇄와 대조한 수치는 39쪽 Native 39.29%·WASM 39.28%, 52쪽 75.86%·75.87%다. [같은 입력의 인쇄와 다시 열기 비교](../assets/pr9435/exported-hwp-p39-p52.png)

두 쪽의 임베디드 이미지 픽셀은 원 HWPX 인쇄와 변환 HWP 인쇄에서 각각 같다. 한컴 PDF가 이미지를 그리는 세로 길이는 39쪽에서 약 152pt에서 342pt로, 52쪽에서는 약 199pt에서 226pt로 바뀌며 하단이 잘린다. rhwp의 파일 재열기 비교는 차이 0건을 보고해 이 인쇄 차이를 발견하지 못한다. [인쇄본별 그림 원점·변환 행렬·픽셀 해시](../assets/pr9435/exported-hwp-figures.json)

기존 #9379는 같은 도식이 있는 원본 HWP의 읽기·렌더링을 다뤘으며 #9411에서 수용했다. 이번에 확인한 재현 경로는 원 HWPX를 rhwp로 HWP에 저장한 다음 한컴에서 인쇄하는 것이다. #9422에서 보고한 원본 HWPX 36쪽의 글상자와 구분한다. [이슈 #9379의 검증 범위](https://github.com/edwardkim/rhwp/issues/9379)

## 정본 보호 문서의 측정 환경

두 소스에 같은 입력과 한컴 PDF를 사용하고 호환 조판 버전, DPI, Native 및 브라우저 빌드를 실행 영수증과 대조했다. HWP3 sample10의 브라우저 렌더링에는 원본이 참조하는 `oracle.gif`, `rdb02.gif`, `s1.jpg`를 명시적으로 공급했다. 등록 글꼴의 순서와 허용된 SHA-256 쌍을 문서별로 적용했다. 정본 표 SHA-256은 `8141acb297ce2f39148f9014e463986431d266120629e8f368665e5917138f9d`이고, 글꼴 공급 manifest는 `be23d03b6d19d55b62635c4f57bb4466caed308ce05ed16aac09dceaed367915`이다.

형식 사양서 Native는 지정된 14개 프로그램을 동일한 순서로 둔 상태에서 Fontconfig가 Windows 시스템의 대체 글꼴을 찾도록 한 뒤 71쪽 모두 하한을 재현했다. 처음 재현되지 않던 3쪽도 이 결과에 포함된다. [형식 사양서 71쪽의 점수와 파일 해시](../assets/pr9435/spec13-system-supply.tsv)

HWP3 sample10의 미달 대표 4쪽은 정본이 허용하는 다른 굴림 파일을 사용해도 같은 차이가 남았다. 브라우저에서는 GPU 사용을 꺼서 미달이 있던 5개 문서 283쪽을 `devel`에서 다시 그렸다. 전쪽의 측정값과 미달 목록이 기본 브라우저 실행과 같았다. 확대 비교에서 화학물질 문서의 글자 굵기, sample16의 빈 원 크기, 시행령의 글머리표와 표 선 차이가 보인다. 그림과 측정만으로 각 행의 원인을 특정하지 못했다. [문서·렌더러별 최대 차이 확대](../assets/pr9435/protected-local-differences.png)

두 소스 각각에 공급한 등록 글꼴 1,690개 항목과 순서를 총 92개 실행 영수증에서 확인했다. 영월 문서 Native는 웹폰트 변환 캐시 생성이 끝난 뒤 두 소스를 순차 실행해 자동 대체 글꼴 목록과 순서까지 일치시켰다. 이 문서의 4쪽도 모두 하한을 충족했다. [문서별 공급 확인](../assets/pr9435/protected-font-checks.tsv)

| 렌더러 | 대조한 쪽 | 기준·후보 PNG 해시 일치 | 양쪽 모두 정본 하한 미달 | 미달 쪽 최저 | 최대 하한 차이 |
| --- | ---: | ---: | ---: | ---: | ---: |
| Native | 2,023 | 2,023 | 170 | 99.42% | 0.58%p |
| 브라우저 WASM | 2,023 | 2,023 | 23 | 99.71% | 0.20%p |

Native는 HWP3 sample10 112행, sample16의 세 버전 44행, 화학물질 표시 기준 14행이다. WASM은 가상융합산업 진흥법 시행령 19행, sample16의 세 버전 3행, 중첩 표 1행이다. 행정업무운영 편람 HWP·HWPX는 각 렌더러에서 각각 383쪽 전부 하한을 충족했다. [보호 4,046행의 하한·수치·PNG 해시](../assets/pr9435/protected-pages.tsv)

2026-11-27 10:47 KST에 GitHub에서 원 PR의 `18eb0184f1`, 필수 Build & Test 성공, `mergeable=true`를 다시 확인했다. 원격 `devel`은 `fef912a6fb`였고 병합 시뮬레이션 트리는 로컬에서 검증한 제품 트리와 같았다. 10:52:12 KST에 자동화가 #9422를 완료 처리했다. [이슈 자동 종료 기록](https://github.com/edwardkim/rhwp/issues/9422#issuecomment-7207641921)

## Merge 후 contributor PR comment 계획

`planet6897` 님에게 36쪽의 아래 테두리와 셋째 줄 복원을 확인한 결과를 전한다. 원 코드 병합 SHA에 포함된 `mydocs/pr/assets/issue_9422/{native,wasm}/{review_036,overlay_036}.png`를 Native·WASM별로 표시한다. 두 한컴 인쇄 빌드의 개선 수치, HWPX 나머지 53쪽과 원 HWP 54쪽의 픽셀 대조, 집중 회귀 25건과 [Visual Sweep 판독 방법](../../manual/verification/visual_sweep_guide.md#github-merge-comment)을 함께 적는다. 보호값의 193행 예외와 변환 HWP 39·52쪽에서 발견한 별도 후속 이슈를 구분해 연결한다. 그림은 링크한 커밋의 바이트를 게시 전후 확인한다.

## 병합 후 기록

원 코드 병합 커밋 `c6fcb1a61211d3d6eb8698660074849431b63e34`의 부모는 검증 기준 `fef912a6fb`와 원 head `18eb0184f1`다. 병합 트리를 API로 조회해 로컬 검증 트리와 같은 `f93a1878ab`임을 확인했다. 기여자 fork의 브랜치는 보존했다. 새 자료에는 파일로 검증한 HWP·HWPX 네 개와 한컴 인쇄본 네 개가 포함되므로 별도 기록 PR로 보존한다. [병합 후 nextest 시간 자료](https://github.com/edwardkim/rhwp/actions/runs/45925000255)
