# 개인정보 원본 글꼴 공급 범위 정정 — C439

Issue: #7445. 원본 조판·기호 계약: #5846, #5714, #7095, #7642.

## 검증한 코드와 입력

- code head: `70708c1b18d380fa3c1a6cf560c6c1e0097c0086` (C438).
- 원본 `samples/task2097/75544_pii_bunseok.hwpx`, SHA-256 `f429768ec02e5a849d08c06678cbee029fdc8618ca9e2bdcdbba485087fe71f1`.
- 독립 한컴2020 Print `pdf/task2097/75544_pii_bunseok-hwpx-2020.pdf`, SHA-256 `0fef38f44b3825f1c97869c2720112536504eaf30cec2a2d2af6d4399ffe8770`, 전66쪽.
- Native frozen171 CLI `eee80c5090e47211760f1030cebcec390c537eb4365aed2e217ac13d2dd1820a`.
- fresh WASM174 `5355e80c7d841cc08eda97bb195da021af3140bea05ffb1694c54c9efa4e9460`, JS `23f085e36c78d952ddac0d91faed9752f33ee1124db78e290382687410917591`.
- runtime2201파일은 ignored `repair171-system-hft-native/source.json`에 고정했다. 이번 기록은 엔진·검사 기대값·비교기·문턱을 바꾸지 않았다.
- WASM174는 Mac locked web/pkg/no-opt 로컬 대체 빌드다. Docker 최적화 빌드로 보고하지 않는다.

## 검증 드라이버 오류와 정정

기존 case producer71의 두 개인정보 조건은 이미 `includeOriginalHftSymbolOverlay:false`였다.
하지만 드라이버137은 이 필드를 소비하지 않고 네 글꼴 경로를 모두 전달했다.
그 결과 독립 HFT 알파벳 검토용 부분 프로그램이 입력의 일반 TrueType family로 선택됐다.
원본 IR183에서 한양신명조의 모든 선언은 altType1(TrueType)이며, 이 부분 HFT 프로그램의 공급 자격이 없다.

제외한 프로그램의 원본 파일 SHA-256은 `687110a537402be23b33235e3cc95b693afd29ad9a0614c502f1eab51bc97428`,
family `한양신명조`, PostScript `HanyangSinMyeongJo-OriginalHFT-SymbolsASCII`다.
전체 cmap99개·공백 제외 ASCII94개·현대 한글0개인 검토 전용 자원이며 완전한 TrueType primary가 아니다.
파일과 표본 PDF는 로컬에만 유지하며 이 증적에 포함하지 않았다.

186은 기존 false 계약에 따라 `independent-ascii-supply-c370/font` 경로만 제외하고 나머지 세 공급 경로를 유지한다.
원본·독립 PDF·프로필·DPI·PDFium·2px90%·설치 글꼴은 그대로다. 제품 기본 글꼴을 변경한 것이 아니다.
별도 원본7336에는 HFT 선언이 있으므로 같은 경로를 일괄 제거하지 않고 기존 네 경로를 유지했다.

137 기본 환경의4/20/21쪽 미달 및22쪽 devel 하락은 잘못된 공급 조건의 진단 기록이다.
현재 올바른 조건의 미달·회귀 근거로 재인용하지 않는다. 과거 PNG/TSV/로그는 삭제하거나 덮어쓰지 않았다.

## 같은 코드에서 공급 조건만 바꾼 집중 대조

| 쪽 | 잘못된 공급 Native | 정정 Native | 잘못된 공급 WASM | 정정 WASM |
| --- | --- | --- | --- | --- |
| 4 | 81.70425% | 100.00000% | 81.96642% | 100.00000% |
| 20 | 83.34811% | 99.98915% | 83.57576% | 99.98926% |
| 21 | 89.77718% | 100.00000% | 89.77198% | 100.00000% |
| 22 | 91.36857% | 99.77965% | 91.36039% | 99.77833% |
| 36 | 100.00000% | 100.00000% | 100.00000% | 100.00000% |

이는184의 선택5쪽 A/B이며 전쪽 수용을 대신하지 않는다.100%는2px 허용 실루엣이며 PNG byte 동일을 뜻하지 않는다.
잘못된 Native4쪽의 글자 원점·자간 이탈과 정정4쪽, 정정WASM22쪽을 review/overlay로 직접 판독했다.
정정4/22쪽 PNG는 이후186의 실제 전쪽 출력과 byte동일하다.

## 현재 코드의 전쪽 결과

Print·192DPI·compat2022·Native Skia/fresh WASM Canvas2D·canonical PDFium raw2px 기준이다.
별도 pdftoppm audit TSV는 canonical 수치와 섞지 않는다. 캡처8·8쪽 묶음·source guard를 유지했다.

| 환경 | backend | 비교/전체 쪽수 | 최저 | 최저 쪽 | 90% 미달 |
| --- | --- | --- | --- | --- | --- |
| 선언 | Native | 66/66 | 94.58132% | 32 | 0 |
| 기본 | Native | 66/66 | 94.58132% | 32 | 0 |
| 선언 | fresh WASM | 66/66 | 94.60117% | 32 | 0 |
| 기본 | fresh WASM | 66/66 | 94.60117% | 32 | 0 |

드라이버186 exit0과4조건264쪽 완료를 확인했다. 모든 TSV는1~66의 빠짐·중복 없는 원래 쪽 번호를 가진다.
네 조건의 전66쪽에서 문단·표·셀 소유와 공백을 제외한 정확한 Unicode 텍스트 순서를 대조해 차이0쪽이다.
이는 backend/환경 간 일치이며 독립 PDF의 텍스트 oracle을 대신하지 않는다.
같은 backend의 선언/기본 PNG는 각각 전66쪽 byte동일하다.

양쪽32쪽 review·overlay를 직접 읽었다. 첫 문단과 일부 굵은 제목의 미세한 기준선·자형 차이,
표 괘선/얇은 획의 래스터 프린지가 남는다.94% 통과를 무차이·전체 시각 완료로 보고하지 않는다.
고정4a07의 같은 세 경로·같은 환경 전쪽 대조188은 이 기록 시점에 실행 중이며, devel 대비 무회귀 판정은 아직 하지 않았다.
현재 전체 문서·전체 nextest·최종 tiers·actual36도 완료되지 않았다.

## 기존 검사가 실제로 검출한 별도 회귀

기대값을 바꾸지 않은 #5846 정식 suite026은 현재1FAIL/231선택 제외(0.690초, exit100)이다.
문단/셀 소유·2/3쪽 독립 px·59/60쪽 내용·기호 검사를 지난 뒤, 마지막 원본 종료 프레임 관계에서57쪽만 실패했다.

독립 source의 cellSz83771HU·common42265HU·바깥여백283+283HU·후속 원점42072HU를 그대로 대조했다.
보정 전 frozen130은 앞 조각42265HU, 뒤41506HU로 후속 원점과 정확히 일치하고 기존 검사1PASS다.
현재171은 앞42266HU, 뒤41505HU로1HU(0.013333CSSpx)를 잘못 배분한다.58쪽의 다른 종료 프레임은 양쪽 동일하다.
C437의 일반 행 예약이 기존1×1 저장 첫 프레임보다 먼저 override를 선택하는 실제 호출을 추적 중이다.
검사·공차를 완화하지 않으며 다음 구현 단계에서 저장 프레임의 예약과 paint를 함께 보정한다.

기존 쪽번호7검사도 현재 그대로 실행해2PASS/5FAIL이다. 원본 Win 프레임과 Print matrix 기대값 검토 및
각주1937의51/독립50쪽 불일치는 아직 남는다. 이 결과를 전체 회귀 통과로 세지 않는다.

## 대표 그림

| 조건 | review | overlay |
| --- | --- | --- |
| 같은 코드, 잘못된 공급 Native4쪽 | [review](wrong-scope-native-p004-review.png) | [overlay](wrong-scope-native-p004-overlay.png) |
| 같은 코드, 정정 공급 Native4쪽 | [review](correct-scope-native-p004-review.png) | [overlay](correct-scope-native-p004-overlay.png) |
| 정정 공급 fresh WASM22쪽 | [review](correct-scope-wasm-p022-review.png) | [overlay](correct-scope-wasm-p022-overlay.png) |
| 정정 공급 Native 최저32쪽 | [review](correct-scope-native-p032-review.png) | [overlay](correct-scope-native-p032-overlay.png) |
| 정정 공급 fresh WASM 최저32쪽 | [review](correct-scope-wasm-p032-review.png) | [overlay](correct-scope-wasm-p032-overlay.png) |

ignored 증거:183 원본/글꼴 감사,184 공급 A/B,186 전쪽·producer·글꼴 실제 선택·소유 감사,
187 기존 쪽번호 검사,188 진행 중 base 대조,189 정식 실패,190 source 전후 진단.
첫186 실행은 PDFium 없는 Python을 선택해 비교 전에 종료했다. INVALID로 보존하고 확인된 PDFium Python에서 재실행했다.
첫189 standalone은 roxmltree 외부 의존성을 빠뜨려 컴파일만 실패했다. 정식 suite 경로로 재실행한 실제1FAIL과 구분한다.
로그·JSON·임시 driver는 ignored output에만 두며 원본/PDF/PNG/공유 target을 보존한다.
