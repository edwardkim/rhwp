# PR #9303 검토 — 표 뒤 빈 문단의 쪽 나눔과 혼합 꼬리말

[#7910](https://github.com/edwardkim/rhwp/issues/7910)의 공개 HWP는 기존 rhwp에서 1쪽, 한컴 Print에서 2쪽이었다. 어울림 표 옆의 좁은 흐름에서 본문 빈 문단을 계상한 뒤, 표가 끝나고 전폭으로 돌아오는 문단에 일반 쪽 맞춤이 적용되지 않았다. 후보는 같은 줄 점유와 배치 순서로 빈 문단을 배치해 끝 문단을 2쪽에 보존한다. `84dee82a`의 꼬리말 그림·사각형·텍스트 배치 보정은 유지했다. [PR #9303](https://github.com/edwardkim/rhwp/pull/9303) · [입력·한컴 Print·배치 근거](../assets/issue_7910/README.md)

| 검증 | 결과 |
| --- | --- |
| 원본 HWP 및 실제 한컴 저장 HWPX | Native Skia·fresh WASM 모두 한컴 Print와 같은 2쪽. 마지막 빈 본문 문단 하나를 제거한 대조는 각각 1쪽이다. |
| 꼬리말 변형 | 번호를 제거해도 일반 문구·그림·사각형이 남았다. 그림을 제거해도 번호와 사각형이 남았다. 사각형 제거 시 기준선은 Native SVG에서 4.00 px, 한컴 Print에서 약 4.00 px 이동했다. 원본 기준선 차이는 약 1.07 px였다. |
| 직접 시각 비교와 회귀 | 두 백엔드 모두 10개 입력·19쪽. 96 DPI 2 px raw 전쪽 최저 Native 99.04%, WASM 99.05%. #7910 Rust 회귀 2건, #9394 인라인 표 대조 각 6건, 공통 보호 8건 통과. |
| 최신 devel 기능 보존 | #9422의 실제 HWPX 36쪽 Native 100.00%, WASM 99.84%. 저장 현재 높이와 크기 보호의 반례 2건 통과. 편람 HWPX 288~290쪽을 두 백엔드에서 재출력해 모두 100.00%를 확인했다. |

## 정본 보호

정본 `8141acb297ce2f39148f9014e463986431d266120629e8f368665e5917138f9d`로 23개 문서의 Native·fresh WASM 각각 2,023행을 비교했다. **4,046행 통과, 하한 미달·누락·신규 하한 조정 0행.** 측정 PDF는 #9437과 같은 PDFium 5.13.0을 썼다. 공급·실측은 문서와 백엔드별로 확인했다. [페이지별 결과](../assets/issue_7910/protected-4046.tsv) · [검증 소스 및 표 해시](../assets/issue_7910/protected-4046.json) · [교차 OS 정본](pr_9437_review.md)

- 종합부동산세법 Native 1·2쪽에서 PDFium 버전만 맞췄을 때는 99.98%·98.75%였다. #9306에서 확인한 Mac 프로세스 글꼴 조건을 적용하자 99.99%·98.86%로 두 하한을 충족했다. 통제 비교에서 SPSMJ, HGSMJ, HGGGT의 등록이 재현에 영향을 주었으며 이미지 배치 좌표는 같았다. 같은 공급의 devel·후보 Native 6쪽 PNG도 모두 일치했다.
- sample16 HWP 2022 Native에서는 #9306의 문서별 한양신명조 별칭을 재현해 64쪽이 모두 통과했고, devel과 후보 PNG가 모두 같았다. sample16 세 형식의 fresh WASM 6쪽은 각각 99.97%로 현행 하한을 충족했다. sample10은 #9306의 Mac 한양중고딕 공급 조건에서 두 백엔드 각각 763쪽이 통과했다. 두 편람의 WASM에는 현행 목록의 HCR 바탕을 적용했다. [#9306의 공급 재현](pr_9306_review.md)
- `ec3dac549a2e`의 4,040행을 재사용했다. #9435의 새 분기는 폭 0 표지가 있는 HWPX 앞면 부동 사각형 글상자에 적용된다. 보호 원본에서 이 속성을 검토했으며 편람에 남은 후보는 289쪽의 인라인·가운데 정렬 상자였다. `997278c7321d`에서 해당 인접 6행을 다시 측정했다. #9438이 적용된 `2373cf77a58d`의 추가 변경은 검토 기록과 별도 fixture이며 검증한 제품 코드와 보호표가 같다. [#9435 직접 대조](pr_9435_review.md)

## 코드와 GitHub 검사

동일 PR의 코드 head `2795f0a89a0d8a115fd483e20a0ed30752fac7a1`에서 CI, Render Diff, CodeQL, Adapter inter-diff, Proptest roundtrip이 성공했다. 최신 devel과 양쪽 작업일지가 보존되는 자동 병합 트리를 확인했다. 기록 커밋의 필수 검사 결과를 확인한 뒤 정상 병합한다. [코드 CI](https://github.com/edwardkim/rhwp/actions/runs/45563324157) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/45563324265) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/45563324231) · [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/45563324229) · [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/45563324169)
