# PR #9526 셀프 리뷰

## 최종 판정

**승인.** 제출 커밋 `088c9724e50297c6ed719752e63ab8c7e8b6338d`의 PDF 가져오기 보정, 회귀 검사와 공개 증적을 검토했다. 작업지시자의 셀프 리뷰·병합 승인을 확인했다. 이 기록을 포함한 새 head의 필수 검사와 최신 `devel` 병합 가능 상태를 확인한 뒤 일반 병합한다.

[PR #9526](https://github.com/edwardkim/rhwp/pull/9526) · [이슈 #9506](https://github.com/edwardkim/rhwp/issues/9506) · [구현 및 검증 보고서](../../report/task_m100_9506_report.md)

## 수정 범위와 재현

PDF의 열린 경로에 유한한 시작점만 있을 때 획을 그리려 하면, 기존 경로 빌더에서 빈 경로 오류가 전파되어 전체 가져오기가 중단됐다. [공통 PDF 경로 변환](../../../src/parser/pdf/path_paint.rs)은 이 경우에 개체를 생성하지 않고 변환을 계속한다. 실제 선분과 명시적인 닫힘을 포함한 경로는 기존 경로 처리로 들어간다. 합성 PDF의 둥근 끝으로 그린 길이 0 획, 일반 선과 채움이 유지되는 것을 확인했다.

## 실행 검증

- 기존 소스에서 실제 PDF와 합성 PDF가 `PDF_LOWERING_FAILED`로 중단됐다. 수정 Native CLI에서 실제 PDF 167쪽을 HWP와 HWPX로 저장한 뒤 다시 열어 쪽수를 확인했다. 새 WASM의 Studio 공통 PDF Worker에서도 원본 전체를 가져와 HWPX로 저장하고 167쪽을 다시 열었다.
- 격리한 최소 글꼴 환경의 전체 Rust 회귀 14,292건, 집중 PDF 관련 검사 15건, 포맷 검사, Native·WASM32·워크스페이스 전체 타깃 Clippy와 워크스페이스 빌드가 통과했다. 전체 회귀 중 기존 설정이 제외한 검사는 45건이다.
- 제출 SHA의 회귀 A~E와 lint가 통과했고 `Build & Test`, Adapter 및 Proptest가 성공했다. CodeQL은 선택된 Rust 분석이 통과했으며 PR 스캔의 열린 경고는 0건이었다. 변경 범위에 따라 다른 언어의 분석 단계가 생략되어 CodeQL 요약은 `neutral`이다. 최종 `CI Impact Policy`는 성공했다. [Build & Test](https://github.com/edwardkim/rhwp/actions/runs/46139784609/job/140985496734) · [Adapter](https://github.com/edwardkim/rhwp/actions/runs/46139784602) · [Proptest](https://github.com/edwardkim/rhwp/actions/runs/46139784565) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/46139784576) · [최종 정책 감사](https://github.com/edwardkim/rhwp/actions/runs/46146223582)
- #9552를 반영한 `devel` `77b3b0d69aaec3d30bc6e701dde29830bea5d4be`에 제출 SHA를 합치는 시뮬레이션에서 충돌이 없었다. 후속 기록 PR #9562를 포함한 최신 `devel`에서도 병합 결과의 PDF 파서, 합성 PDF와 관련 회귀 검사 내용이 검증한 제출 SHA와 같았다.

## 시각 증적

합성 재현 PDF와 시작점 전용 획을 생략한 대조 PDF 각 2쪽을 검증했다. 독립 MuPDF의 96 DPI 출력과 비교한 Native Skia 및 브라우저 WASM Canvas의 전쪽 2px raw 실루엣 일치율은 각각 100.00%였다. 증상이 있던 2쪽의 픽셀 일치율은 각 경로 99.84551%였다. 두 출력의 비교 그림에서 선, 채움과 둥근 점이 모두 보였다. 페이지별 시각 수치는 공개 합성 표본에 해당한다. [Native 전쪽 TSV](../assets/issue_9506/native/silhouette.tsv) · [WASM 전쪽 TSV](../assets/issue_9506/wasm/silhouette.tsv) · [PDF/SVG Visual Sweep 가이드](../../manual/verification/visual_sweep_guide.md#github-merge-comment)

### Merge 후 contributor PR comment 계획

완료 근거를 별도로 게시하게 되면 합성 재현·대조 PDF 각 2쪽의 위 결과와 2쪽 판독을 기록한다. [Native 비교](../assets/issue_9506/native/review_002.png)와 [WASM 비교](../assets/issue_9506/wasm/review_002.png)를 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/issue_9506/{native,wasm}/review_002.png`에 연결한다. 병합 SHA의 자료를 확인하고 게시 승인이 있을 때 UTF-8 `--body-file`로 전송한 다음 API로 본문과 링크를 재확인한다.
