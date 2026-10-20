# 이슈 #7642 Stage 63 — 수식 기준선·숫자 격자·연속 미주 경계 보정

Issue: #7642

## 유지하는 완료 범위

원래 55개 입력·853쪽의 Native/fresh WASM 전쪽 TSV, 알려진 영역의 확대
판독, #1139 전체를 포함한 관련 회귀의 관계형 전환과 전체 Rust/필수 lint는
계속 남아 있다. HML 수식 이관은 유지한다. 기존 회귀 기대값은 아직 바꾸지 않았다.

## Stage 62 실제 결과

고정 source 00bac93e46988fb9a6072dc4bac78a28920ac035의 Native 빌드는
exit 0, 631.01초다. 바이너리 SHA-256은
58eb3e96ab518e3b1fc9d7f339ca87f77e21fc7f56bd0af0e8bac6543a1762ed다.
fmt check는 exit 0, 31.78초, Native Clippy는 exit 0, 113.53초다.

| 대조 | 최저값 | 판정 |
| --- | ---: | --- |
| 원본 비-TAC LADDER 한 쪽 | 79.64318% | 이전 52.11970%보다 개선, 여전히 미달 |
| 48개 LADDER/SLADDER/MATRIX 진단 | 59.18202% | 기준선·일반 행렬의 알려진 차이, 미달 |
| 명령 경계·이항 기호 | 98.78751% | 실제 설치 HY 표시가 진단 투영과 같은 점수 |
| 네 크기 이항 기호 | 96.45015% | 90% 이상, 최신 확대 판독 범위 보존 |
| 인용/비인용 간격 | 99.21736% | 90% 이상 |

원본 review_001.png와 15쪽 단 하단의 새 확대 그림을 직접 표시했다.
위 통과 범위는 현재 추가 변경의 결과나 전체 승인 증거로 승격하지 않는다.

## 수정 전 독립 관측과 원인

1. 비-TAC 원본은 HancomEQN, BaseUnit 1600, 저장 폭/높이
   5910/5445 HU, 기준선 61%다. Stage 62 자연 폭은 76.8이고
   자연 기준선은 높이의 1/2이다. paint 원점에 7.98만큼 추가되어
   괘선과 숫자가 Print보다 아래로 밀렸다. 수치로 원점을 당기지 않고
   한 줄 메트릭에서 줄 묶음의 기준선을 생산한다.
2. 48개 독립 Save의 1/2/3행은 네 크기에서 일관된 기준선 %를 보인다.
   한 줄의 논리 기준선과 행 간격의 절반에서 정수 %를 얻고,
   묶음 중앙으로 옮긴 뒤 저장 정밀도인 정수 %로 올린다.
   결과는 layout_box에 한 번 보존하고 paint가 저장 기준선과 정렬한다.
3. HancomEQN과 HYhwpEQ의 공유 수학 PUA 273개 중 164개의 전진폭이
   다르다. 두 face를 같은 것으로 치환하면 안 된다. 수학 숫자만
   공통 UPM 1024/advance 512를 확인했으므로 Number의 전진폭만
   같은 논리 격자에서 계산한다. 사설 폰트·윤곽은 추가하지 않는다.
4. 일반 MATRIX 대조는 열별 최대 폭 + 0.2em 간격이며 괄호 여백이 없다.
   기존 0.8em 간격과 양쪽 0.2em 여백은 독립 저장 폭과 맞지 않았다.
   첫 줄부터 논리 기준선을 정렬하고 마지막 줄은 표시 상자 끝도 포함한다.
   괄호 있는 행렬은 이 Print 대조만으로 같은 변경을 적용하지 않는다.
5. all-17의 15쪽에서 문23~문26 제목은 Print와 맞는다. 문26 본문
   s0:p222:ci0:note1은 앞 제목과 저장 줄 경계가 연속이며 문단 앞 간격도
   0이다. 공유 HeightCursor의 title_body_gap 경로가 이미 소비한 줄 간격에
   추가 10을 더했다. 디버그 관측은 y_in 985.35 → result 995.35다.
   Print의 첫 본문과 마지막 사건 B 첫 줄은 그 추가 간격이 없다.
   source-유효성이 확인된 연속 저장 경계에서는 실제 소비한 y를 사용하고
   그 원점에 저장 기준도 연결한다. 기존 예외의 수치를 바꿔 맞추지 않는다.

## 실제 소비 경로

- 수식: parser → EqLayout의 숫자 전진폭·행 프레임 → 전체 기준선 →
  layout_for_paint → SVG/Canvas/Skia/Studio의 동일 layout_box.
- 미주: 저장 경계 유효성 → HeightCursor의 제목/본문 원점 →
  endnotes/measure의 fit·예약과 layout/build_single_column의 실제 배치.
  두 호출 지점 모두 같은 커서를 쓰며 원점 이후의 분기도 재검증한다.
- 저장 분할이 dirty이거나 구현이 만든 줄은 새 경계 근거로 수용하지 않는다.
  새 단의 리셋·비연속 저장 gap은 이 변경의 적용 대상이 아니다.

## 소스 진단과 다음 검증

같은 48개 저장본의 폭·높이·기준선과 현재 실제 layout 모듈을 대조했다.
45개는 세 메트릭이 모두 일치한다. 나머지 3개는 결과 행에 빈 칸이 있는
사례로 높이가 25 HU 다르며, 폭과 기준선 %는 맞는다. 이를 완전 일치로
보고하지 않고 새 Native/fresh WASM 확대 판독에서 확인한다.
일반 MATRIX 4개는 이번 변경 뒤 세 메트릭이 모두 일치했다.
원본에 HWPUNIT 또는 px 값 자체를 복사해 맞추지 않았다.

두 face의 독립 Print를 추가 대조하기 위해 같은 48개 입력의 font만
HancomEQN으로 바꾼 MCP Save/Print를 시작했다. 원본을 대체하지 않는다.

변경을 함께 커밋한 뒤 고정 target/pr-review에서 Native 빌드,
원본 LADDER·두 face의 진단·15쪽과 영향 미주 대조를 실행한다.
실제 90% 이상과 확대 판독 전에는 회귀 기대값이나 PR gate를 갱신하지 않는다.
전체 55개 검증과 fresh WASM·전체 회귀·필수 lint는 계속 수행한다.
TSV/JSON/로그/nextrun은 ignored output에만 보존한다.

## 추가 Print 완료와 포맷 결과

HancomEQN 48개 입력도 2024 Save/Print가 완료됐다.
Save job 8788a0c5-34c3-472e-b97a-58b9b0758109,
Print job 251c1260-bc32-4aa4-97cf-1326f12c4977이다.
13.0.0.3901, PrintMethod 0, frame_print_to_pdf_ex_one_up이며
PDF SHA-256은 60c3fc5eec33e5b262edc8886e6802cf5ee5ecda106bc9d43b4322fb853d0b0c이다.
저장본의 font가 HancomEQN으로 남았음을 실제 parser에서 확인했다.
`cargo fmt --all -- --check`와 `git diff --check`는 exit 0이다.
신규 소스 검사는 추가하지 않았고 source-side 기대값도 바꾸지 않았다.
다음 고정 소스의 빌드·확대 판독 결과는 별도로 기록한다.

## sample16의 다음 조사 경계

기존 고정 Native source f6f51dd5d의 22쪽 첫 항목은 원본 문단 439다.
독립 Print에서는 같은 문장이 21쪽 마지막 본문에 있고 22쪽은 문단 440의
4번 제목으로 시작한다. 이 차이는 최신 수식 보정의 해결로 보고하지 않는다.

원본 439는 저장 LineSeg가 없고 ParaShape 앞 간격 1704 HU,
13pt/-12% 글자 간격이다. 440은 pageBreakBefore가 false이며 저장
첫 vpos는 852 HU다. 엔진의 HWP3 변환본 앞 간격 생산·측정·배치가
동일한 결과를 쓰는지, 이전 쪽 마지막 문단의 실제 점유를 확인한다.
현재 실행 중인 81989aafb 빌드의 Rust 소스는 유지했다.

stage63-ladder-before-frame-correction-detail-review.png는 실제 source
00bac93e4의 원본 확대 대조다. 전체 쪽 79.64318% 및 내려간 괘선·숫자를
확인했고 사용자에게 표시했다. 현재 source 81989aafb의 결과가 아니다.

## 실제 보정 후 Native와 경계 진단

Source 81989aafbc9b92f044a30177ce899ad1cb347539의 Native 빌드는
exit 0, 481.25초다. 바이너리 SHA-256은
225401625649e89a89d321d5ddfc0508a8f37e47d2c25a57403e8f8054088963이다.
fmt check는 exit 0, 24.36초, Native Clippy는 exit 0, 81.15초다.

원본 수식-문자처럼취급-아님.hwp의 전체 한 쪽은 100.00000%다.
48개 HYhwpEQ 진단의 전체 세 쪽 최저값은 99.72145%다.
원본의 보정 후 확대 stage63-ladder-after-frame-correction-detail-review.png를
직접 표시하고 괘선·숫자가 Print와 같은 원점에 놓임을 확인했다.
전체 PNG/TSV와 새 source를 연결했으며 이전 캡처 재사용이 아니다.
WASM이나 전체 입력의 통과 판정으로 바꾸지 않는다.

실제 원본 s0:p222의 제목/본문으로 새 경계 helper를 실행했다.
원본 경계와 렌더용 미주 번호 삽입 뒤 경계는 PASS다.
편집 무효화, 구현이 만든 줄, 새 단의 리셋, 비연속 gap은 모두
수용하지 않는 검사도 PASS다. 진단은 ignored output에 있고
정식 렌더링 회귀를 추가한 것이 아니다.

Native lint가 종료됐으므로 같은 source의 fresh WASM 준비가 가능하다.
wrapper는 루트 pkg와 Studio public을 동기화하고 양쪽 SHA-256을 확인한다.
실제 WASM 빌드 종료와 원본 대조 전에는 완료로 판정하지 않는다.

## 두 face 및 미주 단 경계의 최신 Native 결과

같은 source 81989aafbc9b92f044a30177ce899ad1cb347539에서
HancomEQN 진단 세 쪽의 최저값은 99.54211%다. HYhwpEQ의 99.72145%와
구분한다. 원본 all-17의 14/15/16쪽은 각각 98.17741/96.01636/96.51274%다.
이전 source f6f51dd5d의 15쪽 58.32906%에서 96.01636%로 개선됐다.
새 review_015.png를 실제로 표시했다. 사건 B의 첫 줄이 왼쪽 단 하단에
남고, 다음 줄부터 오른쪽 단에 이어지며 문27/문28의 위치가 Print와
대응한다. 수식 기호와 세부 간격의 추가 확대 판독은 별도로 남긴다.

이전 f6f51dd5d 전쪽 Native/WASM 실행기는 실제 /proc의 명령·부모/자식
소유를 확인한 뒤 최신 소스 실행으로 전환했다. 입력·PDF·PNG·TSV·기존
결과를 삭제하지 않았다. ignored stage63-superseded-whole-stop-ledger.json에
대상 PID와 SIGTERM을 기록했다. 종료 사유는 최신 검증 소스로의 전환이며
출력 공백·시간 초과로 빌드를 중단한 것이 아니다. Native 새 전체 실행은
원래 55개·853쪽을 대상으로 한다. fresh WASM 완료 뒤 같은 소스로
대표 영향 쪽과 전체 55개를 실행하는 후속 실행기도 준비했다.

## sample16: 원문 경계와 실제 fit 조건

현재 source의 전체 dump-pages와 source profile을 읽었다. HWP3 계보는
남아 있지만 실제 HWP5 컨테이너이므로 hwp3_layout=false,
native_hwp5_layout=true다. 따라서 HWP3 spacing_before *2 분기는
현재 경계의 원인이 아니다. 기존 제목·단위 추정만으로 변경하지 않는다.

21쪽의 마지막 원본 문단 438 뒤 흐름 높이는 939.1466667이며
본문 가용 높이는 971.2533333이다. 원본 439는 저장 LineSeg가 없고
앞 간격 11.36, 줄 높이 17.3333333, 뒤 줄간격 5.2266667이다.
Print는 439의 문장을 21쪽에 두고 22쪽을 440의 장 제목으로 시작한다.
현재 no_lineseg_heading_precedes_ordered_list는 다음 문자열의 숫자+점만
보고 439를 제목으로 판정한다. 결과적으로 fit_h=33.92의 전체 전진량을
요구하여 439가 다음 쪽으로 간다. 단순 안전마진 변경으로는 이 조건을
고칠 수 없다. 구체적 본문·제목 구분과 원래 #7160 대조 경계의 실제
배치/측정 차이를 함께 조사한다. 기존 판정식이나 회귀를 아직 바꾸지 않았다.

새 전체 Native 산출물의 6쪽 raster를 Visual Sweep compare/overlay/review
helper로 합성했다. 점수 38.80217%로 미달이며 사용자에게 직접 표시했다.
이 쪽은 본문·표가 Print보다 위로 이동한 차이로, 21쪽의 마지막 문단 이월
차이와 같다고 판정하지 않는다. ignored provenance에 source, 원본/Print
경로와 실제 raster 해시를 연결했다. 21~24/26/41쪽과 앞뒤 경계의 분석은
계속 남는다. 현재 fresh WASM의 Rust 소스는 유지하고 있어 빌드 이후
새 변경에는 영향 쪽과 해당 WASM을 다시 검증해야 한다.
