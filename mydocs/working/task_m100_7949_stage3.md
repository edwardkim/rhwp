# Task #7949·#7950 — 기본 WASM 통합 및 범위 추가

- Issue: [#7949](https://github.com/edwardkim/rhwp/issues/7949), [#7950](https://github.com/edwardkim/rhwp/issues/7950)
- 작성일: 2026-10-16 (Asia/Seoul)
- 검증한 제품 source: `14539943a3a6665b82eb547c7402f5dcff2dc4aa`
- 상태: 기존 공동 범위의 통합 검증 성공. 새 SMask·Matte 범위 구현으로 진행

표준 Docker `wasm` 서비스·공유 cache로 기본 on bundle과 명시적 off bundle을 생성했다.
pkg/public 일치와 브라우저 실제 로드 SHA를 확인했다. 기본 WASM SHA-256은
`22f9439b39bece3ccc674e45b65ebeca200d6f052c8ff0f98fe0d1a684aaa1fa`다.
사용자 7700 서버는 보존하고 별도 Studio 7798에서 검증했다.

- 실제 WASM/Worker 계약: **85 assertion, Worker 5개 통과**. 명시적 off와 기존 문서 보존 포함.
- 공용 Worker 단위 검사: **6/6 통과**.
- 기존 PDF resource 검사: **111/111 통과**. 명시적 Mask의 PNG alpha·색상·예산 검사를
  기존 미지원 계약에 통합했다. 새 배치 fixture/golden은 추가하지 않았다.
- 최초 새 assertion의 오류 코드를 진단했다. dangling xref는 bounded loader가 무제한 복구
  scan을 거절하는 `PDF_RESOURCE_LIMIT`이며, 잘못된 sample/Decode의 parse 오류와 구분했다.
- MOF 원본 1쪽: Native **82.79780%**, fresh WASM **83.29803%**. 로고 Mask 표시 확인,
  배경·글꼴/간격 차이는 남는다. 사용자가 이 범위의 시각 예외와 후속 백로그 분리를 승인했다.
- 공개 대조 PDF 12쪽: Native 최저 **90.14609%**, fresh WASM 최저 **90.16832%**.
  Native 전쪽 PNG는 이전 검증 producer와 바이트 동일하다. 독립 PDF와 전쪽 TSV는 보존했다.

원본/결과 비교는 `E:\tmp\issue7949-pdf-review-20261016\index.html`에 준비했다.
로그·TSV·producer/source hash는 ignored `output/pr-review/issue7949/`에 있다.
전체 lint·전체 Rust 회귀 등 PR 최종 게이트는 아직 완료하지 않았다.

다음 입력 `issue-issue7949-type-002.pdf`는 1,199,820바이트·19쪽이며 SHA-256은
`d33ee4e0c6de1abfcd2c93a7baf3a458c6647243c685ad49959892529331c956`다.
첫 image 6의 SMask 7은 DeviceGray 8비트·Flate·Matte `[0 0 0]`이며, 다른 그림에는
1비트 SMask도 있다. 현재 거절은 앞선 명시적 Mask 보정의 회귀가 아니라 미지원 유형이다.
사용자가 이번 작업에 포함하도록 승인해 구현계획에 추가했다. 원격 push·PR 생성은 수행하지 않았다.
