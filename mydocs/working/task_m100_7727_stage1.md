---
kind: working
status: active
canonical: mydocs/working/task_m100_7727_stage1.md
last_verified: 2026-11-15
---

# #7727·#7729·#7730·#7731·#7732 원본 피델리티 분석과 보정

갱신: 2026-11-15T01:24:00+09:00.

Issue: #7727, #7729, #7730, #7731, #7732.

[분석·검증 증적 목차](../pr/assets/issues7727_7732_20261114/README.md)에서 최신 전쪽 TSV,
원인별 근거, 같은 쪽의 review·overlay를 바로 찾을 수 있다. 이 보고서 앞부분은 현재 판정,
`이전`·`이력`으로 표시한 부분은 해당 보정 단계의 관측이다.

## 목표와 현재 판정

다섯 이슈의 실제 원본을 같은 독립 한컴 Print PDF와 대조하여 **모든 쪽 98% 이상**으로
복구하고, 가능한 차이는 100%까지 개선한다. 98% 이상으로 검증된 범위에는 독립 근거에
맞는 회귀 검사를 추가한다. 미달 쪽을 검사에서 빼거나 기준 PDF·글꼴·일치율 판정을 바꾸지 않는다.

브랜치 `task/7727-related-fidelity`, 고정 base/parent는
`53da4d2da28ae0cacf8feac3a989cc6a69fb2622`다. 현재 구현·검증이 진행 중이며 PR 제출 준비가
완료되지 않았다. 아래 수치는 해당 생산자에 한정한다. 뒤의 변경으로 앞선 전쪽 결과를
새 후보의 결과로 바꾸어 부르지 않는다.

## 최신 주석 소스 검증과 기존 회귀의 독립 기대값 보정

제품 소스 `602b0595cac811a94e36858df79b0f9a7501a1cc`의 Native와 fresh WASM은
각389쪽을 완료했고 모두98% 이상이다. CBTA 최저98.11/98.08%, 권고안98.69/98.68%,
시험지99.49/99.49%, API99.55/99.55%, 각주 표98.75/98.75%다.
[생산자·전쪽 TSV·직접 판독](../pr/assets/issues7727_7732_20261114/note-prefix-final/validation.json)에
실행 코드·입력·PDF·정상 설치 글꼴 조건을 연결했다. fresh WASM은 Mac 로컬 대체 빌드
`--target web --no-opt`이며 Docker 최적화 빌드가 아니다.

같은 소스의 각주37·38쪽 Native/fresh WASM review와 overlay를 직접 다시 판독했다.
37쪽24)와25)는 각 본문 뒤의 자기 줄에서 한 번씩 표시되고, 38쪽 병합 꼬리도 유지된다.
원래 수집기가 `TextRun`만 읽어 typed 번호를 빠뜨린 문제와 독립 PDF와 다른 번호 순서를
함께 바로잡았다. 기대 순서는 `정책실현24)재정조달25)`이며 원문 소유·전체50쪽·각주 꼬리·
9·13·38쪽의 기존 정상 괘선 검사는 그대로다. 각주 표50쪽 모두 양쪽98% 이상이므로
지침의 기존 검사 변경 선행 조건을 충족했다. 독립 최소 글꼴 진단은2 PASS이며,
정식 Cargo suite와 전체 회귀를 현재 변경에서 다시 실행한다.

Clippy는 `row + 1 <= first + row_span`에 `int_plus_one` 오류를 보고했다. 원본 행 번호와
걸침 범위의 같은 포함 조건인 `row < first + row_span`으로 바꾼다. 높이 계산·범위·판정 기준은
변경하지 않는다. 이후 최종 검증은 새 code head로 고정하며 앞선602b 결과와 구분한다.

모든 필수 검증과 병합을 완료한 뒤 #7445에서 이번에 완전히 해결한 서브이슈를 닫고
본문에도 완료를 반영한다. 부분 해결 또는 미검증 항목을 닫지 않는다. 현재는 전체 회귀·
lint/build·보호 TSV·편람·실제 소비 경로가 남아 있어 이슈를 닫거나 PR 준비 완료로 보고하지 않는다.

## 이전 번호 문단 보정: 원본 전 389쪽의 98% 목표 충족

parent `fd9207f300` 뒤의 번호 문단 후보는 Native·fresh WASM **각 389쪽**을 모두 다시
출력했다. CBTA 96쪽은 **97.00/96.99% → 99.72/99.74%**로 개선됐다.
원본 389쪽 중 Native는 이 한 쪽만 상승하고 나머지 388쪽의 공식 표시값은 같다.
입력·독립 Print PDF·글꼴 조건·DPI·판정 문턱과 원본 쪽수는 유지했다.

| 문서 | 쪽수 | Native 최저 | fresh WASM 최저 | 98% 미만 |
| --- | ---: | ---: | ---: | --- |
| CBTA | 204 | 98.11% (2쪽) | 98.08% (2쪽) | 없음 |
| 권고안 | 22 | 98.69% (7쪽) | 98.68% (7쪽) | 없음 |
| 영어 시험지 | 8 | 99.49% (3쪽) | 99.49% (3쪽) | 없음 |
| API | 105 | 99.55% (97쪽) | 99.55% (97쪽) | 없음 |
| 각주 표 | 50 | 98.58% (37쪽) | 98.58% (37쪽) | 없음 |

현재 번호·본문 시작을 반영하는 공통 채움은 두 줄을 유지하면서도 `교육훈련` 뒤로
문자 경계를 옮길 수 있다. 기존 경로는 마커 높이가 이미 맞고 줄 수가 같다는 이유로
이 결과를 버렸다. 같은 수·높이의 줄은 새 문자 경계와 **원본의 세로 슬롯 원점**을 함께
발행하도록 했다. 번호 없는 문단·명시적 개행·다른 물리 프레임·다른 높이는 이 가로 캐시
보정으로 재해석하지 않는다. 기존 HCI Poppy 폭 표와 글꼴 프로그램 선택도 바꾸지 않았다.

설치 글꼴 없이 검출하는 새 관계형 반례는 보정 전 **1 PASS / 1 FAIL**, 보정 후 **2 PASS**다.
정식 suite의 Mac 최소 글꼴 검증은 기존 번호 문단·내어쓰기·HCI Poppy 계약과 이 브랜치의
핵심 검사를 합해 **23 PASS / 0 FAIL**, 4.110초다. 선택 밖 2,778건이며 전체 회귀가 아니다.
CBTA 118·162·166쪽은 각각 양쪽 98.44%·99.94%·99.95%로 자격을 확인했으므로,
독립 PDF 그림 테두리 네 변과 종료 행의 닫는 선을 **±2 CSSpx**로 추가 보호했다.
기존 소유·높이 관계 검사도 유지했다. 이 고정 좌표 검사 역시 위 최소 글꼴 실행에서 통과했다.

[전쪽 TSV·생산자·회귀·독립 좌표](../pr/assets/issues7727_7732_20261114/marker-partition/cause.json),
96쪽 Native [review](../pr/assets/issues7727_7732_20261114/marker-partition/native-p096-review.png)·
[overlay](../pr/assets/issues7727_7732_20261114/marker-partition/native-p096-overlay.png),
fresh WASM [review](../pr/assets/issues7727_7732_20261114/marker-partition/wasm-p096-review.png)·
[overlay](../pr/assets/issues7727_7732_20261114/marker-partition/wasm-p096-overlay.png)를 함께 확인한다.

98% 목표는 이 원본 범위에서 충족했지만 모든 쪽이 100%라는 뜻은 아니다. CBTA 2쪽의
목차 쪽번호에는 저장된 명시적 RIGHT 탭 전진과 현재 정지점 재계산 차이가 남아 있어 추가
원인을 확인한다. 관측한 탭 확장은 `[1868,0,515,32,32,32,9]`, 현재 정지점은
642.5333 CSSpx이며 실제 PDF 쪽번호가 더 앞에 있다. 이 차이를 글꼴 불가능 예외로
단정하지 않는다. 전체 회귀·필수 lint/build·정본 보호 TSV·편람·실제 backend 검증도
남아 있으므로 PR 제출 준비 완료로 판정하지 않는다.

## 추가 판독: 각주 표 37쪽의 주석 전용 줄 누락

현재 전쪽 98% 목표를 충족한 뒤 최저 쪽의 review·overlay를 다시 판독했다. 각주 표 37쪽의
`24)`와 `25)`는 모두 Native bbox `(384.3, 884.2, 12.0, 13.3)`에 겹쳐 있다.
이는 글꼴 윤곽이나 색상 경계만의 차이가 아니다. 원본 셀 row44/col2의 두 문단은 각각
본문 네 글자 뒤에 주석 전용 저장 줄을 가지며, 독립 한컴 2020 Print에서도 두 참조 번호는
각자 다음 줄에 있다. PDF 기준선은 각각 688.2878pt·720.2928pt다.

기존 `97e0d9d5ac`의 `NoteLineSlots`가 원시 줄 소속을 계산하지만, 현재 paint 경로는
텍스트 run이 없는 줄에서 번호를 방출하지 않는다. 뒤의 셀 전체 폴백이 문단별로 다시 시작하는
`char_start=0`을 구분하지 못해 두 번호를 첫 문단의 끝에 삽입하는 것을 확인했다.
공통 참조 paint와 서로 다른 저장 줄 소속을 검사하는 관계형 반례는 실제 **0 PASS / 1 FAIL**이다.
원본 글꼴 설치나 절대 px를 검사 조건으로 삼지 않는다.

[원인·원본/PDF SHA·실패 증거](../pr/assets/issues7727_7732_20261114/note-only-line/cause-before.json),
같은 37쪽의 [review](../pr/assets/issues7727_7732_20261114/note-only-line/before-native-p037-review.png)·
[overlay](../pr/assets/issues7727_7732_20261114/note-only-line/before-native-p037-overlay.png)를 보존했다.
진행 중인 전체 회귀는 이 보정 전 `006f3ca043`의 실행이며, 이 새 반례는 그 실행의 준비 목록에 없다.
새 반례와 보정을 포함한 최종 필수 검증을 별도로 완료한다.

### 주석 전용 줄의 공통 폭·paint 보정: Native 검증 완료

텍스트 run이 없는 줄도 `NoteLineSlots`가 정한 자기 주석 슬롯을 소비하도록 했다.
`note_reference::empty_line`에서 제어의 원시 주소에 맞는 글자모양을 골라 공통 참조를 만들고,
폭 계산과 실제 paint가 그 참조를 함께 사용한다. 첫 줄 접두부가 이미 그린 번호는 제외한다.
이제 두 번호는 서로 다른 원본 줄에서 `FootnoteMarker`로 한 번씩 생성되며, 기존 셀 전체 폴백이
다른 문단의 같은 `char_start`를 찾아 중첩시키지 않는다.

새 관계형 반례는 같은 최소 글꼴 격리의 독립 rustc 실행에서 **보정 전1 FAIL → 보정 후1 PASS**다.
현재 제품 소스 Native 전389쪽도 완료했다. 각주37쪽 **98.58%→98.75%**,39쪽
**99.83%→99.92%**이며 나머지387쪽의 공식 표시값은 같다. 쪽수와 원본·PDF·판정 조건을
유지했다. [Native 전쪽·생산자·실패/통과 기록](../pr/assets/issues7727_7732_20261114/note-only-line/native-after.json),
37쪽 [review](../pr/assets/issues7727_7732_20261114/note-only-line/after-native-p037-review.png)·
[overlay](../pr/assets/issues7727_7732_20261114/note-only-line/after-native-p037-overlay.png)를 함께 판독했다.
새 소스의 fresh WASM과 정식 Cargo suite 검증은 아직이며, 이전 WASM 결과를 재사용한 완료 선언은 하지 않는다.

## 전체 회귀에서 확인한 네 실패와 독립 탭 대조군

제품 보정 전 `006f3ca043`의 최소 글꼴 전체 nextest는 **13,559실행 / 13,555 PASS /
4 FAIL / 45 skip**, 실제 테스트372.282초로 끝났다. 새 주석 줄 반례·보정은 이 실행에 포함되지
않았다. [실행 head·실패 목록·로그 SHA](../pr/assets/issues7727_7732_20261114/full-nextest-before-note-repair.json)를
보존했으며, 실패를 수용하거나 검사를 제거하지 않는다.

- #8797: 결재 표→문서번호 표의 저장 간격141.5px가130.1px로 줄었다. 고정 base의
  두 표 y는174.1/315.6, 후보는174.1/304.2다. 원 해결 `2998f76312`의 쪽 원점 계약을 추적한다.
- #6844: 목차 쪽번호 오른쪽 끝의 분산이1.0px 허용을 넘었다. 같은 런 `\t8`의 끝이
  새 저장 탭 처리로692.4px가 됐으며, 나머지 정렬 단위는692.7~693.7px다.
- #1921: 규제영향분석서의 독립37쪽에 대해 base37쪽→후보38쪽을 확인했다.
- 병합 표의 짧은 행: 교육과정 지도165쪽은 base에서120행의 첫 줄을 소유하지만,
  후보는120행 전체를 다음 쪽에 보낸다. 고정 base의 종료 행121·끝 컷과 후보 종료 행120·빈 컷을 확인했다.

CBTA2쪽의 탭 분석에서는 ‘유효한 저장 폭이면 항상 우선한다’는 가설을 독립 Print로 대조했다.
같은 문서 안에서 RIGHT 정지는 고정하고 저장 폭만 바꾼 세 줄의 숫자 원점이 모두 같았다.
HWPX 표본과 저장 줄을 가진 HWP 표본 양쪽에서 확인했으며, 출력 제품은11.0.0.9136·Print method0이다.
따라서 이 근거로 공통 탭을 무조건 저장 폭 우선으로 바꾸지 않는다. 두 형식의 절대 원점이
같다고 주장하지 않으며 원본 CBTA의 원인도 이 대조군만으로 확정하지 않는다.
[실제 입력/PDF SHA·MCP 영수증·문자 원점](../pr/assets/issues7727_7732_20261114/stored-tab-control/cause.json)을
정식 보관했다. 처음 만든 CBTA 파생 HWP는 문서 열기 단계900초 timeout으로 실패했으며,
그 실행을 인쇄 성공이나 정렬 판정의 근거로 쓰지 않는다.

### 네 실패의 단계별 격리와 먼저 복구한 두 계약

저장한 일곱 Native 생산자를 같은 입력에 적용했다. 고정 base부터 RGB 채널 보정까지는
규제영향분석서37쪽과 교육과정 지도165쪽의120행 첫 줄 컷이 유지된다. `77f6bb7418`의
번호 문단 갱신부터38쪽·120행 통째 이월로 바뀐다. 따라서 앞선 완료 행 높이 보정을
원인으로 보거나 되돌리지 않는다. [생산자별 실제 결과](../pr/assets/issues7727_7732_20261114/full-regression-repair/full-failures-producer-bisect.json)를 보존했다.

#8797은 원본 첫 줄852HU와 재구성 뒤 첫 줄0HU의 좌표를 혼용한 문제다. 원본 사다리로
문단 앞 간격을 인증하되, 후속 배치에 전달하는 원점은 현재 줄 좌표에서 같은 간격을 뺀다.
원 해결 `2998f76312`의 결재 표→문서번호 표 간격141.5px 검사는 수정 없이 통과했다.

#6844는 현재 칸 밖의 명시적 RIGHT 정지에서 새 가드가 저장 전진을 우선하면서,
쪽번호8의 오른쪽 끝만692.4px로 당긴 문제다. 공통 정지점 계산을 다시 사용하도록 해당
가드를 제거했다. 기존 목차 오른쪽 끝 분산 검사는 기대값 변경 없이 통과했다.

같은 Native Skia 라이브러리에 연결한 최소 글꼴 독립 실행은 **3 PASS / 2 FAIL**이다.
위 두 기존 검사와 새 주석 줄 검사는 PASS이며, 규제영향분석서·교육과정 지도는 아직 FAIL이다.
이는 정식 전체 suite 결과가 아니며 [실행 로그 SHA·라이브러리 SHA·반환 코드](../pr/assets/issues7727_7732_20261114/full-regression-repair/four-regressions-standalone-result.json)를
기록했다. 새 가로 문자 경계 발행이 원본 물리 슬롯에 미치는 영향을 추가 추적하고 있다.

### 물리 슬롯을 유지하는 문자 경계 갱신과 네 실패 복구

추가 진단에서 규제영향분석서의 번호 문단은 문자 시작 위치만 `0/20/40/60/80`에서
`0/20/40/61/81`로 달라지고 나머지 줄 속성은 모두 같았다. 교육과정 지도에서도 같은
높이의 문자 경계 이동을 확인했다. 그러나 overlay 등록이 문자 경계 변화만으로 셀·조상 표의
저장 물리 프레임을 무효화하여, 기존 쪽 컷을 다시 용량 컷으로 계산했다.
[줄 속성 진단과 로그 SHA](../pr/assets/issues7727_7732_20261114/full-regression-repair/physical-slot-preservation-diagnostic.json)를 보존했다.

번호 문단의 같은 높이 갱신은 **원본 물리 슬롯 전체를 유지하고 문자 시작 위치만** 새 채움에서
가져온다. overlay는 줄 수·전체 줄 속성·세로 원점 출처가 모두 같은 유효한 원본 슬롯을
인증할 때만 쪽 프레임을 보존한다. 높이·폭·기준선·태그·슬롯 수가 달라지거나 원본 출처가
없으면 기존 프레임 무효화 경로를 유지한다. 원본 IR과 독립 PDF는 바꾸지 않았다.

보정 후 여백1·목차2·규제영향분석서4·짧은 행24·주석 줄1·번호 문단2 검사는
최소 글꼴 독립 Native Skia 실행에서 **34 PASS / 0 FAIL**이다. 전체 회귀에서 확인한 네 실패와
기존 양성·음성 대조군이 모두 통과했고, 번호 문단 개선 관계도 유지됐다.
[반환 코드·로그·라이브러리 SHA](../pr/assets/issues7727_7732_20261114/full-regression-repair/physical-slots-standalone-result.json),
[검사 수 요약](../pr/assets/issues7727_7732_20261114/full-regression-repair/focused-after-summary.json),
[실제 Native 생산자](../pr/assets/issues7727_7732_20261114/full-regression-repair/physical-slots-producer.json)를 연결한다.

이 결과는 독립 rustc 실행이다. 새 소스의 원본389쪽 Native/fresh WASM과 정식 전체 suite,
필수 lint/build·보호 TSV·실제 backend 검증까지 완료해야 PR을 제출한다.

### 같은 소스의 Native389쪽 재검증

`17301f0133`에 해당하는 실제 소스를 고정한 새 Native에서389쪽 모두98% 이상이며 쪽수도
각204/22/8/105/50쪽이다. 최저값은 CBTA98.11%(2쪽), 권고안98.69%(7쪽),
영어 시험지99.49%(3쪽), API99.55%(97쪽), 각주 표98.75%(37쪽)다.
[새 Native 전쪽 TSV와 앞 단계 대조](../pr/assets/issues7727_7732_20261114/full-regression-repair/native-comparison.json)를 보존했다.
주석 줄 보정 뒤 결과와 비교하면 권고안4쪽만99.96%→99.86%이고 나머지388쪽은 같다.
이0.10%p 차이를 숨기거나 판정 문턱을 바꾸지 않는다. 같은4쪽의
[review](../pr/assets/issues7727_7732_20261114/full-regression-repair/native-recommendation-p004-review.png)·
[overlay](../pr/assets/issues7727_7732_20261114/full-regression-repair/native-recommendation-p004-overlay.png)를 직접 판독했고,
본문·쪽수 누락은 없지만 목차의 인라인 쪽번호8 위치 차이는 남아 있다. 전체 회귀와
독립 RIGHT 탭 대조군을 함께 확인하며, 모든 쪽100%라고 보고하지 않는다.

이 커밋의 최소 글꼴 정식 전체 nextest는 [13,558 PASS / 2 FAIL / 45 skip](../pr/assets/issues7727_7732_20261114/full-regression-repair/full-nextest-17301-result.json),
테스트370.316초로 완료했다. 앞선 네 실패는 모두 복구됐고 아래의 각주 관련 두 반례가 남았다.
이 실행의 필수 lint/build·Native Skia 후속 단계는 실패 때문에 시작하지 않았다.
새 제품 보정과 독립 출력 검증을 마친 뒤 정식 전체 검증을 다시 진행한다.

### 전체 회귀의 새 반례: 빈 문단에 두 번째 각주를 넣을 때 중복 방출

`17301f0133`의 전체13,560건 실행 중 기존
`footnote_shape_api::new_footnotes_follow_the_section_shape`가 새 실패로 확인됐다.
원 해결은 `9fe4a4df5e`의 각주 모양·새 각주 번호 계약이며, 후속 `7cbcdee652`의
본문 표시 경로도 함께 확인한다. 기존 검사는 그대로 유지한다.

실제 API로 마지막 빈 문단에 각주를 넣으면 controls 두 개의 논리 위치는 `[0,1]`이다.
선두 접두부는 가시 글자가 없는 문단의 두 제어를 모두 위치0에서 한 번씩 그린다.
그러나 새 주석 전용 줄의 제외 조건은 원래 논리 위치를 그대로 비교하여 두 번째 제어를
다시 그렸다. 따라서2쪽에 `[①],[②]` 대신 `[①],[②],[②]`가 생긴다.
[실제 원인·원 해결 SHA·진단 로그 SHA](../pr/assets/issues7727_7732_20261114/full-regression-repair/inserted-empty-paragraph-note-duplicate-before.json)를 기록했다.
전체 실행에서 확인한 두 실패 중 이 항목은 실제 제품 중복이며, 다른 하나는 아래의 기존
TextRun 전용 수집기와 잘못된 번호 순서다. 제품에서는 선두 표시와 빈 줄 제외가 공통
`leading_position`을 사용하도록 수정했다. 빈 문단의 논리 제어 칸0·1은 모두 선두0에
놓는 기존 의미를 두 소비자가 공유한다.

새 Native Skia 최소 글꼴 독립 실행은 기존 각주 모양6개·주석 전용 줄1개를 포함하여
[40 PASS / 0 FAIL](../pr/assets/issues7727_7732_20261114/full-regression-repair/note-prefix-focused-summary.json)이다.
[실행 코드·로그 SHA](../pr/assets/issues7727_7732_20261114/full-regression-repair/note-prefix-standalone-result.json)와
[새 Native 생산자](../pr/assets/issues7727_7732_20261114/full-regression-repair/note-prefix-producer.json)를 고정했다.
이는 정식 전체 suite 통과가 아니며, 새 Native389쪽과 fresh WASM389쪽을 확인하고 있다.

1937의 기존 수집기는 `TextRun`만 읽어 새 typed `FootnoteMarker`를 빠뜨린다. 기존 기대
문자열도 `정책실현24)25)재정조달`이지만, 독립 Print37쪽의 실제 순서는
[정책실현→24)→재정조달→25)](../pr/assets/issues7727_7732_20261114/full-regression-repair/annotated-cell-independent-pdf-order.json)다.
두 입력 경로(samples와 issue7445 asset)의 SHA는 같은
`a3a075594994e4741a7fbe973bc230d95ae75fdee6578ac6017cec0d52844990`임을 확인했다.
새 출력의 양쪽 전쪽 자격 확인 뒤 typed 주석과 이 독립 순서를 검사하도록 보정한다.
쪽수·원문 소유·누락·중복·겹침·기존 정상 좌표 검사는 유지한다.

## 이전 분석: 권고안 4쪽의 배경 채널 보간

parent `6197d8e680` 뒤의 후보는 Native **원본 389쪽과 독립·기존 대조군 12쪽, 총 401쪽**을
모두 출력했다. 권고안 4쪽은 **97.44% → 99.96%**, 전 22쪽 최저는 **98.69% (7쪽)**다.
새 양방향 색상 표본 6쪽과 기존 원형 그러데이션 표본 6쪽은 각각 전쪽 **100.00%**다.
CBTA 전 204쪽은 96쪽 **97.00%**만 목표 미달이며, 영어 시험지 8쪽·API 105쪽·각주 표
50쪽의 최저는 각각 **99.49%·99.55%·98.58%**다. 새 fresh WASM도 원본 389쪽과 대조군 12쪽을 모두 완료했다. 권고안 4쪽은 **99.96%**,
전 22쪽 최저 **98.68% (7쪽)**, 두 대조군 각 6쪽은 **모두 100.00%**다. CBTA는
96쪽 **96.99%**, 나머지 세 원본 최저는 Native와 같은 **99.49%·99.55%·98.58%**다.
아래 완료 행 단계의 WASM 수치는 이전 생산자이므로 이 단계와 구분한다.

권고안의 raw 미일치 4,821픽셀 중 4,748픽셀이 색상 경계 보정에서 해소됐다. 이를 통과
근거로 쓰지 않고 원본 배경과 PDF의 동심원 fill을 대조했다. 공통 `lerp_color`가 최종
RGB 값을 반올림하여 회색 띠를 1단계 어둡게 만드는 것이 실제 원인이었다. 원본 글꼴을
명시한 대조군의 결과가 같으므로 글꼴 교체로 이 차이가 해결된다는 가설은 기각했다.

기존 정상 원형 그러데이션 표본의 크기·셀·내용을 유지하고 RGB 방향과 7/50/255단계만
바꾼 [독립 HWPX](../../tests/fixtures/issue7729/gradient-band-color-probe.hwpx)를 만들었다.
[한컴 2020 Print PDF](../../pdf/issue7729/gradient-band-color-probe-2020.pdf)의 MCP job은
`6c8ae335-f880-41ed-9f83-0687d078a1c4`, 제품은 **11.0.0.9136**, Print method는 **0**이며
6쪽·52,675바이트다. PDF 저장/내보내기나 웹한글기안기 출력이 아니다.

독립 PDF에서 210→40 채널의 1/6 지점은 **182**, 역방향 40→210은 **68**이었다.
따라서 절대값을 일괄 올림하거나 버리는 규칙이 아니라, 시작색에 대한 **부호 있는 변화량의
소수부를 0 방향으로 버린 뒤 더하는 규칙**으로 공통 보간을 수정했다. 띠 수·위치·표 크기와
판정 문턱은 바꾸지 않았다. 중앙의 같은 색 원 두 개는 같은 띠이므로 띠 수를 늘리지 않았다.

새 고정형 RGB 검사 2개는 독립 PDF의 실제 fill을 기대값으로 사용하며, 독립 rustc 실행에서
**보정 전 2 FAIL → 보정 후 2 PASS**다. px 좌표를 고정한 검사가 아니다. 정식 파생 suite의 최소 글꼴 Native Skia 검증도 기존 10개를 포함해 **12 PASS / 0 FAIL**,
3.385초로 완료했다. 선택 밖 1,707건이며 전체 회귀가 아니다. 처음 nextest의 `-j`와
`--test-threads` 중복 지정은 실행 전 오류였고, 빌드 잡은 환경변수 8개로 분리해 재실행했다.

[생산자·MCP 영수증·전쪽 Native TSV](../pr/assets/issues7727_7732_20261114/gradient-channel/cause.json)와
4쪽 [review](../pr/assets/issues7727_7732_20261114/gradient-channel/native-p004-review.png)·
[overlay](../pr/assets/issues7727_7732_20261114/gradient-channel/native-p004-overlay.png)를 함께 보존했다.
분석의 다음 대상은 CBTA 96쪽의 제목 줄바꿈이다. `ICAO` 전진은 실제 Native에서 약54.5px이며
독립 PDF의 뒤따르는 한글 시작점은 약2.42px 앞선다. 설치된 HCI Poppy의 보통/굵은 면 폭은
각52.10/54.42px이나, 이것만으로 굵은 면 선택이 원인이라고 확정하지 않는다. CLI와 같은
글꼴 공급·OTF 측정 준비를 적용한 뒤 실제 줄 구성과 글꼴 프로그램을 확인한다.

Native·fresh WASM의 같은 쪽 [review](../pr/assets/issues7727_7732_20261114/gradient-channel/wasm-p004-review.png)·
[overlay](../pr/assets/issues7727_7732_20261114/gradient-channel/wasm-p004-overlay.png)를 함께 판독한다.

## CBTA 96쪽 보정 전 원인 분리 기록 (이력)

CLI와 같은 문서 글꼴 환경을 적용하고 `prepare_declared_font_fallbacks`·
`prepare_native_pua_fonts`·`prepare_native_otf_fonts`를 마친 실제 Print layer와 측정 trace를
확인했다. `ICAO`는 HCI Poppy의 굵은 전진으로 **54.4533 CSSpx**이고, 뒤 한글은
휴먼명조의 **1em** 전진이다. 늘어난 단어 공백은 약 **4.1644px**의 양쪽 정렬 배분이다.
이 결과만으로 한글 글리프에 무조건 5%가 더해진다고 해석하면 안 된다.

독립 PDF의 첫 `ICAO` 원점과 뒤 한글 원점 차이는 **52.16 CSSpx**다. PDF의 F4 사전은
Type0/Identity-H이며 `/BaseFont /HCI Poppy`의 공백이 escape되지 않아 MuPDF의 일반
글꼴 추출이 실패한다. 이 오류를 두부 글자나 Type3 폭으로 오해하지 않고 원본 content의
600DPI 변환 행렬·TJ와 한글 시작점을 읽었다. 원본 PDF 바이트는 변경하지 않았다.

현재 저장 줄은 `교육훈` / `련 단계 간 비교`, Print는 `교육훈련` / `단계 간 비교`다.
원 해결 계약 `fa7aefdf10`(#8105)은 번호 문단의 낱말 꼬리를 현재 프레임에서 다시 채우지만,
현재 공통 soft-wrap 투영은 일반적으로 **줄 수 감소**를 수용 조건으로 사용한다.
출력용 진단 코어에서 해당 문단만 정식 `reflow_body_paragraph_range_native`로 다시 채우면
다음 줄 시작이 원본 **34 → 36**으로 옮겨 `교육훈련` 뒤 공백까지 첫 줄이 소유한다.
원본 파일과 제품 코드는 바꾸지 않은 원인 분리용 실행이며 해결 결과가 아니다.
이번에는 두 줄을 유지하면서 경계의 한 글자가 이동하므로, 글꼴 차이와 이 캐시 수용 조건을
분리하여 확인해야 한다. 이는 아직 확정 원인이나 보정 완료가 아니다.

HCI Poppy의 원본 굵은 폭 표는 `17225375ff`의 별도 HFT·Print 근거와 기존 회귀가 보호한다.
이번 PDF의 두 원점만으로 그 표를 일반체 폭으로 일괄 바꾸지 않는다. 같은 프레임·번호·
언어 슬롯의 독립 표본에서 줄 경계와 실제 문자 전진을 확인한 뒤 공통 소비 경로를 보정한다.
[글꼴 준비 후 관측과 PDF 원문 근거](../pr/assets/issues7727_7732_20261114/cbta-heading-investigation.json)를
분석 자료로 보존했다. 실행용 Rust 진단기와 전체 trace는 `output/`에만 둔다.

## 이전 완료 행 보정과 실제 검증 범위

parent `9b07db2002` 뒤의 완료 행 보정으로 Native·fresh WASM **각389쪽**을 모두 다시
측정했다. 원본/PDF·정본 도구·96DPI·compat2022·raw2px·문턱32·정상 설치 글꼴은 유지했다.
Native의 CBTA162쪽은96.35%→99.94%,166쪽은96.76%→99.95%이며 fresh WASM도
각각99.94%·99.95%다. 두 경로 모두 원본 쪽수를 유지했다.

| 문서 | 쪽수 | Native 최저 | fresh WASM 최저 | 남은98% 미만 |
| --- | ---: | ---: | ---: | --- |
| CBTA | 204 | 97.00% (96쪽) | 96.99% (96쪽) | 96쪽 |
| 권고안 | 22 | 97.44% (4쪽) | 97.44% (4쪽) | 4쪽 |
| 영어 시험지 | 8 | 99.49% (3쪽) | 99.49% (3쪽) | 없음 |
| API | 105 | 99.55% (97쪽) | 99.55% (97쪽) | 없음 |
| 각주 표 | 50 | 98.58% (37쪽) | 98.58% (37쪽) | 없음 |

[생산자·전후 원인·전체TSV 색인](../pr/assets/issues7727_7732_20261114/complete-row-band/cause.json)에
Native 바이너리·WASM JS/바이너리·소스 패치·전체10개TSV의 SHA를 고정했다.
이 단계의 CBTA 전204쪽 Native는 이전 전쪽 생산자 대비 하락0쪽이다. 바뀐 세 쪽은
캡션118쪽과 종료 행162·166쪽이며, 나머지201쪽의 공식 표시값은 동일하다.
162쪽 [review](../pr/assets/issues7727_7732_20261114/complete-row-band/native-p162-review.png)·
[overlay](../pr/assets/issues7727_7732_20261114/complete-row-band/native-p162-overlay.png),
166쪽 [review](../pr/assets/issues7727_7732_20261114/complete-row-band/native-p166-review.png)·
[overlay](../pr/assets/issues7727_7732_20261114/complete-row-band/native-p166-overlay.png)를 직접 판독했다.

새 관계형 종료 행 검사2개는 같은 원본과 Print 경로에서 보정 전2FAIL→보정 후2PASS다.
정식 파생 suite의 최소 글꼴 NativeSkia 집중 검증은 이전8개도 포함해 **10PASS/0FAIL**,2.834초다.
선택 밖1360건이며 전체 회귀가 아니다. suite 재배정 후 처음 세 target만 선택한 실행은
새2개만 실행했으므로10개 결과로 세지 않는다. 다음 실행의 컴파일 격리 오류는 아래에 구분했다.

## 완료 행 소유와 최소 글꼴 검사의 보정

기존 `1147d7b70a2`가 도입한 refit 뒤의 완전 continuation 경로는 경계를 넘는 병합 칸이
하나라도 있으면 남은 물리 쪽 공간을 끝 행에 먼저 합쳤다. CBTA162쪽 행188은 두 원본 줄
2600HU와 안여백282HU로38.4267px를 소유하지만, 이 단계에서45.2933px로 늘어났다.
166쪽 행252도 같은38.4267px 대신44px를 받았다. 뒤의 실제 병합 잔여 계산은 이미 커진
`current`를 하한으로 쓰므로 이 증가를 되돌릴 수 없었다.

이 선행 확장을 제거하고, 실제 원본 선언·내용·앞 조각이 수용한 행 상자에서 몫을 계산하는
기존 `emit::crossing_span_page_end_band`가 쪽 끝 띠를 한 번만 소유하게 했다. 독립 Print의
닫는 선은 각각724.57064/726.00936CSSpx이며 보정 후 표 끝과 내용 소속을 함께 확인했다.
원본 문서 전체가 아직98% 미만이므로 새 검사는 이 절대 좌표를 고정하지 않는다. 같은 행의
모든 단일 칸에서 원본 마지막 줄 끝+상하 안여백의 최댓값과 실제 마지막 행 높이의 관계를
compact tree의 소수 첫째 자리 직렬화 격자에서 확인한다.

새 검사의 첫 초안은166쪽 col3에도 두 줄이 있다고 잘못 가정했다. 실제 두 줄 소유자는
같은 행의 다른 칸이므로 원본 행 전체의 최댓값으로 바로잡았다. 정정한v2에서 두 실제 결함을
모두 재현하고 보정 후 통과했다. 잘못된 초안 실패는 결함 재현으로 인용하지 않는다.

최소 글꼴 검사에서 새 suite가 공개 `HappinessSansVF.ttf`를 `include_bytes!`로 참조해
기존 격리 규칙의 포괄적인 `ttfs` 읽기 차단에 걸렸다. 이 공개 파일 하나의 읽기만 허용한
새 격리 프로필로 다시 검증했다. 사용자·공유 글꼴 및 font service 차단은 유지했고,
CoreText1875→528개·모든 실제 경로OS 영역·한글 기본 글리프 존재를 다시 확인했다.
이는 공개 내장 fixture의 컴파일 허용이며 개인 글꼴의 설치·등록·탐색 허용이 아니다.

## 영어 공백 보정 단계의 측정 기록 (이력)

영어 시험지의 낱말 공백 기준 em을 보정한 생산자는 parent `d04b21df67`과
소스 패치 SHA-256 `406ae228d79ca7f416b9e6817e0a8eb87daaeaf61b471efa52273fd492ae6761`로
고정했다. Native 바이너리·fresh WASM의 SHA와 측정 요약은
[영어 공백 원인·생산자 기록](../pr/assets/issues7727_7732_20261114/half-space-origin/cause.json)에 있다.
Mac WASM은 locked wrapper의 `--no-opt` 로컬 대체 빌드다.

| 문서 | 최신 측정 범위 | Native 최저 | fresh WASM 최저 | 남은 98% 미만 |
| --- | --- | ---: | ---: | --- |
| 영어 시험지 | 전 8쪽 | 99.49% (3쪽) | 99.49% (3쪽) | 없음 |
| API | 전 105쪽 | 99.55% (97쪽) | 99.55% (97쪽) | 없음 |
| 각주 표 | 전 50쪽 | 98.58% (37쪽) | 98.58% (37쪽) | 없음 |
| 권고안 | 전 22쪽 | 97.44% (4쪽) | 97.44% (4쪽) | 4쪽 |
| CBTA | 96·118·162·166·176·182쪽 | 96.35% (162쪽) | 96.35% (162쪽) | 96·162·166쪽 |

최신 생산자에서는 각 경로 191쪽을 측정했다. CBTA 나머지 198쪽은 이 생산자로 아직 다시
측정하지 않았으므로 전 389쪽 최종 수용으로 보고하지 않는다. 영어 시험지 2쪽은 두 경로 모두
99.96%이며, 전 8쪽 TSV 원문은 [Native](../pr/assets/issues7727_7732_20261114/half-space-origin/exam-native-silhouette.tsv)와
[fresh WASM](../pr/assets/issues7727_7732_20261114/half-space-origin/exam-wasm-silhouette.tsv)에 보존했다.
같은 쪽 [review](../pr/assets/issues7727_7732_20261114/half-space-origin/native-p002-review.png)와
[overlay](../pr/assets/issues7727_7732_20261114/half-space-origin/native-p002-overlay.png)를 직접 판독했다.

최소 글꼴 Native Skia 집중 회귀는 **8 PASS / 0 FAIL**, 2.349초다. 세 파생 suite에서
세 원본 검사 파일만 선택했고 1024개는 선택 밖이다. 새 낱말 원점 고정형 검사도 포함한다.
[격리 정책·실행 요약](../pr/assets/issues7727_7732_20261114/half-space-origin/minimal-font-validation.json)에
기존 7개 검사 실행과 구분하여 기록했다. 전체 회귀·보호 문서·필수 편람·실제 소비 경로는
최종 소스에서 별도로 완료해야 한다.

## 원본과 독립 기준

| 이슈 | 원본 | 독립 PDF | 쪽수·실제 출력 제품 |
| --- | --- | --- | --- |
| #7727 | `samples/issue6764/1613000-202200037-air-traffic-controller-cbta.hwp` | `pdf/issue7727/cbta-original-hancom2020-print-20261106.pdf` | 204쪽·한컴 2020 |
| #7729 | `samples/issue6844/30269-anticorruption-recommendation-toc.hwp` | `pdf/30269-anticorruption-recommendation-toc-2020.pdf` | 22쪽·한컴 2022 |
| #7730 | `samples/exam_eng.hwp` | `pdf/exam_eng-hwp-2020.pdf` | 8쪽·한컴 2022 |
| #7731 | `mydocs/pr/assets/issue7445/hwpctl_API_v2.4.hwp` | `mydocs/pr/assets/issue7731/api-reference/hwpctl_API_v2.4-hwp-2020.pdf` | 105쪽·한컴 2022 |
| #7732 | `mydocs/pr/assets/issue7445/issue1937_rowbreak_footnote_overpagination.hwp` | `pdf/issue7382-regression-review/issue1937_rowbreak_footnote_overpagination-2020.pdf` | 50쪽·한컴 2020 |

파일명의 `2020`과 실제 제품 버전을 구분했다. 웹한글기안기 출력물을 기준으로 쓰지 않았다.
입력/PDF SHA-256·쪽수·Creator·Producer는 작업 증적의 `cases.json`에 고정했다.
시각 검증 조건은 정상 설치 글꼴, Native PNG/fresh WASM, 96 DPI, compat2022,
공통 Visual Sweep raw 2px, 픽셀 차이 문턱 32, 페이지 작업자 총 8개다.
현재 정본 도구의 PDFium 변환을 사용하며 기준 PDF 바이트는 수정하지 않았다.
회귀 검사 실행의 최소 글꼴 조건과 실물 시각 검증의 원본 글꼴 조건은 구분한다.

## 이전 생산자의 전쪽 결과 (이력)

`closing-span-clean-native` 및 `closing-span-clean-pkg`는 API 종료 프레임·CBTA 종료 병합 칸
보정까지 포함한다. 이후의 캡션 테두리·NBSP 압축·밑줄 변경은 포함하지 않는다.

| 문서 | Native 전체 최저: base → 보정 | fresh WASM 전체 최저 | 보정 후 98% 미만 |
| --- | --- | --- | --- |
| CBTA 204쪽 | 96.23% → 96.35% | 96.35% | 96·118·162·166쪽 |
| 권고안 22쪽 | 87.21% → 97.44% | 97.44% | 4쪽 |
| 영어 시험지 8쪽 | 97.12% → 97.12% | 97.11% | 2쪽 |
| API 105쪽 | 24.28% → 99.55% | 99.55% | 없음 |
| 각주 표 50쪽 | 98.58% → 98.58% | 98.58% | 없음 |

두 경로 모두 389쪽의 쪽수가 일치했다. Native의 383쪽이 98% 이상이며 6쪽이 남았다.
API는 Native 77쪽이 공식 두 자리 표시값 100.00%다. 이 결과는 전체 nextest·보호 문서·
최종 실제 소비 경로의 통과를 뜻하지 않는다.

## 원인별 보정 규칙

### API 12→13쪽: 표 앞 바깥여백 중복 소비

`fragment/budget.rs`에서 저장 host 종료 뒤 재동기화한 현재 커서를 opening 프레임의
원점으로 비교했다. 이미 소비한 바깥 위 여백 3.773px를 다시 차감해 행 소유가 달라졌다.
불변 `anchor_y`와 host 앞 점유로 실제 opening 프레임을 판별한다.
기존 #8148 계열의 저장 프레임 계약을 추적했으며 명시적 다음 프레임과 페이지 컷은 유지한다.

### 권고안 6·22쪽: 단 시작 전 닫힌 인라인 개체 줄

`stored_column_spacing.rs`가 한 줄에 여러 도형이 있거나 앞 문단이 중립 컨트롤만 가지면
닫힌 줄을 인정하지 못했다. 원본 문단 99의 두 도형은 3370/3338HU이며 줄 높이는 3370HU다.
같은 줄의 개체 높이는 합이 아닌 최댓값으로 닫고 바깥여백을 포함한다.
저장된 앞 500HU 간격을 누락하던 경로를 복구했다. 기존 #7642·#8917 구현을 대조했다.
6쪽 87.21%→99.95%, 22쪽 91.73%→99.60%다.

### API 35→36쪽: 빈 첫 줄과 동일 원점 재시작

첫 빈 문단과 다음 코드 문단이 모두 vpos=0이어도 첫 빈 문단의 실제 줄 높이가 소유된다.
선언 프레임 닫힘으로 검증한 경우 빈 문단의 종료 전진을 보존하고,
`stored_empty_successor_geometry`가 그 전진을 다시 지우지 않도록 했다.
초기 후보는 36쪽을 악화시켜 폐기했으며, 정정한 후보의 35·36쪽은 모두 100.00%다.

### 권고안 4쪽: 저장 오른쪽 탭의 선언 프레임

문단의 명시적 RIGHT 탭이 현재 프레임 바깥에 있고 실제 저장 확장 정보가 있을 때
그 저장 목표를 소비한다. 프레임 안쪽 탭·자동 오른쪽 탭의 계약은 유지한다.
`inline_right_tab.rs`에서 처리했으며 97.34%→97.44%다. 잔여 차이가 있어 완료가 아니다.

### API 65쪽: 폭 0 어울림 표 host의 앞·뒤 점유

독립 글줄이 아닌 빈 host에서 문단 앞 간격과 표 바깥여백을 중복 소비했다.
공통 `stored_empty_square_host_uses_paragraph_origin`으로 측정과 paint가 같은 원점을 쓴다.
뒤 간격도 원본 식 `34574−500+141+22784+141=57140HU`로 실제 종료를 인증한 경우에만
바깥 아래 여백으로 닫는다. 일반 host의 뒤 간격을 일괄 제거하지 않는다.
65쪽은 72.99%→94.53%(앞만 보정)→100.00%(끝 프레임 함께 보정)다.

### API 27·53·56·67쪽: 첫 재시작과 마지막 종료의 이중 인증

`native_text_frame_terminal_successor_gap`은 셀 placeholder 크기만으로 선언 opening을
제외하고, 실제 종료 줄에 뒤 간격을 더했다. 첫 저장 reset의 줄+안여백이 선언 높이를
닫는지 확인하고 마지막 줄+여백이 다음 문단 원점에 정확히 닿는지도 확인한다.
원본의 두 독립 경계를 만족할 때만 다음 프레임의 간격을 중복 소비하지 않는다.
네 쪽 모두 100.00%다. 문서명·쪽 번호 예외를 넣지 않는다.

### API 69쪽: TAC 표 뒤 독립 빈 줄과 fit 예산

표를 소유한 저장 줄 앞의 실제 prefix와 뒤의 독립 빈 줄을 구분하지 않아,
표 fit 계산에 이미 소비한 줄이 다시 포함되고 paint는 공백 host의 뒤 줄을 생략했다.
표 소유 줄의 실제 인덱스로 뒤 시작을 정하고, source prefix가 실제 전진과 일치할 때
소유 표 줄의 높이·간격만 표 예산으로 쓴다. 별도 빈 줄은 자기 위치를 가진다.
초기 시각 통과 뒤에도 fit의 4px 누락을 확인해 추가 보정했다. 최종 69쪽은 100.00%다.

### CBTA 176·182쪽: 이번 행에서 닫히는 병합 칸의 몫 누락

선행 실제 수정 `0024df763a`의 내용·선언·병합 칸 몫의 최댓값 계약을 추적했다.
`crossing_span_page_end_band`의 엄격 부등식이 이번 행에서 끝나는 병합 칸을 제외해
근거 없는 남은 쪽 공간 전체를 끝 행에 배분했다. 182쪽 행222의 17.0933px가 32.44px로
늘어났지만 독립 Print는 약21.43px다.
종료 병합 칸의 실제 내용과 선언 높이 중 큰 점유에서 이미 수용한 앞 행 상자만 뺀다.
줄·안여백의 전체 최소는 기존 공개 측정 API로 구한다. 뒤로 이어지는 칸의 계약은 유지한다.
176쪽 97.45%→99.94%, 182쪽 96.23%→99.94%이며 전204쪽으로 확인했다.

### CBTA 118쪽: 위 캡션 이동에서 소유 테두리 누락

`layout.rs::offset_inline_image_y`가 그림만 옮긴 뒤 반환해, 형제 테두리를 캡션 위에 남겼다.
원본 image y=212.238px, border y=184.918px로 27.32px 분리됐다.
이미지 ID와 `RectangleNode.image_frame_owner`로 같은 소유자의 테두리를 찾아 함께 이동한다.
좌표·색·문서명으로 테두리를 추정하지 않는다.
별도 Native/fresh WASM 집중 결과는 모두 96.85%→98.44%다. review·overlay를 함께 직접 확인했다.
얇은 테두리의 출력 농도 차이는 남아 있어 100% 해결로 쓰지 않는다.
새 관계형 검사 `issue_7727_top_caption_picture_frame`은 실제 보정 전 1 FAIL→보정 후 1 PASS다.
문서 전체가 아직 98% 미만이므로 절대 위치를 고정하는 검사로 등록하지 않았다.
이는 캡션 보정 당시의 판단이다. 현재는 위 최신 전쪽 검증으로 자격을 확인했으며,
같은 검사 파일에 독립 PDF 테두리 네 변의 ±2 CSSpx 고정형 검사를 추가해 통과했다.

## 영어 시험지의 추가 분석

NBSP의 실제 tracking을 소비하면서 압축 슬롯 수에서는 제외한 공통 분모 불일치를 확인했다.
압축 슬롯에 NBSP를 포함한 후보는 Native 전8쪽에서 최저 97.14%다. 아직 목표 미달이다.
앞서 언어 경계를 강제로 이어받긴 별도 후보는 97.12%→96.85%로 악화시켜 되돌렸다.
그 후보의 결과를 최종 증거로 사용하지 않는다.

밑줄 차이는 별도 한컴 2020 인쇄 대조군으로 분리했다. 동일한 12pt 글자·밑줄에
글자 위치 -12/0/+12를 적용했다. MCP job `9e395d33-57c9-4fb5-932e-eea17e70b484`,
실제 버전 11.0.0.9136, Print method 0, 1쪽, 출력 13초다.

| 글자 위치 | 글자 기준선(pt) | 밑줄 y(pt) | 밑줄−글자 기준선(pt) |
| --- | ---: | ---: | ---: |
| -12 | 79.4731 | 82.9490 | 3.4759 |
| 0 | 100.0906 | 102.1280 | 2.0374 |
| +12 | 120.7081 | 121.3070 | 0.5989 |

밑줄은 약19.179pt인 줄 간격만 따라가고 글자 위치 변화는 따라가지 않는다.
아래 밑줄의 공통 replay에서만 글자 위치를 제외하고, 텍스트·장식의 원본 run은 동일하게
보존한다. 최초 후보가 원본 장식 run을 바꾸어 canonical 소유 검증에 거절된 것을 실제 확인했고,
검증을 완화하지 않고 replay 단계로 보정을 옮겼다. 종이 프레임의 밑줄 원점도 같은 줄 기준선이다.
Native 전8쪽 최저97.79%, fresh WASM 전8쪽 최저97.78%다. 개선됐지만 목표 미달이다.

새 밑줄 관계형 검사는 이전 코드에서 1 PASS / 1 FAIL, 보정 후 2 PASS다.
NBSP 끝 전진 관계를 추가한 3개 검사는 보정 후 모두 PASS다. NBSP의 이전 라이브러리 검사는
폐기한 언어 슬롯 후보를 포함한 진단 생산자이므로 base 자체의 결과로 부르지 않는다.
위 밑줄·취소선과 원본 글자 위치 보존을 대조군으로 함께 확인했다.
API의 독립 PDF 표 경계 13·65·69쪽 고정형 검사와 69쪽 독립 빈 줄 소유 관계를 추가했다.
정확한 base Native에서 3 FAIL, 보정 후보에서 3 PASS다. 임시 독립 rustc 실행이며
정식 파생 suite의 최소 글꼴 Native Skia nextest도 실행해 새3개 파일의 **7 PASS / 0 FAIL**을
확인했다(3.353초, 선택 밖1024건). 첫 실행의 임시 폴더 helper 직접 의존성 누락은 표준
라이브러리의 검사 소유 폴더와 Drop 정리로 수정했다. 전체 회귀는 아직 실행하지 않았다.

Mac 격리에서 CoreText1875→528개, system_profiler의 글꼴 파일235개와 CoreText의
모든 실제 경로가 OS 영역에 있음을 확인했다. 기본 한글 프로그램 파일·표시 PNG의 SHA는
일반 환경과 같고, 일반 프로세스의 사용자 글꼴 등록 목록도 전후 동일했다.
[최소 글꼴 집중 검증 기록](../pr/assets/issues7727_7732_20261114/focused-minimal-font-validation.json)을 따른다.

독립 대조군은 [원본 HWPX](../../tests/fixtures/issue7730/underline-position-probe.hwpx),
[한컴2020 Print PDF](../../pdf/issue7730/underline-position-probe-2020.pdf),
[원인·전후 측정·MCP 영수증](../pr/assets/issues7727_7732_20261114/underline-baseline/cause.json)에
같은 SHA로 보존했다. 글꼴 바이너리는 포함하지 않는다.

### 영어 낱말 공백: 언어 슬롯의 상대 크기와 기준 em 분리

원본 영어 시험지 CS13은 1150HU, 한글 장평 95%, 영문 상대 크기 106%다. 반각 공백이
현재 영문 글리프 크기를 기준으로 커지면서, `bike`의 첫 글자가 독립 Print보다 약7.76px
오른쪽에 놓였다. 글리프 크기·장평·자간을 바꾸지 않고 반각 공백의 기준 em만 한글 슬롯의
크기로 환산하여 측정·줄 구성·paint가 공유하도록 했다. `font_space_em`이 명시되거나
앞 언어에서 상속된 전진은 기존 값을 유지한다. 반각 규칙을 선택하지 않은 경로에는 적용하지 않는다.

독립 한컴2020 Print 대조군은 한글12pt·장평95%를 유지하고 영문 상대 크기100/106/120%만
바꿨다. 영문 글리프는 커지지만 낱말 공백은 약5.7pt로 유지됐다. MCP job은
`e2f7be42-a029-49db-920e-ac993c38cd66`, 제품11.0.0.9136, Print method0이다.
[독립 HWPX](../../tests/fixtures/issue7730/relative-size-half-space-probe.hwpx)와
[Print PDF](../../pdf/issue7730/relative-size-half-space-probe-2020.pdf),
[영수증·원인](../pr/assets/issues7727_7732_20261114/half-space-origin/cause.json)을 함께 보존했다.
이 HWPX의 Native 경로는 `layout_half_space=false`라서 전후93.05%이며, 해당 표본까지
시각적으로 해결했다고 주장하지 않는다. 이 표본은 독립적인 공백 전진의 인쇄 근거다.

실제 원본 HWP 전8쪽은 Native·fresh WASM 모두 최저99.49%로 복구됐다. 새 고정형 검사
`english_response_line_keeps_independent_print_word_origins`는 독립 PDF의 낱말 원점
326.759979pt→435.679972 CSSpx와 기준선308.799988 CSSpx를 ±2 CSSpx로 검사한다.
원점은 보정 전443.44px로 허용 범위를 벗어나며 보정 후 통과했다. 추가한 네 번째 검사를
포함하여 같은 파일의4개 검사와 정식 최소 글꼴 집중8개 검사가 실제 통과했다.
이전 독립 rustc 대조군은 폐기한 언어 슬롯 후보를 포함하므로 정확한 base 전체 결과와 구분한다.

## 남은 분석과 완료 조건

### 2026-11-15 최종 head 검증과 #7445 후속 처리

이 단계에서 검증한 head는 `57a38d59fee77ce7ffa6bb58a69f63c3c57c77c3`이다. Mac 최소 글꼴
전체 nextest는 **13,560 PASS / 0 FAIL / 45 skip**, 실행 361.301초로 완료했다.
fmt·Native/WASM Clippy·workspace build·all-target Clippy·base 고정 suite manifest·unit tier,
Native Skia lib·누락 글꼴 placeholder·직접 PDF export 검사도 모두 통과했다.
[Rust 게이트 실행 범위와 로그 SHA](../pr/assets/issues7727_7732_20261114/note-oracle-final/rust-gates.json)를 따른다.

같은 head의 Native와 fresh WASM도 각각 389쪽 모두 98% 이상으로 완료했다.
[양쪽 전쪽 TSV와 생산자](../pr/assets/issues7727_7732_20261114/note-oracle-final/validation.json)를
정식 assets에 보존했다. WASM은 Mac 로컬 대체 `--target web --out-dir pkg --no-opt` 빌드이며,
루트 pkg와 Studio public의 JS·WASM SHA가 일치한다. 실제 브라우저 초기화·소비 검증은 별도로 남아 있다.

정본 보호 검증의 첫 driver는 원 수용 증적에 기록된 문서별 `font_environment`를
빠뜨렸다. 중첩 표·sample16에서 관측한 하락을 이 상태로 제품 회귀로 확정하지 않는다.
첫 실행과 그 실패 판정은 그대로 보존하고, 정본 manifest의 `condition_evidence` SHA를
검증한 뒤 거기에 기록된 승인 환경으로 다시 측정한다. 동일 조건의 완료 실행만 재사용한다.
필수 편람 gate 자체는 통과했지만, 나머지 보호 결과와 실제 소비 경로가 완료되기 전에는
최종 검증 완료·PR 준비 완료라고 보고하지 않는다. 기준 PDF·보호 TSV·판정 문턱은 바꾸지 않는다.

사용자 지시에 따라 모든 필수 검증과 병합을 완료한 뒤 #7727·#7729·#7730·#7731·#7732의
최신 이슈 본문과 해결 범위를 재확인한다. 완전히 해결된 이슈에는 실제 병합 PR·SHA와
전쪽 TSV·같은 쪽 review/overlay를 포함한 완료 댓글을 남기고 닫는다. #7445의 해당 행도
취소선과 실제 완료 PR로 갱신하되, 대표 이슈와 부분 해결 항목은 열어 둔다.

### 저장 RIGHT 탭의 후속 글자 원점: 독립 Print 반례로 추가 보정

CBTA 2쪽 목차 문단23의 저장 탭은 폭1868HU, 종류 RIGHT다. 같은 런의 `\t10`을
현재 정지점에서 다시 정렬하는 경로와, 교차 런에서 저장 전진을 소비하는 경로가 달랐다.
측정값을 높이기 위해 글꼴 폭 테이블이나 기준 PDF를 바꾸지 않고, 저장 폭의 실제 의미를
독립 인쇄 대조군으로 확인했다.

원본에는 BodyText와 ViewText가 함께 있으며, 해당 문단의 텍스트·스타일·저장 줄은 같다.
파서는 완전한 ViewText를 우선 읽는다. 처음 BodyText만 변경한 임시 진단은 재파싱의
기대 확인에 실패했으므로 제품 반례로 수용하지 않았다. 두 스트림의 해당 탭 폭만 각각
1868→3368HU로 바꾼 사본은 재파싱에서도3368을 확인했다. 독립 한컴2020 Print의 첫 숫자
원점은674.88000488→694.87996419 CSSpx로 이동했다. +1500HU의20 CSSpx와 일치하며,
이 입력의 유효한 저장 줄에서는 탭 폭이 정렬 완료 후의 전진을 소유한다.

[실제 HWP 대조군](../../tests/fixtures/issue7727/cbta-stored-right-tab-plus1500.hwp),
[독립 한컴2020 Print PDF](../../pdf/issue7727/cbta-stored-right-tab-plus1500-2020.pdf),
[MCP 영수증·원본 스트림 대조·좌표 근거](../pr/assets/issues7727_7732_20261114/stored-right-tab/cause.json)를
함께 보존했다. MCP job은 `c8e37f0f-e545-4880-b774-decf06def96e`, 실제 제품11.0.0.9136이다.
저장 줄을 제거한 HWPX 재조판 대조군에서는 이 캐시 변화가 배치를 소유하지 않았다.
두 입력의 서로 다른 저장·재조판 조건을 같은 것으로 취급하지 않는다.

새 관계형 검사 `issue_7727_stored_right_tab_advance`는 공개 문자 배치 API를 사용한다.
저장 폭만 바꾼 이동 관계와 저장 줄 없는 재조판 대조군을 함께 검사하여 보정 전
**1 PASS / 1 FAIL**을 확인했다. 처음 작성한 진단의 필드명·비공개 API 접근 오류는
검사 구현 오류로 수정했으며, 그 오류를 제품의 FAIL 증거로 계산하지 않았다.

보정 후보는 수용된 RIGHT 확장의 하위·상위16비트 폭을 한 번만 소비하여 같은 런도
교차 런의 `StoredStart`와 같은 계약을 따르게 한다. 저장 줄 부재·재조판·자리표의 기존
분기와 명시적 정지·자동 우측 정렬은 유지한다. 특정 문서·쪽 번호 조건은 추가하지 않는다.
최신 라이브러리를 연결한 새 검사는 **2 PASS**다. `--bin` 빌드는
`release-test/deps/librhwp.rlib`을 갱신한다. 이전 root `release-test/librhwp.rlib`을 연결한
첫 임시 실행은 진단기 오류로 분리했으며 새 후보의 실패·통과 근거에 넣지 않았다.
Native 집중 출력은 CBTA2쪽98.11→98.52%, 권고안4쪽99.86→99.96%다.
CBTA2쪽의 같은 쪽 review/overlay도 직접 확인했다. 기존 #6844의 bbox 포괄 검사는
통과했고, 모든 목차 숫자의 우단을 한 곳으로 강제한 보조 검사는 1.3px 편차로 실패했다.
독립 Print와 fresh WASM 자격을 먼저 확인한 뒤 이 기대의 정확성을 검토한다.
원래 결함인 글리프의 bbox 이탈 검출을 지우거나 수치 문턱만 완화하지 않는다.

위57a38 전체 통과를 새 소스의 통과로 바꾸어 쓰지 않는다. 불필요한 구 소스 재측정을
피하려고 보호 교정 driver와 실제 소비·base 대조의 대기 작업은 새 후보 실행 전에
종료했다. 구 소스 보호 캡처의 완료·부분 PNG/TSV/log는 모두 보존하고, 남은 WASM
sample10 캡처만 소유 프로세스를 확인해 중단했다. 새 최종 후보에서 정확한 승인 글꼴
조건으로 정본 보호를 수행한다.

- CBTA 96쪽은 위의 번호 문단 보정으로 양쪽 99.72/99.74%다. 가능한 잔여100% 개선을
  계속 확인하며, 현재 관측한 2쪽 목차 탭 전진의 재계산 경계를 원본과 대조한다.
- CBTA162·166쪽은 위의 완료 행 소유 보정으로 두 경로 모두99.94/99.95%다.
  이후 번호 문단 보정까지 포함한 최신 전389쪽 측정에서도 이 수치를 유지했다.
- 권고안 4쪽은 위 채널 보간 후보의 Native에서 99.96%로 개선됐다. 전 22쪽 최저는
  98.69%다. 새 fresh WASM도 4쪽 99.96%, 전 22쪽 최저 98.68%로 완료했고 정식 최소 글꼴 집중
  12개도 통과했다. 최종 전체 회귀·보호 검증은 아직 남아 있다.
- 현재 번호 문단 소스의 Native/fresh WASM 전389쪽 검증은 위와 같이 완료했다.
  추가 구현으로 제품 소스가 달라지면 영향 범위와 생산자를 구분하여 재검증한다.
- 회귀를 최소 글꼴 환경에서 실행하고, 독립 근거로 자격을 얻은 쪽에는 현행 지침에 따른
  정확한 고정형 또는 관계형 검사를 추가한다. 검사의 상단 주석에 종류·측정 근거를 명시한다.
- 전체 nextest·범위별 lint/build·보호 TSV·필수 편람·실제 소비 경로를 최종 head로 확인한다.
  미실행 항목을 이전 head의 통과나 PR CI로 대신하지 않는다.

## 2026-11-15 03:40 KST — 저장 RIGHT 탭의 편집·반복 저장 경계

`353989d9f2`의 Native/fresh WASM 전389쪽은 각각 모두98% 이상이다.
CBTA2쪽은98.52%, 권고안4쪽은99.96%이며 같은 쪽 review/overlay를 직접 판독했다.
권고안 전22쪽 최저는98.69/98.68%다. 전체 회귀13,560 PASS 기록은 여전히
이전57a38 소스의 결과이며353989의 전체 회귀로 인용하지 않는다.

기존 #6844의 원 해결 `c1a1a5dcf3`과 현재 출력 계약을 대조했다. 글리프가 bbox를
벗어나는 원 결함 검사는 그대로 보존한다. 모든 숫자 우단을 한곳에 강제하던 보조 가정은
각 런의 bbox 끝이 자기 charX 끝과 일치하는 관계로 바꿨다. 원래1px 허용폭을 늘린
수정이 아니다. 실제 API의 폭만20.3px 짧게 또는 길게 바꾸는 음성 대조에서 각각
2개 실패,1개 실패를 확인했다. 이 음성 대조는 출력 변이이며 과거 제품 실행과 구분한다.

공개 `insert_text_native`로 본문을 편집한 후 HWP를 두 번 저장·다시 열자, 오래된
TAB 폭1868/3368HU가 살아남아 같은 편집 결과의 숫자 위치를20px 다르게 만들었다.
모든 본문 문단의 텍스트와204쪽은 보존됐으므로 단순 누락·파서 실패로 분류하지 않는다.
새 회귀는 원본 저장 폭 수용·저장 줄 없는 재조판·실제 편집 후 반복 저장을 함께 검사하며,
수정 전3건 중2 PASS /1 FAIL을 확인했다.

원인은 `try_reflow_line_segs_impl`이 새 줄을 발행하면서 TAB의 옛 정렬 완료 전진을
그대로 두는 데 있다. 수정 후보는 유효한 상자에서 실제 재조판할 때 RIGHT 폭의
두 워드만0으로 만들어 현재 TabDef가 계산하게 한다. 정렬 종류·채움·탭 순번과 다른
종류의 확장은 유지하며, 재조판하지 않은 원본 저장 폭은 바꾸지 않는다.
HWP/HWPX 각각 편집→저장→열기→재저장→열기에서 텍스트·기하를 확인하고
최소 글꼴 전체 회귀·정본 보호·실제 소비를 새 head로 검증한 뒤 제출한다.
수정 후 Native 빌드가 성공했고 새 탭 검사3개와 bbox 검사2개가 모두 통과했다.
새 탭 검사는 HWP/HWPX 각각2회 저장에서 내용과 기하 관계를 확인했다. 이 실행은
현재 설치 글꼴 환경의 집중 검사이며 최소 글꼴 정식 suite·전체 검증은 별도로 수행한다.
위353989 전쪽 결과를 새 수정의 통과로 쓰지 않는다.

실행 로그: `output/pr-review/issues-7727-7732-20261114/stored-right-tab-repair/`의
`edit-invalidation-before-test.log`, `edit-invalidation-native-build.log`.

## 2026-11-15 04:04 KST — Undo 메타 복원을 실제 입력 무효화와 구분

`fe8425e66d`의 새 Native 전389쪽은 모두 98% 이상이며 직전353989의 페이지 점수와
달라진 쪽이 없다. 그러나 최소 글꼴 전체 회귀는 **13,558 PASS / 5 FAIL / 45 skip**으로
끝났다. 테스트 실행356.908초, 컴파일 포함829.311초다. 이 head를 최종 통과로 쓰지 않는다.

실패는 본문1건과 표 셀·중첩 셀·글상자·그림 캡션4건의 Undo 메타 복원이다.
원 해결은 본문 PR #3223 및 셀 경로 PR #3324, 실제 셀 통합 커밋`a9f4729bdd`다.
원래 탭 메타100HU를 그대로 복원하던 계약을 이번 무조건 캐시 삭제가0으로 바꿨다.
원 검사·기대값은 변경하지 않는다. 원인과 전체 실패 목록은 로컬
`stored-right-tab-repair/edit-undo-full-regression-failure.json`에 보존했다.

수정 경계는 실제 텍스트 교체 helper가 기존 `invalidate_layout_inputs`로 입력 변경을
게시하는 지점이다. 재조판은 그 dirty 출처가 있는 실제 탭에 대해서만 이전 RIGHT 전진을
비우고, Undo가 원본 메타를 복원한 경우와 실제 글을 바꾸지 않은 빈 입력은 보존한다.
파일·쪽·문구 조건이나 Undo 전용 수치 복사로 통과시키지 않는다.
실제 CBTA 가시 탭의 병합 Undo와 빈 입력 반례를 추가했으며 보정 전 **3 PASS / 2 FAIL**이다.
새 후보의 Native 빌드와 최소 글꼴 집중12개(새 탭5·기존 셀 계열 Undo5·bbox2)가 모두
통과했다. CoreText 실제 목록528개에 사용자·추가 시스템 경로의 글꼴은0개였고,
OS 기본 AppleSDGothicNeo의 한글 glyph는 사용 가능했다.
같은 본문 Undo가 전체 페이지네이션 재사용 후보와 다르다는 진단이 나오지만, 이 경로는
실제로 계산한 full pagination을 유지하고 잘못된 재사용을 하지 않는다. 이 기록을
페이지별 Print 대조의 통과로 바꾸어 쓰지 않는다. 새 전체 게이트를 이어 진행한다.

## 2026-11-15 04:43 KST — Undo 경계 보정 후 전체 회귀 완료

제품 소스 `1123f322a65cd8eaa414a8f32c3e2b99462bbda0`의 Mac 최소 글꼴 전체 회귀는
**13,565 실행 / 13,565 PASS / 0 FAIL / 45 skip**이다. 테스트 333.989초,
컴파일을 포함한 명령 710.630초다. 이전 실패한 본문·셀 계열 Undo 검사는 기대값을
유지한 채 통과했다. fmt·Native/WASM Clippy·workspace build·all-target Clippy·suite
manifest·unit tier·Native Skia 라이브러리·placeholder·직접 PDF 검사도 모두 exit 0이다.

같은 소스의 Native 전 389쪽은 모두 98% 이상이다. CBTA 최저 98.13%(58쪽),
권고안 98.69%(7쪽), 시험지 99.49%(3쪽), API 99.55%(97쪽), 각주 98.75%(37쪽)다.
직전 `fe8425e66d`와 현재 Native raw PNG 389개 및 페이지별 점수는 모두 같다.
수정은 실제 입력과 복원 경계를 구분하며 원본의 수용된 저장 배치를 바꾸지 않았다.

고정 base `53da4d2d`의 동일 조건 Native 전 389쪽과 비교해29쪽 상승·3쪽 하락이다.
CBTA 3쪽 99.42→99.41%, 4쪽 98.31→98.26%, 5쪽 99.34→99.28%이며 최대 하락 0.06%p다.
이 세 쪽의 base/후보 review·overlay 여섯 쌍을 직접 판독했다. 목차 문구·순서·쪽번호·줄
소속은 유지됐고 새 누락·중복·넘침은 관측하지 않았다. 이 결과는 일반 2%p 하락 문턱
대조이며 특별 보호 TSV의 하락 금지 검증을 대신하지 않는다.

fresh WASM은 저장소 루트 `pkg`에 `--target web --no-opt`로 새로 빌드했고,
빌드 4분 56초·root/Studio 공개 JS·WASM 해시 일치를 확인했다. 현재 전 389쪽 WASM과
이후 정본 보호 36조건·실제 소비 90건·고정 base WASM 대조를 순서대로 진행 중이다.
과거 source의 결과를 이 소스의 보호·소비 완료로 사용하지 않는다.

보정 전 전체 CBTA 편집 진단 사본 2개는 MCP의 문서 열기 단계에서 각각 1800초
시간 초과했고 PDF는 생성되지 않았다. 이 결과는 실패로 보존하며 원본 독립 Print를
대체하지 않는다. 현재 소스의 공개 편집 API로 별도의 작은 표본을 편집·2회 저장한
HWP/HWPX의 독립 Print를 요청했다. 아직 성공·시각 일치로 판정하지 않는다.

실행 기록: `output/pr-review/issues-7727-7732-20261114/`의
`stored-tab-owned-final-rust-gates/state.json`, `stored-tab-owned-final/native-fixed-base-comparison.json`,
`stored-tab-owned-final/native-decline-review/reading.json`, `stored-tab-owned-final/state.json`.

## 2026-11-15 05:04 KST — 소형 편집 저장본의 독립 Print와 namespace 분리

현재 fresh WASM `1123f322a6`에서 작은 독립 탭 표본의 첫 문단에 `x`를 입력하고,
HWP/HWPX 각각 두 번 저장·다시 열었다. HWP Print는 MCP job
`cdcdae80-619b-4296-b43a-0a121f00faaa`에서 성공했다. 처음 생성한 HWPX는
`HANCOM_DOCUMENT_OPEN_REJECTED`로 거절됐으므로 같은 파일을 재시도하거나 보안 설정을
낮추지 않았다. 이 실패와 PDF 0바이트 기록을 보존했다.

파일을 직접 검사하니 `header.xml`의 번호 정의에 선언되지 않은 `ns0:paraHead`가 있었다.
원본 표본에서 쓴 namespace 별칭이 raw numbering splice에서 선언 없이 재사용된 결과다.
이 splice는 고정 base에도 존재하는 `3f564107368`의 기존 경로이며 이번 탭 수정으로
추가하지 않았다. 원본 XML에서 글자와 RIGHT 폭만0으로 직접 바꾼 대조군은 독립 Print
job `9c4e9762-2f14-4613-979f-56db9b479438`에서 성공했다. 따라서 이 거절을 폭 0 자체의
한컴 형식 위반으로 해석하지 않는다. 기존 실패 원본과 정상 대조군을 혼합하지 않는다.

별칭만 표준 `hh/hc/hp`로 정리한 별도 표본에서 XML expanded name·속성·text·tail의
완전 동일성을 확인했다. 이 입력을 실제로 편집·두 번 저장한 HWPX는 모든 XML이 정상이며
job `7a2bc452-42a6-4f4d-be21-4a233768366a`에서 한컴 2020 `11.0.0.9136` Print에 성공했다.
HWP는 별칭 정리 전후의 모든 CFB 스트림뿐 아니라 전체 파일 SHA도 동일하다.
두 독립 Print는 1쪽, 본문 `xABC 10 / ABC 10 / ABC 10`과96DPI PDF 픽셀이 같다.

Print의 숫자 문자 원점은 세 줄 모두324.46415 CSSpx이며 rhwp 문자 배치 JSON은
324.7 CSSpx다. 차이 0.23585 CSSpx를 기록했다. 이는 독립 원점 측정이며 아직 실제
Native/fresh WASM 래스터와의 시각 대조 완료를 뜻하지 않는다. 기존763쪽 보호 문서와
캡처8개 한도를 공유하므로 대기 중 비교가 끝난 뒤 소형 표본을 순차 출력한다.
임의 namespace 별칭을 가진 HWPX의 일반 무손실 재저장까지 해결했다고 주장하지 않는다.
이번 다섯 원본은 HWP이며 원본과 기준 PDF는 바꾸지 않았다.

## 증적 위치

현재 작업 증적은 저장소 기준 `output/pr-review/issues-7727-7732-20261114/`다.
임시 driver·실행 파일·글꼴/SVG 중간 자료는 이 경로에만 두며 Git에 추가하지 않는다.

- `cases.json`: 입력과 기준 PDF의 SHA·쪽수·출처.
- `closing-span-clean-native-state.json`, `closing-span-clean-wasm-state.json`: 동일 source 전쪽 결과.
- `closing-span-clean-native-producer.json`, `closing-span-clean-wasm-producer.json`: 생산자 SHA.
- `logs/closing-span-clean-native-driver.log`, `logs/closing-span-clean-wasm-driver.log`: 전쪽 실행 로그.
- `caption-frame-native-producer.json`, `logs/caption-frame-cbta.log`: 캡션 집중 후보와 측정.
- `logs/caption-frame-before-test.log`, `logs/caption-frame-after-test.log`: 새 검사 전후 실패/통과.
- `caption-frame-direct-review/review/review_118.png`와 `overlay/overlay_118.png`: 같은 쪽 직접 판독 쌍.
- `underline-position-cause.json`, `underline-position-print-{start,status,download}.json`: 독립 밑줄 대조군 영수증.
- `half-space-origin-{native,wasm}-state.json`: 최신191쪽 측정과 남은4쪽.
- `logs/half-space-origin-minimal-font-focused-nextest.log`: 정식 최소 글꼴 집중8개 PASS.
- `cbta-row188-diagnostic.log`: 162쪽 종료 행·걸침 셀의 원본 선언 높이와 저장 줄.

영어 공백의 확정 증거는 정식 `mydocs/pr/assets/issues7727_7732_20261114/half-space-origin/`에
복사했고, 독립 표본은 `tests/fixtures/issue7730/`와 `pdf/issue7730/`에 보존했다.

위 경로는 진행 중인 로컬 증적이다. 최종 제출 때에는 필요한 PNG·TSV·영수증과 독립 표본을
정식 assets/PDF 경로로 선별해 함께 커밋하고 이 문서의 링크와 판정을 갱신한다.

### 2026-11-15 05:46 KST 최소 글꼴 회귀와 원본/보호 검증 완료, 실제 소비 경로 점검

- source `1123f322a65cd8eaa414a8f32c3e2b99462bbda0`에서 최소 글꼴 전체 nextest는
  13,565 PASS / 0 FAIL / 45 skip이다. Native/fresh WASM 원본 5종 전389쪽은 모두98% 이상이며,
  필수 편람을 포함한 보호36조건 전3,542쪽은 공식 두 자리 TSV 하락0이다.
- 실제 편집→2회 저장·재열기한 작은 독립 표본은 HWP/HWPX 각각 Native99.90%, fresh WASM100.00%다.
  같은 쪽 review와 overlay를 네 경로 모두 열어 세 줄 내용·순서·숫자 RIGHT 끝 정렬을 판독했다.
  HWPX는 기존 raw numbering namespace 직렬화 문제와 분리한 canonical prefix 대조군이다.
  원본 namespace alias 상태의 수동 XML 진단은 Native55.95%·gate FAIL로 별도 보존했으며,
  이 추가 진단을 정상 fixture 또는 실제 편집 저장 네 경로의 성공으로 계산하지 않는다.
- 자동화 브라우저의 Local Font Access 열거가0개여서 처음 캡처가 두부로 나온 것은 무효로 보존했다.
  실제 설치 글꼴1,220 face를 기존 HostFontProvider 공개 API에 공급하고 읽은 bytes의 SHA를 검증한
  캡처에서는 한글이 표시된다. 이를 브라우저 Local Font Access 정상 증거로 바꾸어 설명하지 않는다.
- CBTA204쪽은 CanvasKit의 기존 문서 preflight128쪽 한도를 넘는다. 강제 요청도 준비 단계에서
  Canvas2D로 대체되며 CanvasKit PASS가 아니다. 제품 guard/한도는 변경하지 않고 원 gate 실패·
  실제 선택 backend·전체 preflight를 보존한다. 실제 지원 경로와 의도된 한도 대체 경로를 구분해
  전체 캡처와 고정 base fresh WASM 비교를 계속한다. PR 제출·하위 이슈 close는 아직 하지 않았다.

### 2026-11-15T06:39:32+09:00 — 실제 CanvasKit 소비에서 원형 반경 전달 누락 확인

정식 원본 Native/fresh WASM389쪽·보호3,542쪽과 별도로 실제 Studio 출력도 비교했다.
설치 SFNT bytes와 실제 이름 정보를 Local Font Access 인터페이스 adapter에 공급한
권고안4쪽의 Canvas2D Print는98.30%, 실제 CanvasKit compat/default Print는 각각53.71%다.
readiness와 fallback gate가 통과해도 독립 PDF 피델리티를 보장하지 않으므로 제출을 보류한다.
이 adapter는 실제 Chrome의 OS 열거 성공 증거가 아니며 실제 설치 bytes를 사용하는 제품
렌더링 소비 검사다. 초기0개 열거·두부 결과와 HostFontProvider 전용 결과도 구분해 보존한다.

원 해결 `0dc43871465ef312c1e93b571c240a0a2e7394f1`은 Native·Rust Canvas의 원형 반경을
공통 `gradient_geometry::radial_circle`의 가장 먼 모서리 거리로 보정했다. 현재
`src/paint/json.rs::write_gradient`는 중심 비율만 전달하고
Studio `makeShapeGradientShader`는 긴 축 절반을 다시 계산한다. 물리 반경을 제품 공통
결과에서 JSON에 전달해 CanvasKit이 소비하도록 보완한다. 원뿔 형식3의 기존 근사는
별개 계약으로 보존하고, 사각형·이동 중심·비정사각형 독립 Print 대조군으로 확인한다.
목차 하위 항목의 글꼴/전진 차이도 보이므로 그라데이션만 보정해 완료로 판정하지 않는다.

완료 후 #7727·#7729·#7730·#7731·#7732의 실제 해결 범위와 병합 결과를 확인해
각 완료 댓글·close와 #7445의 해당 행 취소선을 처리한다. 미완료 항목과 대표 이슈는 유지한다.

### 2026-11-15T06:47:18+09:00 — 공통 producer와 CanvasKit 소비 계약 보완

`gradient_geometry::radial_circle → paint JSON radialCircle → makeShapeGradientShader`로
물리 원 전달을 연결했다. Native·Rust Canvas의 산식은 바꾸지 않았고 형식3의 기존 원형 근사도
유지했다. 독립 Print6쪽을 사용하는 기존 정식 검사에 JSON의 실제 물리 원 전달을 추가했으며
최소 글꼴2PASS, Studio shader replay 계약도 PASS다. 실물 CanvasKit 재출력은 아직 대기다.

목차 하위 글꼴은 같은 family에 HMKMM.TTF와 HMKSM.otf가 공존한다. CanvasKit이 공통
resolver의 선택 뒤에 캐시 전체를 다시 순회하면서 스타일 거리가 같은 첫 후보로 덮어쓰는
코드가 있었다. 공통 선택을 동점에서 보존하고 더 정확한 굵기/기울기가 있는 경우만 기존처럼
교체하도록 했다. 두 캐시 순서·Bold/Italic 대조군2PASS다. 실제 반례의 회복 여부는 새 WASM과
실제 CanvasKit 출력으로 확인하며, 분석 가설만으로 피델리티 완료를 주장하지 않는다.

진단 중 normal JSON을 Canvas 보충 메트릭 상태에서 직접 내보낸 추가 wrapper는 제품 guard에
거절됐다. 이 wrapper 오류를 원본 렌더링 결함이나 성공 결과로 계산하지 않으며 실패 기록을 보존했다.

### 2026-11-15T07:01:14+09:00 — 실제 CanvasKit 원형 배경 회복

새 source3f7e4340ae의 Mac 로컬 --no-opt fresh WASM은5분19초에 빌드됐다. WASM SHA는
`332fe003097de7043f0261044b08af720de8382d22502adbb23aa387d253f2d9`다. 루트 pkg·Studio public·
실제 초기화 응답을 검증했고 같은 원본/PDF/글꼴 인터페이스/96DPI Print에서 CanvasKit
권고안4쪽53.71%→98.26%, Canvas2D98.30% 유지다. review와 standalone overlay를 모두
판독해 배경 원형 띠·외곽 프레임·목차 순서·쪽번호가 보존됨을 확인했다. 하위 목차의
글리프/작은 간격 차이는 남아 있으므로100%로 주장하지 않는다.

동점 face 보존만 적용하고 이전 WASM을 쓴 분리 진단의 CK PNG는 수정 전과 바이트가
같았다. 따라서 이 표본의 점수 상승 원인을 그 face 변경으로 설명하지 않는다. 실제 상승은
새 공통 radialCircle 전달 후 관측했다. CanvasKit 준비 실패 후 실제 Canvas2D로 대체된
경로는 원 fallback gate의 exactDiffPixels0을 확인해 별도로 기록하며 CK 성공으로 세지 않는다.

독립 자료는 `mydocs/pr/assets/issues7727_7732_20261114/canvaskit-common-3f7e4340/`에
선별 보존했다. 원본389쪽 새 WASM 실측과 최소 글꼴 전체 회귀·lint 묶음은 새 source에서
진행 중이다. Native Skia/직접 Canvas의 그리기 경로는1123f322와 바이트가 같고 추가 Rust
제품 변경은 paint JSON 투영뿐임을 재사용 감사에 기록했다. 과거 실행 SHA를 새 실행으로
바꾸어 표기하지 않는다.

### 2026-11-15T07:35:17+09:00 — 최종 source 검증과 제출 증적 확정

source3f7e4340의 최소 글꼴 전체 nextest는13,565PASS·0FAIL·45skip(테스트461.749초),
필수 Rust14단계는 전부exit0이다. Native Skia lib는3,852+15+165+2PASS(12ignored),
placeholder2건과 P37 독립 PDF 경로도 통과했다. Studio 전체는2,791PASS·0FAIL·2skip,
tsc·build·shader replay 계약도exit0이다.

새 fresh WASM389쪽을 전부 산출해 최저98.13%와 기존 자격 PNG389개 바이트 동일을 확인했다.
추가 Rust 제품 차이가 paint JSON뿐인 점과 실제 읽기 경로 SHA를 기록해 Native389쪽·보호36조건
3,542쪽의 기존 실행을 재사용한다. 공식 보호 TSV 하락0·상승0, 편람 Native gate PASS다.

canonical Visual Sweep과 같은 정상 설치 프로그램을 공통 Rust API로 선언한 실제 Studio
Canvas2D Print15쪽은 최저98.21%다. 대표7쪽의 review와 overlay를 직접 판독했다.
설치 SFNT의 실제 bytes를 공급한 플랫폼 adapter 조건을 기록하며 실제 OS 열거 검증이나
일반 자동 선택 경로의 모든 피델리티 해결로 확대하지 않는다. 지원 CanvasKit 권고안4쪽은
같은 조건98.26%로 회복했고 의도된 Canvas2D fallback은 CK PASS로 세지 않는다.

최종 증적은 `canvaskit-common-3f7e4340/`에 보존한다. 다음 단계는 승인된 PR 생성·정확한 code
head CI·review/오늘할일 trailing·trailing CI·병합 및5개 하위 이슈 완료 처리다.
