# SO-SUEOP의 남은 표시 차이

2026-11-09 KST 직접 확대 판독. 기준 `b768bee1c4`, runtime `4fabb47ae2`, Native producer `d2137e0ccf`(runtime 동일), fresh WASM `4fabb47ae2`. 96 DPI, print profile, compat 2022. 원본과 독립 Print는 기존 저장소 자료다. [전쪽 수치](README.md)를 확인한다.

## 확인한 개선과 남은 범위

- 매핑 8종은 원 source에 없던 문자를 새로 발명하는 변경이 아니다. 독립 한컴 변환본과 실제 저장 source의 본문·각주에서 “/” 31/32개, ‧ 4개, — 1개, ∣ 6개, ↳ 18개, 반각 책괄호 PUA 각 3개가 일치한다. Native·최적화 WASM의 실제 브라우저 저장에서 확인했다.
- [화살표](arrow-p13.png)와 [책괄호](pua-p25.png)는 누락이 복구됐다. 본문·각주의 원 좌표·다른 텍스트의 보존을 함께 대조했다.
- [23쪽 긴 줄표](symbols-p23.png)는 source에 U+2014가 보존되고 양쪽에서 표시된다. Native는 뒤 `입` 글자와 잉크가 겹치는 잔여 문제가 있다. `TextRun`은 x=209.5946667, 다음 글자는 x=216.368로 반각 점유를 소비한다. 이 값은 진단 관측이며 새 px assertion으로 고정하지 않는다.
- 실제 SVG 선언은 긴 줄표에 `한양신명조`/`HanyangSinMyeongJo` 원 메트릭과 95% 장평·음수 자간을 사용한다. 공급한 `SPSMJ.otf`·`HGSMJ.otf`·`HJSMJ.otf`·`ENSMJ.otf`의 Unicode cmap에는 U+2014가 없다. 대체 글리프의 자연 잉크와 원 반각 점유를 대조해야 한다. 원본 파일·특정 숫자에 맞춘 이동이나 전역 음수 자간의 축소로 고치지 않는다.
- Native는 `src/renderer/skia/text_replay.rs`의 원점/fit 경계 → 실제 선택 font의 `measure_str` → 공통 `canvas_cluster_fit`를 소비한다. 공통 fit는 음수 자간을 일반 글리프 축소로 바꾸지 않는 계약이다. 단순 Unicode 분기 추가나 좌표 clamp는 제안하지 않는다. 이 후속 원인의 확정·보정 범위는 현재 승인된 HWP3 이웃 탐색 보정과 구분한다.
- [45쪽](legacy-p45.png)의 본문 굵기·열 사이 흐름 차이는 base부터 존재한다. Native 최저 61.69502→61.69802%, WASM 61.97765→61.97235%이며, 새 매핑의 정당성과 문서 전체 fidelity 통과를 같은 판정으로 취급하지 않는다.

## 글꼴 공급 대조

사용자가 제공한 HFT 변환 OTF 디렉터리, HCR Batang/Dotum TTF, Gulim TTC를 명시했다. 별도로 원 TTF `HMKMM.TTF`(휴먼명조)를 추가한 23·45쪽 대조도 실행했으나 미달을 해소하지 못했다. 글꼴 파일과 원시 SVG는 공개 commit에 넣지 않는다. 이 결과를 해결 불가능한 정식 글꼴 예외 승인으로 바꾸지 않는다.

## 기존 renderer 문제인지 확인한 대조

후보가 만든 파일로만 원인을 분류하지 않았다. 기존 독립 한컴 변환본 [samples/SO-SUEOP.hwpx](../../../../samples/SO-SUEOP.hwpx), SHA-256 `ebbd6f5c86d6eda195bb9c4ba1fc35b329432dcc5ef1dc0fffbfb4542fce74b8`를 **수정 전 base Native renderer**로 열었다. 이 경로는 #8789의 HWP3 Johab decode를 사용하지 않으며 U+2014가 원래 Unicode로 존재한다.

같은 글꼴 공급·print·compat 2022에서 23쪽의 긴 줄표가 뒤 `입`의 잉크와 겹치는 것을 재현했다. baseline의 TextRun은 x=214.1, 폭=6.8 px이며 같은 대체 글리프 문제가 보인다. [독립 HWPX의 base / 원 HWP3의 후보 확대 대조](base-independent-dash-control.png)를 직접 판독했다. 전체 원본·쪽수·fidelity 통과 증거로 이 진단 crop을 대신하지 않는다.

따라서 이 표시 한계는 PR이 새 renderer 코드를 도입해서 생긴 오류가 아니며, 기존 Native의 원 메트릭·fallback 잉크 소비 문제다. Unicode 매핑 8종의 정당성과 이 renderer 한계의 수용 범위는 구분해 결정한다. 이 대조가 자동 시각 예외를 부여하지는 않는다.

## 메인테이너 수용 결정

자동 수치 하락은 Native 최대 0.31024%p, WASM 0.27353%p로 2%p 이내지만 전쪽 90% 조건과 직접 판독은 별도다. 90% 미달과 위 표시 문제는 남아 있다. 2026-11-09 KST 메인테이너가 “기존 차이 한정 예외·2건 원격 처리 승인”으로 결정했다. 예외는 이 문서의 기존 본문 굵기·흐름과 독립 base 대조로 확인한 Native 긴 줄표 fallback 간격에 한정한다. source 문자 복원을 수용하며 표시 문제를 해결 완료로 선언하지 않는다. 새 회귀·데이터 손실·쪽수 불일치·미실행 검사로 범위를 넓히지 않는다.

관련 전체 피델리티 이슈 [#7807](https://github.com/edwardkim/rhwp/issues/7807)은 전쪽 90%·양 형식 검증의 완료 조건을 충족하지 못해 OPEN을 유지한다. PR 수용 예외를 이슈 종료로 확대하지 않는다. [개선·잔여 범위 후속 댓글](https://github.com/edwardkim/rhwp/issues/7807#issuecomment-6738899526).
