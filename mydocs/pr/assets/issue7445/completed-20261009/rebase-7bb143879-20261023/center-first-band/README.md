# 원본 첫 띠와 Center 여유의 검사 교정

Issue: #7445 / #7470. 렌더링 구현을 바꾸지 않은 검사 교정 체크포인트다.

- 부모 HEAD: `e30ef94440ee792519ac8ceffa1055330609c058`.
- 원 해결 PR #7567의 통합 `1d6bc70767`과 첫 띠 검사 추가 `0fa99ae38d`를 대조했다.
- 원본 `rowbreak-problem-pages.hwp`의 첫 제목 셀은 Center다. 저장 앞 띠 1000HU와 문단 앞 간격 1000HU가 최소 공간이며, 남은 물리 공간의 Center 여유까지 0으로 고정하지 않는다.
- 현재 실제 앞 여유 14.58667px와 뒤 여유 14.62000px가 직렬화 오차 안에서 대응한다. 제목·국가/절 제목 소속·중첩 꼬리·후속 본문·원본 빈 띠 검사를 유지했다.
- 독립 Print의 첫 제목 기준선은 354.720001pt = 472.960001CSSpx, Native 논리 기준선은 472.933333CSSpx다. 잉크 외곽을 기준선으로 쓰지 않았다.
- 기존 정확히 2000HU라는 검사에서 1 FAIL을 확인했다. 현재는 앞 띠+문단 간격의 최소 보존과 실제 내용의 Center 앞뒤 여유를 검사하며 같은 0.1px 직렬화 정밀도를 유지한다.
- 마지막 내용 끝에는 실제 TextLine 상자를 사용하므로 빈 줄도 소유 높이로 보존한다. 임의의 허용량이나 새로운 문서/쪽 분기를 제품에 추가하지 않았다.
- CLI 조판 세대는 같은 한컴 2024 PDF에 맞춰 명시적 compat2024로 고정했다.
- 전체 18쪽 Native/fresh WASM이 90% 이상이고, 13쪽은 각각 **98.77313/98.77812%**다. 독립 기준선 ±2CSSpx 검사도 같은 기존 함수에 추가했다. 페이지/제목은 원본 내용 소유로 선택하며 화면 좌표로 고르지 않는다.

## 실행과 증거 재사용

검사 교정 전 1건 **0 PASS / 1 FAIL**, 4.102초다. 최종 교정 후 원본 모듈 전체는 **6 PASS**이며 삭제·skip·조용한 return을 추가하지 않았다. 제품 소스 14개 SHA가 Stage69와 같아서 같은 Native/fresh WASM producer의 전쪽 TSV와 PNG를 재사용한다. 새 제품 빌드/캡처라고 보고하지 않는다.

- [HWP Native 전 18쪽](../bitmap-area/rowbreak-hwp-native.tsv) / [fresh WASM 전 18쪽](../bitmap-area/rowbreak-hwp-wasm.tsv).
- [HWPX Native 대조 18쪽](../bitmap-area/rowbreak-hwpx-native.tsv) / [fresh WASM 대조 18쪽](../bitmap-area/rowbreak-hwpx-wasm.tsv).
- [Stage69 producer와 전체 결과](../bitmap-area/README.md).

| 조건 | Native 최저 | fresh WASM 최저 | 미달/쪽수 불일치 |
| --- | ---: | ---: | ---: |
| 원본 HWP 18쪽 | 93.06876% | 92.99291% | 각 0 |
| HWPX 대조 18쪽 | 92.85921% | 92.82532% | 각 0 |

## 13쪽 직접 판독

첫 제목·회색 설명 상자·마지막 소유 줄·셀 하단과 다음 내용의 순서를 확인했다. 제목 아래의 굵은 문장 자간·자형 가장자리, 테두리 위상·배경 색은 미세한 차이가 남는다. 이 교정이 새 화면 개선이나 RGB 완전 일치라는 뜻은 아니다.

- rowbreak-hwp native: [review](rowbreak-hwp-native-p013-review.png) / [overlay](rowbreak-hwp-native-p013-overlay.png).
- rowbreak-hwp wasm: [review](rowbreak-hwp-wasm-p013-review.png) / [overlay](rowbreak-hwp-wasm-p013-overlay.png).
- rowbreak-hwpx native: [review](rowbreak-hwpx-native-p013-review.png) / [overlay](rowbreak-hwpx-native-p013-overlay.png).
- rowbreak-hwpx wasm: [review](rowbreak-hwpx-wasm-p013-review.png) / [overlay](rowbreak-hwpx-wasm-p013-overlay.png).

## 잔여

직전 집중 목록 중 task1725·overlap partition12·7379 캡션은 아직 남았다. C354의 devel 대비 하락과 C356 진단, 브랜치 전체 nextest·범위 gate·최종 devel 비교·실제 소비 검증도 미완료다. 이 기록은 PR 수용 판정이 아니다.
