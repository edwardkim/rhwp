# #9316 리베이스와 합성 글꼴 제거 검증

기준 `devel`은 `8a196ca7aea2aa3cf32d394f5b7da2bdf4520479`, 제품 후보는 `954226e426517dc02ef0a15cc739899ab54b7fae`다. 두 합성 글꼴을 추가했던 변경과
해당 전용 검사 2건을 PR 커밋 범위에서 제거했다. 기존 sample10 원본 474쪽 위치 검사는
유지했다. 렌더러가 같은 병합 검증 빌드에서 실제 Native PNG와 macOS locked wrapper
`--no-opt`의 fresh WASM Canvas를 사용했다.

#9316은 같은 원본과 한컴 Print, 실제 글꼴 파일의 SHA-256을 Native와 WASM 조건마다 고정했다.
보호 기록 46조건·4,046쪽은 합성 fixture 없이 생산됐다. sample10은 양쪽의 기록에 실제
`HBATANG.TTF`가 있다. [공급 기록 감사](9316-font-supply-audit.json)

| 문서·경로 | 결과 | 증적 |
| --- | --- | --- |
| sample10 474쪽 Native | 100.00% | [review](sample10-p474/native-review.png) · [overlay](sample10-p474/native-overlay.png) · [TSV](sample10-p474/native-tsv.tsv) |
| sample10 474쪽 fresh WASM | 100.00% | [review](sample10-p474/wasm-review.png) · [overlay](sample10-p474/wasm-overlay.png) · [TSV](sample10-p474/wasm-tsv.tsv) |
| #9316 원 입력 3종, 19쪽 | Native·fresh WASM 각각 전쪽 표시값 동일. 직접 PDF 19쪽 텍스트·래스터 동일 | [비교 기록](9316-output-comparison.json) |
| 화학 표기 103쪽 | Native·fresh WASM 각각 최저 99.99%. 고정 기준 대비 하락 0쪽 | [Native TSV](chemical/native/candidate.tsv) · [WASM TSV](chemical/wasm/candidate.tsv) |

화학 표기 문서는 변환본 `SPJGT.otf`와 실제 `HDOTUM.TTF`를 공급했다. 이미 유효한 기호의
원래 윤곽을 유지하고 빈 CFF 가운뎃점을 복원했다. 교차 검증은 96 DPI·2px 이웃 raw 실루엣의
두 자리 표시값을 썼다. [sample10 입력·공급 영수증](sample10-p474/validation.json)

31쪽의 가운뎃점과 주변 문자는 Native와 fresh WASM에 표시됐다. 독립 Print와의 대조에서
일부 글자 잉크와 윤곽에 차이가 보인다. 100.00%는 2px 이웃 실루엣의 표시값이다.
[Native review](chemical/native/review/review_031.png) · [Native overlay](chemical/native/overlay/overlay_031.png) ·
[WASM review](chemical/wasm/review/review_031.png) · [WASM overlay](chemical/wasm/overlay/overlay_031.png)

## 정본 보호 전쪽 결과

Native와 fresh WASM 46조건·4,046쪽을 정본 표시값과 대조했다. 미달과 누락은 0쪽이다.
정본 TSV의 SHA-256을 일치 확인했다. 이전 실행과 공급 목록 순서가 다른 조건은 결과에 표시했다.
[쪽별 대조 TSV](protected-pages.tsv) · [검증 요약](protected-summary.json)

## 제출 사전 검증

- 편람 HWPX Native 383쪽 `--handbook-gate`와 기여자 공통 보호 8조건을 통과했다.
  [검사 요약](formal-gates.json) · [편람 전쪽 TSV](handbook-gate.tsv)
- 실제 Studio 화면·인쇄 24건에서 최신 WASM 초기화 응답을 확인했다.
  실제 백엔드는 Canvas2D 20건과 CanvasKit 4건이다. [Studio 결과](studio-consumer.json)
- 전체 nextest 13,951건, Native Skia 라이브러리 4,032건, 자리표시자 2건, 직접 PDF 37건이 통과했다.
  [Rust 검증](rust-validation.json)
