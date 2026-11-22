# PR #9322 병합 이후의 리베이스와 기능 보존 검증

새 `upstream/devel`은 `93e861523623bcd84d57b808edb62af5af2aa82a`다. 이전 기준
`622af208ee12c4cb16357d465e4802e7fd4726bc` 위의 103개 커밋을 순서대로 리베이스했다.
리베이스 직후 후보는 `d322327c0f6d71c4343a5a2835dfaa6e4459a24c`이며, 새 기준과 후보의
병합 모의 트리는 후보와 같은 `47b9e2b83cc0c35ffe04d2df8d461e264905adda`다. 기존 렌더·테스트
프로세스가 사용하는 작업 디렉터리는 유지하고 별도 sparse worktree에서 충돌을 해결했다.

#9322는 #9280의 HWP 버전별 본문 기호, 단독 실로티 나그리 결합 부호, 누락 글리프의
표시와 전진폭을 저장·측정·Native·WASM·Studio 경로에 반영했다. `native_pua_fonts.rs`에서
같은 등록 지점에 추가된 두 분석을 합쳤다. 원래 face에서 가시 glyph 0의 전진폭과 가시
옛한글 호환 glyph의 전진폭을 각각 수집하고, compact face로 바뀐 뒤에는 각 소비자가
자신의 측정값을 쓴다. 버전별 기호의 표시·저장 처리와 기존 87개 기호의 공급 순서도 보존했다.

[원 PR 검증 자료](../pr/assets/pr9280/README.md)의 Native·WASM 37쪽씩 원본 TSV 74개와
37종의 입력·독립 한컴 Print PDF 해시를 새 base의 Git 객체에 대조했다. 원 PR 생산 소스
`9595602fe1024fc314bb0c9918a8f7343a91bb75`의 Rust·Studio·스크립트 입력은 새 base와
같았다. 이 기록은 #9322 검증 입력의 출처다. 새 후보의 실제 출력, HWP 버전 경계에서의
HWPX 저장·재열기, 결합 부호의 내용·편집 위치와 실제 화면·인쇄 backend는 새 base와 비교한다.

37종의 입력·PDF를 쪽수와 해시로 고정했다. 검증에 사용하는 글꼴 다섯 파일은 원 PR의
해시와 일치한다. 굴림은 현재 TTC의 첫 face를 독립 한컴 PDF 16개가 실제 사용한 GID 0과
대조해 4,539개 표시에 쓰인 윤곽과 전진폭을 확인했다.
[검증 입력 목록](../verification/issue7754-nine-fidelity/incoming-pr9322/inputs.tsv)과
[글꼴 프로그램 대조](../verification/issue7754-nine-fidelity/incoming-pr9322/font-supply.json)를 사용한다.

Native에서 이 입력과 독립 한컴 Print 37쪽을 같은 글꼴 공급으로 비교했다. 기준과 후보의
PNG 37쌍이 일치했고 최저 99.17%였다. 함초롬돋움에서 빠진 기호는 한컴 PDF의 실제 사용
기록대로 굴림을 지정해 재현했다. 결합 부호, 함초롬돋움의 누락 글리프, 현대 도장란의 연속
누락 글리프를 원본 Print와 직접 대조했다.
[Native 쪽별 TSV와 비교 PNG](../verification/issue7754-nine-fidelity/incoming-pr9322/print/native-validation.json)

최적화한 fresh WASM에서도 같은 37쪽을 대조했다. 기준과 후보의 PNG 37쌍이 일치했고
독립 Print 대비 최저는 98.79%였다. Native와 같은 세 사례를 직접 판독했다.
[WASM 쪽별 TSV와 비교 PNG](../verification/issue7754-nine-fidelity/incoming-pr9322/print/wasm-validation.json)

현대 HWPX 재저장 대조군에서 리베이스 전 두 생산본은 같은 여섯 문단의 BMP 문자를
Plane 15 기호로 바꿨고 굵은 구간의 UTF-16 시작 위치도 1만큼 밀렸다. 한컴 HWPX와 같은
원문과 굵은 구간을 새 생산본에서 확인하는 데 사용한다.
[재저장 대조군의 실제 차이](../verification/issue7754-nine-fidelity/incoming-pr9322/modern-hwpx-save-before-pr9322.json)

새 `devel` 생산본에서 원본·편집본 11종을 Native와 fresh WASM으로 각각 44개 저장 산출물로
재현했다. 각 경로의 HWPX 33개는 독립 한컴 저장본과 문단 텍스트·굵은 구간이 일치했다.
후보 Native와 fresh WASM도 11종의 저장·재열기에서 같은 한컴 문단 텍스트와 굵은 구간을
보존했다. WASM의 원본·재열기 표시 60개와 결합 부호 편집·재열기 13개 상태가 기준과 일치했다.
[기준과 후보의 산출물 해시 및 비교 결과](../verification/issue7754-nine-fidelity/incoming-pr9322/roundtrip-verification.json)

기준과 후보의 Studio에서 원본·저장본 10종을 기본 글꼴과 Windows 검증 글꼴로 열고,
Canvas2D·CanvasKit의 화면·인쇄 80조건을 각각 실행했다. 두 생산본 모두 CanvasKit 40조건을
요청한 렌더러에서 완료했다. 같은 조건의 PNG 80쌍은 바이트까지 일치했다. 두 렌더러의 대표
인쇄 캡처에서 단독 결합 부호의 점선 원과 정상 결합을 확인했다.
[실제 렌더러와 PNG 해시](../verification/issue7754-nine-fidelity/incoming-pr9322/studio/studio-verification.json)

새 기준과 후보의 Native·fresh WASM에서 중첩 표 HWP와 한컴 HWPX에 각각 30줄·70줄을
편집했다. 편집 직후와 HWP·HWPX 저장 후 재열기의 12상태 모두 줄 순서·개수·쪽 소속이
독립 Print와 일치했다. 각 경로에서 기준과 후보의 쪽별 점수와 PNG 42쌍이 모두 일치했고
실루엣 일치율의 최저값은 98.07%였다.
[Native 쪽별 TSV와 산출물 해시](../verification/issue7754-nine-fidelity/after-pr9322/nested/native/validation.json) ·
[WASM 쪽별 TSV와 산출물 해시](../verification/issue7754-nine-fidelity/after-pr9322/nested/wasm/validation.json)

좌표 회귀가 지정된 다섯 쪽은 현재 후보에서도 두 경로 모두 98% 이상이었다. #7754의
안내서 2쪽은 Native·fresh WASM에서 각각 99.99%였고 #7765의 네 쪽은 각각 100.00%였다.
전쪽 실행에서 얻은 다섯 쪽의 PNG는 리베이스 직전 확대 판독에 사용한 PNG와 모두 일치했다.
#7765 HWP 10쪽은 새 WASM 확대 이미지에서 일곱 줄을 다시 판독했다. 독립 Print와
각 줄의 첫 잉크 행 차이는 최대 1px였다.
[선정 다섯 쪽의 TSV와 확대 판독](../verification/issue7754-nine-fidelity/after-pr9322/coordinate-pages/validation.json)

규제분석 HWP는 64쪽 전체에서 Native 최저 98.08%, fresh WASM 최저 98.11%였다.
좌표 검사 대상 10·28·29쪽은 두 경로에서 모두 100.00%였다. 10쪽의 fresh WASM PNG는
선정 쪽 실행과 전쪽 실행에서 서로 달랐다. 전쪽 실행의 본문 일곱 줄 첫 잉크 행은
독립 Print와 각각 일치했다. 전쪽 fresh WASM PNG 64쪽은 리베이스 직전 후보 및 현재
기준과 모두 일치했다.

규제분석 HWPX 64쪽의 최저는 Native 98.69%, fresh WASM 98.69%였다.
사유 표 첫 행의 높이를 확인한 26쪽은 Native·fresh WASM 모두 100.00%였고,
전쪽 실행의 PNG와 확대 판독 이미지가 일치했다. fresh WASM은 현재 기준보다 21쪽이 개선됐고,
64쪽의 PNG가 리베이스 직전 후보와 모두 일치했다.

새 기준 Native의 원래 아홉 이슈 583쪽과 보조 사례 50쪽을 직전 기준과 대조했다.
43조건에서 같은 글꼴 공급과 PDF 래스터 조건을 사용했으며, 633쪽의 PNG와 TSV가
바이트까지 일치했다.
[기준 Native 633쪽의 원본 TSV와 산출물 확인](../verification/issue7754-nine-fidelity/after-pr9322/native-base-before-and-after-pr9322.json)

시장구조조사 실행 전에 완료한 기준 fresh WASM 42조건·318쪽을 #9318과 #9322 병합
전 같은 조건의 출력과 대조했다. PNG 312쪽이 일치했다. 규제분석 13·37·54쪽은
HWP에서 각각 0.02%p 낮아졌고 HWPX에서 각각 0.04%p 높아졌다. HWPX의 기준 최저
19.52%는 두 기준에서 같았다.
[기준 fresh WASM 318쪽의 리베이스 전후 대조](../verification/issue7754-nine-fidelity/after-pr9322/wasm-base-before-market.json)

현재 후보 Native도 43조건의 전체 633쪽을 같은 공급 조건으로 대조했다. 이슈 대상
583쪽의 최저는 98.08%였고 389쪽이 100.00%였다. 보조 사례 50쪽의 최저는 98.07%였고
42쪽이 100.00%였다. 현재 기준보다 73쪽이 개선됐으며 하락은 없었다. 리베이스 직전
후보와 633쪽의 PNG가 일치했고, 원본 TSV 86개의 바이트도 기존 정식 증적과 일치했다.
좌표 회귀 5쪽과 대표 구간 5쪽의 Native와 독립 Print PNG는 확대 판독에 쓴 PNG와 모두 일치했다.
[현재 Native의 전쪽 결과와 TSV](../verification/issue7754-nine-fidelity/after-pr9322/native-target-and-feature-comparison.json)

현재 후보 fresh WASM은 이슈 대상 583쪽에서 최저 98.11%, 100.00% 378쪽이었다. 보조 사례
50쪽은 최저 98.07%, 100.00% 42쪽이었다. 현재 기준보다 91쪽이 개선됐고 하락은 없었다.
633쪽 모두 리베이스 직전 후보와 PNG가 일치했다. 좌표 검사 5쪽과 대표 구간 5쪽의
전쪽 실행 PNG도 확대 판독에 쓰인 PNG와 일치했다. 시장구조조사 후보 315쪽은
최저 98.40%, 100.00% 212쪽이었다.
[현재 fresh WASM의 전쪽 원본 TSV와 10쪽 판독](../verification/issue7754-nine-fidelity/after-pr9322/wasm-target-and-feature-comparison.json)

복학원서·배포용 의회운영 문서·공개 HWP의 fresh WASM 전쪽 최저는 각각 98.99%, 99.80%,
99.94%였다. 대표 쪽의 Native·fresh WASM·독립 Print PNG가 리베이스 직전 확대 판독
이미지와 일치했다. 복학원서 1쪽의 fresh WASM 일치율은 현재 기준의 96.59%에서 98.99%로 올라갔다.
[세 문서의 현재 대표 구간 대조](../verification/issue7754-nine-fidelity/representative-print-review/review.json)

온새미로 HWP와 HWPX는 각각 46쪽이며 fresh WASM 전쪽 최저는 두 형식 모두 99.98%였다.
HWP 44쪽과 HWPX 44쪽이 100.00%였다. 각 형식의 46쪽은 리베이스 직전
후보의 PNG와 일치했다. 도형 옆 줄 경계를 판독한 두 형식의 5쪽은 Native와 fresh WASM에서
100.00%였으며, 현재 기준의 fresh WASM은 94.71%였다.
[온새미로 HWP·HWPX 5쪽 판독과 전쪽 집계](../verification/issue7754-nine-fidelity/representative-print-review/review.json)

수출입 현황 19쪽의 전쪽 최저는 Native 99.81%, fresh WASM 99.86%였다. fresh WASM의
기준과 후보 PNG 19쪽이 모두 일치했다. 10쪽의 흑자국·적자국 표와 하단 무역 추이 그래프를
한컴 Print에 직접 대조했다.
[수출입 현황의 현재 전쪽 결과와 10쪽 증빙](../verification/issue7754-nine-fidelity/after-pr9322/monthly-trade-print.json)

시장구조조사 315쪽의 기준 Native 출력은 리베이스 전후로 동일했다. 각 쪽의 출력·독립
Print PNG와 TSV를 실제 바이트로 대조했다. 기준에서 98% 미만인 3쪽(95.99%)과
100쪽(26.80%)도 리베이스 전후에 같은 값이었다.
[기준 Native의 전쪽 대조](../verification/issue7754-nine-fidelity/after-pr9322/issue7755-base-native.json)

시장구조조사의 fresh WASM 기준은 리베이스 전 증빙이 남은 1~288쪽을 새 기준과 대조했다.
1~245쪽의 PNG와 점수는 같았고 246~288쪽은 43쪽 모두 개선됐다. 리베이스 전 기준은
부록 B-2의 시작을 246쪽에 배치했다. 새 기준은 독립 Print와 같이 247쪽에서 제목과 표가
시작했다. 글꼴 공급과 독립 Print PNG는 대조 범위에서 일치했다. 현재 기준은 315쪽의
TSV·PNG를 모두 검증했다. 98% 미만은 3쪽(95.95%)과 100쪽(27.35%)이며,
리베이스 전 증빙 범위를 벗어난 289~315쪽의 최저는 99.98%였다. 현재 후보에서도
부록 경계인 246·247쪽의 PNG가 새 기준과 같았고, 각각 99.97%와 100.00%였다.
[현재 기준 fresh WASM 전쪽 TSV와 후보의 부록 경계](../verification/issue7754-nine-fidelity/after-pr9322/issue7755-wasm-base-before-and-after-pr9322.json)

시장구조조사 첫 32쪽의 fresh WASM 후보 최저는 98.40%였고, 32쪽의 PNG가 리베이스 직전
후보와 일치했다. 3쪽은 현재 기준보다 Native에서 95.99% → 99.40%, fresh WASM에서
95.95% → 99.34%로 개선됐다. 연구진 제목과 테두리 및 이름 세 줄을 한컴 Print와 대조했다.
[3쪽 TSV 수치와 확대 비교](../verification/issue7754-nine-fidelity/after-pr9322/issue7755-page-003.json)

시장구조조사 100쪽의 현재 후보 Native는 독립 한컴 Print 대비 100.00%였다. 기준의
제3절 소결 제목은 PDF보다 61px 위에서 시작했고, 후보의 시작 행은 PDF와 1px 차이였다.
후보의 제목과 본문 위치 및 줄 순서를 확대 대조했고 PNG가 리베이스 직전 후보와
바이트까지 일치하는 것을 확인했다.
[100쪽의 현재 Native 판독 근거](../verification/issue7754-nine-fidelity/after-pr9322/issue7755-native-page-100.json)

같은 100쪽에서 fresh WASM은 현재 기준의 27.35%에서 후보 100.00%로 개선됐다. 후보의
제3절 소결 제목은 독립 한컴 Print와 같은 279행에서 시작했고, 전쪽 실행에서 얻은 PNG는
리베이스 직전 후보의 PNG와 일치했다.
[100쪽의 현재 WASM 판독 근거](../verification/issue7754-nine-fidelity/after-pr9322/issue7755-wasm-page-100.json)

시장구조조사의 기준 Studio는 95·100·106·247·315쪽을 화면과 인쇄 프로필로 실행했다.
요청한 렌더러 세 조건을 포함한 30개 PNG가 리베이스 전후로 모두 일치했다. 전체 315쪽이
CanvasKit의 사전 검사 한도인 128쪽을 넘는 네 조건에서 실제 Canvas2D 출력을 확인했다.
[시장구조조사 기준 Studio 화면·인쇄 증적](../verification/issue7754-nine-fidelity/after-pr9322/issue7755-base-studio.json)

시장구조조사의 후보 Studio 화면 15조건은 리베이스 직전과 PNG가 일치했다. 100쪽의 현재
기준과 후보 Canvas2D PNG를 이전 확대 판독 이미지와 각각 대조했다. 후보의 제목 시작 행은
한컴 Print와 같은 279px였다.
[100쪽 Studio 화면 확대 판독](../verification/issue7754-nine-fidelity/studio/market-page100-screen/review.json)

100쪽 Studio의 Canvas2D 인쇄도 재실행했다. 기준과 후보의 PNG가 각각 리베이스 직전과
일치했고, 후보의 제목 시작 행은 화면과 한컴 Print에서와 같은 279px였다.
[100쪽 Studio 인쇄 확대 판독](../verification/issue7754-nine-fidelity/studio/market-page100-print/review.json)

현재 기준과 후보의 Studio 화면·인쇄를 각각 216조건에서 검증했다. 원래 192조건은 양쪽
모두 리베이스 직전의 각 PNG와 일치했고, #9318의 24조건은 현재 기준과 후보가 일치했다.
시장구조조사 30조건의 차이는 100쪽의 6조건에서 확인했다. 시장구조조사의 CanvasKit 요청은
315쪽이 사전 검사 한도 128쪽을 넘으므로 Canvas2D 출력을 대조했다.
[Studio 전체 화면·인쇄 검증](../verification/issue7754-nine-fidelity/after-pr9322/studio-screen-and-print.json)

#7754의 암호 HWPX는 같은 독립 한컴 Print PDF로 대조했다. Native와 fresh WASM 모두
암호 원본과 평문 대조군의 24쪽 PNG가 각각 바이트까지 일치했다. 기준의 전쪽 최저값은
Native 98.17%와 WASM 98.07%였고, 후보는 두 경로 모두 99.48%였다.
[암호 HWPX 쪽별 TSV와 PNG 해시](../verification/issue7754-nine-fidelity/after-pr9322/issue7754-encrypted-parity/validation.json)

별도 암호 검증을 전쪽 평문 실행과도 대조했다. 두 실행 사이에서 24쪽의 실루엣 값은
모두 같았다. 10쪽의 파란 손가락 기호 네 개에서만 획 윤곽이 달랐고, 그 픽셀 변화가 각 경로의
현재 기준과 후보에서 정확히 일치했다. fresh WASM 글꼴 등록 기록과 확대 비교를 증빙에 담았다.

#9318의 OLE 입력은 새 기준과 후보의 Native·fresh WASM에서 HWP와 HWPX로 각각 두 번
편집·저장하고 다시 열었다. 변환본과 편집본 20개의 독립 CFB·XML 검사에서 두 OLE 개체의
속성과 payload가 일치했으며 편집한 텍스트도 보존됐다.
[반복 저장의 실제 산출물](../verification/issue7754-nine-fidelity/incoming-pr9318/ole-repeat-save-after-pr9322.json)

#9318에서 반영한 원본 6종의 후보 Native·fresh WASM 출력을 재검증했다. 각 경로의 전체
167쪽에서 리베이스 직전 후보와 현재 후보의 PNG·쪽별 점수·공급 조건이 모두 일치했다.
독립 Print 대비 최저값은 Native 98.14%, WASM 98.15%였다. 수용 당시 기록과 점수가
달랐던 쪽도 두 후보에서 같은 값을 확인했다.
[후보 Native TSV와 PNG 해시](../verification/issue7754-nine-fidelity/incoming-pr9318/candidate-native-after-pr9322.json) ·
[후보 WASM TSV와 PNG 해시](../verification/issue7754-nine-fidelity/incoming-pr9318/candidate-wasm-after-pr9322.json)

#9318 원본 6종의 현재 기준과 후보 fresh WASM를 전쪽 대조했다. 167쪽의 PNG가 모두 일치했고 최저는 98.15%였다. 기준 대비 하락과 누락은 없었다. 리베이스 직전 후보의
PNG와 점수도 전쪽 일치했다. 최초 수용 기록보다 낮은 10쪽의 수치는 리베이스 직전 후보와
같았고, 가장 차이가 큰 온라인 검색 제안요청서 2쪽은 확대 판독했다.
[현재 기준과 후보 fresh WASM의 전쪽 검증](../verification/issue7754-nine-fidelity/incoming-pr9318/after-pr9322/wasm/coverage.json)

온라인 검색 제안요청서 2쪽의 fresh WASM은 #9240 수용 증빙 99.94%에서 99.72%로 달라졌다.
두 증빙의 독립 한컴 Print는 RGB 픽셀이 같았으며, 비교 대상을 교차해도 같은 점수를 얻었다.
차이는 목차 첫 다섯 항목의 점선 양 끝에 집중됐다. 현재 devel과 후보의 PNG는 일치했다.
[2쪽의 교차 측정과 확대 판독](../verification/issue7754-nine-fidelity/incoming-pr9318/online-search-rfp-page-002-wasm-comparison.json)

#9318 원본 6종의 현재 기준과 후보 Native를 전쪽 대조했다. 167쪽의 PNG가 모두 일치했고 최저는 98.14%였다. 기준 대비 하락과 누락은 없었다.
[현재 기준과 후보 Native의 전쪽 검증](../verification/issue7754-nine-fidelity/incoming-pr9318/after-pr9322/native/coverage.json)

#9318의 Studio 편집·입력 동작 7종을 새 기준과 후보에서 실행했다. 실제 브라우저가 받은
WASM 해시를 대조했다. 후보의 호스트 글꼴 입력 검사 첫 실행에서 준비 호출 21회가 기록됐다.
동일한 검사와 WASM을 양쪽에서 다시 실행하자 20회로 통과했고, 편집 후 내용·쪽 소속과 조판·
그리기 해시가 일치했다. [Studio 기능별 검증 결과](../verification/issue7754-nine-fidelity/incoming-pr9318/studio-feature-journeys-after-pr9322.json)

#9318의 Studio 원본 4종에서 화면·인쇄와 렌더러 요청 3종을 실행했다. 24조건 모두
리베이스 전 검증과 같은 실제 렌더러를 사용했으며, 현재 기준과 후보의 PNG 24쌍이
바이트까지 일치했다.
[리베이스 후 Studio 화면·인쇄 PNG 해시](../verification/issue7754-nine-fidelity/incoming-pr9318/studio-screen-and-print-after-pr9322.json)

후보 생산본과 공개 글꼴 497개를 사용해 공통 보호 문서 4종을 Native·fresh WASM으로
실행했다. 8조건 모두 공식 표시값 100.00%였다.
[공개 글꼴의 쪽별 TSV와 비교 이미지](../verification/issue7754-nine-fidelity/after-pr9322/public-font-gate/validation.json)

후보 Studio의 TypeScript와 렌더러 계약 검사가 통과했다. 단위 검사 2,834개 중 웹 WASM 환경에서
실행된 2,832개와 같은 소스의 Node WASM 바인딩으로 재실행한 2개가 통과했다. 실제 WASM의
글꼴 추적 계약 검사 2개도 통과했다.
[Studio 로컬 검증](../verification/issue7754-nine-fidelity/after-pr9322/studio-local-validation.json)

최소 글꼴 환경에서 후보의 Native Skia 라이브러리 전체를 실행했다. 공개 글꼴 다섯 개를
확인했고, 실행 파일 네 개에서 4,033개가 통과했다. 실패는 없었고 기존 ignore 표기가 붙은
12개는 실행에서 제외됐다. CI가 지정한 Native Skia 선택 검사 21개를 같은 환경에서 모두 실행해
194개가 통과했다. CLI PNG, 직접 PDF, 글꼴 선택 추적과 네이티브 그림 검사를 포함한다.
[Native Skia 라이브러리와 선택 검사 실행 근거](../verification/issue7754-nine-fidelity/after-pr9322/rust-native-lib.json)

추가 Native 집중 검사 8종에서 48개가 통과했다. #5140 본문 기호 검사에서는 격리 환경이
이전 체크아웃의 자료를 사용해 컴파일 입력 3개를 찾지 못했다. 현재 후보의 자료를 연결해
추가·변경 파일 323개의 해시와 공개 글꼴 5개를 확인했다. 해당 검사를 다시 실행해
11개가 통과했고 나머지 집중 검사도 완료했다.

저장소 루트, Native Skia 라이브러리, 워크스페이스 전체 타깃 및 WASM 라이브러리의 Clippy가
모두 통과했다. 워크스페이스 Clippy는 디스크 공간 부족으로 한 차례 종료됐다. 검증을 마친
36개 단계의 로그를 확인했다. 기존 검토 타깃 경로를 유지하면서 격리된 임시 파일시스템에
새 빌드 출력을 기록해 워크스페이스 Clippy부터 다시 실행했다. 같은 환경의
워크스페이스 빌드도 통과했다.
최소 글꼴 환경에서 release-test 전체 Nextest는 55개 바이너리의 13,995개 검사를 모두 통과했다. 러너가 제외한 검사는 45개였다.
[Clippy·빌드 명령과 저장공간 복구 기록](../verification/issue7754-nine-fidelity/after-pr9322/rust-local-validation.json)

새 기준과 후보의 Native 정본 보호 문서 23종·2,023쪽을 완료했다. 쪽별 TSV와 실제 PNG를
대조해 하락과 누락이 0쪽이고, 기준과 후보의 PNG 2,023쌍이 모두 일치하는 것을 확인했다.
행정업무운영 편람 HWPX 383쪽을 포함한다.
[Native 정본 보호 전쪽 비교](../verification/issue7754-nine-fidelity/after-pr9322/protected/native/native-validation.json)

fresh WASM의 새 기준도 정본 보호 23종·2,023쪽을 완료했다. 같은 공급 조건에서 리베이스 직전
기준과 전체 PNG 및 쪽별 값이 일치했다. 보호 하한 미달은 없었으며 sample10에 공급한 원본 연결 그림
세 파일의 해시도 대조했다.
[fresh WASM 기준 정본 보호 전쪽 비교](../verification/issue7754-nine-fidelity/after-pr9322/protected/wasm-base-before-and-after-pr9322.json)

새 기준과 후보의 fresh WASM 정본 보호 23종·2,023쪽에서 PNG가 모두 일치했다. 보호 하한
미달과 쪽수 누락은 없었다. 자동 공급 문서의 실제 글꼴 선택 기록과 sample10의 원본 연결
그림을 대조했다. Native를 포함한 정본 보호 46조건·4,046쪽을 모두 확인했다.
[fresh WASM 정본 보호 전쪽 비교](../verification/issue7754-nine-fidelity/after-pr9322/protected/wasm/wasm-validation.json)

리베이스 직전 기준의 보호 정본 46조건·4,046쪽은 `devel`과 후보의 PNG 해시가 일치했고,
하락과 누락은 0쪽이었다. [리베이스 전 전쪽 보호 결과](../verification/issue7754-nine-fidelity/protected/validation-before-pr9322.json)

리베이스 전 생산본 `b75ce19774a4b059b26c2d134d49d5f7ce4a93b2`의 최소 글꼴 전체 Rust 검사는
13,984개 통과, 실패 0개, 설정에 따라 제외한 45개로 끝났다.
[검사 명령과 종료 기록](../verification/issue7754-nine-fidelity/local-checks.json)

같은 생산본의 fresh WASM은 원래 아홉 이슈 583쪽에서 최저 98.11%, 보조 검증을 포함한
633쪽에서 최저 98.07%였다. [원본별 전쪽 TSV와 측정 기록](../verification/issue7754-nine-fidelity/wasm/candidate-before-pr9322.json)

`sample10` 1쪽은 기존 연결 그림 공급 절차로 확인했다. 원본 그림 3개의 SHA-256이 기존 증거와
일치했다. 기준과 후보의 fresh WASM에서 그림을 실제로 공급해 1쪽을 각각 99.96%로 재현했고
PNG가 바이트까지 일치했다. 표지의 로고·구성도·인물 삽화를 독립 Print와 직접 대조했다.
[1쪽 연결 그림, TSV와 비교 PNG](../verification/issue7754-nine-fidelity/after-pr9322/protected/sample10-linked-images/validation.json)

Stage 20의 보호 검증과 원래 아홉 이슈의 증빙은 이전 기준과 생산 SHA에 연결한다. 새 base와
후보에서 동일한 Harness와 글꼴·연결 그림 공급으로 원래 대상, 보호 정본, #9318·#9322 기능을
다시 대조한다. 시각·최소 글꼴 Rust·Studio 필수 검증을 완료한 뒤 PR을 제출한다.
