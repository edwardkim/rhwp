# Task M100 #1388 — 3단계 완료 보고서 (게이트 동승)

- 브랜치: `local/task1388`
- 작성일: 2026-06-12
- 수정 파일: `src/serializer/hwpx/roundtrip.rs` (`SectionPageDef` 동승 + 테스트 3종)

## 1. 구현 내용

### 1.1 variant + Display (3.1)

- `IrDifference::SectionPageDef { section, detail }` 추가.
- Display: `section[{i}] page_def: {detail}` — detail 은 불일치 필드별
  `field: expected=.. actual=..` 세미콜론 연결.

### 1.2 비교 동승 (3.2)

- `diff_page_def(a, b) -> Option<String>` 헬퍼 신설: width/height + margin 7필드 +
  landscape + binding 비교.
- `diff_documents` 섹션 대응 루프(ParagraphCount 직후)에서 호출 — `roundtrip_ir_diff`,
  `tests/hwpx_roundtrip_baseline.rs`, 배치 IR_DIFF 판정 3곳 자동 동승 (#1380 패턴).
- 비교 제외 필드와 사유를 doc 주석에 명기:
  `attr`(binding/landscape 와 의미 중복 — 해석 필드 쪽을 비교),
  `pagination_bottom_tolerance`(렌더러 내부 허용치 — 파일 포맷 필드 아님).

## 2. 단위 테스트 (3.3)

| 테스트 | 검증 |
|--------|------|
| `task1388_page_def_in_gate` | 여백·제본 차이 주입(온새미로 실측값) → `SectionPageDef` 검출 + detail 문자열 고정 |
| `task1388_page_def_equal_is_empty` | 동일 PageDef → 차이 0 (attr/tolerance 제외 확인 겸용) |
| `task1388_roundtrip_preserves_page_def` | 비템플릿 여백 실샘플(ta-pic-001-r) parse→serialize→재parse → 게이트 0 |

`cargo test --lib serializer::hwpx::roundtrip` — **27 passed** (기존 24 + 신규 3).

## 3. baseline 전수 (게이트 동승 후)

`cargo test --test hwpx_roundtrip_baseline` — **4 passed, 0 failed** (35.3s).

- 신규 xfail **0** — 1단계 사전 판정(4절) 적중. 기존 xfail(#1382 1건, #1384 4건)·제외
  (hwpx-01) 변동 없음.
- `cargo fmt --check` 통과.

## 4. 다음 단계

4단계 — 전수 검증(배치/페이지 수 대조/SVG) + 매뉴얼·최종 보고서 + CI급 검증.

승인 요청드립니다.
