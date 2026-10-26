# #8153: 재조판한 단독 글상자 줄의 점유와 음수 간격

## 사양·원본·독립 기준

같은 두 편람 원본·383쪽의 독립 2024 Print와 96dpi를 유지한다. HWP5 §4.3.4
표 62는 줄 높이와 줄간격을 각각 INT32로 구분한다. 문단의 Percent 간격과 실제로
재조판한 줄 상자를 함께 소비하며, 빈 글자 표시와 TAC 개체의 줄 점유를 구분한다.

289쪽 원본 section10/para26은 글자 없는 TAC 사각형 글상자다. 원본 프레임은
2400HU, 내용 때문에 커졌던 currentHeight와 저장 줄 높이는 4066HU다. 그 안의 짧은
제목을 현재 폭에서 한 줄로 재조판하면 프레임도 2400HU가 된다. 문단의 글자모양
340은 1250HU, 줄간격은 90%, 뒤 간격은 1000HU다. 새 구성 줄은 2400HU 점유와
1250×(90−100)/100 = −125HU의 간격을 각각 소유한다. 원본 저장 간격 −124HU와
새 계산값을 혼동하지 않는다. 두 값의 차이는 1HU이고 출력 차이를 맞춘 상수가 아니다.

독립 Print의 표 상단은 191.02132px, 첫 답변 기준선은 302.23999px다. source9bab의
실제 Native에서 표 상단이 약 1.7px 아래이며, 289쪽에는 23개의 미대응 픽셀이 있다.
이 차이를 글자 기준선에 직접 더하거나 그림/표를 이동하는 방식으로 보정하지 않는다.

## 기존 해결과 실제 소비 경로

기존 a78053a141은 PR #7611 검토 과정에서 내용 증가로 커졌던 글상자와 호스트 줄을
함께 재조판하도록 했다. `mydocs/pr/archives/pr_7611_review.md`의 당시 보정·검증
범위와 실제 코드 변경을 읽었다. 이후 7488ca31e4는 일반 TAC의 줄 점유를 유지하고
독립 저장 사다리가 닫힌 음수 간격 호스트만 별도 단축에 수용한다. 그 저장 계약과
#6551의 문단 앞 간격, #6665의 양수 저장 꼬리 간격을 보존한다.

실제 경로는 `normalized_single_line_inline_textbox_host → reflow_line_segs →
RenderNormalizedSection의 같은 paragraphs/composed → HeightMeasurer·typeset의
줄 높이/간격 → layout_composed_paragraph_in_frame → TAC Shape 높이 바닥값`이다.
재조판한 줄도 저장 guide처럼 접어 높이를 0으로 만든 뒤, 뒤의 바닥값이 도형 높이와
문단 뒤 간격만 다시 더해 음수 줄간격을 잃는지 확인한다. source9bab의 실제 공개
렌더 트리에는 높이 32px의 도형과 그 아래 높이 0의 호스트 TextLine이 동시에 있다.

## 보정과 검증 계획

원래 저장 guide 줄을 접는 기존 분기에서, 실제 재조판 줄은 제외한다. 측정과 조판이
이미 소비한 같은 composed 줄 높이/간격을 paint도 사용한다. 원본 저장 줄의 특수
회계, backend별 처리, 문서명/쪽/상수 분기를 새로 추가하지 않는다.

두 원본 289쪽의 줄이 자기 TAC 프레임을 포함하는지와 뒤 표까지의 signed 간격을
공개 API 진단으로 수정 전 FAIL/후 PASS 대조한다. 원본 저장·쪽수·소유와 원문 순서를
유지하고, 직접 Print 확대 이미지·Native/fresh WASM과 주변 정상 쪽을 확인한다.
#6551·#6280·#7196·기존 #8153 글상자/문단 경계도 확인한다. 새 회귀는 시각 선행
조건을 충족한 뒤 `tests/cases/issue_8153_*.rs`에 등록한다. 전쪽 Native/fresh WASM,
최종 표준 최적화 WASM·전체 Rust·lint·Skia·Studio와 PR 준비는 아직 완료되지 않았다.

진단 원문은 ignored `remaining-raw100-diagnostic/continuous-text-shadow/
p289-301-source-contract/`의 원본 XML, 독립 PDF 원점·도형, 실제 flow trace와
`diagnose-inline-frame.log`에 있다. upstream 동기화·원격 쓰기는 하지 않는다.

## sourceb3ad의 실제 Native 검증

`b3adcdc39c7bd368acfcdabe4bf9bc330f7fbf86`의 Native 빌드는 고정 target/pr-review와
jobs 8에서 6분 55초·exit 0이었다. 원본 HWP/HWPX의 같은 진단 파일이 이전 source9bab에서
2 FAIL, 새 source에서 2 PASS였다. 이전 호스트 TextLine은 도형 바닥에 높이 0으로 있었지만,
새 줄은 도형과 같은 y=147.41333px·높이 32px를 포함한다. 뒤 표의 상단은
192.74667 → 191.09333px로 바뀌어 독립 Print 191.02132px와 차이가 약 0.072px다.
작은 위치 공차를 새로 맞춘 것이 아니라, TAC 포함 관계와 원본 Percent의 signed 간격을
각각 검사했다. 원본 HWP 내보내기 바이트도 전후 같다.

기존 #6551·#7196·#6280·#8153 문단/그림/이어받기/글자 폭/한자와 실제 overflow 대조를
포함한 16개 harness는 총 35 PASS·0 FAIL·0 ignored였다. 이는 전체 Rust suite가 아니다.
두 원본 36쪽씩의 Native 72조건을 같은 PDF·DPI·공급·마스크로 대조했고 미대응 증가가 없다.
289쪽은 두 형식 모두 23 → 0픽셀, raw 99.98886 → 100%로 개선됐다. 291·301쪽의
각 20픽셀은 이 수정으로 해결되지 않았으며 별도의 셀 그림 clip 경계로 추적 중이다.
136·166·228쪽과 기존 정상 TAC 경계도 유지했다. 289쪽 review PNG를 직접 확인했다.

`inline-shape-native/native-focus/result.json`, `inline-shape-native/contracts-result.json`,
`inline-shape-original-contract.log`와 문서별 `p289/review/review_289.png`에 연결한다.
전쪽 Native와 fresh DEV WASM은 별도로 계속 실행한다. 이 결과를 최종 표준 최적화 WASM이나
전 1,532조건의 100%·정수 미대응 0, 정식 새 회귀 등록 또는 최종 전체 검증 완료로 쓰지 않는다.
