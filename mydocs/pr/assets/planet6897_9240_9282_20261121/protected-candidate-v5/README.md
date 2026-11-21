# 보호 문서 후보 전쪽 결과

제품 소스 `65b6fe383c3bd8cbcf83de7909c7a6c24eb32c35`의 Native와 fresh WASM에서 보호 문서 23개의 **46조건·4,046쪽**을 모두 측정했다. 기존 정본의 소수 둘째 자리 raw 실루엣 표시값과 비교해 하락·누락은 0쪽이다. [전쪽 TSV·입력·기준 PDF·생산물 SHA](validation.json) · [정본 보호 TSV](../../visual-protected-baseline/final.tsv)

시행령 `78494-virtual-convergence-industry-decree.hwpx`의 fresh WASM 8쪽에서 **99.99% → 100.00%**를 관측했다. 본문, 영향평가 표, 쪽번호가 한컴 Print의 같은 쪽에 있다. 같은 글꼴 공급의 고정 `devel`도 8쪽에서 100.00%이고, 74쪽 TSV가 후보와 바이트 단위로 일치했다. 수용 당시의 글꼴 공급 설정과 기록 해시가 달라 기존 하한 99.99%를 유지한다. [74쪽 TSV](wasm/decree.tsv) · [기준 대조 영수증](decree-wasm-paired.json) · [8쪽 review](decree-wasm-review_008.png) · [8쪽 overlay](decree-wasm-overlay_008.png)

같은 입력·Print·글꼴 공급 조건에서 고정 `devel`과 후보를 46조건·4,046쪽 전부 대조했다. 45조건의 TSV는 바이트 단위로 같았다. 유일한 공식 표시값 변화는 `sample10` fresh WASM 474쪽의 99.96% → 100.00%다. 편람 HWPX Native 383쪽은 수용 당시 기록한 글꼴 설정으로 재실행했으며, 후보와 고정 기준의 TSV가 바이트 단위로 일치한다. 일부 과거 글꼴 파일은 동일성을 확인하지 못해 현재 동일 공급 조건의 대조와 과거 기록의 재현 여부를 함께 기록했다. [고정 기준 전쪽 대조](fixed-base-comparison.json)

캡처 중 별도 로컬 검토 브랜치의 checkout SHA가 일부 영수증에 기록됐다. 출력은 이 검토용으로 앞서 고정한 Native 실행 파일과 WASM 패키지를 사용했다. 자료를 읽는 sweep과 exporter, 보호 입력·Print의 신원을 대조했다. [checkout 및 출력 신원 감사](validation.json)

편람 HWPX의 fresh WASM은 수용 당시와 같은 글꼴 설정에서 고정 `devel`과 후보의 383쪽 TSV가 바이트 단위로 일치했다. 130쪽의 후보와 기준 RHWP 이미지 및 PDF 이미지의 SHA도 같다. 독립 Print와의 차이는 그림 내부의 미세한 색과 윤곽에 남는다. [편람 전쪽 TSV](wasm/handbook-hwpx.tsv) · [기준 대조 영수증](handbook-hwpx-wasm-paired.json) · [130쪽 review](handbook-hwpx-wasm-review-130.png) · [130쪽 overlay](handbook-hwpx-wasm-overlay-130.png)
