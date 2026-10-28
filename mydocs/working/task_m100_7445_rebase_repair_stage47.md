# Stage 47: 온전한 자식 표의 행 결과와 clip 소유 복원

- 시작 HEAD: `6c977c63c92ba9444415ab405046333c8e99c7bb`.
- 선행 분석: 원 보정3fe47ee64f와 pre-rebase5e23738의 no-cache 순수 행 결과 소비 및 ordinary child clip을 대조했다. 실제source는 HWPX 원본의 cell row2/col0, 문단81/control0의3행7셀이다. 현재 upstream의 row-span 배분·패딩 재검사 뒤에 저장 tail fitting 소비가 빠져 있었고 온전한 Exclusion child도 글줄 clip에 갇혔다.
- 보정: 기존 `fit_stored_declared_tail_row_heights`의 확정 행 결과를 캐시 유무와 무관하게 사용한다. 편집·재조판·저장 reset 제외 계약을 유지하고 RowBreak/CellBreak의 같은 원본 규칙을 사용한다. 실제 마지막 빈 여유260HU만 회수하며 머리행·내용 하한은 보존한다. ordinary child가 원본 시작과 전체창을 소유할 때만 공유 `ParagraphFloatPlacement.occupied_bottom`을 clip이 소비한다. pagination 예산과 cursor는 바꾸지 않았다.
- Native 실제 원장: 두 머리행17.093333px 유지, 마지막71.08→67.613333px, 총105.266667→101.8px로 원본7635HU와 일치. 온전한 자식4검사 모두PASS, 집중20건16 PASS/4 FAIL(3.973초). 남은4건은 별도 #4138 분할본 소유 실패다. 기존 테스트와 독립 좌표 기대값을 바꾸지 않았다.
- 같은 글꼴·Print96DPI/PDFium/2px90%에서 base/전/후 × HWP/HWPX 각115쪽, 총690쪽을 재산출했다. 두 후보 최저97.01264%, 미달0,4쪽97.05212→99.07270%. 양쪽 모두4쪽만 PNG 변경, 전체 TextRun 순서/소유/개수7919개 불변, base/전 대비0.1%p 이상 하락0이다. HWP의2024 Print와 HWPX의2020 Print를 혼동하지 않았다.
- 직접 판독: review와overlay에서 마지막 행 세 내용·하단 괘선이 복원됐고 본문과 다음 쪽의 소유가 유지됨을 확인했다. 일부 글자 가장자리·괘선 굵기는 남는다. 새 px 회귀는 추가하지 않았다. 이전의 정식4검사를 그대로 복구했다.
- all fmt 및 diff 검사는 통과했다. `canvaskit_policy.rs` import 정렬과 `legacy_font_advances.rs` match arm은 rustfmt만 적용했다. 최초 시각 요약 도구가 PDF PNG의 밑줄 파일명을 사용해 그림 생성 전에 멈췄고, 실제 하이픈 파일명으로 고쳐 저장 PNG로 재실행했다. 전체 캡처를 중복하지 않았다.
- [전체 TSV·대표 review/overlay·출처](../pr/assets/issue7445/completed-20261009/stage47-complete-child-frame-20261022/verification.md). 생성 JSON/SDK/로그는 output에만 보존한다. 이 단계는 전체PR 검증 완료가 아니다. 남은 Native 회귀/시각 보류, 새 full nextest/fresh WASM/실제 경로 검증을 계속한다. PR/push/원격 게시는 하지 않았다.

- 최종 같은 소스의기존 실패56건 재검사는24 PASS/32 FAIL(47.878초)이다. 수정 전 실패35건과 이름별 대조에서 새 실패0, 위3건 복구를 확인했다. full nextest를 대신하지 않는다.
