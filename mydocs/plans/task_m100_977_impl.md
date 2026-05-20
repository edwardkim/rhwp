# 구현 계획서 — Skia replay 경로 개요번호 x 우측 밀림 (M100 #977)

- 이슈: edwardkim/rhwp#977
- 브랜치: `local/task977`
- 수행 계획서: `task_m100_977.md` (승인 완료)

## 개요

Skia replay 경로에서, 선두 공백 CharShape가 본문과 다른(다른 폰트 + 장평 95%) 목차 문단의 개요번호가 ~9~10px 우측으로 밀린다. SVG 경로는 정상. 분기점이 render tree 구성인지 Skia 글리프 배치인지 미확정 상태이므로, 1단계에서 진단으로 확정한 뒤 정정한다.

---

## 단계 1 — 진단 및 분기점 확정

**목표**: `bbox.x` 오류(render tree 구성)인지, Skia 글리프 배치 오류(`text_replay.rs`)인지 확정.

- 어긋난 문단 / 정상 문단의 render tree TextRun을 식별
- 각 TextRun의 `bbox.x`, 적용 `style`(폰트 인덱스·장평·font_size), `char_positions`를 임시 디버그 출력으로 덤프
  - 디버그 출력은 환경변수/디버그 플래그 가드로 추가, 정상 경로 영향 없음
- `svg.rs`의 동일 문단 출력 x좌표와 대조
- 판정:
  - `bbox.x`가 SVG와 Skia에서 다름 → render tree 구성 단계 버그
  - `bbox.x`는 동일하나 Skia 그리기 결과만 밀림 → `text_replay.rs` 글리프 배치 버그
- **소스 정정은 하지 않는다.** 진단 코드만 추가.
- 산출물: `task_m100_977_stage1.md` — 분기점 확정 결과 + 2단계 정정 방향

## 단계 2 — 정정 구현

**목표**: 확정된 분기점을 정정해 Skia replay의 후속 런 x를 SVG와 일치시킨다.

- 1단계 확정 결과에 따라:
  - render tree 버그 → 런 원점 산출 로직 정정
  - 글리프 배치 버그 → `text_replay.rs`의 장평·폰트 적용 정정
- 1단계의 임시 진단 코드 제거
- 정정 범위는 원인 지점으로 한정 (다른 장평/폰트 조합 회귀 방지)
- 네이티브 검증: `cargo build`, `cargo test`, `cargo clippy` 통과 / `export-svg` 출력 무회귀
- 산출물: `task_m100_977_stage2.md`

## 단계 3 — WASM 빌드 및 회귀 검증

**목표**: Skia replay 경로 실측 정렬 회복 확인 + 광범위 회귀 없음.

- Docker로 WASM 빌드 (`pkg/`)
- rhwp-studio에서 대상 페이지 시각 검증 — 개요번호 정렬 회복, 픽셀 측정으로 SVG와 일치 확인
- 장평·다폰트 혼용 샘플 다수로 회귀 점검 (SVG ↔ Skia 양 경로)
- 산출물: `task_m100_977_stage3.md`, 최종 보고서 `task_m100_977_report.md` (`report/`)

---

## 검증 기준

- 어긋난 개요번호가 형제 항목과 정렬 회복
- SVG 경로와 Skia replay 경로 출력 좌표 일치
- `cargo test` / `cargo clippy` 통과, 기존 레이아웃 테스트 무회귀
- 장평≠100% 또는 다폰트 혼용 문단에서 회귀 없음

## 승인 요청

본 구현 계획서 검토 후 승인을 요청합니다. 승인 시 단계 1(진단)부터 진행합니다.
