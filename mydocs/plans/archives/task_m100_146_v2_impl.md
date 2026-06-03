# 구현계획서 v2: text-align.hwp SVG ↔ 한컴 PDF 렌더링 차이 보정

- **타스크**: [#146](https://github.com/edwardkim/rhwp/issues/146)
- **마일스톤**: M100
- **브랜치**: `local/task146`
- **작성일**: 2026-04-23
- **상위 문서**: `mydocs/plans/task_m100_146_v2.md`
- **이전 버전**: `task_m100_146_impl.md` (원본, 과대 범위)

## 0. 진입점 확정

- **수정 대상**: `src/renderer/layout/text_measurement.rs:845-858`
- **추가 범위**: `'\u{25A0}'..='\u{25FF}'` (Unicode Geometric Shapes Block, 96 글리프)
- **대표 글자**: ■(U+25A0), □(U+25A1), ▲(U+25B2), ◆(U+25C6), ○(U+25CB), ● (U+25CF), ◇(U+25C7) 등

## 1. 단계 (2단계)

### 단계2 — is_fullwidth_symbol 범위 추가 + 단위 테스트

**코드 변경**
- `text_measurement.rs:855-857` 기존 범위 리스트 끝에 다음 범위를 추가:
  ```rust
  || ('\u{25A0}'..='\u{25FF}').contains(&c) // Geometric Shapes (□■▲◆○ 등, 섹션 머리 기호)
  ```
- 기존 주석 순서에 맞춰 CJK Compatibility(U+3300-U+33FF) 앞 또는 Misc Symbols(U+2600-U+26FF) 앞에 삽입(주석 블록 일관성 유지).

**테스트 추가**
- `src/renderer/layout/tests.rs` (혹은 `text_measurement` 동일 파일 내 `#[cfg(test)] mod tests`) 에 신규 테스트:
  - `test_is_fullwidth_symbol_geometric_shapes`: □■▲◆○●◇△▽▼▶◀ 등 대표 6~8개 글자에 대해 `is_fullwidth_symbol(c) == true` 를 확인
  - `test_compute_char_positions_geometric_shape_bullet`: 
    - text = "□ 가", font_size = 20.0, letter_spacing = -1.6 (자간 -8%)
    - 기대: positions[2] (가의 x) ≈ 20 + (20 − 1.6) + ... (계산식 명시, 이전 버그와의 delta 10px 차이 보장)
- `is_fullwidth_symbol` 은 현재 `pub(crate)`/`pub` 가 아닌 가능성 높음 → 테스트 모듈이 같은 파일이거나 해당 함수 export 범위 확인 후 필요 시 `pub(super)` 로 제한적 노출.

**재현 검증**
- `cargo run --bin rhwp -- export-svg samples/text-align.hwp -o output/svg/text-align/`
- 생성된 SVG 에서 제목 "국" 의 x 좌표 grep → 약 105.39px 수준으로 수정되었는지 확인 (PDF 환산 좌표와 ±1px 일치)
- Chrome headless 150dpi PNG 재생성 → `output/compare/text-align/pdf-1.png` 와 나란히 시각 확인

**커밋**
- 커밋 1: 소스 수정 (`text_measurement.rs`) + 단위 테스트 (`tests.rs`)
- 커밋 2: 단계2 보고서 (`mydocs/working/task_m100_146_stage2.md`) + 수행/구현 계획서 v2

### 단계3 — 통합 검증 + 회귀 방지 + 보고서

**테스트 스위프**
- `cargo test --lib` 전체 통과 재확인
- `cargo test --test svg_snapshot` 실행, 실패 테스트 목록 수집
  - 실패가 없으면: 기존 스냅샷이 이번 범위 변경과 무관 또는 영향 샘플 내 Geometric Shapes 문자 없음 → 추가 작업 없음
  - 실패가 있으면: 각 실패 샘플에 대해 변경 전/후 좌표를 비교, "PDF 기준으로 가까워짐" 증빙 캡처 후 일괄 golden 재생성 (`scripts/` 내 재생성 루트 확인 후 실행)
- `cargo clippy -- -D warnings` 통과

**시각 회귀 스모크**
- `samples/` 내 Geometric Shapes 사용 가능성이 높은 문서 후보 3~5개 선정하여 150dpi 렌더 전후 비교
  - 후보 예시: `exam_kor.hwp`, `biz_plan.hwp`, `samples/basic/` 내 관목식 문서들

**결과보고서 작성**
- `mydocs/report/task_m100_146_report.md` 신규 작성
  - 문제 현상 → 원인 규명 (is_fullwidth_symbol 범위 누락) → 수정 → 전후 좌표 / 이미지 대비 요약
  - svg_snapshot golden 변경 있으면 샘플 목록 + 사유
  - 좌표 기반 정량 검증 테이블 (PDF pt vs SVG pt 변환 일치)

**orders 갱신**
- `mydocs/orders/20260423.md` 를 v2 스코프로 갱신 (5단계 → 3단계 체크리스트)

**커밋**
- 커밋 1: (필요 시) svg_snapshot golden 업데이트
- 커밋 2: 최종 보고서 + orders 갱신

## 2. 롤백 전략

- 단 한 줄의 범위 추가이므로 롤백이 간단하다. 회귀 발견 시 `text_measurement.rs:855` 근처 추가 줄 제거 + 테스트 제거로 되돌린다.

## 3. 산출물 체크리스트

- [ ] `src/renderer/layout/text_measurement.rs` Geometric Shapes 범위 추가
- [ ] `src/renderer/layout/tests.rs` 신규 유닛 테스트 2건
- [ ] `mydocs/working/task_m100_146_stage2.md`
- [ ] `mydocs/report/task_m100_146_report.md`
- [ ] svg_snapshot golden (영향 시)
- [ ] orders/20260423.md 갱신
