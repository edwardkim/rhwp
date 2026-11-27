# hyunmyungJaneLee의 Word 호환 문단·셀 그림·인라인 표 PR 검토

## 판정

세 원 기여의 커밋을 포함한 메인테이너 보완본을 수용할 수 있다. 기준 `devel`은 `cbebdbeeca060efe670cb8702355e4d79cb7e70e`, 제품 코드와 테스트를 검증한 커밋은 `9e601ad6a862a30b019ad9563a1d597f6a33222b`다. [최초 검토와 재현한 경계](hyunmyungjanelee_20261126_review_impl.md)

| 원 PR | 보존한 head | 실제 입력에서 확인한 결과 |
| --- | --- | --- |
| [#9416](pr_9416_review.md) | `ae243b34c1744e68dfbe32ab2866148228b9a53b` | 굴림의 한글·영문 이름, Word 줄 높이와 공백, 문서의 호환 대상·개별 플래그를 확인했다. 같은 Windows 글꼴과 같은 한컴 인쇄에서 두 이름의 결과가 일치했다. |
| [#9418](pr_9418_review.md) | `0553e7137303efa88a5c93722fa835c4de32a4c4` | 분할 칸의 그림 양옆에 자리 잡은 글이 같은 기준선을 사용했다. 두 쪽에 걸친 문서에서 행과 뒤 본문을 보존했다. |
| [#9420](pr_9420_review.md) | `6485652352171c06446d5ce3ba2540675ce46250` | 양수 세로 오프셋으로 놓인 본문 표의 앞뒤 줄을 각각 한 번 배치했다. 실제 글자, 공백, 누름틀로 시작하는 기존 문서를 함께 대조했다. |

원 PR은 번호순으로 이력을 병합했다. 세 원 head를 포함한 소스는 `3f09b6d71d`다. #9414에 대한 CodeQL 검토 의견은 해당 원 PR에 게시했다. [#9414의 기존 댓글](https://github.com/edwardkim/rhwp/pull/9414#issuecomment-7178890959)

## 한컴 인쇄 및 저장

제출 원본 22개를 한컴 2020·2024에서 다시 인쇄해 기준 PDF 25쪽의 픽셀을 확인했다. 보완 경계를 포함한 54개 문서는 Native와 브라우저 각각 60쪽이다. #9416은 45쪽, #9418은 10쪽 모두 100.00%였고 #9420의 5쪽은 Native 최저 99.80%, 브라우저 최저 99.75%였다. 원 기여의 쪽별 측정 50행 가운데 20행은 개선됐고 30행은 같았다. [입력·인쇄본·전쪽 수치](../assets/hyunmyungjanelee-20261126/final/README.md) · [원 head별 대조](../assets/hyunmyungjanelee-20261126/final/contributor-comparison.tsv)

누름틀 줄 다음에 놓인 두 표가 중간 보완본에서 9.7px 겹친 회귀를 최종 후보에서 수정했다. 한컴 2020·2024와 재저장본을 포함한 10개 대조가 Native·브라우저에서 모두 100.00%였다. 앞서 병합된 #9412·#9417의 문서 7개도 14쪽씩 100.00%였고 두 소스의 PNG 28쌍이 같았다. [누름틀과 표의 전쪽 결과](../assets/hyunmyungjanelee-20261126/final/field-layout-pages.tsv) · [기존 PR 기능과 목표 문서](../assets/hyunmyungjanelee-20261126/final/paired.tsv)

7개 호환 설정으로 HWP↔HWPX 저장·재열기 28회를 검사했다. HWP의 비표준 플래그를 한컴이 HWPX로 변환할 때의 줄 배치는 실제 한컴 변환본을 기준으로 삼았고 HWP로 복귀한 문서는 원본 플래그와 대조했다. 한컴에서 다시 인쇄한 저장본 22개·24쪽이 각 기준과 일치했다. Studio 배포 빌드에서 대표 파일 3개를 열어 1·2·1쪽과 사용하는 WASM의 해시를 확인했다. [저장 회차별 구조](../assets/hyunmyungjanelee-20261126/final/cycle-structure.tsv) · [한컴 데스크톱 인쇄](../assets/hyunmyungjanelee-20261126/final/desktop-pixels.tsv) · [Studio 실행 결과](../assets/hyunmyungjanelee-20261126/final/studio-browser.json)

## 보호 문서와 검증 환경

23종의 보호 문서에서 Native 2,023쪽과 최적화한 fresh WASM 2,023쪽을 각 쪽의 공식 두 자리 하한과 대조했다. 하락·누락은 0건이다. 같은 공급 조건에서 기준과 후보의 보호 PNG 4,046쌍이 모두 같았다. [4,046행 결과](../assets/hyunmyungjanelee-20261126/final/protected-pages.tsv) · [기준과 후보의 입력·도구·공급 순서](../assets/hyunmyungjanelee-20261126/final/protected-runs.tsv)

중첩 표의 Native 10쪽에서는 같은 Linux 환경의 기준과 후보가 모두 100.00%였고 PNG도 쪽마다 같았다. 이 쪽의 정본에는 개별 승인한 macOS 공급 조건의 하한을 적용한다. [운영체제별 공급 조건과 공식 조정 근거](../assets/visual-protected-baseline/updates/planet6897-mac-native-20261125.json)

두 문서의 승인된 OTF 33종은 원래 지정한 해시와 함께 소형 및 원본 크기 로컬 파일의 해시를 등록했다. 개요 번호는 9쪽 모두 100.00%, 종합부동산세법은 6쪽 최저 98.86%였다. 소형 파일을 사용한 Native·브라우저의 PNG 30쌍과 원본 크기 파일을 사용한 Native의 PNG 15쌍이 기준과 후보에서 각각 같았다. 브라우저 단일 글꼴 한도는 32 MiB이며 큰 파일 6개가 이를 초과한다. [파일별 허용 해시와 순서](../assets/hyunmyungjanelee-20261126/final/accepted-local-otfs.tsv) · [실제 브라우저 글꼴 등록](../assets/hyunmyungjanelee-20261126/final/verified-browser-registration.tsv)

## 필수 로컬 검사와 연결 이슈

최소 글꼴 환경에서 전체 회귀 14,194건, Native Skia 4,227건, 집중 29건이 통과했다. 기본 배포 빌드와 배포 프로필 라이브러리 테스트 3,976건, Native·WASM·workspace Clippy, workspace 빌드와 문서 테스트 8건도 통과했다. Studio 타입 검사와 배포 빌드가 성공했고 테스트 2,835건이 통과했다. 전체 회귀에서 45건, Studio에서 2건은 테스트 실행기가 제외한 것으로 기록했다. [개별 검사 종료 시각·결과·로그 해시](../assets/hyunmyungjanelee-20261126/final/local-validation.json)

공개 글꼴 기여자 공통 검사도 독립 실행해 문서 4종의 Native·브라우저 8쪽 모두 100.00%를 확인했다. [원본 TSV와 쪽별 비교 그림](../assets/hyunmyungjanelee-20261126/final/contributor-gate/README.md)

통합 뒤 #8579의 그림 뒤 본문과 #8463의 표 이월 결과를 각 이슈에 반영한다. #8463은 기여자가 공개한 재현물과 파생 대조군으로 검증했다. #8582에는 Word 호환 문서의 탭 간격이 남는다. 원 기여 브랜치는 기여자가 관리한다.

## 병합 결과

[통합 PR #9428](https://github.com/edwardkim/rhwp/pull/9428)의 제출 커밋 `d2b181555a979356d7d3ffd544a723e9c7ad829b`에서 필수 검사와 보안·렌더링·정책 검사가 성공했다. [메인테이너 셀프 리뷰](https://github.com/edwardkim/rhwp/pull/9428#pullrequestreview-6070130915) 뒤 원 기여 커밋을 보존한 [병합 커밋 `79f17a994a6f`](https://github.com/edwardkim/rhwp/commit/79f17a994a6fb42b85cf3ad06672d537361ce9a8)으로 `devel`에 반영했다. [검사별 GitHub Actions 결과](../assets/hyunmyungjanelee-20261126/final/github-ci.tsv)

병합 뒤 [nextest 실행 시간 갱신](https://github.com/edwardkim/rhwp/actions/runs/45776612420)이 성공해 [측정 데이터 커밋 `414ceea878f5`](https://github.com/edwardkim/rhwp/commit/414ceea878f58d8c67804f1d004545f3becb677d)을 게시했다.

[이슈 자동 종료 작업](https://github.com/edwardkim/rhwp/actions/runs/45776612430)이 #8579와 #8463을 완료로 닫았다. Word 호환 탭 간격이 남은 #8582는 열려 있다.

| 원 PR | 상태 | 연결 이슈 | 상태·처리 범위 |
| --- | --- | --- | --- |
| [#9416](https://github.com/edwardkim/rhwp/pull/9416) | MERGED | [#8582](https://github.com/edwardkim/rhwp/issues/8582) | OPEN · Word 탭 간격은 후속 |
| [#9418](https://github.com/edwardkim/rhwp/pull/9418) | MERGED | [#8579](https://github.com/edwardkim/rhwp/issues/8579) | CLOSED · 그림 뒤 본문까지 검증 |
| [#9420](https://github.com/edwardkim/rhwp/pull/9420) | MERGED | [#8463](https://github.com/edwardkim/rhwp/issues/8463) | CLOSED · 공개 재현본과 파생 대조군 검증 |
