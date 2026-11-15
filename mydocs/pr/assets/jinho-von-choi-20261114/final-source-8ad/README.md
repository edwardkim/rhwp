# JinHo-von-Choi 3건의 최종 로컬 검증

Rust 제품 source는 `8ad1c5c08aa4cd68accbd2af6b9c60bc458d8992`다.
최신 base는 `2d83fa4c8a5ed3513afcde26367fb5ffebeb294c`이며, 실제 비교용으로 빌드한
`e7ee272cfa33a960b0d0d924983d29e6509903e0`와 제품·시각 harness가 같다.
[실제 source 대응](source-equivalence.json), [Native](native-producer.json)·[최적화 WASM](wasm-producer.json)의
원본 생산 SHA와 바이트 해시를 보존한다. 아직 원격 push·새 PR·공개 댓글·merge는 수행하지 않았다.

| 검증 범위 | 경로별 쪽수 | Native 최저 | fresh WASM 최저 |
| --- | ---: | ---: | ---: |
| 바탕쪽·색 없음·괘선·정상 대조군 및 앞선 devel 기능 | 134 | 93.31% | 92.35% |
| 최신 devel의 추가 기능 보존 | 389 | 98.22% | 98.22% |
| RowBreak 첫 조각 원 사례 | 14 | 97.04% | 99.59% |
| 정본 보호 전체 | 1,771 | 98.63% | 98.65% |

원 사례의 직접 렌더 재검증 9쪽은 134쪽과 중복되는 별도 증거다.
편람 HWPX 383쪽도 보호 전체에 포함하며 중복 합산하지 않는다.
[전쪽 입력·PDF·실제 프로그램·TSV](feature-validation.json), [보호 36개 데이터셋](protected-validation.json),
[원 사례의 exact-base 정밀 비교](original-base-paired-results.json)·[전쪽 TSV](original-base-paired.tsv),
[추가 devel 문서의 정밀 비교](latest-base-paired-results.json)·[전쪽 TSV](latest-base-paired-pages.tsv)를 보존한다.
모든 쪽은 90% 이상이고 reference PDF 쪽수와 대응한다. 원 사례와 추가 문서 모두 2%p 초과 하락은 0쪽이다.
[직접 확대 판독](review/README.md)도 완료했다.

## 예외 없는 보호 판정과 하한 상향

갱신 전 정본 `8f073dddd67cf8f44bea7c0315c3f9950ba73641e9393747cdbc52d24b33f9a1` 대비
3,542행의 공식 두 자리 raw 하락·누락은 0행이다. 사용자가 거절한 107행의 기존 차이 예외는 적용하지 않았다.
같은 최종 전쪽 결과에서 상승한 Native 10행·WASM 9행을 새 하한으로 반영했다.
[상향 영수증](../../visual-protected-baseline/updates/pr9070-9072-20261115.json)의 새 TSV 해시는
`43c59cf77a4a86b3744d3df30eb71469de70b584ccc9cb165fa8f7b7884d2e1f`다.
중간 실행의 페이지별 최댓값을 조합하지 않았다. 90%·98%·2%p·공식값 하락 금지 기준도 유지한다.

[실제 `--handbook-gate` 383쪽 실행](handbook-executed-gate.json)은 갱신 전 하한에서 통과했다.
픽셀·소스·프로그램이 같은 완성된 실제 캡처를 [상향된 하한으로 다시 판정](handbook-new-floor-evaluation.json)해
383쪽 모두 통과했다. 별도의 새 렌더 실행으로 바꾸어 기록하지 않는다.

## 필수 실행과 실제 사용자 계약

[17개 필수 단계](local-validation.json): 최소 글꼴 전체 회귀 13,608건 통과·45건 기존 skip,
Native 공식 3종 4,073건 통과·lib의 기존 ignored 12건, fmt·root/WASM32/workspace Clippy·workspace build·manifest 통과.
정적 TTF 형식 보존의 default 집중 21건과 Native 21건, HFT 24건·웹폰트 8건·행 이월 4건도 통과했다.
실제 브라우저 26건·Studio 2,794건 통과(기존 skip 2건), TypeScript·build·WASM binding 검사를 통과했다.
공급 선택 Python 14건·실제 브라우저 등록 2건·보호 하한 계약 21건·새 devel CI 계약 50건도 통과했다.

[실제 CLI의 TAC 5개 텍스트·도움말](cli-text-help.json),
[CLI의 HWP/HWPX 본문 편집·두 번 저장 재열기](cli-repeated-save.json),
[실제 최적화 WASM의 필드 주소·삽입·저장·범위 거부](typed-frame-wasm-functional-repeated-result.json),
[증적 입력 2개의 실제 보안 탐지 6개 결과](extra-input-security.json)를 보존한다.
[입력·독립 PDF 118개의 실제 Git blob 대조](input-inventory.json)는 미커밋 입력 0개다.
상용 글꼴 프로그램 자체는 커밋하지 않았다. 사용자 지정 HBATANG은 SHA-256
`35f84328500fc2c3ee0b148aa75de0ed384bf9eee8dee9148c01b0a11a27fe05`인 실제 파일을 사용했다.
Windows의 Hwp.exe·COM·인쇄 자동화는 실행하지 않았다.


## PR 본문의 원본 TSV·대표 이미지

[문서별 원본 TSV 인덱스](tsv-original-index.json)는 base/후보·Native/WASM의 파일별 SHA-256·바이트 수와 경로를 제공한다. 이미 생성된 silhouette.tsv는 바이트 그대로 보존했고, full 모드만 기존 실제 페이지 metric JSON을 같은 공식 다섯 열·정밀도로 내보냈다. [네 개의 전쪽 통합 TSV 인덱스](aggregate-tsv-index.json)도 역할·문서·쪽을 구분해 원 열을 유지한다. [대표 review·standalone overlay](representatives/receipts.json)는 같은 실제 PNG 쌍에 canonical 함수를 적용한 결과이며 새 렌더나 마스크 변경을 뜻하지 않는다.

## 제출 중 최신 devel 반영

#9093의 Studio 글꼴 별칭 변경을 `1805d8f08c`에서 로컬로 병합하고 새 경로를 실제 브라우저로 재검증했다.
앞선 `2d83fa4c8a` 비교 기록은 그 실행 시점의 근거이며, 현재 base와의 대응·새 검증은
[추가 검증 원장](incoming9093/README.md)을 따른다. 후보 Studio 전체 결과는 2,798 PASS·2 기존 SKIP이다.
