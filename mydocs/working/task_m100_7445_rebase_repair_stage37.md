# 단계 37: 리베이스 뒤 그림 여백과 원 PUA 전진폭 복구

## 범위와 출처

- base `2e087a5b4bc1515ad877e8020b30114639a59392`, 보정 전 HEAD `4eeea6d7b40a0edf237b24534bbad3999b0cf537`.
- #7752의 새 표 수용 원장은 유지한다. 이번 단계는 그림 슬롯의 여백 중복과 실제 공급한 PUA 프로그램의 측정 우선순위를 고친다.
- 원 PUA 계약은 upstream의 `774d414cd998d1c04abcc6eac18aeed40d36e90f` 및 기존 `native_pua_host_font_protocol`을 대조했다.
- 그림의 독립 한컴 재저장 근거와 원본 전체 Native/fresh WASM 수용 이력은 [fixture 설명](../../tests/fixtures/issue7445_picture_source_frame/README.md)을 따른다. 이번 결과를 새 후보 전체 WASM 통과로 쓰지 않는다.

## 공통 원인과 보정

1. `inline_picture_occupied_size_hu`가 이미 그림 본체와 바깥여백을 합산한다. `inline_control_size_hwp` → `inline_control_occupied_size_hwp` → `inline_frame_input::project`가 이 점유 상자를 소비하는데 마지막 투영이 같은 여백을 다시 더했다. 투영은 공통 결과를 그대로 사용한다. 독립 기대 5980/5083/6580HU에 대해 보정 전 실제 값은 9980/8483/10580HU다.
2. 원 PUA 공급 뒤 표시 단계는 원 glyph를 유지했지만 `char_width_decision`이 Unicode 표시 문자로 다시 바꿔 원 hmtx 분기에 도달하지 못했다. 실제 프로그램에 해당 원 glyph가 있을 때 원 키를 먼저 측정하며, 없으면 기존 표시 대체를 유지한다. 합성 원본 SFNT의 독립 hmtx 0.9em에 대해 보정 전 배치는 1em이었다. 원본 문서·글꼴 바이너리를 수정하지 않는다.
3. 기존 그림 검사는 합성 첫 줄 캐시가 생략되는 HWPX 저장본에서 캐시를 직접 읽어 먼저 실패했다. 5980/5083/6580 기대값은 유지하고 실제 편집 모델에서 검사한다. 재열기는 후속 원문 `AFTER`의 유일한 소유와 실제 원점 일치도 검사한다. 검사 삭제·허용치 완화·새 픽셀 핀 추가는 없다.

## 실제 전후 실행

- 같은 그림 검사 본문을 frozen 보정 전 Native 라이브러리로 실행: 1 FAIL, `(9980, 8483) != (5980, 5083)`.
- 원 PUA 기존 검사 본문의 보정 전 실행: 1 FAIL, advance=13.3333333333, size=13.3333333333, 독립 기대 0.9em.
- 보정 후보의 정식 generated suite/nextest: 그림 6 PASS, PUA 4 PASS. 함께 실행한 #4068 이미지 스택 2건은 계속 FAIL이다. 전체 12건 중 10 PASS/2 FAIL, 필터로 752건 제외, 테스트 0.252초. 필터 제외를 전체 검증 skip으로 혼동하지 않는다.
- Cargo jobs 8/threads 8, 공유 target 순차 실행. manifest를 다시 prepare한 실제 target은 `regression_suite_001`, `002`, `012`다. 이전 target 번호를 고정 재사용하지 않았다.
- source hash·명령·실행 시간은 [집중 검증 결과](../pr/assets/issue7445/completed-20261009/stage37-rebase-contracts-20261022/focused-native-result.json), 독립 기대와 전후 원장은 [증거 요약](../pr/assets/issue7445/completed-20261009/stage37-rebase-contracts-20261022/before-after-proof.json)에 보존한다.
- v3 producer에는 다음 표 호스트 원인 추적용 `RHWP_TABLE_DRIFT` 조건부 진단도 포함된다. 이 진단은 이번 커밋의 두 생산 보정과 별도이며 새 표 수용 보정 완료를 뜻하지 않는다.

## 전체 검증과 남은 회귀

- 보정 전 4eee 전체 nextest: 11,539 실행, 11,392 PASS/147 FAIL/45 skip, 테스트 827.413초, 컴파일 포함 1383.841초, exit 100. [정본 결과](../pr/assets/issue7445/completed-20261009/stage37-rebase-contracts-20261022/full-regression-before-repair.json). 이 결과는 이번 보정 후보의 전체 결과가 아니다.
- 같은 Mac·글꼴·PDFium·96DPI·Print/compat2024·2px 기준으로 편람 HWP/HWPX 각 383쪽의 base/후보 Native TSV를 완주했다. 4eee 후보는 양쪽 최저 92.41307%, 90% 미만 0이나, 각 24쪽에서 새 base 대비 0.1%p 이상 하락하여 보류다. HWPX 94쪽은 92.69771→92.41307%, HWP는 92.63808→92.41307%다.
- 277쪽은 base HWPX 88.76094/HWP 88.85188에서 후보 99.99781%로 개선됐다. 이 개선으로 다른 쪽의 하락을 상쇄하지 않는다. 94/277/294쪽 review와 overlay를 대조했고 277쪽 제목·목차·배경 소유 및 294쪽 원문을 확인했다. 세부 래스터/간격 잔여를 완전 일치로 쓰지 않는다.
- `issue2004_cell_image_stack.hwp`는 base 8쪽/후보 9쪽이며 후보 4쪽에 표 호스트만 있고 그림·괘선이 없다. 기존 #4068의 4~8쪽 기대를 바꾸지 않고 첫 조각의 호스트 줄 예약을 추적 중이다.
- 새 후보 전체 회귀, Native/fresh WASM 전쪽·실제 소비 경로 및 기타 제출 게이트는 남아 있다. PR 생성·push·외부 게시는 하지 않았다. 지침 변경도 없다.

## ignored 원본 증적

`/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-7752-20261022/` 아래 `full-regression-v1`, `repair-focused-v3`, `picture-margin-repair`, `pua-source-advance-repair`, `handbook-whole`, `image-stack-border-owner-diagnostic`에 로그·PNG·전쪽 TSV·frozen producer를 보존한다. `.log`는 커밋하지 않는다.
