# 원 PR #9327 후속 검토

[통합 당시 검토](pr_9327_review.md)를 이어서 확인했습니다. 원 PR `39d018f7a363910d0cd3a483a078214b66a6b423`은 #9372로 통합됐습니다. 식량안보의 이월 표와 제주 관광교육 계획의 배치·인쇄를 독립 한컴 Print와 다시 대조했습니다.

## 조판 원칙과 검증 입력

저장 글줄·표 경계의 소속과 순서를 조판에서 확정하고, 그림·캡션 원점과 차트의 인쇄 피복을 출력이 소비하는 경계에서 확인했습니다. 표 2-5의 두 조각과 이어지는 글줄은 92·93쪽에 보존됩니다. 독립 PDF에 있는 괘선과 차트의 관계를 원본 레코드와 출력에서 대조했습니다. 실행한 원본과 한컴 Print는 후보 커밋에 포함된 파일과 SHA-256으로 대조했습니다.

## 원 PR 주장과 메인터너 보정

식량안보 표 2-6은 96·97쪽에 원본 행이 이어지고 89~93쪽의 공백 호스트와 글줄 원점을 보존했습니다. 제주 240·244쪽은 그림과 캡션의 쪽 소속, 299쪽은 분자와 분모의 논리 줄 간격을 대조했습니다. 제주 47~50쪽은 정상 2024 전체 Print로 보정했고 외곽 괘선의 끊김을 제거했습니다.

- 제주 313쪽은 [원본 HWP](../../../samples/issue8759/6500000-201700118_jeju_tourism_education_plan.hwp)와 [한컴 2024 PDF](../../../pdf/issue8759/6500000-201700118_jeju_tourism_education_plan-2024.pdf)를 `--compat 2024`로 비교했습니다.
- 식량안보 141쪽은 [원본 HWP](../../../mydocs/pr/assets/planet6897_clean20_20261113/counterexamples/1543000-202100035-food-security-report.hwp)와 [한컴 2020 PDF](../../../pdf/issue8997/1543000-202100035-food-security-report-2020.pdf)를 `--compat 2022`로 비교했습니다.

| 문서·기준 Print | 쪽수 | Native 최저 | fresh WASM 최저 | 원 PR 대비 하락 |
| --- | ---: | ---: | ---: | ---: |
| 제주 관광교육 종합계획 · 한컴 2024 | 313 | 98.18% | 98.15% | 0 |
| 식량안보 보고서 · 한컴 2020 | 141 | 98.25% | 98.18% | 0 |

| 대상 | Native | fresh WASM |
| --- | --- | --- |
| 식량안보 92쪽 이월 표 | [review](../assets/issue_9326/20261124/food/native/review/review_092.png) · [overlay](../assets/issue_9326/20261124/food/native/overlay/overlay_092.png) | [review](../assets/issue_9326/20261124/food/wasm/review/review_092.png) · [overlay](../assets/issue_9326/20261124/food/wasm/overlay/overlay_092.png) |
| 식량안보 97쪽 표 2-6 | [review](../assets/issue_9326/20261124/food/native/review/review_097.png) · [overlay](../assets/issue_9326/20261124/food/native/overlay/overlay_097.png) | [review](../assets/issue_9326/20261124/food/wasm/review/review_097.png) · [overlay](../assets/issue_9326/20261124/food/wasm/overlay/overlay_097.png) |
| 제주 47쪽 차트 바깥 괘선 | [review](../assets/issue_9326/20261124/jeju/native/review/review_047.png) · [overlay](../assets/issue_9326/20261124/jeju/native/overlay/overlay_047.png) | [review](../assets/issue_9326/20261124/jeju/wasm/review/review_047.png) · [overlay](../assets/issue_9326/20261124/jeju/wasm/overlay/overlay_047.png) |
| 제주 50쪽 원형 차트 | [review](../assets/issue_9326/20261124/jeju/native/review/review_050.png) · [overlay](../assets/issue_9326/20261124/jeju/native/overlay/overlay_050.png) | [review](../assets/issue_9326/20261124/jeju/wasm/review/review_050.png) · [overlay](../assets/issue_9326/20261124/jeju/wasm/overlay/overlay_050.png) |
| 제주 240쪽 그림·캡션 | [review](../assets/issue_9326/20261124/jeju/native/review/review_240.png) · [overlay](../assets/issue_9326/20261124/jeju/native/overlay/overlay_240.png) | [review](../assets/issue_9326/20261124/jeju/wasm/review/review_240.png) · [overlay](../assets/issue_9326/20261124/jeju/wasm/overlay/overlay_240.png) |
| 제주 299쪽 한글 수식 | [review](../assets/issue_9326/20261124/jeju/native/review/review_299.png) · [overlay](../assets/issue_9326/20261124/jeju/native/overlay/overlay_299.png) | [review](../assets/issue_9326/20261124/jeju/wasm/review/review_299.png) · [overlay](../assets/issue_9326/20261124/jeju/wasm/overlay/overlay_299.png) |

| 문서 | Native 전쪽 | fresh WASM 전쪽 |
| --- | --- | --- |
| 제주 관광교육 종합계획 · 한컴 2024 | [TSV](../assets/issue_9326/20261124/jeju/native/candidate-silhouette.tsv) · [비교](../assets/issue_9326/20261124/jeju/native/comparison.json) | [TSV](../assets/issue_9326/20261124/jeju/wasm/candidate-silhouette.tsv) · [비교](../assets/issue_9326/20261124/jeju/wasm/comparison.json) |
| 식량안보 보고서 · 한컴 2020 | [TSV](../assets/issue_9326/20261124/food/native/candidate-silhouette.tsv) · [비교](../assets/issue_9326/20261124/food/native/comparison.json) | [TSV](../assets/issue_9326/20261124/food/wasm/candidate-silhouette.tsv) · [비교](../assets/issue_9326/20261124/food/wasm/comparison.json) |

표 2-5의 문단 925·개체 0은 92·93쪽의 두 조각에 남아 있습니다. 92쪽에 시작한 단계 2의 첫 줄은 다음 쪽에서 이어지며 두 쪽의 경계 글줄은 독립 한컴 2020 Print와 같습니다. [표 2-5 쪽 소속과 실제 글줄](../assets/issue_9326/20261124/food/table-2-5-page-ownership.json)

## 차트 근거

보정 전 SVG에서 0.48px 획으로 재생된 외곽 괘선은 격자 경계에서 피복이 줄었습니다. 독립 한컴 2024 PDF의 4개 외곽 괘선은 0.120pt 회색 벡터 경로입니다. 보정 후 같은 Print에 대조한 결과는 아래와 같습니다. [보정 전 피복과 원본 VtBackdrop](../assets/issue_9326/20261124/jeju/chart-frame-source-evidence.json)

| 제주 쪽 | Native 보정 전 → 후 | fresh WASM 보정 전 → 후 |
| ---: | ---: | ---: |
| 47 | 97.99% → 99.94% | 98.39% → 99.97% |
| 48 | 99.35% → 100.00% | 97.64% → 100.00% |
| 49 | 99.36% → 100.00% | 98.72% → 100.00% |
| 50 | 97.96% → 100.00% | 97.57% → 100.00% |

[Native PNG 해시 변화 범위](../assets/issue_9326/20261124/native-png-identity.json). 제주 47~50쪽 이외의 309쪽 및 식량안보 141쪽의 Native PNG는 보정 전과 해시가 같습니다.

## 검증 커밋과 입력

코드 검증 head는 `24ec51a6f9186d509d3f3285faa6f9f6e067af5e`, 비교 devel은
`2c0c1d205a54b8d5bd262241bac1391cffe0d801`입니다. 전쪽 비교에 사용한 실행 파일과 WASM 패키지는
이 커밋에서 다시 빌드한 산출물과 해시가 같습니다. [산출물 해시](../assets/issue_9326/20261124/producer-identity.json)

[입력·기준 PDF 해시와 전쪽 TSV 목록](../assets/issue_9326/20261124/visual-evidence.json)

macOS 최소 글꼴 환경의 전체 Nextest는 14,074건 통과, 실패 0건, 45건 건너뜀입니다. fmt·suite manifest·Native 및 fresh WASM 빌드도 통과했습니다.

[HWP 저장 제품과 기준 Print PDF 생산 이력](../assets/issue_9326/20261124/print-provenance.json)

Native Skia 단위 검사는 4,032건 통과, 실패 0건, 제외 12건입니다. Native Skia 집중 검사에서 누락 그림 자리표시자 2건과 37쪽 PDF 직접 내보내기 관련 37건을 통과했습니다. Native Clippy, WASM32 Clippy, workspace all-target Clippy와 workspace 빌드를 통과했습니다.

정본 보호 46개 조건·4,046쪽의 공식 두 자리 표시값 하락은 0쪽입니다. 종합부동산세법 Native·fresh WASM 각 6쪽과 sample10 Native 763쪽은 문서별 허용 SHA-256으로 기존 하한을 재현했습니다. 중첩 표 Native 17쪽은 100.00%이며 2·7·17쪽의 보호값을 상향했습니다. [보호 비교 요약](../assets/issue_9326/20261124/protected/README.md) · [갱신 영수증](../assets/visual-protected-baseline/updates/issue9326-20261124.json)

실제 Studio Canvas2D에서 두 원본의 주장 쪽 14건을 확인했습니다. CanvasKit은 제주 차트 축소 대조군 2건에서 선택·렌더링됐습니다. 전체 원본의 CanvasKit 준비는 devel과 후보 모두 200,000 작업 한도에 닿았습니다. 식량안보는 devel 92쪽·후보 91쪽, 제주는 양쪽 197쪽까지 검사한 상태였습니다. 후보가 Canvas2D로 전환한 14건은 Canvas2D를 직접 선택한 같은 쪽과 PNG 해시가 일치했습니다.

보호 TSV 해시 변경 후 편람·기여자 공통·글꼴 해시 계약 검사 33건과 기여자 공통 실물 비교 8건을 다시 통과했습니다.

[#9326 미달 쪽과 #8997 이월 표 경계의 대응 결과](../assets/issue_9326/20261124/issue-page-coverage.json)

기여자 공통 보호 검사는 4개 문서의 Native·fresh WASM 8개 조건을 모두 통과했습니다. [기여자 공통 증적](../assets/issue_9326/20261124/contributor/validated-evidence.json)

## 보호 공급 대체 진단

이전 Windows 글꼴 조합을 사용한 중첩 표 Native는 고정 devel과 후보의 PNG가 같았고 정본 대비 8쪽이 낮았습니다. 이후 기존 정본 WASM 공급 목록과 같은 21개 파일에 Native의 문서별 글꼴 선택 설정을 적용했습니다. 이 조건으로 devel과 후보를 다시 전쪽 캡처해 모두 100.00%임을 확인한 뒤 정본에 반영했습니다.

최신 devel의 #9385는 기존 인쇄 PDF를 사용한 조판 검증의 입력·제품 소스·검증 스크립트·정본을 바꾸지 않았습니다. 병합 시뮬레이션의 제품·검증 경로도 검증한 후보와 같았습니다.

## Merge 후 contributor PR comment 계획

[원 PR #9327](https://github.com/edwardkim/rhwp/pull/9327)의 식량안보 141쪽 복원과 그림·목차·표의 쪽 소속 보존에 감사합니다. 후속 검토에서 식량안보 90~93쪽의 저장 경계와 제주 차트의 한컴 2024 인쇄 대조를 확인했습니다. 병합 후 최종 SHA에 있는 아래 직접 증적만 인라인으로 게시하고, 최신 통합 PR과 검증 결과를 연결합니다.

| 주장 쪽 | Native | fresh WASM |
| --- | --- | --- |
| 식량안보 표지와 TAC 로고 1쪽 | 100.00% [review](../assets/issue_9326/20261124/food/native/review/review_001.png) · [overlay](../assets/issue_9326/20261124/food/native/overlay/overlay_001.png) | 100.00% [review](../assets/issue_9326/20261124/food/wasm/review/review_001.png) · [overlay](../assets/issue_9326/20261124/food/wasm/overlay/overlay_001.png) |
| 식량안보 목차 표 2쪽 | 99.84% [review](../assets/issue_9326/20261124/food/native/review/review_002.png) · [overlay](../assets/issue_9326/20261124/food/native/overlay/overlay_002.png) | 99.84% [review](../assets/issue_9326/20261124/food/wasm/review/review_002.png) · [overlay](../assets/issue_9326/20261124/food/wasm/overlay/overlay_002.png) |
| 식량안보 표 2-6의 2008년 행까지 96쪽 | 100.00% [review](../assets/issue_9326/20261124/food/native/review/review_096.png) · [overlay](../assets/issue_9326/20261124/food/native/overlay/overlay_096.png) | 100.00% [review](../assets/issue_9326/20261124/food/wasm/review/review_096.png) · [overlay](../assets/issue_9326/20261124/food/wasm/overlay/overlay_096.png) |
| 식량안보 표 2-6의 2009년 이후 97쪽 | 98.89% [review](../assets/issue_9326/20261124/food/native/review/review_097.png) · [overlay](../assets/issue_9326/20261124/food/native/overlay/overlay_097.png) | 98.89% [review](../assets/issue_9326/20261124/food/wasm/review/review_097.png) · [overlay](../assets/issue_9326/20261124/food/wasm/overlay/overlay_097.png) |
| 식량안보 표 다음 본문 98쪽 | 100.00% [review](../assets/issue_9326/20261124/food/native/review/review_098.png) · [overlay](../assets/issue_9326/20261124/food/native/overlay/overlay_098.png) | 100.00% [review](../assets/issue_9326/20261124/food/wasm/review/review_098.png) · [overlay](../assets/issue_9326/20261124/food/wasm/overlay/overlay_098.png) |
| 제주 그림·캡션 쪽 소속 240쪽 | 100.00% [review](../assets/issue_9326/20261124/jeju/native/review/review_240.png) · [overlay](../assets/issue_9326/20261124/jeju/native/overlay/overlay_240.png) | 100.00% [review](../assets/issue_9326/20261124/jeju/wasm/review/review_240.png) · [overlay](../assets/issue_9326/20261124/jeju/wasm/overlay/overlay_240.png) |
| 제주 그림·캡션 쪽 소속 244쪽 | 100.00% [review](../assets/issue_9326/20261124/jeju/native/review/review_244.png) · [overlay](../assets/issue_9326/20261124/jeju/native/overlay/overlay_244.png) | 100.00% [review](../assets/issue_9326/20261124/jeju/wasm/review/review_244.png) · [overlay](../assets/issue_9326/20261124/jeju/wasm/overlay/overlay_244.png) |
