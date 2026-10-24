# Stage 41: 일반 행의 선언 최소를 조각 수용과 이어받기에 보존

- 시작 HEAD: `37c7979fa7`(Stage 40).
- 원 보정 `2cfed9e6e4bf142ff321b68f9ca0fa611ca106f5`와 독립 한컴 Print를 대조했다. 리베이스 통합에서 일반 행의 물리 컷 인증이 저장 reset만으로 좁아져, 용량 컷의 앞뒤 내용이 있는 경우 독립 최소를 잃었다. 원본 셀 최소는 69.6266667px인데 기존 수용 합은 44.8533333px였다.
- 공통 원인 보정: 현재 저장 줄 인증 생산자를 이용해 유효한 원본 일반 셀의 독립 최소를 조회한다. 앞뒤 내용이 남는 용량 컷에도 해당 근거를 전달한다. 이미 끝난 원본에 꼬리를 만드는 조건으로 확장하지 않는다. 새 upstream의 accepted-row 원장과 스캔·배치 파이프라인은 유지한다.
- 소비: 공통 물리 컷 → `stored_ordinary_row_end_frame` → 스캔의 수용 상자 → emit의 물리 높이/잔여 → 다음 조각 override → 공통 부분 행 높이·정렬 → 실제 paint. 실제 동일 컷 `[1,1]`은 첫 상자 39.4px와 잔여 30.2266667px를 소유한다.
- 검사: 기존 #1133 두 실패 → 두 PASS. 완료 원본의 새 빈 꼬리 방지와 #8105·#8101·#8060·걸침 밴드·첫 상자 remainder·저장 원점0 정상 대조군을 합쳐 실제 Native 19 PASS/0 FAIL(2.641초). 기대값·검사 수·원본을 바꾸지 않았다.
- 시각: 동일 frozen base `2e087a5b4bc1515ad877e8020b30114639a59392`/후보, 같은 글꼴 713개, canonical PDFium 152.0.7947.0 / smooth RGB / 96DPI / raw2px. 4입력 각3쪽, base/후보 합24쪽 TSV를 새로 산출했다. 후보 최저는 bottom15/footer15/원본 HWP 98.05890%, 원본 HWPX 98.04532%. 후보 미달0, 쪽수 모두3/3, base 대비0.1%p 이상 하락0이다. 두 여백 대조군의3쪽은 base50.30717% → 후보100%, 원본 두 입력의3쪽은100% 유지다.
- 직접 판독: bottom15의2쪽(앞쪽 종료 문단/괘선)과3쪽(이어받는 일정 문단·최종합격·담당자·후속 행·쪽번호)의 review/overlay를 확인했다. PNG와 전쪽 TSV·producer·비교표는 [증적](../pr/assets/issue7445/completed-20261009/stage41-ordinary-row-20261022/)에 보존했다.
- 첫 시각 호출은 PDFium 없는 시스템 Python에서 PDF raster 전에 중단됐다. 해당 폴더에 INVALID를 남겼으며 수용 증거로 쓰지 않는다. 이전 실행과 같은 `/Users/tsjang/rhwp/venv/bin/python`에서 별도 v7b 전체 실행을 완료했다. PNG 패널의 처음 생성 호출은 compare 폴더 누락으로 실패했고 폴더 생성 후 정상 산출했다. 제품 실패로 집계하지 않는다.
- 로그: ignored `rebase-7752-20261022/repair-focused-v7/nextest.log`, `ordinary-row-native-v7b.log`, `ordinary-row-frame-cause/{before,after}.log`.
- fresh WASM, 남은 실패 보정 및 최종 전체 게이트는 미완료다. 이전 head의 fresh 결과로 현재 후보 수용을 주장하지 않는다. 렌더링 기대값이나 새 픽셀 회귀도 추가하지 않았다.
