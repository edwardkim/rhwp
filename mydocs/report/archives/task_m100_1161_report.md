# 최종 결과보고서 — Task M100-1161: rhwp-studio 셀 안 picture 복사(Ctrl+C) 지원

## 1. 목표 / 결과

표 셀(중첩 표 포함) 안 inline picture 를 객체 선택 후 Ctrl+C 로 복사해도 시스템/내부
클립보드에 저장되지 않던 결함을 해소. **셀 picture 복사 → 붙여넣기 정상 동작 확인**(작업지시자
localhost 시각 판정). 추가로 떠 있는 개체 반복 붙여넣기 cascade offset(한컴 정합) 구현.

## 2. 근본 원인

복사 경로의 native 4종(`copy_control_native` / `export_control_html_native` /
`get_control_image_data_native` / `get_control_image_mime_native`)이 **본문 컨트롤만 접근**하고
셀 경로(cell_path)를 받지 못함. 더 깊게는 렌더 트리 **ImageNode 가 TextRun 과 달리 다단계
cellPath 를 보유하지 못해**(단일 레벨 스칼라만) 프런트가 중첩 셀 picture 의 경로를 복원 불가.

## 3. 구현 (Stage 1~4 + 추가)

| Stage | 내용 | commit |
|------|------|--------|
| 1 | clipboard native 4종 `cell_path` 인자 + 공통 헬퍼 `resolve_control_para` + 회귀 테스트 | 4bae87cc |
| 2 | wasm_api 4 래퍼 `cell_path_json` 연결(`parse_cell_path`) | a83a7d24 |
| 3 | **ImageNode `cell_context` 필드 + 레이아웃 3 site 채움 + rendering `cellPath` 방출**(#1171 공유 기반) | 6ee45cff |
| 4 | TS: 선택 ref 가 cellPath 보유 + 복사/오려두기/컨텍스트 메뉴 전 지점 배선 + 타입 보정 | 63edacbb |
| 추가(옵션 B) | 떠있는 개체(tac=false) 붙여넣기 cascade offset | 33b35cab |

### 설계 핵심
- **cellPath 단일 진실원**: ImageNode `cell_context`(TextRun 정합) 권위값, 기존 단일 레벨 스칼라는
  innermost 투영으로 유지(하위호환, 회귀 0). 셀 picture 생성 chokepoint `layout_picture_full` 등
  3 site 에서 동일 `cell_ctx` 로 채움.
- **inline vs floating 구분**(한글 5.0 스펙 표 70 bit 0 "글자처럼 취급"): inline(tac=true)은
  텍스트 흐름이 위치를 정하므로 cascade 미적용, floating(tac=false)만 cascade.
- 검증 데이터: `pic-in-table-01.hwp` p16 셀 picture 는 외부표→셀5→내부표→셀1/3/5/7 의 **2단계 중첩**,
  cellPath `[(0,5,0),(0,1,0)]` 방출 확인.

## 4. #1171 공유 기반

이슈 #1171(사각형→글상자→picture 이중 nested click/dialog)과 **동일 뿌리**(ImageNode 다단계 cellPath
부재). Stage 3 이 그 기반(ImageNode cell_context + `resolve_paragraph_by_path` 의 표 셀·글상자 공통
탐색)을 구축 → #1171 은 이를 소비하면 됨. 추가(additive)·스칼라 불변·chokepoint 1곳 → #1171 회귀 위험 낮음.

## 5. 검증

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | 1471 + cascade 2 + cellpath-json 1 passed, 0 failed |
| `cargo test --tests` (전 통합) | 0 failed |
| `tests/issue_1161_copy_picture_in_cell.rs` | 4 passed (셀 복사/본문 회귀) |
| `tests/issue_1161_image_cellpath.rs` | 1 passed (2-엔트리 cellPath 방출) |
| cascade 테스트(floating 누적 / inline 불변) | 2 passed |
| `tsc --noEmit` (WASM 재빌드 후) | 0 errors (canvaskit 기존 제외) |
| WASM 빌드(docker) | 성공 |
| **작업지시자 시각 판정** | 셀 picture 복사→붙여넣기 정상 / cascade 계단식 동작 확인 |

## 6. 범위 밖 / 후속

- **표 복사 분기**(getSelectedTableRef): #1161 은 picture. 중첩 표 복사는 기존 처리.
- **cut 의 삭제**(`deletePictureControl` cellPath 미지원): 중첩 셀 picture *삭제*는 후속. cut 의 *복사*는 정상.
- **cascade step**(567 HU ≈ 2mm): 한컴 정밀 정합은 추후 미세조정 가능.
- **🔴 신규 이슈 #1227** — 직렬화기 `mini_cfb` 가 출력 >7.14MB 시 DIFAT 미작성으로 CFB 손상(대용량 저장
  데이터 손실). #1161 시각 판정 중 발견했으나 **진단 결과 #1161 과 무관한 기존 직렬화 버그**로 확인
  (`parse_document→bin 추가→serialize_document` 만으로 재현, paste/clipboard 코드 0줄). 별도 처리.

## 7. 결론

셀(중첩 포함) picture 복사 지원 완료, #1171 공유 기반 구축, cascade 보강. 저장 시 발견된 대용량 CFB
손상은 무관한 기존 버그로 #1227 분리 등록. **#1161 머지 가능 상태.**
