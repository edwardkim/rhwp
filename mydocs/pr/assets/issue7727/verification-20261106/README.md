# #7727 CBTA 원본 전쪽 검증과 기존 회귀 교정

고정 devel `170b886f167b705b6e630c5b34bb638d92fae0a9`와 코드
`fc378076f23aa60838a2871a7532d09830384216`을 같은 원본·독립 Print로 비교했습니다.
이번 변경은 기준 PDF 세대와 기존 검사의 대상 선택 교정이며 renderer는 바꾸지 않았습니다.
원격 PR·merge·이슈 종료 및 전체 오래된 이슈 작업 완료를 뜻하지 않습니다.

## 독립 Print와 원본 보존

원문 `samples/issue6764/1613000-202200037-air-traffic-controller-cbta.hwp`의 SHA-256은
`8ef9de3f35690bf9d7994527f77cb02d4a4fcff447c219a78fbc2855d64be6e7`입니다.
기존 `pdf/1613000-202200037-air-traffic-controller-cbta-2020.pdf`는 이름과 달리 실제
Creator가 `Hwp 2022 0.0.0.0`입니다. 이 PDF와 원문은 덮어쓰지 않았습니다.
그 기준의 32쪽 raw Native 89.06875% / WASM 89.10083%도 이력으로 유지합니다.
색 경계 보조값이나 비교 예외로 미달을 통과 처리하지 않았습니다.

원문은 한컴 2018 저장본입니다. 새 Print 영수증 (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue7727/verification-20261106/independent-print-receipt.json`)의
job `a4dfe6a7-10fd-4155-925e-c1baba91e1a2`는 engine2020에서 61초에 성공했고,
Hwp2020 `11.0.0.9136`·PrintMethod0·`hancom2020_pdf_driver_one_up`·204쪽을 확인했습니다.
[새 정식 PDF](../../../../../pdf/issue7727/cbta-original-hancom2020-print-20261106.pdf)는
4,970,756바이트, SHA-256 `0db1eb55eda0f1a4985eeaf187656c23499616b7dccddcbb7c874c0937e1b2e0`입니다.
기존 2022 PDF의 그림 JPEG 전달에 맞추려고 올바른 현재 renderer를 변경하지 않았습니다.

## 전 204쪽 비교

| 경로 | base 최저 | 후보 최저 | 90% 미만 | 쪽 누락 | 같은 base/후보 PNG |
| --- | ---: | ---: | ---: | ---: | ---: |
| Native | 92.05232% | 92.05232% | 0 | 0 | 204/204 |
| fresh WASM | 92.16419% | 92.16419% | 0 | 0 | 204/204 |

최저 쪽은 모두 11쪽입니다. [Native base](native-base.tsv)·[후보](native-candidate.tsv),
[WASM base](wasm-base.tsv)·[후보](wasm-candidate.tsv)는 canonical 96DPI·compat2022·
PDFium·2px raw이며 전쪽을 다시 산출했습니다. 새 raw 하락은 0입니다.
producer 재사용 근거 (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue7727/verification-20261106/runtime-reuse-provenance.json`)의 runtime 소스와 실제 PNG hash를
각 전쪽 다시 대조했습니다. 미변경 runtime의 fresh WASM 산출물을 재사용했으며 새 테스트
코드까지 이전 WASM 빌드에 포함됐다고 표현하지 않습니다.
`--png-pair`의 자동 gate는 `not_evaluated`입니다. 그 상태를 자동 PR 승인으로 쓰지 않고
별도 쪽수·내용 소속 감사와 아래 직접 판독을 함께 확인했습니다.

## 대표 review와 overlay

- 11쪽: [Native review](native-p011-review.png)·[overlay](native-p011-overlay.png),
  [WASM review](wasm-p011-review.png)·[overlay](wasm-p011-overlay.png).
  본문·줄 소속은 보존되며 미세 간격 차이는 남습니다. 100% 또는 px 핀 자격으로 보고하지 않습니다.
- 32쪽: [Native review](native-p032-review.png)·[overlay](native-p032-overlay.png),
  [WASM review](wasm-p032-review.png)·[overlay](wasm-p032-overlay.png).
  새 Print raw는 99.50563% / 99.49742%입니다.
  [기존 2022 review](old-hancom2022-native-p032-review.png)·[overlay](old-hancom2022-native-p032-overlay.png)도 보존합니다.
- 185쪽: [Native review](native-p185-review.png)·[overlay](native-p185-overlay.png),
  [WASM review](wasm-p185-review.png)·[overlay](wasm-p185-overlay.png).
- 186쪽: [Native review](native-p186-review.png)·[overlay](native-p186-overlay.png),
  [WASM review](wasm-p186-review.png)·[overlay](wasm-p186-overlay.png).
  앞 29×3 표의 계속 조각·과목 2 제목·후속 46×3 표의 순서와 번호가 보존됩니다.
- 187쪽: [Native review](native-p187-review.png)·[overlay](native-p187-overlay.png),
  [WASM review](wasm-p187-review.png)·[overlay](wasm-p187-overlay.png).
  후속 표의 원문 21~45행이 해당 쪽에서 계속됩니다.

원문 소유 감사 (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue7727/verification-20261106/source-content-audit.json`)는 양쪽 원문 155개 셀의 전쪽 조각을
이어 붙여 글자 순서·누락·중복을 검사했고 차이 0입니다. 29×3 표의 26행 원문 셀
52~54는 185/186쪽에 합법적으로 나뉘므로 그 계속을 중복으로 오인하지 않습니다.

## 회귀 검사 교정과 실제 실행

원 해결 `d87b3037e5`·`29194a603f`·`c547a52b2a`·`26812ee08d`·`cabcd9a07c`는
모두 고정 devel의 조상입니다. 과거 엔진 결함을 이번에 새로 재현했다고 주장하지 않습니다.
BEFORE (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue7727/verification-20261106/old-selector-before.json`)는 기존 인덱스 181에 원문 29×3 표의 소유 사전조건을
추가한 실제 Native 1FAIL입니다. 현재 그 쪽은 landscape 247×7 표이며 고정 1122.5px도 다른 용지입니다.

기존 파일·검사 이름을 유지해 원문 구역 2·문단 1/4·컨트롤 0의 표 소속, 실제 용지와 SVG paint,
제목의 글자·번호·앞뒤 순서, 전 155개 셀 내용과 독립 204쪽을 검사하도록 교정했습니다.
임시 전쪽 최대 초과 100px를 새 허용치로 늘리지 않았습니다.
185~187쪽은 Native/WASM 모두 99.9%대라 독립 PDF 괘선 중심·단어 bbox (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue7727/verification-20261106/independent-print-anchors.json`)를
96/72로 환산한 ±2px 보조 검사를 추가했습니다. 182쪽 96.23%대는 px 대상으로 쓰지 않습니다.
AFTER 최소 글꼴 Native (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue7727/verification-20261106/native-focused.json`)는 4PASS/0FAIL입니다.
이 결과는 대상 선택의 BEFORE 1FAIL → AFTER 4PASS이며 새 renderer 보정이 아닙니다.

기존 corpus 이관 목록의 재활성화는 아래 복원 단계에서 별도로 확인했습니다.
커밋 `98e5afbb7a`에서 Native Skia 필수 3종 (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue7727/verification-20261106/native-required.json`)을 Mac 최소 글꼴 환경으로
순차 실행했습니다. 라이브러리 4,039PASS/0FAIL/12ignore, PNG 2PASS, 직접 PDF 22PASS이며
Native feature 전체 target Clippy와 fmt도 통과했습니다. 기본 전체 nextest로 이 세 검사를
대신하지 않았습니다. 새 검사 포함 전체 nextest·나머지 범위별 lint 묶음·최종 제출 head 편람
gate·다음 오래된 이슈 작업은 아직 완료로 표시하지 않습니다.


## 기존 코퍼스 검사 복원

기존 원장과 같은 계산 (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue7727/verification-20261106/corpus-count-comparison.json`)에서 고정 devel/후보 모두 전 204쪽의
본문 하단 넘침·용지 밖·글자 겹침·셀 줄 넘침이 각 0임을 확인했습니다.
복원 범위 (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue7727/verification-20261106/corpus-restoration.json`)의 원본 두 경로를 공통 6개 축의 제외 TSV에서 해제하고,
본문/글자 겹침의 개별 제외 목록도 해제했습니다. 본문·글자 겹침·용지 밖 원장은
이 입력에만 0을 명시합니다. 다른 문서의 행·문턱·예외·입력은 유지합니다.

Native 실행에서 복원 전 제외를 검출한 3PASS/1FAIL (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue7727/verification-20261106/corpus-before-native.json`) →
복원 후 4PASS/0FAIL (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue7727/verification-20261106/corpus-after-native.json`)을 확인했습니다. 등록된 검사 수는 4건으로 같습니다.
본문과 그림 배치는 바뀌지 않으며, 새 전체 회귀에서 복원한 코퍼스 수집 경로도 실행합니다.


## 코퍼스 복원 커밋의 최종 로컬 검증

커밋 `9b4c850535b3d260abecc22f833f878c804b3952`의 최종 검증 (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue7727/verification-20261106/final-gates.json`)은
Mac 최소 글꼴·jobs 8·threads 8·Cargo 순차 조건입니다. 전체 nextest는
12,657PASS/0FAIL/45skip, 테스트 실행411.620초로 완료했습니다. 복원한 네 검사도 실제 포함됐습니다.
Native Skia 라이브러리4,039PASS/0FAIL/12ignore, PNG2PASS, 직접PDF22PASS와
fmt·Native feature/기본/WASM/all-target Clippy·workspace build·고정base manifest/unit-tier가 모두 통과했습니다.
runtime은 이전 코드와 같고 새 WASM 구현 변경은 없으므로 기존 frozen fresh WASM의 producer와
전204쪽 A/B를 재사용했습니다. 새 테스트와 원장은 이 최종 전체 회귀에서 검증했습니다.

#7727의 로컬 기준·회귀 복원 검증은 완료했습니다. 원격 PR·통합·이슈 close와
최종 제출 head의 편람 gate 및 전체 오래된 이슈 목표는 별도로 남아 있습니다.
