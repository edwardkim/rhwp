# 최종 보고서 — Task M100-1323

- 이슈: https://github.com/edwardkim/rhwp/issues/1323
- 제목: 글상자/표 셀 안 이미지(컨트롤) 붙여넣기가 조용히 누락 — merge_from 컨트롤 미병합·빈 텍스트 early-return
- 브랜치: `local/task1323`
- 작성일: 2026-06-11

## 1. 결과 요약

`Paragraph::merge_from`(model 계층)을 보강하여 컨트롤 보유 문단 병합 시
`controls`/`ctrl_data_records`/`control_mask`가 보존되도록 했다. 이로써:

- 글상자/표 셀(중첩·캡션 포함)에 이미지 copy→paste 시 그림 컨트롤과 CTRL_DATA가
  보존되고 SVG에 실제 렌더된다.
- 본문/셀 백스페이스 문단 병합, 다중 문단 선택 삭제 등 merge_from을 경유하는
  모든 경로의 동일 잠재 결함이 함께 해소됐다.
- 클립보드/편집 명령 계층과 프론트엔드는 수정하지 않았다(기존 라우팅 그대로 동작).

## 2. 변경 내역

| 파일 | 변경 |
|---|---|
| `src/model/paragraph.rs` | `merge_from` 컨트롤 병합 보강 (early-return 조건, utf16_end 갭 인코딩, controls/ctrl_data_records/control_mask 병합, char_count·has_para_text 정합) |
| `src/model/paragraph/tests.rs` | merge_from 컨트롤 단위 테스트 5건 + 헬퍼 |
| `src/wasm_api/tests.rs` | 통합 테스트 7건(셀/path 셀/캡션 paste, 본문·셀 백스페이스 병합, HWP5 round-trip, SVG 렌더링) + `find_table_pos` 헬퍼 |
| `src/document_core/commands/object_ops.rs` | `paste_picture_into_textbox` 테스트 추가, "별개 결함" 주석을 해소 사실로 갱신 |
| `mydocs/plans/task_m100_1323.md` | 수행 계획서 |
| `mydocs/plans/task_m100_1323_impl.md` | 구현 계획서 |
| `mydocs/working/task_m100_1323_stage{1,2,3}.md` | 단계별 보고서 |

## 3. 설계 요점 (직렬화·렌더링 계약 정합)

1. **컨트롤 위치 = char_offsets 갭**: 2차 병합(right_half) 시 trailing 컨트롤의
   8 code unit을 utf16_end에 가산하여, 렌더링(`control_text_positions`)과 HWP5
   직렬화(`serialize_para_text` 갭 분석)가 병합 지점(커서 위치)에 컨트롤을 복원한다.
2. **char_count**: 텍스트 + 8×컨트롤 수 + 1 — `split_at` 및 HWPX 직렬화의
   컨트롤 수 역산(`serializer/hwpx/section.rs`)과 정합.
3. **ctrl_data_records[i] ↔ controls[i] 정렬**: self 쪽 None 패딩 후 other 이어붙임 —
   HWP5 직렬화의 인덱스 기반 소비와 정합.
4. **control_mask**: OR 병합.

## 4. 검증

### 4.1 단위·통합 테스트

- `cargo test` 전체 — **2139 passed, 0 failed**, 6 ignored
- 신규 테스트 13건: 단위 5(모델 병합 의미론) + 통합 7(paste/병합/round-trip) +
  SVG 렌더링 1
- HWP5 저장 round-trip: 셀에 붙여넣은 그림이 serialize→parse 후 보존
  (char_count 역산·갭 인코딩 정합 실증)
- SVG 렌더링: 셀/글상자에 붙여넣은 **부동(treat_as_char=false)** 그림이 실제
  `<image>` 요소로 방출 — 리스크 항목(떠 있는 개체 셀 anchor) 자동 검증

### 4.2 정적 검사

- `cargo clippy --all-targets` — 무경고
- `rustfmt --check` (변경 파일 한정) — 통과

### 4.3 WASM 빌드

- `docker compose --env-file .env.docker run --rm wasm` — 통과, `pkg/` 갱신

### 4.4 작업지시자 시각 판정

rhwp-studio 수동 검증 4항목 통과 (2026-06-11 작업지시자 확인):

1. 본문 이미지 복사 → 글상자 안 붙여넣기 → 렌더 정상
2. 본문 이미지 복사 → 표 셀 안 붙여넣기 → 렌더 정상
3. 텍스트 붙여넣기(본문/셀/글상자), 본문 이미지 붙여넣기 무회귀
4. 셀 안 백스페이스 문단 병합 시 그림 보존

## 5. 범위 밖 관찰 (별도 이슈 후보)

- `merge_from`이 `tab_extended`를 병합하지 않음 — other 텍스트에 탭이 있으면
  탭 확장 데이터(너비/종류) 소실 가능.
- 프론트 `clipboard_has_control_native`가 클립보드 첫 문단만 검사 — 다중 문단
  클립보드의 컨트롤 처리(본문 pasteControl이 첫 문단만 붙여넣는 문제 포함)는
  별도 이슈 영역.
- html_import의 merge_from 우회 분기(`has_controls` 검사)는 본 수정으로 단순화
  가능해졌으나 본 타스크 범위 밖.

## 6. 후속 절차 (CONTRIBUTING.md 컨트리뷰터 워크플로우)

- PR 전 체크리스트: `cargo fmt --all -- --check` / `cargo test` /
  `cargo clippy -- -D warnings` 통과 확인
- Fork(`johndoekim/rhwp`)에 작업 브랜치 push → `edwardkim/rhwp` `devel` 대상
  PR 생성
- CI 통과 + 메인테이너 리뷰 승인 후 merge, Issue #1323 close는 메인테이너 수행
