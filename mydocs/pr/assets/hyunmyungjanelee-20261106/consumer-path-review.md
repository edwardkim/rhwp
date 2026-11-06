# 공통 결과와 실제 소비 경로 검토

현재 제품 source tree는 `c1ea19fbc94dd6aac77d4368755b6da5a9544e42`이다. 줄 번호는 아래 파일 경로의 현재 함수 이름과 함께 확인한다.

| PR | 생산 → 측정/소유 → 실제 배치·저장 | 적용 경계와 비적용 이유 |
| --- | --- | --- |
| #8515 | `shape_layout.rs::layout_textbox_content`의 첫 문단 배치 패스 `paragraph_origins` → 같은 세로 정렬 translation → 둘째 control 패스 `para_start_y`; `textbox_floating_table_offset_px` → `layout_embedded_table`의 x/y | non-TAC InFront/Behind, 가로 Column/Para Left·세로 Para Top. TAC는 게시된 inline shape 원점/같은 문단 줄 계획을 사용한다. 그림·표의 paint 하단과 실제 본문 흐름을 구분한다. 최대 signed offset + margin은 f64에서 정확히 더하며 clamp하지 않는다. |
| #8599 | `table_partial.rs`의 소유 컷/줄 수 → `uncut_control_only_nested_table`의 문단 보존 → `para_y_before_compose`와 저장 오프셋 → 실제 Shape/Image node | 컷이 없는 온전한 행의 저장 줄 없는 control-only TAC 도형을 구제한다. 컷이 있는 조각의 기존 소유 유닛·요구 높이·예약·이월·종료는 바꾸지 않는다. 줄 수 0의 Para 그림은 뒤 본문 문단이 없는 경우 원점을 복원한다. 전체 2쪽 화살표·그림과 원본 순서/누락을 비교했다. |
| #8608 | `model/table/contradictory_grid.rs` 하한 경계(HU) → `table.rs::fallback_paragraph_frame_owner_widths_with_storage` 및 `row_frames.rs` → `table_layout.rs`의 모순 표 경계 소비 | 짧은 행·모순 폭·표 폭 안의 해가 모두 있을 때만 적용한다. 정상 등식·범위 밖/해 없음은 기존 경로다. Storage 그래프·queue·하한 벡터 할당과 오류를 같은 요청으로 전달한다. 96HU는 두 독립 한컴 Print의 모순 경계 최소 간격 관측값이며 형식 사양의 보편적 최소 폭으로 주장하지 않는다. |
| #8631 | HWP5 DocInfo BIN_DATA 레코드 순번 → Link 빈 자리 보존 → 기존 `content_index` → 그림 데이터/크기/crop → CFB·HWPX 실제 쓰기 | storage_id와 레코드 순번을 혼동하지 않는다. Link 빈 자리만 실제 BinData 파일 쓰기에서 제외한다. 일반 id 조회·외부 파일 해석을 바꾸지 않는다. |
| #8656 | `model/image.rs` 꼬리 길이/효과 flag → `parser/control/shape.rs` 원본 크기·선택적 alpha → Image crop/크기 → `serializer/control.rs` 저장 | 효과 flag 4의 4바이트 f32 반경 다음 확장 크기를 해석한다. 21/22바이트 공개 합성 대조군과 기존 꼬리를 구분한다. blur나 다른 효과 전체의 렌더링 완료 주장이 아니다. |

#8599의 기존 컷·행 이어받기와 #8659의 Storage/편집 계약은 집중 27개·전체 12,752개·Native 공식 3종 4,064개, 원 사례/경계/정상 각각 12쪽과 최신 devel 보존 55쪽, 실제 WASM 공개 API 12그룹의 결과로 확인했다. 최대 오프셋 경계와 기존 뒤 본문 겹침은 별도 JSON·직접 판독으로 구분했다. 이 기록은 아직 실행하지 않은 검사를 통과로 표시하지 않는다.
