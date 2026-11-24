# PR #9394 표 높이 편집·영역 누름틀·HWPX 본문 저장 줄 셀프 리뷰

semanticist21 님의 세 기여를 메인테이너 보완 후 수용한다. 렌더링 기준은 `e961a7877186de626643a6466cbecb137874c530`, 최종 제품 소스는 `ff9194594e1d7396c3730b92f387e2a4d3c62b3a`다. 문서 8개가 추가된 제출 기준 `dd1dc8509769df46593ca91e23c06167d0436607`에서도 제품과 검증 입력이 동일했고, 제출 커밋 `1cdbd4e7936663c5c6fa2cc3adc69d48fc507db8`의 GitHub CI가 성공했다. [통합 PR #9394](https://github.com/edwardkim/rhwp/pull/9394) · [로컬 검증과 독립 인쇄 증거](../assets/semanticist21-9337-9354-9377/README.md)

| 원 기여별 검토 | 제출 커밋에 보존한 원 기여 SHA |
| --- | --- |
| [#9337 표 높이 편집](pr_9337_review.md) | `b69c815caa964dbbba1d55a9aae6654c875a5a05` |
| [#9354 네 영역의 누름틀 입력](pr_9354_review.md) | `fc4e2cf07a1439760d2cfb67c8230cb3140e2a78` |
| [#9377 HWPX의 편집 본문 줄 좌표](pr_9377_review.md) | `b3301e1905edd7fe504fe400c3560e27c421ad33` |

## 직접 확인한 동작

- 같은 병합 셀 높이를 속성 창과 크기 조절로 바꿔도 표 뒤 간격은 12.4 px다. 독립 빌드한 `devel`에서는 같은 조작으로 0.6 px까지 줄었다. 이웃 셀이 높이 변화를 서로 보상하는 공개 문서는 두 저장 형식 모두 원 표 외곽 17,932 HU를 지켰다. [셀 편집과 실행 취소·다시 실행](../assets/semanticist21-9337-9354-9377/browser-editing.json) · [보상 편집 전후](../assets/semanticist21-9337-9354-9377/compensated-resize.tsv)
- 각주·미주·머리말·꼬리말에서 같은 위치의 두 누름틀 중 선택한 하나만 글자를 소유했다. 한컴 인쇄에서 줄 너비 표가 다음 줄로 넘어갈 때 앞줄의 글자가 양끝에 놓이는 것을 확인했다. 본문의 ‘천원)’과 표가 차지하는 줄은 어절·글자 나눔 설정을 각각 따랐다. [필드별 결과](../assets/semanticist21-9337-9354-9377/corrected-fields.tsv) · [표 앞 본문 줄 확대](../assets/semanticist21-9337-9354-9377/body-overflow-comparison.png)
- 26개 문단을 각각 편집한 뒤 HWP·HWPX로 다시 열었다. 52건 모두 24쪽이고 최종 Chrome WASM에서 365개 본문 문단의 텍스트와 쪽 소속이 원본과 일치했다. [재열기 결과](../assets/semanticist21-9337-9354-9377/corrected-reopen.tsv)

## 검증과 수용 범위

최소 글꼴 환경의 전체 Rust 회귀 14,091건, Native 공식 3종 4,072건과 Studio 2,835건이 통과했다. 한컴 인쇄를 기준으로 목표 20개 문서의 경로별 92쪽과 추가 8개 입력의 경로별 32쪽을 확인했다. 기준과 후보의 보호 문서 4,046행은 모두 같은 점수와 같은 PNG였다. 작업지시자는 공식 100.00%에 대해 99.87~99.99%인 기존 WASM 14쪽에 한해 수용했다. 최종 측정에서 그 범위에 해당하는 쪽은 14쪽이다. 중첩 표 Native에는 이 세 PR에 한해 승인된 동명 로컬 글꼴 5종을 포함해 총 21개를 같은 순서로 공급했다. 기준과 후보 모두 17쪽 전부 100.00%였다. [검사별 명령·결과·환경](../assets/semanticist21-9337-9354-9377/local-checks.json) · [TSV 원문 목록](../assets/semanticist21-9337-9354-9377/page-tsv/index.tsv) · [공식 하한 미달 쪽의 해시](../assets/semanticist21-9337-9354-9377/protected-below.tsv)

각 원 기여의 최종 커밋에서 재생성한 Native·WASM 184행은 같은 조건의 통합 후보에서 하락이 없었다. 작업 중 병합된 #9388의 식량안보 141쪽·제주 관광교육 313쪽도 양쪽 backend에서 최신 devel과 같은 PNG였다. [원 PR별 대조](../assets/semanticist21-9337-9354-9377/original-candidate.tsv) · [#9388 전쪽 보존](../assets/semanticist21-9337-9354-9377/9388-preservation.tsv)

세 PR의 자동 종료 연결 이슈는 0개다. 참고한 #6714·#6882·#5847·#8310은 이미 닫혀 있다. 별도 PR #9298은 게시한 인쇄 보완 요청의 답변을 기다린다. [#9298 보완 요청](https://github.com/edwardkim/rhwp/pull/9298#issuecomment-7089121727)

## 제출 커밋의 GitHub 검사

성공 34개, 실행 조건으로 건너뛴 검사 5개, 실패와 대기 0개를 확인했다.

| 검사 | GitHub 실행 |
| --- | --- |
| CI | [실행 45402601922](https://github.com/edwardkim/rhwp/actions/runs/45402601922) |
| CodeQL | [실행 45402601858](https://github.com/edwardkim/rhwp/actions/runs/45402601858) |
| Render Diff | [실행 45402601737](https://github.com/edwardkim/rhwp/actions/runs/45402601737) |
| Adapter inter-diff | [실행 45402601829](https://github.com/edwardkim/rhwp/actions/runs/45402601829) |
| Proptest roundtrip | [실행 45402601757](https://github.com/edwardkim/rhwp/actions/runs/45402601757) |
| CI Impact Policy | [성공](https://github.com/edwardkim/rhwp/actions/runs/45408523578) |

이 문서와 작업 기록을 담은 최종 커밋에서 필수 검사와 정책 판정을 다시 확인한 뒤 일반 merge commit으로 통합한다.
