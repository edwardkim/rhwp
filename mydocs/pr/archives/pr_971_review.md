# PR #971 검토 — HWP3 margin_bottom 원본값 보존

## 1. PR 정보

| 항목 | 값 |
|------|-----|
| 번호 | #971 |
| 제목 | fix: HWP3 margin_bottom 원본값 보존 — 쪽 테두리/페이지 번호 위치 정상화 |
| 작성자 | oksure (Hyunwoo Park) — 외부 컨트리뷰터 |
| base ← head | `devel` ← `contrib/fix-hwp3-margin-bottom` |
| 연결 이슈 | closes #951 |
| mergeable | MERGEABLE (mergeStateStatus: BEHIND — devel 뒤처짐, 충돌 아님) |
| CI | Build & Test ✅ / CodeQL ✅ / 기타 SUCCESS |

## 2. 배경 (이슈 #951)

`src/parser/hwp3/mod.rs`에서 HWP3 last-line tolerance를 구현하려고
`margin_bottom`을 `saturating_sub(1600)`으로 직접 감산.

- 부작용: 용지 하단 여백 값 자체가 변조됨
  - 쪽 테두리가 한컴 대비 ~5.6mm 위로 이동
  - 페이지 번호 위치 오류 (rhwp 4.4mm vs 한컴 10.0mm 하단 여백)
- 발견 경위: 이슈 #920 쪽 테두리 수정 시각 판정 중 작업지시자 발견

## 3. 변경 내용

변경 파일 5개:

| 파일 | 변경 |
|------|------|
| `src/model/page.rs` | `PageDef.pagination_bottom_tolerance: HwpUnit` 필드 추가 (기본 0) |
| `src/parser/hwp3/mod.rs` | `margin_bottom` 원본값 복원, tolerance를 `1600u32.min(bottom*4)`로 분리 |
| `src/renderer/page_layout.rs` | `PageLayoutInfo.pagination_tolerance_px` + `available_body_height()`에 가산 |
| `src/renderer/page_number.rs` | 테스트 fixture 필드 추가 |
| `src/renderer/layout/integration_tests.rs` | 회귀 테스트 y좌표 1061.4→1050.8 갱신 |

## 4. 검토 의견

### 설계 — 적절

- `margin_bottom`을 변조하던 책임을 신규 필드 `pagination_bottom_tolerance`로 분리.
  용지 여백 의미와 페이지네이션 허용치를 명확히 구분 — 이슈 #951의 수정 방향과 일치.
- `1600u32.min(margin_bottom*4)` clamp는 기존 `saturating_sub(1600)`의
  언더플로 상한 동작과 수학적으로 동치 — 회귀 위험 없이 상한 보존.
- tolerance를 `available_body_height()`에만 반영 → paginator에만 영향,
  `body_area`(쪽 테두리/페이지번호 기준)는 불변. 부작용 원인을 정확히 차단.

### 프로젝트 규칙 정합

- CLAUDE.md "HWP3 전용 로직은 `src/parser/hwp3/` 안에서만" 준수:
  공통 모듈(`page.rs`/`page_layout.rs`)에는 포맷 중립 필드만 추가,
  HWP3 고유값(1600) 주입은 파서 내부에 한정.

### 회귀 테스트

- `integration_tests.rs`의 쪽번호 y좌표 갱신(1061.4→1050.8)은
  margin_bottom 복원에 따른 정상 보정 결과와 일관.

## 5. 검증 항목

- [ ] `cargo test` 전체 통과
- [ ] `cargo clippy -- -D warnings` 0 warnings
- [x] 시각 판정 (작업지시자) — **통과**
- [ ] 빌드 산출물 확인

## 6. 판단

설계·구현·프로젝트 규칙 모두 적절. 시각 판정 통과.
검증(test/clippy) 완료 후 merge 권고. 최종 판단은 `pr_971_report.md`.
