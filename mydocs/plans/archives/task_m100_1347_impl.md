# 구현계획서 — 단 구분선 부분 페이지 길이 복원 (M100 #1347)

## 변경 대상: `src/renderer/layout.rs` (단일 파일)

`4e7f191f`가 도입한 `prev_zone_sep_full_body` 플래그와 조건부 `sep_y_end`를 제거하고, 두 emit 지점 모두 `prev_zone_y_end`(콘텐츠 높이)를 직접 전달. body 하단 캡은 `emit_zone_column_separators` 내부에 유지되어 꽉 찬 페이지 자연 처리.

### 수정 지점 (3 + 선언 1)

1. **선언 제거** (≈line 2432): `let mut prev_zone_sep_full_body: bool = false;` 삭제
2. **중간 zone emit** (≈2483-2487): 조건부 sep_y_end → `prev_zone_y_end` 직접 전달
3. **기록부** (≈2561-2573): #1333 콘텐츠높이 주석 복원, `prev_zone_sep_full_body` 대입 2곳 제거
4. **마지막 zone emit** (≈2731): 조건부 sep_y_end → `prev_zone_y_end` 직접 전달

## 단계

### Stage 1 — 소스 수정
- 위 4지점 편집. 결과적으로 #1333(91c5cebe) 동작 복원.

### Stage 2 — 검증
1. `cargo build --release`
2. `export-svg -p 22`(p23): 중앙 구분선 y_end가 콘텐츠 하단(PDF 44.7%)으로 단축됐는지 측정
3. `export-svg -p 15`(p16): 전체 높이 유지 확인
4. shortcut.hwp 9개 구분선 보존·중복 0
5. `cargo test --release` 회귀 무결
6. 단계 보고서 `working/task_m100_1347_stage2.md`

### Stage 3 — 최종 보고서 + PR
- `report/task_m100_1347_report.md`
- 계획서 archives 이동, 커밋
- fork(origin) push → upstream(stream) PR (squash)

## 완료 정의
- p23 구분선 PDF(44.7%) ±1.5% 정합, p16 전체 유지
- shortcut.hwp 구분선 회귀 없음, cargo test 통과
