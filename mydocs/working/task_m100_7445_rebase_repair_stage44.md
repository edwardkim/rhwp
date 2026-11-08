# Stage 44: 첫 중첩 분할 프레임의 예약·정렬 소비 복구

- 시작 HEAD: `66ba7e3cdd80c7bda647cad6bc0060c08e40e35f`.
- 선행 분석: 원 보정 `aba8a59923`와 Stage25, 통합 전 `5e23738a`를 대조했다. 원본 자식의 저장 프레임을 확인하는 `stored_mixed_opening_frame`은 남았으나 모듈 등록과 수용·높이·정렬 소비가 빠져 있었다. 새 upstream의 행 수용 원장과 물리 예산은 유지한다.
- 공통 보정: 현재 확정 컷을 같은 질의에 전달해 첫 선언 프레임의 물리 높이를 예약하고, 그 프레임 안에서 끝난 자식의 실제 내용 높이로 가운데 정렬한다. 실제 `cut_units`를 구성 창보다 먼저 한 번 확정해 구성·정렬이 공유한다. 기존 높이 계산은 `height_without_mixed_opening`으로 보존해 공통 질의의 재귀를 피한다. 문서·쪽·좌표 예외는 없다.
- 실제 소비: 확정 source 컷 → 수용 스캔의 물리 예산 → `CellCutFrame`/행 높이 → 같은 컷의 부분 셀 구성 창 → `effective_align` → 최종 `text_y`. 후단 정렬 분기까지 연결했다. 원본 cell height27451HU, 패딩141+141HU, 물리 높이366.013333px이다. 자식 내용355.36px를 사용해 최종 첫 줄 y646.633333px, `align=Center`, `mixed_declared=true`를 확인했다. 앞선 v10b는 높이 질의만 연결하고 후단 정렬 소비가 누락되어 해당 관계 검사가 실패했다.
- 기존 Native 검사: v10b의14 PASS/1 FAIL → v10c의15 PASS/0 FAIL(2.788초). HFT 첫 분할 가운데 정렬·이어받기, #1488 저장 소유, 첫 행 잔여 상한, #7751 분리 셀 프레임, zero-origin 재시작, #8105 선언 빈 띠 대조군을 실행했다. 테스트 기대값은 바꾸지 않았다. 변경3파일의 rustfmt와 diff 공백 검사도 통과했다. 전체 fmt는 이번에 변경하지 않은 `canvaskit_policy.rs`와 `legacy_font_advances.rs`의 기존 서식 차이가 있어 최종 게이트 전에 별도로 처리한다.
- 전쪽 시각 대조: HWP(compat2024)·HWPX(compat2022) 각각18쪽을 fixed base/수정 전/후로 전부 산출했다. 같은713개 글꼴·Native Print96DPI·PDFium152.0.7947.0 smooth RGB·raw2px90% 조건이며 총108쪽이다. HWP7쪽90.18709→99.83073%, HWPX7쪽87.01856→99.60978%. 수정 전 대비0.1%p 이상 하락은 두 입력 모두0쪽이다. HWP의 과거 compat2022 결과와 같은 환경이라고 주장하지 않는다.
- 직접 판독: HWP7쪽 review/overlay에서 제26조 첫 줄·중첩 상자·좌측 제목과 앞뒤 조각 경계를 확인했다. 작은 글꼴·래스터 차이는 남아 있다. 새 px 핀은 추가하지 않았다. fresh WASM 전쪽 조건을 아직 충족하지 않았다.
- **미완료**: HWP11/12쪽80.85851/62.37510%, HWPX11/12쪽80.14767/36.58076%는 이번 보정 전후 동일하다. 후보 전체 미달은 HWP11·12쪽, HWPX11·12쪽이며 전체 수용이 아니다. 앞 표의 실제 상자는 같지만 다음 흐름이 약8.7px 더 전진해 뒤 표의 컷이 달라지는 별도 원인을 계속 추적한다. fixed base도 HWP7·16쪽 및 HWPX7쪽 미달이 있다.
- [증적](../pr/assets/issue7445/completed-20261009/stage44-mixed-opening-20261022/): producer와 수치, 전조건 TSV, 대표 review/overlay. 로그·SDK·초기 실패는 ignored `rebase-7752-20261022/{repair-focused-v10,repair-focused-v10b,repair-focused-v10c,mixed-opening-frame-cause,mixed-opening-native-v10c}/`에 보존했다. 최초 v10은 삭제된 helper 호출로 컴파일 실패해 INVALID 처리했고 제품 검사 결과로 사용하지 않았다. 패널 진단의 잘못된 API 인자는 정정 후 다시 생성했다.
- 전체 회귀·fresh WASM·실제 소비 경로는 미완료다. PR/push/원격 게시는 수행하지 않았다.
