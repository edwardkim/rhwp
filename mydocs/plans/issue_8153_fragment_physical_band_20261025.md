# #8153: 같은 원본 종료선을 소유한 HWPX 조각의 빈 띠 보존

## 실제 차이와 원본 계약

314쪽 HWPX의 표 세로선은 약 638.7px에서 끝나 별도 가로선과 떨어진다.
독립 한컴 Print의 세로선 끝은 483.197998pt, 약 644.264px다.
같은 독립 출력의 HWP는 현재 100%이고 HWPX에는 모서리 미대응 9픽셀이 남는다.
여러 후속 법령 대조 페이지에서도 같은 모서리 차이가 반복된다.

고정 producer `2b91f8bdc2`에서 두 원본을 실제 실행해 확인했다.
양쪽 모두 6..10행 조각, 종료 컷 `[4, 1]`, block=false,
마지막 예약 높이 110.773333px로 같다. HWPX만 paint 뒤에
`trail=6.29, slack=13.24, fold=6.29`를 적용한다.
HWPX의 원본 표는 section11의 103×2 표로 선언 높이는 41288HU다.
독립 호스트의 저장 줄은 vertpos=0, 높이=1000HU이고 변경된 입력이 아니다.

원본 행의 쪽 재시작은 `stored_text_frame_owner.rs`와
`native_stored_row_has_text_frame_boundary`가 인증한다. 공통 원본 수용 함수는
HWP/HWPX를 모두 검사하지만, 진입 함수가 HWP5만 허용해 HWPX를 거절한다.
배치는 같은 저장 재시작 행의 빈 띠를 다시 줄간격으로 접어 버린다.

추가 trace와 원본 XML에서 끝 행의 첫 셀은 같은 문단의 저장 줄이
0·1522·3044·4566HU 뒤 0HU로 재시작함을 확인했다. 두 번째 셀은 빈 문단이며
이미 소비한 소유자다. 미소비 소유자 둘을 요구하는 별도 본문 공동 종료 계약으로
대신 판정하지 않는다. 그 계약의 `original_body_closed_table_bottom`은 변경하지 않는다.

## 생산·소비와 보존 범위

기존 `0ec1379beb`·`4fc53bca6c`의 본문 공동 종료 인증은 유지한다.
원본 HWP/HWPX가 같은 저장 행·물리 프레임 계약을 만족하면 같은 인증을 사용한다.
재조판·편집, 단순 문단 로컬 원점, 합성 줄, 개체·caption·겹침 등 기존 거절은 유지한다.
배치 좌표를 늘리거나 접기량만 강제로 0으로 만드는 처리를 추가하지 않는다.

`원본 표 높이/행 합 → 저장 행 재시작 인증 → scan의 조각 예약 →
동일 컷의 행 종료 소유 → 실제 셀·세로선`을 확인한다.
캡처·원본 XML·실행 trace는 ignored 작업 루트의 `remaining-raw100-diagnostic/`에 둔다.

## 검증

314쪽의 두 셀과 세로선 끝을 독립 Print ±2px 및 내용·소유 관계로 검사한다.
수정 전 HWPX 실패 / HWP 정상 대조를 확인한다. 같은 종료선의 후속 페이지와
비대상 본문·앞뒤 조각의 내용 순서, 누락·중복·383쪽 유지도 대조한다.
최신 Native/fresh WASM 전쪽 TSV·정확한 미대응 픽셀 및 확대 판독으로 완료를 판정한다.

## 실제 Native 후보 결과

공통 수용 변경 source는 `68dbcdc78f1e5037905c5056b26435db5c08b9c8`이다.
독립 PDF의 314쪽 끝점을 사용한 실제 회귀는 변경 전 HWP 정상·HWPX 실패이며,
변경 후 두 형식 모두 통과했다. 새 검사와 기존 정상 대조를 포함한 집중 회귀는
23건 통과, 실패 0건이다. 전체 Rust 회귀의 완료를 뜻하지 않는다.

같은 producer의 Native 전쪽 766개를 직전 `c5383665af`와 대조했다.
HWPX 314·325·331·338·340·349·357·363쪽은 각각 미대응 9 → 0픽셀이며
raw 100%가 됐다. 나머지 758개에는 미대응 증가와 raw 하락이 없다.
HWPX는 292/383쪽, HWP는 295/383쪽이 미대응 0픽셀이다.
fresh WASM의 같은 source 검증은 진행 중이며 이 Native 결과로 대신하지 않는다.

ignored 검증 루트 `handbook-source-band-candidate/`의 `producer/receipt.json`,
`before-test-run.log`, `selected-regressions-run.log`, `whole-native-exact-audit.json`,
`whole-native-comparison.json`에 실제 source·입력·출력과 비교 결과를 연결한다.

같은 source의 표준 최적화 fresh WASM 전쪽 766개도 종료 코드 0으로 완료했다.
HWPX 8쪽의 미대응 9 → 0픽셀 개선이 동일하며, 나머지 쪽의 미대응 증가와 raw 하락은
없다. Native/fresh WASM의 미대응 0은 각각 HWPX 292·292쪽, HWP 295·294쪽으로
총 1173/1532개다. 전체 최소 raw는 98.93458%이며 목표의 전쪽 100%는 미완료다.
`whole-wasm-exact-audit.json`, `whole-wasm-comparison.json`,
`validation-controller-result.json`과 최적화 WASM 영수증에 source·실제 출력 SHA를 고정했다.
