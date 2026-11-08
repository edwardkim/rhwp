# 이슈 #7642 — 근호의 논리 em과 피근호식 글자 전진의 소비

시작 커밋: `7ce0b246d`. 기존 회귀 실패는 모두 현재 브랜치에서 보정한다.

## 독립 근거와 원인

Stage28의 정상 한컴 Save/Print에서 숫자·변수·근 지수·분수·중첩 근호
70건 모두 `부모 폭 − 몸체 폭 = 논리 em + 0.17 × 글자 크기`가 일치했다.
현재 `layout_sqrt`는 명목 1em과 0.1em을 사용하므로 소속은 맞아도
몸체의 원점·근호의 예약 폭·뒤 수식의 전진이 다르다. 글꼴의 논리 em은
짝수 ppem의 공통 전진 메트릭을 소비해야 한다. 근호·각 기호의 독립
UPM 1024 전진과 70건의 관측을 연결하며 문서 ID·절대 px 좌표를 쓰지 않는다.

소비 경로는 `논리 em → layout_sqrt의 몸체 x와 전체 width → Row/inline 점유 →
SVG·Canvas·Skia의 몸체와 근호 종료점`이다. Source face 외 경로는 기존
공개 MATH 글꼴 계약을 유지한다. 세로 여백을 가로 여백과 같은 수치로 바꾸지 않는다.
높이·기준선·그림 기하는 아직 독립 관측과 대조하여 보정할 범위다.

7pt `x`의 자연 전진은 375HU, 정상 한컴 전진은 450HU였다. 단순한 근호 폭
보정만으로 몸체의 잘못된 전진을 숨기지 않는다. 이탤릭 알파벳을 14 크기로
독립 Save/Print하고 글리프 원점·저장 폭을 대조해 정체·이탤릭의 공통 논리
크기와 힌팅 차이를 확인한다. 사설 글꼴 바이트·윤곽은 커밋하지 않는다.

## 검증과 남은 범위

원래 근호 28건, 추가 154건, 이탤릭 대조 입력과 원본 문제 페이지를 먼저
Native로 비교·확대 판독한다. 90% 미만인 페이지와 미세 차이를 보존한다.
fresh WASM과 관련 전체 페이지가 선행 기준을 만족한 뒤에만 필요한 기존
검사를 소속·순서·보존·포함·겹침 관계로 변경한다. 새 golden을 만들거나
기준값을 완화하지 않는다. 전체 회귀 110건·52개 원본 검증은 계속 수행한다.

## 몸체 글자 전진의 독립 측정

`pdf/issue7642-italic-font-advance/`에 입력 HWPX·정상 HWP·Print PDF를 만들었다.
14 크기의 56개 문자열이다. M/Z/m/z 뒤에 대조 글자를 두어 마지막 글자의
전진도 다음 글리프의 원점으로 읽었다. 52개 글자 각각의 전진 합이 정상
저장 폭과 56/56 일치했다. 7pt `x`는 450HU이며 현재 자연 전진 375HU와 다르다.
7/8pt·10/11pt·13/14pt의 같은 논리 크기는 52개 글자 전진이 전부 같다.

Save job `cb64ee78-2f4f-45ba-8d72-d6ef58c11ce5`, Print job
`c20194bb-bab4-4b36-9014-c6d157792259`의 정상 산출을 확인했다. 한컴은
`11.0.0.9136`, Print는 2쪽이며 `pdf_print_method: 0`과
`hancom2020_pdf_driver_one_up`이다. 입력·출력 해시와 원점·합산 검사는 ignored
증적 경로의 `italic-font-advance-independent-print.json`,
`italic-font-advance-print-glyphs.json`, `italic-font-advance-measured-glyphs.json`에 있다.

이 독립 메트릭을 `source_italic_advance`로 묶어 측정과 paint가 공유할 글자
원점을 구성한다. 다른 face·굵은 글자·독립 메트릭 없는 크기의 fallback을
검증된 이탤릭과 혼동하지 않는다. source font 바이트·윤곽은 포함하지 않으며,
진단 파일은 회귀 fixture/golden을 새로 추가한 것이 아니다.

근호만의 중간 빌드는 exit 0, SHA-256
`087b22afb4da45fe007cab1497f95c2ee53bcd45478abf9255e3eff7f9d334d8`다.
`root-logical-em-native-provenance.json`으로 소스를 고정한다. 이 빌드에는
이탤릭 메트릭의 후속 구현이 없으므로 최종 코드의 검증으로 보고하지 않는다.
근호·이탤릭을 함께 반영한 새 빌드와 직접 시각 검증은 진행 중이다.

## 현재 구현의 소단계 검증과 미충족 범위

근호·이탤릭을 함께 반영한 Native 빌드는 exit 0이며 바이너리 SHA-256은
`ae6eb958b49a59469d3b0b58f5a6de55561ca143292944b5759c987692488bb4`다.
`root-italic-advance-native-provenance.json`으로 코드·명령을 고정했다.
Native Clippy는 exit 0(53.02초), 포맷과 `git diff --check`도 exit 0이다.

| 실제 확인 범위 | 결과 | 증적 |
| --- | --- | --- |
| 이탤릭 문자열 56건의 공통 layout 폭 | 정상 한컴 저장 폭과 56/56 일치 | `root-italic-advance-width-contract.json` |
| 정체·함수 98건의 공통 layout 폭 | 98/98 일치 유지 | 같은 계약 파일 |
| 단순 근호 70건의 전체 폭 | 67/70 일치. 8·11pt 분수 몸체와 8pt 근 지수의 폭 차이가 남음 | 같은 계약 파일 |
| 이탤릭 대조 Native 전2쪽 | 최저 97.70073%, 누락 없음, exit 0 | `root-italic-advance-diagnostic-native/italic-advance/summary.json` |
| 기존 근호·함수 대조 Native 1쪽 | 94.57617%, 누락 없음, exit 0 | `root-italic-advance-diagnostic-native/radical-function/summary.json` |
| 원본 2022년 11월 18·21쪽 | 96.87208%, 97.03835%, 누락 없음, exit 0 | `nov2022-root-italic-advance-native/summary.json` |
| 추가 근호 대조 Native 전5쪽 | 4쪽 89.64494%, 5쪽 82.69128%, exit 1 | `root-italic-advance-diagnostic-native/radical-frame/summary.json` |

독립적인 근호 추가 폭 규칙 70/70과 전체 구현의 67/70을 구분한다. 분수
몸체의 잔여 폭과 복합 근호의 차이는 테스트 허용치로 숨기지 않는다. 특히 추가
대조 5쪽은 이전 84.23074%보다 낮아졌다. 공통 예약 폭과 기존 근호 그림 기하가
서로 다른 계약을 소비하는 남은 차이를 직접 확인하고 후속 보정을 진행한다.
원본 21쪽의 개선이나 글자 전진의 일치를 이 미달의 해결 근거로 사용하지 않는다.
현재 변경은 중간 구현이며 PR 제출·승인 상태가 아니다. 새 회귀·기대값·golden은
변경하지 않았다. fresh WASM·전체 회귀·나머지 필수 lint는 최종 코드에서 실행한다.
