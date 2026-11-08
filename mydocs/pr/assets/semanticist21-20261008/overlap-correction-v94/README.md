# clip 진단·중첩 source frame 보정의 현재 Native 직접 비교

현재 미커밋 production source는 통합 기록 진행77에 연결한다. code HEAD12d96686 위 보정이며 Native CLI default SVG export artifact SHA`c6b4492b91d7b5ed6b8bb5df627538df0aa65539c7cf5bfcf5fea3a4d52a3299`다. native-skia 실행으로 확대하지 않는다.

- rowbreak 원본 `samples/rowbreak-problem-pages.hwp`/독립 `pdf/rowbreak-problem-pages-hwp-2024.pdf`: 실제 총18대18쪽. 영향7/8/9쪽93.01408/98.18550/99.63160%. Native/fresh WASM 전18쪽 모두 최저92.75031%/누락0/18대18쪽이며 actual raster18개가 바이트 동일하다. 현재7–9쪽을 직접 읽어 제26조의 꼬리가8쪽에서 이어지고 제27조 본문과 겹치지 않으며9쪽의 시행령 표로 순서가 이어지는 것을 확인했다. 활자 획·회색 괘선/점선의 작은 차이는 남는다.
- shortcut 원본 `samples/basic/shortcut.hwp`/독립 `pdf/basic/shortcut-2022.pdf`: Native 전7쪽 최저96.30135%, 모든 쪽90% 이상/누락0/7대7쪽이다. 이전 올바른 글꼴의 Native/fresh WASM 증거와 현재 fresh WASM 재검증은 통합 기록에서 구분한다. 새 후보 진단은 paper clip 이후의 상자를 반환한다. 원본 장식 쪽번호의 소유·뒤쪽 master layer를 검사하는 기존 계약은 유지한다.
- 실행 exit0/source 불변: rowbreak 영향3쪽237.071초, shortcut 전7쪽44.185초. TSV/원시 로그/JSON은 ignored output에 보존한다. 이 부분 개선과 대표 PNG를 전체 통합 gate나 모든 PR 승인으로 사용하지 않는다.


## 전18쪽 범위 보완

- 현재 Native 전18쪽은18대18쪽/누락0/최저12쪽92.75031%로 완료했다. fresh WASM 영향7–9쪽은 같은 점수이며 세 PNG가 Native와 바이트 동일하다. WASM 전18쪽도 동일한 최저92.75031%로 완료했다.
- 새 대표 `rowbreak-p011/p012/p018-review.png`와 overlay를 직접 판독했다. 11쪽 끝 목차 표가12쪽 머리의 표 조각으로 이어지며 내용 순서·후행 본문·쪽 번호를 확인했다. 최저12쪽의 문단 시작과 본문 줄바꿈·하단 간격, 마지막18쪽의 두 절과 끝 문장이 같은 쪽에 보존된다. 11쪽 표 머리의 파랑/회색 채우기, 가는 괘선·회색 띠와 작은 기호/활자 획 차이는 남으며 완전 동일 출력으로 보고하지 않는다.


## 실제 backend 소비

현재 Studio PageRenderer Canvas2D의6935 전3쪽은91.99433/95.07682/96.33170%이고 actual JS/WASM 응답·최신package가동일하다. 2쪽을직접판독해열린이어받기표하단/앞표닫힘/뒤내용을확인했다. `issue6935-studio-p002-review.png`/overlay가해당새캡처다. 작은활자획·회색괘선차이는남는다.

현재Native Skia의rowbreak 영향7–9쪽은90.46831/97.79471/99.17244%다. 직접PNG/print profile/compat2024/96dpi이며새 `rowbreak-native-skia-p007/p008/p009-review.png`/overlay를보존한다. 7/8쪽을직접판독해제26조꼬리의이월·제27조소속·외곽과순서를확인했다. 활자획·회색/점선차이는남는다. Native Skia3종공식회귀도모두exit0이며전체통합head승인으로확대하지않는다.

rowbreak fresh WASM 전18쪽도 현재Native와동일한최저12쪽92.75031%/누락0/18대18쪽으로완료했다. font cmap135개/18쪽passed, actual raster18개모두Native와바이트동일하다. 전체통합gate는별도다.
