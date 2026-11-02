# PR #8389·#8397 페이지별 기준 대조

2026-11-03 KST 기준 대조. 이전 통합에서 제출한 페이지별 TSV와 같은 입력·PDF,
기존 글꼴 환경을 사용했다. 판정은 Visual Sweep의 소수 둘째 자리 표시값으로 하며,
상승은 허용하고 하락은 허용하지 않는다. 두 PR에서 겹치는 기준 행은 출처를
남겨 각각 표시한다.

| 경로 | 대상 | 하락 | 완료 여부 |
| --- | ---: | ---: | --- |
| [Native 표시값 대조](native-before-crop-comparison.tsv) | 2,153행 | 0행 | 재단 표시 보정 전 코드에서 17개 문서 산출 완료 |
| [fresh WASM 표시값 대조](wasm-before-crop-comparison.tsv) | 2,153행 | 0행 | 재단 표시 보정 전 코드에서 17개 문서 산출 완료 |

Native 실행 파일 SHA-256은 `5c2c5c23f0d9f30d5dacab78069a6ec689ba3bbb999c1a200e90475f42683990`,
비교 대상 코드의 제품 변경은 `6896862b1cfb638304dc1106915fcbc7bbbb9d03`과 같다.
기준 표 SHA-256은 `21e3ca2a3035927bfbf5c17710657ea2b2b8691025c6fb55571fdc781690384e`다.
WASM 바이너리 SHA-256은 `c7e6eb6f43570abcbdda1920f8baea208858cc883e1fb4f87f9876922d7f1ede`이고,
WASM 대조 TSV SHA-256은 `a1968c0ae8c3920d8c816beb70be5d566a292841b5646de6dfdf7c64098a72b0`이다.
17개 문서의 각 실행 종료 코드 0과 완료 manifest를 확인했다.
이 기록은 이후 재단 표시 및 저장 보정의 최종 후보 대조를 대신하지 않는다.

## 2026-11-03 현재 코드 대조

제품 코드 `d56930dda723fff1c4e45675979b44604997f0e2`로 빌드한 Native
실행 파일의 SHA-256은 `7d35357fe91cf213bb4143675a7c148bf3f6a9e4facbf93fea765081661b25c5`다.
같은 입력·독립 Print PDF·기존 대체 글꼴 설정으로 두 PR의 기준 2,153행을
전부 다시 측정했다. [Native 최종 표시값 대조](native-final-comparison.tsv)의
2,153행 모두 기준 이상이며 하락·미완료는 각각 0행이다. 대조 TSV SHA-256은
`1551697a713dfae8c508449bebec076270f165b3b9d881439c8f3735f42ce744`다.
같은 제품 코드에서 새로 빌드한 fresh WASM 바이너리의 SHA-256은
`eda26e7ad93a87c7e9aed712e83ae0cddce993aa9939d25cfaf427c8cbe685c1`이다.
같은 입력·독립 Print PDF·기존 대체 글꼴 설정으로 두 PR의 기준 2,153행을
전부 다시 측정했다. [fresh WASM 최종 표시값 대조](wasm-final-comparison.tsv)의
2,153행 모두 기준 이상이며 하락·미완료는 각각 0행이다. 대조 TSV SHA-256은
`a1968c0ae8c3920d8c816beb70be5d566a292841b5646de6dfdf7c64098a72b0`이다.
이 대조는 원 PR 입력의 최신 `upstream/devel` 대비 2%p 하락 게이트와 별도로 수행했다.
