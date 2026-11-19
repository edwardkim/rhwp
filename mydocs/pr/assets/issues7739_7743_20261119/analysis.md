# #7739–#7743 원본 피델리티 분석

## 원본과 대조 자료

| 이슈 | 원본 | 독립 Print PDF | 쪽수 |
| --- | --- | --- | ---: |
| #7739 | [화학제품 표시도안 연구](../../../../samples/issue6782/1480000-201900042-chemical-product-labeling-study.hwp) | [한컴 2022](../../../../pdf/1480000-201900042-chemical-product-labeling-study-2020.pdf) | 103 |
| #7741 | [분할 표 HWPX](../../../../samples/rowbreak-problem-pages.hwpx) | [한컴 2022](../../../../pdf/rowbreak-problem-pages-hwpx-2020.pdf) | 18 |
| #7742 | [사이버대학 신청서](../../../../samples/issue6795/1341000-201100013-cyber-university-application.hwp) | [한컴 2020 재인쇄](../../../../pdf/issue7742/cyber-university-application-hancom2020-print.pdf) | 45 |
| #7743 | [교육과정 연결맵](../issue7445/education-curriculum-map/1342000_edu_curriculum_map.hwp) | [한컴 2020](../issue7445/education-curriculum-map/current-source-hancom2020-print.pdf) | 415 |

#7740은 #7382 검토 중 복원한 정상 회귀의 범위와 후속 실패를 검토한 기록이다.
당시 110함수 중 94개가 통과했고 16개가 실패했다. 코퍼스 검사 86건은 63건 통과와
23건 실패를 기록했다. 현재 활성 검사에 과거 이름 그대로 남은 함수는 49개다.
아래 원본의 검사와 당시 복원 기록은 [기존 검증 원장](../issue7445/scope_restore_validation.json)에서 추적한다.

## 화학제품 표시도안 연구

16·88쪽의 RGB 32비트 DIB에는 알파 채널이 없고, 한컴 인쇄 PDF에는 같은 이미지가
RGB555 디더링과 JPEG로 기록되어 있다. 원본 DIB에 기존 4×4 디더링과 Q58 4:2:0 인쇄
인코딩을 적용하면 두 PDF JPEG의 디코딩 픽셀을 재현한다. Windows 3.x의 40바이트 헤더와
`BI_RGB` 규격의 32비트 DIB를 기존 인쇄 경로에 연결했다.

64쪽에는 셀 안의 공백 문단 하나에 두 그림이 매달려 있다. 음수 세로 오프셋을 분할
프레임에서 그대로 사용하면 위 행을 침범한다. 그림이 수용되는 경우 같은 셀의 실제
내용 원점을 사용한다. 독립 PDF에서 두 그림의 위쪽은 445.040과 445.041px다(96 DPI).

## 분할 표 HWPX

5쪽 후속 본문의 저장 세로 위치는 앞 표 조각보다 이르다. 표 조각을 끝내며 이미 수용한
앞 간격이 후속 문단에서 다시 적용되어 본문이 아래로 밀렸다. 같은 문단 서식과 원본
위치 관계로 중복 간격을 식별한다. 같은 입력을 한컴에서 다시 인쇄한 대조에서도
후속 첫 글줄 기준선이 동일했다.

11쪽의 제목 간격은 원문이 지정한 한양중고딕 계열의 영문 공백을 사용한다.
아래 표의 조각은 이미 올바르게 나뉘어 있었고, 앞에 남은 마지막 본문 줄의 전진량
440 HWPUNIT가 그리기에서 빠졌다. 원본의 명시적 재시작과 물리 쪽 끝에 일치하는
선언 높이를 확인해 앞 조각을 닫는다. 4.2는 11쪽에, 4.3은 12쪽에 속한다.

## 사이버대학 신청서

1쪽 관인은 원문 `BinData/BIN0001.jpg`에 저장되어 있다. [기존 PDF](../../../../pdf/1341000-201100013-cyber-university-application-2020.pdf) 첫쪽에는 관인이 빠져 있다.
입력 SHA-256을 유지하여 한컴 2020 `11.0.0.9136`에서 `Print`, 방식 0, 1-up으로
다시 인쇄했고 45쪽과 첫쪽 관인을 확인했다. 새 PDF의 첫쪽 수치는 기존 출력에도 적용해
기준 PDF를 교정한 효과를 확인했다.

28쪽의 부모 셀에는 첫 일곱 행을 담은 중첩 표 조각이 있다. 원본 선언의 11077 HWPUNIT는
해당 일곱 행에 맞는다. 그 선언과 부모 본문 절단이 같은 원본 프레임을 닫는 경우 높이를
공유하도록 보정했다. 다음 쪽은 여덟 번째 행을 이어받는다. 인쇄본 28쪽 중첩 표의
위쪽과 아래쪽 선은 694.363과 842.035px다(96 DPI).

## 교육과정 연결맵

- **8쪽**의 흰 글자 위 취소선에는 검정 `COLORREF` 값 0이 명시돼 있다.
  선택 색상이 없는 구형 레코드는 종전의 글자색을 따르며, 명시된 검정은 각 출력에 전달한다.
- **89·93쪽**은 목록 머리가 점유하는 폭으로 새 글줄을 계산한다. 저장 물리 프레임과
  줄 높이를 확인해 같은 상자에서 줄과 문자의 소속을 발행한다.
- **175쪽**은 단어 끝의 ‘기’가 독립 Print에서도 다음 줄에 놓인다. 앞줄 27432 HWPUNIT에
  마지막 글자의 폭 1000 HWPUNIT를 더하면 상자 28408 HWPUNIT를 넘는다. 900 HWPUNIT의
  압축 전진만 사용한 후속 계산은 이 글자를 앞줄에 옮겼다. 원본 저장 줄을 비우기 전에
  같은 목록 상자의 말미 줄 증거를 전달해 탐색과 실제 발행의 폭 판정을 맞춘다.
  250쪽의 ‘갈등 현상 탐색하기’가 한 줄을 차지하는 조건도 대조한다.
- **223·227·230–235쪽**은 병합 칸에서 완료한 줄과 현재 물리 행의 높이 소유가 섞였다.
  앞 조각에서 이미 소비한 긴 병합 칸의 잔여를 독립된 본문 행에서 다시 예약하지 않는다.
  단기 병합의 실제 잔여와 저장 여백은 현재 조각의 수용 기록으로 확인한다.

최종 제출 증적에는 두 백엔드의 전쪽 TSV, 기준과 후보의 입력·출력 해시,
변경 지점의 review와 overlay, 실제 Native 및 Studio 소비 경로 결과를 연결한다.

## 전쪽 비교

고정 base는 `9f605a0d0aa2410754ec1470078d831a0c09688f`, 보정 출력의 생산 소스는
`d77d3e0faa0f75d0bfcd49453aa8f144f6b56b68`이다. 같은 원본과 독립 Print PDF를
96 DPI, 2px 이웃, 색상 경계 보정 전 raw 실루엣으로 각각 대조했다.

| 이슈 | 쪽수 | Native 최저 | fresh WASM 최저 | 실제 raw 100% Native / WASM |
| --- | ---: | --- | --- | ---: |
| #7739 | 103 | 98.79596% (65쪽) | 98.79775% (65쪽) | 73 / 65 |
| #7741 | 18 | 98.32256% (16쪽) | 98.36100% (16쪽) | 7 / 6 |
| #7742 | 45 | 98.31892% (29쪽) | 98.54594% (29쪽) | 7 / 7 |
| #7743 | 415 | 98.21994% (134쪽) | 98.18712% (134쪽) | 376 / 374 |

각 경로에서 581쪽 모두 98% 이상이다. 고정 base 대비 2%p를 초과한 하락은 없다.
사이버대학 29쪽의 Native raw는 98.66897%에서 98.31892%로 바뀌었다. 다음 쪽으로
이어지는 여덟 번째 행과 표의 글자 소속을 28·29쪽 양쪽에서 확인했다.
[전쪽 TSV·최저 쪽·해시·대표 증적 목록](visual-evidence.json)과
[581쪽의 Native/WASM 렌더 트리 글줄 대조](page-content-comparison.json)를 제공한다.

| 이슈와 대표 쪽 | Native TSV | fresh WASM TSV | 직접 증적 |
| --- | --- | --- | --- |
| #7739 64쪽 그림 | [전 103쪽](chemical-6782/native/silhouette.tsv) | [전 103쪽](chemical-6782/wasm/silhouette.tsv) | [Native review](chemical-6782/native/review_064.png) · [overlay](chemical-6782/native/overlay_064.png) · [WASM review](chemical-6782/wasm/review_064.png) · [overlay](chemical-6782/wasm/overlay_064.png) |
| #7741 11쪽 표 하단 | [전 18쪽](rowbreak-hwpx/native/silhouette.tsv) | [전 18쪽](rowbreak-hwpx/wasm/silhouette.tsv) | [Native review](rowbreak-hwpx/native/review_011.png) · [overlay](rowbreak-hwpx/native/overlay_011.png) · [WASM review](rowbreak-hwpx/wasm/review_011.png) · [overlay](rowbreak-hwpx/wasm/overlay_011.png) |
| #7742 28쪽 표 조각 | [전 45쪽](cyber-6795/native/silhouette.tsv) | [전 45쪽](cyber-6795/wasm/silhouette.tsv) | [Native review](cyber-6795/native/review_028.png) · [overlay](cyber-6795/native/overlay_028.png) · [WASM review](cyber-6795/wasm/review_028.png) · [overlay](cyber-6795/wasm/overlay_028.png) |
| #7743 175쪽 목록 말미 | [전 415쪽](curriculum-2287/native/silhouette.tsv) | [전 415쪽](curriculum-2287/wasm/silhouette.tsv) | [Native review](curriculum-2287/native/review_175.png) · [overlay](curriculum-2287/native/overlay_175.png) · [WASM review](curriculum-2287/wasm/review_175.png) · [overlay](curriculum-2287/wasm/overlay_175.png) |

## #7740 회귀 복원 기록

[과거 110개 함수의 활성 위치와 변경 SHA](historical-test-audit.json)를 대조했다. 과거 이름이
유지된 49개는 현재 경로에 남아 있다. 이름이 달라진 61개의 변경 이력에서 #8160에 통합된
sample16 검사, 시험지 이관, 독립 증거에 맞춰 바뀐 기존 검사의 위치를 추적할 수 있다.
과거 실패 중 현재 이름이 남은 #1133, #5846, #6031, #6409, #1811은 집중 재실행에서 통과했다.

대상 원본 네 종의 body, off-canvas, overflow cell, text overlap 제외를 해제했다. 한컴 독립
Print 쪽수를 oracle과 렌더 원장에 반영했으며, 교육과정에서 이관된 83쪽의 도 교육청
소속 검사도 복원했다. 사이버대학 원본의 인쇄 설정에는 모아 찍기 선언이 있으므로
공통 인쇄 장수 oracle의 범위에서 제외된다. 실제로 재인쇄한 1-up PDF의 45쪽은
렌더 원장과 사이버대학 전용 검사에서 고정했다. 415쪽 쪽수의 고정값은 파일 경로를 추적하는 공통 oracle에 한 번 등록했다.
별도 추적 중인 다른 원본의 유효한 기존 검사는 계속 해당 경로에서 수행한다.

사이버대학 10쪽은 Native 98.85684%, fresh WASM 98.95071%다. 상단 셀의 여는 괄호와
닫는 괄호는 각각의 줄에 있고 독립 PDF에서도 각각 보인다. 검사기의 두 run 상자는
높이 14px, 세로 간격 8.4px로 겹친다. 같은 고정 base에서도 동일한 상자 쌍 한 건이
보고되므로 이 문서의 예전 기준 2건을 1건으로 낮춰 래칫을 복원했다.
[Native review](cyber-6795/native/review_010.png) · [overlay](cyber-6795/native/overlay_010.png) ·
[WASM review](cyber-6795/wasm/review_010.png) · [overlay](cyber-6795/wasm/overlay_010.png).

## 로컬 회귀와 저장 대조

최소 글꼴 환경의 최종 전체 회귀 13,831건이 통과했고 45건이 건너뛰어졌다. 최초 집중 45건,
원장 수정 후 집중 2건, Native Skia 라이브러리·placeholder·직접 PDF와 필수 fmt/Clippy/구성
검사도 통과했다. 교육과정 원본 편집 뒤 HWP와 HWPX를 각각 재저장해 415쪽, 편집 문자열의
단일 소속과 취소선의 색 속성을 확인했다. [실행 범위 및 종료 영수증](local-validation.json)을
보존한다.

## 실제 Studio 화면과 보호 기준

실제 문서 열기와 페이지 표시를 거쳐 Native Skia 12쪽, Studio Canvas2D 12쪽,
CanvasKit의 준비 검사를 통과한 분할 표 3쪽을 대조했다. Canvas2D는 각 문서의
원본 글꼴 프로그램을 Host Font Provider로 공급한 뒤 다시 캡처했다. 준비 전 화면에
나왔던 두부 문자를 안정화 뒤 다시 확인했다. 분할 표 12쪽의 후속 문단, 사이버대학
29쪽의 평가 내용, 교육과정 8쪽 목차 본문과 쪽번호가 정상 표시된다. 이 세 화면의
인쇄 대조값은 각각 98.36753%, 99.86233%, 100.00%다.

화학제품 64쪽의 첫 사진과 다음 조각, 사이버대학 28·29쪽의 중첩 표, 교육과정
175·230쪽의 목록과 표 연속부도 직접 대조했다. CanvasKit 요청 중 화학제품은
글꼴·그림 레이어 준비, 사이버대학은 보충 메트릭·글꼴 준비, 교육과정은 전 문서
사전검사 한도에 걸려 Canvas2D로 전환됐다. 요청 9쪽의 전환을 검사 결과에 기록했다.
[실제 출력과 브라우저 초기화 해시](actual-consumer-validation.json)에 확인한
쪽·backend·폰트 프로그램·판독용 PNG를 연결했다.

교육과정 8쪽의 명시 검정 취소선은 Native 97.07%→100.00%, WASM 97.10%→100.00%다.
원본 COLORREF 0의 검정 선이 도트 리더 영역에서 보이는 것을 렌더러별로 확인했다.
[Native review](curriculum-2287/native/review_008.png) ·
[overlay](curriculum-2287/native/overlay_008.png) ·
[WASM review](curriculum-2287/wasm/review_008.png) ·
[overlay](curriculum-2287/wasm/overlay_008.png).

[보호 문서 46조건·4,046쪽](protected-validation.json)은 페이지 단위의 공식 표시값
하락과 누락이 0건이다. [법령 표 WASM 19쪽의 99.73%→99.74% 상승](../visual-protected-baseline/updates/issues7739-7743-20261120.json)을
보호 기준에 반영했다. 편람 HWPX 383쪽 및 기여자 공통 8조건의 기준값은 그대로다.
