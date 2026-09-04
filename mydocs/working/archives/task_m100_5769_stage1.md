# Stage 1 완료 보고 — Task M100-5769: 선택 삭제의 참 역연산 (조각 저장소)

- 일자: 2026-08-22
- 브랜치: `fix/5769-delete-inverse` @ `304cb031b`
- 계획서: `mydocs/plans/task_m100_5769.md` Stage 1
- 이슈: edwardkim/rhwp#5769

## 한 것

`delete_range_native` 가 제거분을 drop 하던 자리에 **조각(fragment) 저장소**를
추가했다. 삭제 직전 `paragraphs[start..=end]` 원본을 통째로 보관하고 undo 시 그
자리에 되돌려 끼는 LibreOffice SwNodes 묘지 패턴이다.

### 조각이 되돌리는 4요소 — 각각이 깨지면 저장 바이트가 어긋난다

| 요소 | 깨지는 경로 |
|---|---|
| 범위 문단 전체 클론 | delete_text_at 의 char_shapes 클램핑·병합(#3576/#4271)이 사후 재구성 불가 |
| **꼬리 line_segs 저널** | `recalculate_section_vpos` 가 start_para~구역 끝 vertical_pos 를 덮어쓰고 HWP5 직렬화기(body_text.rs LINE_SEG)가 그 값을 그대로 기록 |
| 구역 raw_stream/provenance | delete 가 `raw_stream = None` 을 박아 원본 바이트 재사용 경로 상실 |
| **캐럿(doc_properties)** | DocInfo 봉인 다이제스트(`doc_info_model_digest`)가 DocProperties 전체를 포함 — 캐럿만 남겨도 raw 재사용이 깨져 IR 재직렬화 폴백 |

### 디버깅 기록 — @888 불일치의 정체

첫 구현에서 5개 게이트 중 4개가 실패했고, 실패 지점은 이전 시도(osk 노드 기록)와
동일한 @888 이었다. 결정적 단서는 **구역 선두 [0..1] 만 통과**한 것: 복원 후 캐럿이
(0,0) 으로 돌아오는 유일한 케이스였다. 즉 문단·raw 복원은 옳았고, DocInfo 쪽
봉인이 캐럿 변화로 깨져 폴백 직렬화가 나던 것. 조각에 캐럿 스냅샷을 추가해 해소.

### 안전장치

- 복원 전제 검증: 현재 문단 수 ≠ 캡처 시 −(범위−1) 이면 거부 — 캡처 후 삭제가
  적용되지 않았거나 다른 편집이 끼었을 때 무음 중복 삽입 차단.
- 코어 자동 축출 없음 — 조각은 KB 급이라 스냅샷 #2328 클래스 무통보 축출을 만들
  이유가 없다. TS 히스토리 discard 계약으로 해제.

## 검증

- 통합 게이트 `tests/cases/issue_5769_delete_fragment_byte_identity.rs` 7/7 통과:
  단일 부분 삭제·중간 범위·표 포함·선두 경계·끝 경계에서 **저장 바이트 왕복 동일성**
  + 스냅샷 경로 대조 + 전제 검증 거부 + discard 계약.
- lib 전체 3,893 통과 / clippy clean / fmt 적용.
- unit-tier 게이트: src 신규 #[cfg(test)] 모듈 금지 정책(`newCfgTestModules:
  forbidden`)에 걸려 테스트를 tests/cases 로 이동 — 4225 기준선 복원 확인.
  (`local_validation.md` §integration test 절 준수)
- 파생 하니스(tests/generated/)는 로컬 검증용으로만 생성·gitignore — stage 안 함.

## WASM 표면

`captureDeleteRange(sectionIdx,startPara,endPara)` ·
`restoreDeleteFragment(id)` · `discardDeleteFragment(id)`.

## 다음

Stage 2(TS DeleteSelectionCommand 배선) — kind:'command'·selectionBefore·
snapshotResourceCount 0 수렴 계약 유지. Stage 3(붙여넣기), Stage 4(페이지·구역 설정).
