# 원 PR의 실제 입력·페이지에 대응하는 최종 증적

제품 source: `c95f9ade4e8acf9d901c01136ef2b9bacd60c497`. [검증 기록](../../verification-record.md)과 [실제 공급 조건](conditions.md)에 Native·WASM 생산물 및 입력·기존 독립 PDF를 연결했다. PDF를 재인쇄하지 않았다.

[Native 전 14쪽](native_silhouette.tsv)·[fresh WASM 전 14쪽](wasm_silhouette.tsv)은 모두 raw 100.00%다.

| 같은 원본의 대응 쪽 | Native | fresh WASM |
| --- | --- | --- |
| 4쪽 | [review](native/review_004.png) · [overlay](native/overlay_004.png) | [review](wasm/review_004.png) · [overlay](wasm/overlay_004.png) |
| 5쪽 | [review](native/review_005.png) · [overlay](native/overlay_005.png) | [review](wasm/review_005.png) · [overlay](wasm/overlay_005.png) |
| 6쪽 | [review](native/review_006.png) · [overlay](native/overlay_006.png) | [review](wasm/review_006.png) · [overlay](wasm/overlay_006.png) |

새 제품으로 실제 재출력한 Native·WASM 전 28쪽 PNG는 이전 직접 판독 PNG와 각각 바이트가 같다. 따라서 같은 입력·같은 쪽·같은 배율의 기존 직접 판독을 재사용했다. 점수만으로 그림 동일성을 추정하지 않았다.

4쪽의 두 열·줄바꿈·점선·밑줄·표 외곽과 쪽표시를 확인했다. 5쪽은 표 첫 줄 한 개, 6쪽은 나머지 두 줄이며 7쪽에 후속 문단 43 「Ⅱ. 규제의 적정성」이 있다. 두 표 조각을 합친 텍스트가 원본 셀 전체와 같고 대응 PDF 쪽에 각 조각이 한 번만 존재한다.

100.00%는 공식 2px 내용 실루엣 지표이며 RGB 동일성을 뜻하지 않는다. [Native 명시적 글꼴 환경](native-font-environment.json)은 검증 입력 설정이고 글꼴 프로그램 자체는 배포하지 않는다. 원시 render tree·폰트 catalog·실행 로그는 output에만 보관한다.

[보호 전쪽 검증](../protected/README.md)은 별도 완료 결과이며 원격 CI·병합 결과는 PR review에서 확정한다.


## 같은 최신 base의 변경 전 출력

base `f8234f6f90cb25d9601c9d5d762e0a565aea19e4`도 같은 원본·독립 PDF·공급 조건에서 Native와 WASM 모두 15쪽이다. 실제 표 조각은 6쪽 한 줄·7쪽 두 줄이고 후속 제목은 8쪽이다. 후보는 독립 PDF처럼 5·6·7쪽으로 복구한다.

[변경 전 Native 대응 1~14쪽 TSV](base-native_paired_silhouette.tsv) · [변경 전 WASM 대응 1~14쪽 TSV](base-wasm_paired_silhouette.tsv). base의 전체 출력 15쪽 중 PDF와 짝이 있는 14쪽만 계산했다. PDF에 없는 15쪽의 점수를 만들지 않으며 이 변경 전 자료를 정상 통과나 새 보호값으로 수용하지 않는다.

Native 변경 전 [5쪽 review](base-native/review_005.png)·[overlay](base-native/overlay_005.png), [6쪽 review](base-native/review_006.png)·[overlay](base-native/overlay_006.png)를 같은 원본·같은 쪽으로 직접 판독했다.
