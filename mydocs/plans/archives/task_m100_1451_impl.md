# Task M100 #1451 구현계획서 — HWPX serializer legacy 도형 shapeComment 직렬화

- 이슈: #1451
- 브랜치: `local/task1451`
- 작성일: 2026-06-21
- 수행계획서: `mydocs/plans/task_m100_1451.md`

## 구현 개요

`render_common_shape_xml` (`src/serializer/hwpx/section.rs:1141`) 의 caption 방출 직후,
`</hp:{tag}>` 닫기 전에 `shapeComment` 를 OWPML 순서대로 방출한다. 기존
`shape.rs:715 write_shape_comment` 는 `Writer<W>` 기반이라 String 빌더 경로와 맞지 않으므로,
`writer_to_string` 어댑터로 감싸 재사용한다(중복 로직 방지, 빈 description 규칙 일원화).

---

## 1단계 — shapeComment 방출 구현

**대상**: `src/serializer/hwpx/section.rs` `render_common_shape_xml`

- caption push 블록(`section.rs:1174-1179`) 직후, `out.push_str(&format!("</hp:{tag}>"))`
  (`section.rs:1180`) **직전**에 shapeComment 방출 추가.
- 구현 방식: 기존 `write_shape_comment` 재사용.
  ```rust
  // 설명 (#1451) — caption 직후 (OWPML AbstractShapeObjectType: outMargin→caption→shapeComment)
  // picture.rs:104 선례와 동일 순서. legacy 경로(ellipse/arc/polygon/curve/chart/ole) 보존.
  match writer_to_string(|w| super::shape::write_shape_comment(w, c)) {
      Ok(xml) => out.push_str(&xml),
      Err(e) => eprintln!("[hwpx] Shape({tag}) shapeComment 직렬화 실패: {e}"),
  }
  ```
- 빈 `description` 미방출은 `write_shape_comment` 내부 가드로 자동 보장 → 빈 경우 빈 문자열 반환.

**완료 기준**: 빌드 통과. `samples/table-vpos-01.hwpx` round-trip diff 2→0,
재직렬화 출력 shapeComment 3→5 (수동 측정).

## 2단계 — 회귀 검증 (게이트 + 보존 fixture)

- `cargo test --profile release-test --lib` serializer/roundtrip 모듈 통과 확인.
  특히 `task1392_*` 게이트 테스트군 통과.
- 보존 fixture 회귀 없음 확인 (`aift.hwpx` / `tac-img-02.hwpx` / `business_overview.hwpx` diff 0 유지).
- `cargo test --test hwpx_roundtrip_baseline` 회귀 게이트 통과.
- 필요 시 legacy 도형 shapeComment 보존을 직접 가드하는 단위 테스트 1건 추가
  (Polygon description round-trip → diff 0). 기존 게이트가 Ellipse loss 검출만 하므로
  "보존 성공" 방향 가드를 보강한다.

**완료 기준**: 위 게이트 전부 통과 + 신규 보존 테스트 통과.

## 3단계 — 최종 검증 + 보고서

- `cargo test --profile release-test --tests` 전체 통과.
- `cargo fmt --check` (수정 파일 한정 diff 확인) + `cargo clippy` 0 warning.
- 최종 보고서 작성 (`mydocs/report/task_m100_1451_report.md`) — 수치표(전/후), 제보자 크레딧.
- 오늘할일(`mydocs/orders/20260621.md`) #1451 상태 갱신.

**완료 기준**: 전체 CI급 검증 통과 + 보고서/오늘할일 커밋.

---

## 변경 파일 예상

| 파일 | 변경 |
|---|---|
| `src/serializer/hwpx/section.rs` | `render_common_shape_xml` shapeComment 방출 추가 (~5줄) |
| `src/serializer/hwpx/roundtrip.rs` (테스트) | (선택) Polygon 보존 가드 1건 |
| `mydocs/working/task_m100_1451_stage{N}.md` | 단계별 보고서 |
| `mydocs/report/task_m100_1451_report.md` | 최종 보고서 |

## 위험 / 주의

- OWPML 순서 위반 시 한컴 재적재 실패 가능 → picture.rs:104 선례(aift 실물 9건)와 동일 순서 엄수.
- `writer_to_string` 가 section.rs 내 caption 경로에서 이미 사용 중 → import/가시성 추가 불필요(확인).
- shape.rs `write_shape_comment` 는 `pub(super)` → 동일 크레이트 모듈에서 호출 가능(확인).
