# planet6897 저장 조판 통합 검증

## 최종 후보와 증적

기준 devel은 `626a5b30ab773a48aec004a22ad789e508ae9c49`, 제품 검증 소스는
`e70dc6231a22ec0bd569537c4e5cb0ff930fd606`다. 재적용한 패치 32개의 정합을 확인했다.
리베이스 직후 제품·테스트·입력·시각 도구의 파일 내용은 이전 후보와 동일했다.

최종 검토 기록은 [#9289](../../archives/pr_9289_review.md),
[#9305](../../archives/pr_9305_review.md), [#9311](../../archives/pr_9311_review.md),
[#9317](../../archives/pr_9317_review.md), [#9324](../../archives/pr_9324_review.md),
[#9327](../../archives/pr_9327_review.md), [#9363](../../archives/pr_9363_review.md),
[#9364](../../archives/pr_9364_review.md)에서 확인한다.

#9364의 추가 보정은 함초롬바탕의 반각 낫표를 명시한 환경에서 저장 물리 줄을
보존한 채 문자 경계를 다시 계산한다. 96 DPI·2px raw 기준 Native와 fresh WASM이
각각 전쪽 3/3쪽 100.00%였다. 한컴바탕의 전각 낫표를 사용한 별도 Print 대조는
두 경로에서 전쪽 100.00%를 유지했으며 보정 전후 픽셀이 일치했다.
[PDF 영수증·실행 SHA·TSV·PNG 검증 기록](pr-9364/maintainer-wrap/validation.json)

#9363·#9364의 이전 통합 전쪽 TSV와 #9365의 12개 공개 대조군에는 동일한 입력을 사용했다.
#9365의 비교는 Native 12장과 WASM 12장이 최신 기준과 픽셀이 일치했다.
#9317 17쪽은 최신 기준과 후보의 Native·WASM PNG가 일치했다.
[이전 통합 실행 증적](rebase-claim-validation.json) · [영향 쪽과 재사용 근거](affected-and-reused-pages.json)

정본 보호 문서 46개 출력 설정에는 이번 보정이 활성화되는 슬롯별 자연 전진 선택이 없다.
아래 보호 결과는 앞선 소스에서 산출했고, 이번 변경의 적용 범위를 확인해 재사용했다.
새 글꼴 파일은 제출 자료에 포함하지 않았다.

최종 제품 후보의 최소 글꼴 전체 회귀 14,028건 전부 통과, 실패 0건, 제외 45건. Native Skia 라이브러리·placeholder·직접 PDF, fmt, Native/WASM/워크스페이스 Clippy, 워크스페이스 빌드, suite manifest, unit tier도 통과했다.

## 선행 통합 검증 기록

`upstream/devel` 기준 `321ab8f97791ecf58b56f178f49b7795a836fb79`에서 여섯 PR을 순서대로 적용했다.
제품 검증 소스는 `330af8608ac741b14041d3e372284a6111346b9c`다. Native Skia와 fresh WASM
브라우저 PNG는 같은 입력 및 데스크톱 한컴 `Hancom PDF` Print에 비교했다. fresh WASM은 Mac의
locked wrapper `--no-opt` 빌드를 사용했다. [입력·PDF SHA](claim-inputs.json) · [인쇄 영수증](hancom-print-receipts.json)

| 원 PR | 직접 검증한 범위 |
| --- | --- |
| [#9289](../../archives/pr_9289_review.md) | 같은 문단의 표·그림 원점과 앞 간격 |
| [#9305](../../archives/pr_9305_review.md) | 쪽을 넘겨 시작한 표의 첫 행과 후속 내용 |
| [#9311](../../archives/pr_9311_review.md) | 용지 기준 자리차지 표 바로 앞 빈 줄의 소속 |
| [#9317](../../archives/pr_9317_review.md) | 분할 표의 첫 조각에 사각형 한 번 배치 |
| [#9324](../../archives/pr_9324_review.md) | HWP/HWPX 교차 저장의 원점 표지와 두 번의 저장·다시 열기 |
| [#9327](../../archives/pr_9327_review.md) | 식량안보 보고서의 141쪽 분할·내용과 제주 문서의 그림 소속 |

전체 nextest는 424.721초 동안 14,016건 실행, 14,016건 통과, 제외 45건이다. focused 회귀,
Native Skia의 라이브러리·placeholder·직접 PDF 검사, Native/WASM/워크스페이스 Clippy,
워크스페이스 빌드, fmt, suite manifest, 기여자 편람 gate를 확인했다. 검토 판정은 원 PR의 주장
범위를 기준으로 삼았다. 식량안보 보고서의 별도 90~93쪽 흐름은 #8997에서 계속 추적한다.

## 정본 보호 문서

23개 원본의 Native/fresh WASM 46개 데이터셋에서 4,046쪽을 비교했다. 공식 두 자리 raw 하락은
0쪽이며 nested WASM 3쪽, 화학 문서 양쪽 backend 4쪽의 보호값이 올랐다.
자동 공급 조건의 두 문서는 같은 Mac에서 기준·후보 PNG 및 render tree 775쪽의 동일성도 확인했다.
[전쪽 검증과 TSV 목록](protected-validation.json)

nested 16쪽은 원본 한양신명조 언어 슬롯 5, 글꼴 ID 7의 `‧` 지원이 빠져 대체 글자의 위치가 달랐다.
기존 명조 보완 순서에서 `SPSMJ.otf`가 로마 숫자와 괄호를 먼저 공급하고 `HDOTUM.TTF`가 남은
기호를 공급하도록 설정했다. Native의 기존 확인 결과와 함께 WASM은 기준·후보 17쪽 모두
두 자리 100.00%를 확인했다. 후보와 기준의 PNG·render tree도 17쪽 전부 일치했다.
[공급 기록](../../../verification/protected-font-supply/records/nested-table-wasm-original-programs-validation.json) · [후보 TSV](protected/nested/wasm/silhouette.tsv) · [16쪽 review](protected/nested/wasm/review/review_016.png) · [16쪽 overlay](protected/nested/wasm/overlay/overlay_016.png)
