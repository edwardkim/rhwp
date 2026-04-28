# 구현계획서 — Task #409

## 개요

`prev_has_overlay_shape` 가드(`src/renderer/layout.rs:1366-1370`)를 확장하여 TopAndBottom Picture/Shape (vert=Para) 다음 문단에서 vpos 보정이 차트 높이를 이중 반영하지 않도록 수정.

## 단계 구성 (3단계)

---

### Stage 1: 진단 재현 + 회귀 베이스라인 수집

**목적**: 수정 전후 비교용 베이스라인 확보 + 가설 수치 재확인.

**작업 항목**:
1. 대상 페이지 베이스라인 수집
   - `rhwp export-svg "samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx" -p 20` 의 LAYOUT_OVERFLOW 로그 캡처 (수정 전)
   - `dump-pages -p 20`, `dump -s 0 -p 172`, `dump -s 0 -p 173`, `dump -s 0 -p 174` 출력 캡처
   - 21페이지 SVG → PNG 변환하여 시각 비교 자료 보관 (`mydocs/working/task_m100_409_stage1_before/`)
2. 회귀 검증용 베이스라인 수집
   - `cargo test --lib` 통과 개수 확인
   - `cargo test --test svg_snapshot` 통과 개수 확인
   - TopAndBottom 그림 포함 다른 문서 식별 (grep `TopAndBottom` + Picture in samples)

**완료 조건**:
- 베이스라인 수치 + 로그가 stage1 보고서에 기록
- 가설(차트 높이 ≈ 31470 HU 이중 반영)이 수치로 재확인됨

**산출물**: `mydocs/working/task_m100_409_stage1.md`

---

### Stage 2: `prev_has_overlay_shape` 가드 확장 구현

**목적**: 가드 로직을 Picture + TopAndBottom(vert=Para)까지 확장.

**작업 항목**:
1. `src/renderer/layout.rs:1366-1370` 가드 식 수정
   - `Control::Picture` (non-TAC) 분기 추가
   - 기존 InFrontOfText/BehindText에 더해, TopAndBottom + vert_rel_to=Para 케이스 추가
   - 변경 의도 설명 주석 갱신 (왜 TopAndBottom 도 우회해야 하는지)
2. 명확성을 위해 가드 추출
   - 가드 식이 길어지므로 helper 함수 또는 클로저로 분리 검토
   - 중복 호출 시 캐시 — 현재 위치는 단일 호출이므로 인라인 유지 가능
3. `cargo build --release` 통과 확인
4. 21페이지 SVG 재생성 → LAYOUT_OVERFLOW 로그 검증
   - pi=174 (2x1 표) overflow 사라짐 또는 정상 페이지 분할
   - pi=192 (10x5 표) overflow 사라짐
   - PDF와 시각적으로 일치 확인 (qlmanage 변환 후 비교)

**완료 조건**:
- 21페이지 차트 바로 아래 2x1 표가 정상 위치 배치
- LAYOUT_OVERFLOW: pi=174~191 사라짐, pi=192는 다음 페이지로 흘러감
- 빌드/clippy 경고 없음

**산출물**: `mydocs/working/task_m100_409_stage2.md`, `src/renderer/layout.rs` (변경)

---

### Stage 3: 회귀 검증 + 최종 보고

**목적**: 전체 테스트 통과 + 다른 문서 무회귀 확인.

**작업 항목**:
1. 단위/통합 테스트
   - `cargo test --lib` (목표: stage1 베이스라인 동일 통과)
   - `cargo test --test svg_snapshot`
   - 실패가 나오면: 골든 SVG 의도된 변경(올바른 페이지네이션)인지 검토 후 갱신
2. 샘플 회귀 검증
   - TopAndBottom 그림 포함 대표 샘플 1~2건 페이지 수 / 시각 비교
   - 그림 없는 일반 문서 샘플 1건 페이지 수 비교
3. ir-diff (해당 문서 HWPX↔HWP 가능 시)
4. 최종 보고서 작성
   - 변경 전후 LAYOUT_OVERFLOW 비교표
   - 21페이지 SVG 스크린샷 (전/후)
   - 회귀 테스트 결과 요약

**완료 조건**:
- `cargo test --lib` 통과
- `cargo test --test svg_snapshot` 통과 (또는 의도된 골든 갱신만 발생)
- 21페이지 PDF 일치 (시각 판정 통과)

**산출물**: `mydocs/working/task_m100_409_stage3.md`, `mydocs/report/task_m100_409_report.md`

---

## 위험 요소 및 대응

| 위험 | 영향 | 대응 |
|------|------|------|
| 가드 확장이 다른 페이지 페이지네이션 변경 | 기존 골든 SVG 실패 | stage 3에서 case-by-case 검토. 변경이 PDF에 더 가까워지는지 확인하여 골든 갱신 |
| Picture 내부에 caption 이 있는 경우 vpos 처리 | caption 영역도 vpos에 반영될 수 있음 | dump 로 vpos 차이 검증, 필요 시 가드 조건 세분화 |
| Square/Tight wrap 도 동일 문제 가능성 | 잠재 회귀 | 본 타스크는 TopAndBottom 만 다룸. Square/Tight 는 별도 이슈로 분리 |

## 커밋 분리

- Stage 1: `Task #409 Stage 1: 베이스라인 수집 + 가설 검증`
- Stage 2: `Task #409 Stage 2: prev_has_overlay_shape 가드 확장 (Picture + TopAndBottom/vert=Para)`
- Stage 3: `Task #409 Stage 3: 회귀 검증 통과 + 최종 보고서`
