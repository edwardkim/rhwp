# 정본 보호 조건 재현 분석

제품 source `167921b3c5`의 Native 18종·1,771쪽을 이전 공식 두 자리 TSV와 대조했다.
초기 조건에서 Helvetica 1쪽·issue7905 2쪽·outline 8쪽·sample10 474쪽이 하락으로 표시됐다.
정본 점수·PDF·반경·이진화 문턱을 낮추거나 바꾸지 않는다.

## 복원한 공급 조건

- Helvetica: Mac Helvetica query를 남긴 예전 JSON은 Print의 Arial과 다르다.
  원본이 선언한 Arial과 실제 프로그램 네 개를 공급하면 base Native·후보 Native·fresh WASM이
  100.00%다. [별도 근거](declared-arial-print-condition.md)를 따른다.
- issue7905: 예전 JSON의 한컴바탕→Batang 강제 치환을 사용한 base·후보 PNG는 완전히 같다.
  원본 설치 글꼴을 자동 선택하면 같은 독립 Print 대비 양쪽 2쪽이 98.86%로 복원된다.
  후보 Native 전 6쪽도 재산출해 공식 값 하락 0쪽을 확인했다.
- outline: 예전 JSON을 쓴 base·후보 PNG는 8쪽에서 완전히 같다.
  원본 설치 글꼴 자동 선택은 양쪽 8쪽 100.00%이며 후보 전 9쪽도 하락 0쪽이다.

## sample10 474쪽 — 실제 미일치 위치와 공급자 복원

base와 후보는 예전 JSON·원본 글꼴 자동 선택 양쪽 모두 99.96%였다.
base binary SHA `a8f9b5c0f8f0ee7c14926a743aab2e4784ebfe57ac2e2506f3c08e5237cbce19`는
직전 보호 기록의 Native producer SHA와 같고 입력·PDF·Visual Sweep 스크립트도 같다.
따라서 이번 일곱 PR의 새 악화로 단정하거나 보호값 100.00%를 낮추지 않는다.

초기에는 하이픈 줄의 회색 차이를 의심했다. 그러나 canonical 판정은 최소 RGB <232이며,
<128 검정 화소의 차이는 공식 미일치 위치를 증명하지 않는다. 실제 2px raw 마스크를
계산한 결과 미일치 25화소는 (376,380)–(380,384)의 5×5 영역에만 있었다.
하이픈 줄·종이 CTM을 원인으로 본 초기 가정은 폐기하고 제품 코드를 바꾸지 않았다.

해당 원문은 문단 17095의 `=` 10자와 U+F0827 25자다. 독립 Hancom Print의
이중선 공급자는 Haansoft Batang이며 반각 전진을 갖는다. 기존 환경 JSON의
`RHWP Hancom Junggothic Symbols`는 이 Mac에 실제 설치된 글꼴이 아니었다.
Native 준비 후 기호의 실제 전진은 0.485em이었고 마지막 끝이 짧아졌다.
이는 원문의 명조 HFT 하이픈 프로그램과 구별한다.

기존 본문 선택을 보존하면서 해당 글리프 공급자만 독립 PDF에서 확인한 설치 원본
Haansoft Batang으로 명시하자 같은 base·후보 Native 474쪽이 모두 100.00%로 복원됐다.
기호 전용 폰트를 새로 만들거나 원문·PDF·점수·반경·임계값을 변경하지 않았다.
[집중 영수증](sample10-symbol-provider/validation.json), [같은 쪽 review](sample10-symbol-provider/review-474.png)·
[overlay](sample10-symbol-provider/overlay-474.png)를 직접 판독했다.
이 집중 결과만으로 전 763쪽과 fresh WASM 보호 검증 완료를 주장하지 않는다.

## WASM 공급 조건의 추가 확인

Native에서 복원한 issue7905·outline은 fresh WASM의 자동 선택에서 각각 98.68%·99.99%였다.
실제 공급 receipt와 Native 선택을 대조 중이며, Native 통과를 WASM 통과로 재사용하지 않는다.
