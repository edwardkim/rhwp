# 원 PR의 실제 입력·페이지에 대응하는 최종 증적

[검증 기록](../../verification-record.md)은 제품 소스·Native 실행 파일·fresh WASM·
[실제 공급 조건](conditions.md)·입력·PDF와 최종 대표 파일의 SHA-256을 연결한다. 기존 PDF를 재인쇄하지 않았다.

- [Native 전 14쪽](native_silhouette.tsv)·[fresh WASM 전 14쪽](wasm_silhouette.tsv): 각각 모든 쪽 raw 100.00%.
- Native [4쪽 review](native/review_004.png)·[overlay](native/overlay_004.png), [5쪽 review](native/review_005.png)·[overlay](native/overlay_005.png), [6쪽 review](native/review_006.png)·[overlay](native/overlay_006.png).
- fresh WASM [4쪽 review](wasm/review_004.png)·[overlay](wasm/overlay_004.png), [5쪽 review](wasm/review_005.png)·[overlay](wasm/overlay_005.png), [6쪽 review](wasm/review_006.png)·[overlay](wasm/overlay_006.png).

두 경로의 review와 overlay를 직접 판독했다. 원본 셀 전체 내용과 두 조각을 이어 붙인
내용을 대조했고, 각 조각이 대응 독립 PDF 쪽에 한 번씩 존재함을 확인했다.
첫 소유 줄은 5쪽, 나머지 두 줄은 6쪽, 후속 문단 43 「Ⅱ. 규제의 적정성」은 7쪽에 있다.


원시 render tree·분석 JSON·폰트 catalog·실행 로그는 output에서 관리하며 제출하지 않는다.
[Native 명시적 글꼴 환경](native-font-environment.json)은 검증의 입력 설정이다.
글꼴 파일은 배포하지 않는다. 100.00%는 공식 2px 실루엣 지표이며 RGB 동일성을 뜻하지 않는다.
전체 보호 Native·fresh WASM 및 편람 gate는 [전쪽 보호 기록](../protected/README.md)에서 확인한다.
원격 CI·병합은 이후 실제 결과로 확정한다.

4쪽의 조문 대비표도 새 source833 Native/fresh WASM의 같은 쪽 review·overlay로 직접 판독했다.
두 열의 문장·줄바꿈·점선·밑줄·표 경계와 하단 쪽표시를 보존한다. 글자·선 가장자리의
미세한 농도 차이는 남으며 RGB 동일성을 주장하지 않는다. 새 출력 전14쪽 PNG가 각각
source43 판독본과 바이트가 같다는 재검증 기록은 [리베이스 분석](../../rebase-f823-analysis.md)을 따른다.
