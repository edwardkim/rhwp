# 최종 결과보고서 — Task M100 #2230: placeholder 선택 + 그림 삽입 편집 기능

- 이슈: #2230 (#2225 후속) / 브랜치: `local/task2230` / 작성일: 2026-07-12
- 계획: `plans/task_m100_2230.md` + `plans/task_m100_2230_impl.md` (승인됨)
- 단계 보고: `working/task_m100_2230_stage{1..4}.md`
- 검증 문서: 36389312 결재문서 "심볼" 필드 (bin_id=0 미지정 그림)

## 요약

한컴 편집기처럼 그림 미지정 placeholder(#2225 MissingPicture)를 **클릭
선택**하고 **더블클릭 → 파일 선택 → 그림 지정**할 수 있게 했다. 지정은
Undo/Redo 정합 + HWPX/HWP 저장 왕복 보존. **작업지시자 실사용 판정 통과**
(선택 → 더블클릭 삽입 → undo/redo → 저장 → 재열기 유지).

## 단계별 결과

| 단계 | 내용 | 커밋 |
|------|------|------|
| 1 | placeholder 에 문서 좌표(control_ref kind="picture") + cell_context 배선, 컨트롤 레이아웃에 `type:"image"` + `missing:true` + cellPath 방출 → 클릭 선택 TS 무수정 성립 | `15d59a21` |
| 2 | `register_embedded_bin_data` 헬퍼 추출(insert 와 규칙 공유) + `assign_picture_image_native` 신설(대상 검증 선행, 틀 크기 유지, 캐시 무효화) + `assignPictureImage` wasm 노출 | `e586fe46` |
| 3 | studio: missing 마커 hit 3경로+선택 저장 관통(소실 지점 정정), 더블클릭 분기 → 파일 선택 → 스냅샷(Undo) 지정 → full refresh | `b471a435` |
| 4 | HWPX/HWP 저장 왕복 표적 테스트 + 전체 게이트 + WASM 빌드 | `d1b5fe94` |

## 핵심 설계

- **hit-test 소스 재사용**: studio 개체 선택의 본선은
  `getPageControlLayout` — placeholder 를 `type:"image"` 로 방출하면
  findPictureAtClick/선택 테두리/핸들이 기존 로직 그대로 동작한다.
  `missing:true` 마커만 더블클릭 진입 분기 근거로 추가.
- **BinData 규칙 단일 원천**: 등록 규칙(storage id 최댓값+1 채번)을 헬퍼로
  추출해 insert/assign 이 공유 — 저장 왕복이 첫 실행에 통과한 근거.
- **레이아웃 불변**: 지정은 bin_data_id·crop 만 갱신, 개체 틀 크기·배치
  속성 유지(한컴은 틀에 그림을 맞춤).
- **Undo 는 studio 스냅샷 패턴** — Rust 측 undo 스택 작업 불필요.

## 검증 총괄

| 게이트 | 결과 |
|--------|------|
| 표적 테스트 (`tests/issue_2230_placeholder_selection.rs`) | 6건 (선택 방출 2 + 지정 2 + 왕복 2) — 1단계 수정 전 FAILED 실증 |
| fmt / clippy (release-test all-targets) | 통과 / 0 |
| 전수 `--tests --no-fail-fast` | **3058 / 0** (golden 포함) |
| OVR 5샘플 (±2px, 분리 폴더) | 회귀 0건 |
| studio tsc / vite build / npm test | 클린 / 성공 / 206/0 |
| WASM 빌드 | 성공 (`assignPictureImage` 심볼 포함) |
| **작업지시자 실사용 판정** | **통과** (2026-07-12) |

## 잔여/관찰

- 머리말·꼬리말 안 미지정 그림: 좌표 미전파 경로는 기존 #2225 표시-만
  동작 유지(선택 불가). 실수요 발생 시 후속.
- OLE placeholder(kind="ole") 동작 불변 — 분기 추가만.
