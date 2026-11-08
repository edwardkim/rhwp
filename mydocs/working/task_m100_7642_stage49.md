# 이슈 #7642 Stage 49 — 원본 수학 축의 공통 전달

Issue: #7642

## 독립 근거와 원인

시작 source는 `7021d4342fd469298de1de4a4497ffaeee55e236`이다.
정상 한컴 저장·Print의 `1 over 2`를 6·9·12·18pt에서 대조하면 본문 기준선과
분수선 중심의 거리가 명목 em의 0.328257~0.328264다. PDF 글자 행렬의 실제
em으로 환산하면 약 0.3285다. 독립 측정은 ignored output의
`stage48-independent-simple-fraction-axis.json`에 보존했다. 기존 분수는
표시용 Latin Modern Math의 0.25em을 원본 수학 축으로 재사용했다.
합 기호도 같은 축을 쓰며, Studio CanvasKit은 분수선을 본문 기준선에 그린다.

## 구현 계약과 실제 소비 경로

- HYhwpEQ 원본의 축과 일반 표시 글꼴의 축을 구분한다. 문서 ID·절대 px 위치로
  보정하지 않는다. 일반 EqLayout의 기존 0.25em 계약은 유지한다.
- `EqLayout → 분자·분모 배치와 Fraction.axis_height → SVG / Canvas /
  Skia / paint JSON → Studio CanvasKit`에 같은 계산값을 전달한다.
- 원본 합의 논리 기준선도 같은 축으로 정한다. 합의 공개 글리프 자체는 표시
  글꼴의 축을 사용하며, 위·아래 첨자는 부모 원점과 함께 이동한다.
- AST·구조 덤프의 Fraction match는 새 필드를 수용하되 기존 검사 항목과
  기대값을 완화하지 않는다. 렌더링 회귀 변경은 전체 Native/fresh WASM 90%
  선행 조건을 충족한 뒤 진행한다.

## 검증 범위

정상 저장·Print의 전체 수식 32건과 구성요소 32건, 이미 보정한 2022 HWPX
18/19쪽 및 2023 HWP/HWPX 19쪽을 같은 입력과 기준 PDF로 재검증한다.
18pt 합·분수·근호와 사용자 지적 미주 수식은 직접 확대 판독한다.
현재 별도 진행 중인 55개 문서의 이전 source 조사는 최종 head의 통과 근거로
사용하지 않는다. 그림 소유 원점과 뒤 페이지 배치 미달은 별도 원인으로 계속 보정한다.

## 실행 결과

Native release-test 빌드는 493.93초, exit 0이다. 빌드 전후 source 1453개 파일의
해시가 일치한다. frozen binary SHA-256은
`6e9cefb8591f1b463b4f4bc97ff9bbbcd0335454ae6622a58b7594694bfe24c2`다.
포맷 검사와 Native Clippy `-- -D warnings`는 각각 44.22초·106.43초,
exit 0이다. Studio `npx tsc --noEmit`도 exit 0이다.

| 동일 원본·독립 Print | 변경 전 Native | 변경 후 Native |
| --- | --- | --- |
| 정상 저장 수식 진단 1쪽 | 93.31814% | 97.89256% |
| 정상 저장 수식 진단 2쪽 | 89.86917% | 95.46958% |
| 정상 저장 구성요소 32건, 1쪽 | 97.93179% | 98.66658% |
| 2022 HWPX 18/19쪽 | 97.51434% / 97.72013% | 97.60151% / 97.76191% |
| 2023 HWP 19쪽 | 95.94694% | 95.94555% |
| 2023 HWPX 19쪽 | 96.05167% | 96.05956% |

정상 저장 수식·구성요소 진단은 모두 Native에서 90% 이상이다. 이 자료는 기존
원본 corpus의 Print를 대체하지 않는다. 18pt 양쪽 합·분수의 소유 Equation
상자에 1.5em 여백을 더한 새 확대 review는 94.07704%다. 분수선과 합의
기준선 차이는 개선됐지만 합 위첨자의 ink 간격·표시 글리프 인상은 아직 다르다.
이 차이를 페이지 점수로 면제하지 않고 후속 확대 보정을 계속한다.

전체 증적은 ignored `source-math-axis-focused-native/queue.json`,
`source-math-axis-large-sum-detail/provenance.json` 및 그 하위 review PNG에
보존했다. PNG는 실제 새 실행 산출물로 사용자에게 표시했다.

변경된 목표 파일의 HWP/HWPX 56개 경로도 확인했다. 실물 원본 53개 경로 중
`mydocs/pr/assets/issue7445/exam_eng.hwp`는 `samples/exam_eng.hwp`와 바이트가
동일하다. 나머지 `mini_repo`의 세 경로는 12바이트 `placeholder`이며 파일명
대조 도구의 fixture다. 실제 HWP/HWPX가 아니므로 시각 측정값을 만들거나
통과로 집계하지 않는다. 해당 사실은 ignored `updated-goal-source-inventory.json`에
기록했다. 첨부의 제거 커밋 두 개와 브랜치는 현재 로컬 Git에서 확인되지 않았고,
원격의 같은 브랜치도 조회 결과가 없어 제거·복원됐다고 단정하지 않는다.

위 Native 1차 결과만으로 fresh WASM·전체 최종 검증·관계형 전환·PR 준비
완료를 판정하지 않는다. 이후 같은 source의 WASM 결과는 다음 절에 기록한다.

## 같은 source의 fresh WASM 재검증

코드 커밋은 `c1cd0367f3db737ec424262c0bf916774f383926`이다. 루트 wrapper의
web 빌드는 1008.72초, exit 0이며 source 해시를 빌드 전후 대조했다.
frozen package의 JS·WASM과 Studio public 대응 파일의 SHA-256이 각각 일치한다.
WASM SHA-256은
`121161ed98b47cd1532ef85f7937361682ae2e05d1f187b75105ba6b335eab14`다.

앞 표의 7개 페이지를 같은 원본·독립 Print로 fresh WASM에서도 실행했다.
5개 실행의 gate가 모두 `passed`이고 Native와 페이지별 일치율이 같다.
7개 대응 PNG도 SHA-256이 모두 같다. 확대 판독한 Native 이미지와 fresh WASM의
실제 raster가 같은지 `source-math-axis-native-wasm-parity.json`으로 확인했다.
정상 저장 수식 진단 전체 2쪽은 97.89256%·95.46958%, 구성요소 전체 1쪽은
98.66658%다. 2022 HWPX는 18/19쪽, 2023 HWP/HWPX는 19쪽만 재검증했으므로
그 문서의 전체 페이지 통과를 주장하지 않는다. Studio CanvasKit 변경의 실행
검증도 SVG 기반 WASM 비교나 타입 검사로 대신하지 않으며 아직 미검증이다.

최신 목표 파일에서도 입력 56개 경로와 Rust 파일 91개의 목록은 동일하다.
현재 91개 파일은 원래 경로나 `tests/cases` 아래에 모두 존재한다. 첨부의
삭제 상태를 현재 브랜치 상태로 단정하지 않는다. 입력·검사 목록은 ignored
`updated-goal-source-inventory.json`·`updated-goal-regression-inventory.json`으로
연결하고, 미달 원본의 실제 출력 보정 뒤 관계형 전환을 진행한다.
