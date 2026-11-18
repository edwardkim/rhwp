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

## Native 전쪽 완료와 WASM 공급 조건의 추가 확인

Native에서 복원한 issue7905·outline은 fresh WASM의 자동 선택에서 각각 98.68%·99.99%였다.
기존 수용 조건의 software Chrome(`--disable-gpu`)을 적용한 집중 검사는 각각 98.86%·100.00%로 복원됐다.
이 조건의 문서 전체와 같은 base를 다시 비교하며 Native 통과를 WASM 통과로 재사용하지 않는다.

후보 Native 18종·1,771쪽의 [최종 전쪽 TSV 집계](native-protected-final.json)는
보호값 하락·누락·쪽수 불일치 0쪽이다. sample10 763쪽도 수정한 같은 공급 조건으로
전쪽을 다시 산출했으며 최저 99.57%·하락 0쪽이다. 474쪽의 집중 최고값만 전쪽 결과에 섞지 않았다.


## sample10 WASM 757쪽: 이전 수용의 직접 프로그램 공급 재현

앞선 Mac 전용 선택 JSON과 전체 사용자 글꼴 1,233 face를 사용한 WASM은 757쪽이 99.56%였다.
같은 입력·PDF·2px·96DPI·compat2022에서 base/후보와 GPU/소프트웨어를 대조해 모두 재현했다.
base/후보 PNG도 같으므로 현재 기능 변경의 새 회귀로 보지 않지만, 이 조건의 보호 실패는 보존한다.

정본이 연결한 `bd26dedc3ee867bd32a847006bf3bb35d81a2a7e`의 수용 기록은
Mac Native용 `selectedPrograms` JSON이 아닌 `--font-path` 직접 공급이다. 그때 공급한 80개 중
현재 필요한 원본 프로그램 66개를 동일 SHA-256으로 확보해 별도 경로에 공급했다.
미확보 Windows 프로그램 14개를 같은 파일이라고 주장하지 않는다. 후보 전 763쪽의 실제 Canvas
요청은 HFT 7개 계열과 Haansoft Batang이며, 미확보 바탕/TTC·굴림·Courier·Times·Arial 계열의
paint 요청은 없었다. 추가 시스템 글꼴이 같은 이름/별칭으로 등록되는 넓은 공급과 구분했다.

이 조건의 후보 fresh WASM 전 763쪽은 최저 99.57%·공식 보호값 하락 0쪽이며,
474쪽은 100.00%, 757쪽은 99.57%다. 원본·기준·계산 코드·기준 하한은 바꾸지 않았다.
전쪽 한 실행의 TSV를 사용했고 쪽별 최고값을 섞지 않았다. 같은 조건의 base 및 적격 Native
전쪽 확인도 후속 실행에서 완료했으며 아래 최종 집계로 연결한다.

[새 전쪽 후보의 757쪽 review](sample10-original-supply-pages/wasm/review/review_757.png)·
[overlay](sample10-original-supply-pages/wasm/overlay/overlay_757.png)와
[474쪽 review](sample10-original-supply-pages/wasm/review/review_474.png)·
[overlay](sample10-original-supply-pages/wasm/overlay/overlay_474.png)를 직접 판독했다.
제목·본문·명령·쪽번호가 유지되며 기존 ASCII 순서도의 미세 선 차이는 남아 있다.
이는 이번 원 PR 범위의 새 개선이나 RGB 완전 동일성으로 계산하지 않는다.


### Native의 명시적 PUA 공급과 브라우저 준비의 구분

원본 프로그램 66개를 Native에 자동 공급만 한 대조에서는 2쪽이 98.25%였고 목차 머리의
선 폭·가운데 위치가 기준과 달랐다. 이 잘못된 공급 조건의 실행은 진단으로 중단·보존했다.
기준의 원본 CFF 프로그램 20개 선택과, 독립 Print에서 확인한 `HY Junggothic`/`한양중고딕`의
PUA 공급자 `Haansoft Batang`을 함께 명시하자 같은 입력·PDF에서 Native base와 후보의 2쪽은 100%, 474쪽은 100%,
757쪽은 99.57%를 복원했다. 이 세 페이지의 Native PNG는 기존 적격 Native 결과와의
바이트 비교도 [집중 기록](sample10-native-provider-focus.json)에 남겼다.

브라우저는 canonical 글꼴 등록 단계가 PUA 공급자를 준비하며, Native는 이 명시 환경을 쓴다.
공급 방법을 같은 것으로 표기하거나 원본 이외 프로그램으로 점수를 고른 것이 아니다.
제품 코드·기준·판정은 바꾸지 않았고 실제 공급자 차이를 구분해 같은 유효 프로그램으로 재현했다.
이 집중 결과와 별도로 Native 전쪽 실행을 완료했다.

`--font-path`는 Mac의 CoreText 설치 글꼴까지 격리하지 않는다. 따라서 원 수용 프로그램의 선택과
PUA 공급 선언을 함께 고정한 결과로 기록하며, 어느 설정 하나만이 유일한 원인이라고 단정하지 않는다.
이 선언은 해당 보호 조건의 재현용이고 일반 Native의 TTF→TTC→OTF/CFF 우선순위를 바꾸지 않는다.

## 최종 전쪽 확인

[원본 공급 재현 영수증](sample10-original-supply-validation.json)의 base/후보 Native/fresh WASM 네 실행은
각각 763쪽·최저 99.57%·공식 보호 하락 0쪽이다. [최종 보호 집계](protected-final.json)는
18문서 × Native/WASM 총 36조건·3,542행에서 하락 0쪽·상승 0쪽이며 정본 TSV를 수정하지 않았다.
초기 자동 공급 98.25%와 잘못된 넓은 공급의 99.56%는 진단 실패 기록으로만 보존한다.
