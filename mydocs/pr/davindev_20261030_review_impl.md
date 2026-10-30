# davindev 3건 검토·통합 계획

- 기준 devel: `2d535febe00a61585b6770cb95860fee912bbe54`.
- 브랜치: `review/davindev-20261030`. 다른 `review/pblsketch-20261030`는 보존한다.
- #8134 → #8147 순으로 원 커밋을 보존해 적용한다. #8147이 #8134를 포함하므로 원 head merge 한 번으로 두 PR의 기여 이력을 유지한다.
- #8175의 원 head를 merge하고, 승인된 SVG·Canvas2D 충돌만 최신 글꼴 계약을 보존해 해소한다.
- 먼저 최신 base의 같은 재현 입력을 출력한다. 이어 통합 후보를 빌드해 원 문제 개선, 대조군 보존, 실제 backend·전쪽 시각 검증을 완료한다.
- 테스트의 현재 규약과 실행 결과를 확인하고, 필요한 보정은 독립 기준을 확보한 뒤 적용한다. 구현 결함이면 범위를 명시하고 실제 재검증한다.
- 최종 로컬 검증 후 PR별 판정과 원격 처리안을 제시한다. 원격 push·새 통합 PR·원 PR merge·comment는 승인 범위를 확인한 뒤 수행한다.

## 검토 중 최신 devel 반영

2026-10-30 KST에 #8389·#8390이 통합되어 기준을
`cb9c45cf774a03eddb5d56fdb1fd28919f10a969`로 갱신했다. 기존 후보와 충돌 없이
merge한 현재 검증 소스는 `d189352a1b3b0e09718022f7da47daaa5205fab9`다.
원 기여 커밋과 승인된 #8175 보정을 유지하며 제품 차이는 7개 파일이다.

이전 기준의 실행 결과와 asset은 보존하고, 새 기준의 producer·TSV·로그는
`output/pr-review/davindev-20261030/{producers/latest-base,producers/latest-candidate,latest-base,latest-candidate}`에
분리한다. 세 PR의 원본 5개와 #8389의 편람 HWP/HWPX를 같은 글꼴·독립 Print PDF로
대조한다. 편람은 두 원본 각각 전 383쪽을 확인하며, 먼저 9·130·205쪽으로 글꼴 공급과
출력 경계를 진단한다. 진단 3쪽의 결과를 전쪽 완료로 기록하지 않는다.

Native 글꼴 입력 한도를 지키기 위해 편람의 한컴바탕만 로컬 subset으로 공급한다.
원본 HWPX의 Unicode 집합에 포함되는 986개 문자의 원본·subset 윤곽과 advance가
동일함을 확인했다. KoPub 6종·HCR Batang·Wingdings는 원본 프로그램을 공급한다.
이 private 글꼴과 글꼴을 포함한 SVG는 공개 asset에 추가하지 않는다.

## 원격 처리 승인

2026-10-30 작업지시자가 회귀 테스트 통과 후 원격 push와 머지를 승인했다.
세 원 PR의 커밋을 조상으로 보존한 통합 브랜치를 제출하고 최신 head CI와 self-review를
확인한 뒤 merge commit으로 통합한다. 원 PR별 판정·기여 범위는 별도로 기록한다.

## 로컬 검증 완료와 제출 브랜치

전체 회귀 12,198건, Native lib 4,040건, 그림 누락 2건, 직접 PDF 출력 14건이 통과했다.
원격 제출 전 브랜치명을 `review/integration-20261030-davindev-rendering`으로 정렬했다.
원본 head 세 개가 모두 조상인 것을 확인했다. 본인 열린 PR #8392의 CI 완료를 먼저 확인한다.
