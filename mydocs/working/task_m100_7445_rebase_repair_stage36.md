# #7445 보정36 — #7752 리베이스와 공통 수용 계약 통합

Issue: #7445. 기능 보존 대상: #7752 / 통합 PR #8148, #8132, #8135.

## 리베이스와 충돌 처리

- 이전 base `80968497c2c178eeecb3b2e1d04d086ad27a0dc7`, 보존 head `5e23738ad2074b8046176bd80719b6fb51203af2`다.
- 새 base `2e087a5b4bc1515ad877e8020b30114639a59392`에 로컬 333개 커밋을 리베이스했다. 리베이스 직후 head는 `970c47bf1b8f05ba468198db7bc060fca474905b`다.
- 사용자의 #7752 우선 지시에 따라 새 `accepted_start/end/partial_row_box → fragment_row_heights → 부분 표 paint` 계약을 생산·스캔·예약·배치에서 함께 유지했다. 구형 `physical_rows/rowspan_physical_space/row_height_overrides` 원장을 병행하지 않는다.
- 파일 전체를 고른 초기 충돌 처리에서 빠진 독립 helper와 생산자를 전체 source 3-way 대조로 복구하고, 새 표 계약과 양립하는 그림·글꼴·각주 호출을 연결했다. 처음 Native compile의 124개 API 오류를 보정했다. 단순 충돌 해소를 기능 보존 완료로 기록하지 않는다.
- 실제 소비 사슬은 새 표 컷 query → scan의 수용 상자 → fragment budget/emit → `PageItem.fragment_row_heights` 및 `layout_partial_table`이다. 끝 각주 소유, 표 괘선의 edge 소유, 문단의 확정 쪽 원점은 각각 해당 공통 결과를 이어받는다.

## 글꼴 프로그램 변경 뒤의 공백

독립적인 표시용 fallback과 명시적 TrueType 선택을 구분했다. 원본 HFT 폭을 보존한 뒤 호출자가 TTF를 선택했을 때 문자 폭만 바뀌고 `font_space_em`이 남던 오류가 실제로 검출됐다. 일반 해소와 명시적 변경이 같은 `resolved_space_advance_em` 결과를 소비하게 했다. 자동 표시 대체는 원본 메트릭을 보존한다.

- 첫 집중 실행: 40개 중 39 PASS / 1 FAIL. 실패는 `declared_hft_gothic_keeps_its_monospace_ascii_punctuation`의 명시 TTF 공백 대조군이다(3750HU 대 2250HU).
- 보정 후 같은 집중 실행: **40 PASS / 0 FAIL**. #7752 HWPX 완료 서식, HWP/HWPX 부록 길이, #7092 명시 대체 TTF와 기존 source glyph/기호·언어 소유 검사를 포함한다.
- 전체 test API 확인에서 구형 prepared-state 필드 세 개와 두 case helper의 빈 `row_height_overrides` 초기화만 새 타입에 맞췄다. assertion·독립 기대값·허용치는 유지했다.
- 소스의 test 이름 보존 감사: control의 12개 검사는 새 모듈로 이동한 것을 확인했다. 원본 자식 폭 확대 검사는 새 devel의 `8d2ab06810`에서 `parent_and_child_height_order_does_not_expand_absolute_child_width`로 계약이 교정됐다. 사용자 우선순위에 따라 그 새 대조군을 유지했다.

## 실제 검증과 잔여

- Native Skia lib/CLI check 통과. 현재 전체 `--tests` API check 통과는 **테스트 실행 통과와 별개**다.
- Visual Sweep Python 84개, PNG/font helper 28개, Studio host font provider 15개 통과. 변경 Rust fmt·manifest check·공백 검사를 확인했다.
- [같은 Mac의 고정 devel 전쪽 TSV](../pr/assets/issue7445/completed-20261009/stage36-rebase-7752-20261022/README.md)를 Native 두 형식 각 383쪽으로 산출했다. HWPX 최저 88.76094%, HWP 최저 88.85188%이며 둘 다 277쪽 한 쪽이 미달이다.
- 후보 Native의 94/277/375쪽을 두 형식에서 진단했다. 모두 383쪽이며 raw는 각각 **92.41307 / 99.99781 / 100.00000%**다. 375쪽의 서식·내용 소유를 review/overlay로 직접 대조했다. 이는 선택 쪽 진단이며 후보 전쪽 수용이 아니다.
- 같은 Mac 기준의 94쪽은 92.69771% → 92.41307%로 **0.28464%p 하락**했다. 후보 전쪽 driver는 캡처 전에 종료했고, 기준 수집은 끝까지 보존했다. 기준·후보의 원본 image layout bbox는 같으며 Print paint의 그림 원점·크기 변환 차이를 독립 PDF CTM과 대조 중이다. 임의로 OutputDevice 표식을 바꾼 SDK 실험은 진단 전용이며 제품 구현으로 채택하지 않았다.
- 새로운 head의 전체 nextest, fresh WASM 전쪽, 원래 브랜치의 전체 영향 범위·#8064 Native 래칫과 실제 소비 경로 검증은 아직 남았다. 이전 head의 전체 통과를 재인용하지 않는다. **PR 준비 미완료·제출 보류**다.

작업 로그·단계별 source/충돌 blob·실제 PID/producer·선택 쪽 PNG는 ignored `output/pr-review/7445-completed-20261009/rebase-7752-20261022/`에 보존한다. 백업 ref `backup/7445-before-7752-20261022-122452`도 유지한다.
