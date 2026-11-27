# PR #9298 JPEG 사진 방향 검토

## 최종 판정

원 기여 `a588481833`을 보존한 보완본 `d47b744b78b26389836a451dab248d00c587d899`을 메인테이너가 승인했고, 원 PR을 `b18310bed34e56bad4ec0a9e49e4c615a1f79b64`로 병합했다. 규제영향분석서 8쪽에서 PR 등록 전부터 나타났던 JPEG 회전과 옆 사진의 겹침이 해소됐다. 한컴 2024 인쇄 기준 Native와 fresh WASM의 2px 실루엣 일치율은 각각 98.58%다. 원본의 나머지 36쪽과 보호 문서 4,046쌍은 기준 devel과 PNG가 같다. GitHub 검사 35개가 성공했고 조건부 검사 6개는 건너뛰었다. [원 PR](https://github.com/edwardkim/rhwp/pull/9298) · [커밋별 CI](https://github.com/edwardkim/rhwp/actions/runs/45544007193) · [최종 CI 정책 검사](https://github.com/edwardkim/rhwp/actions/runs/45547402450) · [승인 리뷰](https://github.com/edwardkim/rhwp/pull/9298#pullrequestreview-6051896845) · [병합 커밋](https://github.com/edwardkim/rhwp/commit/b18310bed34e56bad4ec0a9e49e4c615a1f79b64)

작업지시자는 원본의 기존 미달 8쪽과 보호 문서의 기존 WASM 14쪽에 한해 이번 병합을 승인했다. 중첩 표 Native 검증에는 기록된 글꼴 해시 다섯 쌍을 허용했으며 해당 17쪽은 모두 100.00%였다. 전체 조판의 남은 차이는 [기존 이슈 #7778](https://github.com/edwardkim/rhwp/issues/7778)에서 추적한다.

semanticist21 님이 수정한 규제영향분석서 8쪽의 행사 안내판 사진은 가로 1,321·세로 2,137픽셀이며 EXIF 방향 값이 6이다. 한컴 2022·2024는 원래의 세로 사진에 HWP 개체 배치를 적용한다. 11월 23일 검토에서 브라우저 SVG 개선을 확인했지만, 37쪽 문서의 전쪽 기준 미달이 남아 공개 댓글로 병합을 보류했다. 이후 세 건을 통합한 #9394에서는 #9298을 별도로 분류해 기여자의 보완 답변을 기다리기로 했다. 이번 작업에서 메인테이너가 보완을 맡았다. [원 PR](https://github.com/edwardkim/rhwp/pull/9298) · [당시 판단과 전후 사진](https://github.com/edwardkim/rhwp/pull/9298#issuecomment-7089121727) · [#9394의 처리 범위](pr_9394_review.md)

병합 후 실행 시간 갱신 작업이 PR 회귀 B–E의 네 실측을 확인해 메타데이터 커밋 `ce7b57ae52`로 게시했다. [갱신 실행](https://github.com/edwardkim/rhwp/actions/runs/45547612825)

## PR 이전의 증거

| 시점 | 같은 원본에서 확인한 사항 |
| --- | --- |
| 7월 9일 | 커밋 `55ad911b66`에 추가된 원본 HWP는 현재 파일과 SHA-256(`da9f9c986247`)이 같다. 방향 값이 6인 JPEG도 그때부터 들어 있었다. |
| 10월 10일 | #7778이 10월 3일의 검토를 인용해 규제영향분석서 전체 조판을 추적했다. 당시 Native 37쪽 중 8쪽을 포함한 19쪽이 90% 미만이었다. 이때 8·11·12쪽의 그림이 각 쪽에 속하는지 확인하는 검사도 유지했다. |
| 11월 14일 | #9007 수용 당시 Native와 WASM 모두 12·14·15·16·20·25·28·37쪽이 한컴 2022 Print 대비 90% 미만이었다. 37쪽 최저값은 각각 38.34%였다. 같은 날 #9057도 새 기준에서 원본 37쪽을 두 경로로 다시 그려 기존 미달 8쪽을 확인했다. [#9057의 재검증](pr_9057_review.md) |
| 11월 22일 | #9298이 브라우저 JPEG 방향 수정을 제출했다. |
| 11월 24일 | `devel`의 `8a26c00869`에서 같은 파일을 다시 열었다. 한컴 2022·2024 각각을 기준으로 두 렌더러 모두 동일한 8쪽이 미달이었다. 실제 Studio에서 8쪽의 안내판 글씨도 옆으로 누워 있었다. |

[#7778 전체 조판 추적](https://github.com/edwardkim/rhwp/issues/7778) · [#9007의 검토 기록](pr_9007_review.md) · [이전·현재 판정값 74행](../assets/pr9298/source-history.tsv)

기여자는 PR을 열기 42분 전에 당시 `devel` SVG에서 사진이 옆으로 누운 모습과 한컴의 세로 사진을 나란히 커밋했다. 이 비교 그림과 현재 Studio 재현으로 수정 전 증상을 확인했다. 공개 이슈·PR의 제목과 본문에서 등록 전의 `EXIF`와 ‘사진 방향’을 검색했으며 사진 회전을 따로 신고한 더 이른 건은 찾지 못했다. 10월의 #7778에는 8쪽 그림이 어느 페이지에 속하는지 확인하는 회귀 검사가 기록돼 있다. [기여자가 PR 생성 전에 기록한 화면](https://github.com/semanticist21/rhwp/blob/04ff7a15311c20a2611dcce19be7326e9f764037/mydocs/pr/assets/jpeg-exif-orientation/59043-p8-hancom-vs-devel-svg.png) · [7월의 원본과 기준 PDF](https://github.com/edwardkim/rhwp/commit/55ad911b6608f8a6683b0ad4f315372265c56617)

## 메인테이너 보완

원 기여 `a5884818331908d5d6085aac3f9ca3430f27a098`를 최신 기준에 병합했다. 원 기여는 화면 SVG와 WebCanvas의 JPEG를 고치지만, 같은 8쪽에서 Studio가 레이어의 그림 키로 읽는 바이트에는 방향 6이 남았다. 원 PR의 마커 처리기를 그대로 Chromium 156에서 실행하면 보통 JPEG는 1,321×2,137로, APP1 앞에 채움 바이트 한 개가 있는 같은 JPEG는 2,137×1,321로 열렸다. 두 문서를 한컴 2024에서 재인쇄한 1쪽 출력은 96·192 DPI 모두 픽셀이 같았다. [JPEG 채움 바이트 규정 B.1.1.2](https://www.w3.org/Graphics/JPEG/itu-t81.pdf) · [독립 인쇄·입력 해시](../assets/pr9298/print-control.json)

공통 JPEG 마커 탐색을 재사용하고 Studio가 그림 키와 inline 레이어에 싣는 색상 JPEG에도 같은 방향 처리를 적용했다. 렌더링용 사본에서 EXIF의 방향 항목을 1로 바꾼다. 정상 색상 JPEG에서는 압축 영상을 다시 인코딩하지 않는다. 메인테이너 보완 제품 커밋은 `e0c103e2b9`와 `09abb57777`이다. [구현 계획과 두 표시 경로](pr_9298_review_impl.md)

## 원본 8쪽의 두 사진

실제 Studio에서 37쪽 원본을 열면 보완 전 가로로 누웠던 안내판 글씨가 보완 후 바로 선다. 사진 한 장만 남긴 문서는 보완 전에도 바로 서 있어 이 차이를 검출하지 못했다. 원본 안내판 전체 영역(136×289px)의 한컴 인쇄 대비 평균 절대 RGB 차이는 255단계에서 66.608에서 41.158로 줄었다. [같은 사진의 한컴·기존 Studio·보완 Studio 화면](../assets/pr9298/studio-source-comparison.png)

같은 칸에는 사진이 두 장 있다. HWP의 7행 1열 첫 문단에 안내판(`bin_id=2`, 어울림)과 오른쪽 사진(`bin_id=3`, 글자처럼 취급)이 들어 있다. 원본 저장 줄의 시작은 10,119 HWPUNIT, 폭은 11,821 HWPUNIT이다. 한컴 2024 인쇄에서 안내판의 오른쪽 끝은 x=243.5px, 다음 사진의 왼쪽은 x=243.9px이다. 종전과 JPEG 보완 후 Native의 오른쪽 사진은 모두 x=176.6px에서 시작해 안내판 오른쪽 약 67px를 가린다. 96 DPI에서 Native와 새 WASM의 그림 배치를 대조했다. [두 사진 전체 비교](../assets/pr9298/page8-neighbor.png)

한컴 2024에서 원본을 HWPX로 저장한 뒤 해당 표의 한 행과 사진 두 장을 분리했다. 첫 사진의 배치를 ‘어울림’으로 유지한 대조군과 ‘글 앞으로’로 바꾼 대조군을 각각 한컴에서 HWP로 다시 저장하고 PDF로 인쇄했다. ‘어울림’ 저장본에서는 첫 사진의 가로 오프셋 −84에 그림 폭 10,203을 더한 위치 10,119에서 다음 줄 영역이 시작한다. ‘글 앞으로’ 저장본에서는 줄 시작이 0이고 전체 폭은 21,940이다. 한컴은 이 설정에 따라 두 번째 사진을 x=243.87px와 x=176.39px에 놓는다. JPEG 보완본의 Native와 fresh WASM은 HWP·HWPX 양쪽에서 ‘어울림’ 78.88%, ‘글 앞으로’ 99.85%였다. [두 설정의 한컴·Native 화면](../assets/pr9298/cell-wrap-controls.png) · [입력과 PDF 해시, 좌표, 쪽별 측정값](../assets/pr9298/cell-wrap-controls.tsv)

‘글 앞으로’ 대조군에서는 두 그림의 좌표가 한컴과 일치하지만 겹친 부분의 색상이 다르다. 한컴은 안내판을 위에 그리며, Native는 오른쪽 사진을 위에 그린다. 99.85%는 두 그림 외곽의 실루엣 측정값이다. 이 대조군에서 색상과 앞뒤 순서까지 일치한다고 판정할 수 없다.

## 표시·저장 검증

사진 한 장과 JPEG 마커 앞에 채움 바이트를 넣은 대조군은 모두 Native·fresh WASM 99.98%였다. 원본 37쪽은 인쇄용 Native·fresh WASM 모두 `devel`과 보완본의 쪽별 PNG가 같았다. 독립 한컴 2024 PDF 대비 원본 8쪽은 각각 92.93%·92.94%, 37쪽 전체 최저는 38.23%·38.20%였다. 인쇄 비교에서 드러나지 않은 8쪽 화면의 회전은 별도로 실제 Studio 캡처에서 판독했다.

사진 한 장 대조군 세 종류(원래 EXIF 방향 6, 마커 앞 채움 바이트, 이미 정방향인 사진)를 실제 브라우저에서 HWP·HWPX로 두 번씩 저장하고 다시 열었다. 저장본 12개에서 원본 JPEG 압축 바이트와 방향 값이 보존됐고 저장 변환 보고의 손실 항목은 0개였다. Studio 테스트 2,835개 통과·2개 건너뜀, 배포 빌드 성공을 확인했다.

37쪽 원본도 `devel`과 보완본 양쪽의 실제 브라우저 저장 API로 HWP·HWPX 두 세대씩 재검증했다. 여덟 저장본 모두 37쪽으로 다시 열렸으며 원본 JPEG 15개 전부의 SHA-256과 개수가 같았다. 안내판의 EXIF는 저장본에서 6이고, 여덟 번의 저장 보고에서 손실 항목은 0개였다. [원본 37쪽 저장 검증](../assets/pr9298/full-source-roundtrip.tsv)

보호 문서 Native 2,023쪽과 fresh WASM 2,023쪽의 PNG를 최신 `devel`과 제품 수정 `09abb57777`에서 새로 생성해 대조했다. 4,046쌍 모두 SHA-256이 같다. WASM 14쪽은 정본 하한 100.00%에 대해 99.87~99.99%로 양쪽 모두 미달이다. 직전 #9394의 14개 PNG와도 해시가 같다. [전체 근거와 쪽별 수치](../assets/pr9298/README.md)

최소 글꼴 환경에서 전체 회귀 14,096개와 JPEG·기존 색상 처리 집중 검사 8개가 통과했다. 전체 회귀에서 선택된 테스트 중 실패는 0개이며, 실행기의 별도 건너뜀 집계는 45개다.

같은 코드에서 Native Skia 라이브러리 4,033개, 누락 그림 검사 2개, 직접 PDF 내보내기 37개가 통과했다. Native·WASM·workspace Clippy, workspace 빌드, 포맷과 회귀 목록 검사도 성공했다. [검사별 건수와 로그 SHA-256](../assets/pr9298/validation-gates.tsv)

위 검증의 기준 커밋은 `8a26c00869`이다. 검증 종료 조회에서 [#9393](https://github.com/edwardkim/rhwp/pull/9393)이 `devel`의 `0fbee2c04e`로 병합된 것을 확인했다. 이 변경은 표·그림 배치와 페이지별 인쇄 글꼴 공급을 수정했다. 아래에서 새 기준으로 재측정한다.

## #9393 병합 뒤의 원본 재검증

#9298을 적용하기 전의 `devel`을 `0fbee2c04e`에서 다시 빌드했다. 한컴 2022·2024 독립 PDF를 각각 기준으로 원본 37쪽을 새로 그렸다. Native와 fresh WASM 모두 12·14·15·16·20·25·28·37쪽이 90% 아래였다. 2024 인쇄 기준 8쪽은 Native 92.92%, WASM 92.94%였다. [기준본의 원본·대조군 결과](../assets/pr9298/devel-0fbee2c04e/devel-original-and-controls.json)

같은 소스의 브라우저에서 실제 그림 리소스를 디코딩하면 8쪽 안내판은 가로 2,137·세로 1,321픽셀이다. 브라우저의 렌더 트리에서 옆 사진은 x=176.6px였고 안내판과 가로 67.1px 겹쳤다. 한컴에서 다시 저장한 한 쪽 대조군에도 같은 결과가 나왔다. [브라우저가 반환한 사진 크기·두 그림 좌표](../assets/pr9298/devel-0fbee2c04e/devel-browser-geometry.json)

보완본 `13f283cfbf`에서 실제 Chromium의 같은 사진은 가로 1,321·세로 2,137픽셀로 표시됐다. 원본 8쪽과 한컴에서 다시 저장한 두 대조군에 적용됐다. 옆 사진의 x좌표와 67.1px 겹침은 같은 세 문서에서 그대로 재현됐다. 화면용 Native SVG에서도 안내판 JPEG의 방향 항목이 기준본 6, 보완본 1임을 확인했다. [보완본 브라우저 측정](../assets/pr9298/devel-0fbee2c04e/candidate-browser-geometry.json) · [두 SVG에서 추출한 JPEG](../assets/pr9298/devel-0fbee2c04e/screen-svg-p8.json)

첫 빌드에서 공유 Cargo 캐시가 기준본과 같은 바이너리를 돌려줘 그 실행의 결과를 폐기했다. 보완본은 Native와 WASM 각각 별도 빌드 디렉터리에서 컴파일했다. 양쪽 백엔드에서 기준본·보완본의 서로 다른 산출물 SHA-256과 위 실제 동작을 확인했다. [소스·빌드 산출물·로그 해시](../assets/pr9298/devel-0fbee2c04e/builds.json)

최신 기준과 보완본의 원본 37쪽은 Native·fresh WASM에서 두 인쇄 기준 모두 쪽별 PNG가 일치했다. 두 형식으로 두 차례씩 저장한 37쪽 문서 8개의 내장 JPEG 15개도 각각 원본과 같았다. 새로 반영한 #9393의 법률 개정 이유서·동물복지 사육관리 지침 출력 152쌍과 보호 문서 4,046쌍은 모두 SHA-256이 같았다. 기존 WASM 하한 미달은 14쪽이며, 중첩 표 Native 공급 글꼴 중 5개가 지정 해시와 달랐다. [최신 기준의 쪽별 비교·글꼴·저장 근거](../assets/pr9298/devel-0fbee2c04e/README.md)

최소 글꼴 전체 회귀 14,115개, Native Skia 4,227개, Studio 2,835개, JPEG 집중 검사 8개가 통과했다. 포맷·lint·workspace와 Studio 배포 빌드도 통과했다. [검사 원장](../assets/pr9298/devel-0fbee2c04e/validation-gates.tsv)

## JPEG 방향만 보완한 시점의 판정

JPEG 방향은 메인테이너 보완으로 해결했다. 원본 8쪽을 확대하면 오른쪽 사진이 여전히 안내판의 글씨를 가린다. 이 증상은 PR 등록 전 화면과 최신 `devel`에서도 확인되며, 두 장을 분리한 ‘어울림’ 대조군은 78.88%다. 현 후보의 병합은 보류한다. 두 사진의 줄 배치와 겹칠 때의 앞뒤 순서를 추가 보완할지, 문서 전체를 추적하는 #7778에 기존 겹침을 남기고 JPEG 범위만 수용할지가 남은 결정이다. [실제 원본 8쪽 대조](../assets/pr9298/devel-0fbee2c04e/studio-source-comparison.png) · [두 배치 설정의 판독](../assets/pr9298/cell-wrap-controls.png)

## 셀 안 두 사진의 메인테이너 보완 결과

기여자의 JPEG 방향 수정을 보존하고, 일반 표와 이어지는 표에서 둘째 사진을 소유 글줄의 실제 인라인 위치에 놓았다. 셀 안에서 의도적으로 사진을 겹치면 문서에 저장된 앞뒤 순서를 적용한다. 표의 괘선은 해당 그림을 포함한 셀 내용 위에서 이어진다. 소스는 `ad48653ab3`, 정식 회귀는 `0d5795f135`다.

원본 8쪽의 Native·fresh WASM 일치율은 한컴 2022 인쇄에서 99.98%·99.99%, 한컴 2024에서 모두 98.58%가 됐다. 전체 37쪽에서 나머지 36쪽은 각 기준의 devel PNG와 같다. 어울림·겹침 순서는 실제 한컴 저장본으로 검사했고 회귀 3개는 같은 프로그램으로 최신 devel에서 모두 실패, 보완본에서 모두 통과했다. 보호 문서 4,046쌍의 PNG도 devel과 모두 같았다. [실제 Studio 전후 사진](../assets/pr9298/cell-pictures/source-page8-studio.png) · [인쇄·편집·저장 검증 원장](../assets/pr9298/cell-pictures/README.md)

원본의 기존 90% 미달 8쪽, 보호 문서의 기존 WASM 하한 미달 14쪽, 중첩 표 Native의 글꼴 해시 5쌍을 수용 판단 대상으로 기록한다. 37쪽 원본의 나머지 조판은 기존 이슈에서 추적한다. [#7778](https://github.com/edwardkim/rhwp/issues/7778)

최종 검증에서 집중 20개, 전체 회귀 14,118개, Native Skia 4,227개, Studio 2,835개와 필수 lint·배포 빌드가 통과했다. #9393의 152쪽 인쇄 결과도 모두 보존했다. 최신 원격 devel `0fbee2c04e` 및 원 기여 `a588481833`를 포함한다. [검사 원장](../assets/pr9298/cell-pictures/validation-gates.tsv) · [#9393 보존 검증](../assets/pr9298/cell-pictures/incoming-9393.tsv)

## Merge 후 contributor PR comment 계획

semanticist21 님에게 원 기여의 JPEG 방향 수정을 인정하고, 같은 원본의 8쪽에서 옆 사진이 안내판을 가렸던 위치까지 보완한 결과를 전달한다. 비교 대상은 규제영향분석서 본문의 실제 행사 안내판과 오른쪽 룰렛이다. 한컴 2022·2024, Native·fresh WASM의 96 DPI 측정에서 8쪽을 각각 한 쪽씩 검사했고 자동 배치 차이 후보는 각 0/1쪽이었다. 전체 픽셀 일치율은 종이 여백을 포함한다. 내용 픽셀 일치율은 Visual Sweep의 `visual_accuracy_proxy_percent`다.

| 한컴 인쇄 | 출력 | 전체 픽셀 일치율 | 내용 픽셀 일치율 | 2px 실루엣 일치율 |
| --- | --- | ---: | ---: | ---: |
| 2022 | Native | 94.80776% | 86.06684% | 99.98% |
| 2022 | fresh WASM | 94.75646% | 85.94371% | 99.99% |
| 2024 | Native | 94.34125% | 84.81484% | 98.58% |
| 2024 | fresh WASM | 94.27345% | 84.66362% | 98.58% |

실제 Studio 화면에서도 안내판의 ‘100% 증정’ 글씨가 바로 서고 옆 사진의 룰렛이 함께 보인다. 병합 커밋의 비교 그림과 37쪽 전쪽 검증 원장을 공개하고 나머지 조판은 기존 이슈의 추적 범위임을 알린다. [8쪽 비교 그림](https://raw.githubusercontent.com/edwardkim/rhwp/b18310bed34e56bad4ec0a9e49e4c615a1f79b64/mydocs/pr/assets/pr9298/cell-pictures/source-page8-studio.png) · [측정 원장](https://github.com/edwardkim/rhwp/blob/b18310bed34e56bad4ec0a9e49e4c615a1f79b64/mydocs/pr/assets/pr9298/cell-pictures/README.md) · [Visual Sweep 측정 절차](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)
