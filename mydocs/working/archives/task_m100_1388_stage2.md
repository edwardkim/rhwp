# Task M100 #1388 — 2단계 완료 보고서 (serializer 수정)

- 브랜치: `local/task1388`
- 작성일: 2026-06-12
- 수정 파일: `src/serializer/hwpx/section.rs` (`replace_page_pr` 확장 + 테스트 4종)

## 1. 구현 내용

### 1.1 margin 7필드 치환 (2.1)

`replace_page_pr` 내부에 `TEMPLATE_PAGE_MARGIN` 고정 문자열 anchor를 추가하고, IR
`PageDef`의 header/footer/gutter/left/right/top/bottom 7필드로 `replacen(..., 1)` 치환.
미스매치 시 원본 유지(silent no-op) 정책은 #1166과 동일.

### 1.2 gutterType 치환 (2.2)

pagePr 여는 태그의 `gutterType="LEFT_ONLY"` 고정값을 IR `binding` 역매핑으로 동적화:
`SingleSided→LEFT_ONLY` / `DuplexSided→LEFT_RIGHT` / `TopFlip→TOP_BOTTOM`
(parser `parse_page_pr` 매핑의 역방향 — 추정 없음).

## 2. 단위 테스트 (2.3)

| 테스트 | 검증 |
|--------|------|
| `task1388_page_margin_reflects_page_def` | 7필드 비기본값(온새미로 sec0 실측값) 방출 + 템플릿 값 잔존 없음 + #1166 width/height 회귀 없음 |
| `task1388_gutter_type_reflects_binding` | binding 3종 → gutterType 매핑 |
| `task1388_template_mismatch_keeps_original` | anchor 부재 입력 → 원본 유지 |
| `task1388_template_anchor_present_in_template` | `EMPTY_SECTION_XML`의 anchor 존재 직접 보장 — 템플릿 변경 시 silent no-op 으로 빠지지 않도록 즉시 실패 |

`cargo test --lib serializer::hwpx` — **159 passed, 0 failed** (기존 155 + 신규 4).
`cargo fmt --check` 통과.

## 3. 전수 배치 재실행 (2.4)

`rhwp hwpx-roundtrip --batch samples/hwpx -o output/poc/task1388`

| 항목 | 수정 전 (1단계 실측) | 수정 후 |
|------|---------------------|--------|
| margin 변형 | 27/45 파일 (sec0 기준) | **0/63 섹션 (전 섹션)** |
| gutterType 변형 | 1건 (온새미로) | **0건** |
| 배치 요약 | PASS 48 / IR_DIFF 1 / SERIALIZE_FAIL 4 / PARSE_FAIL 1 | **동일** (신규 실패 0, xfail #1382·#1384·제외 hwpx-01 기지 정합) |

다중 섹션 파일(보도자료 5섹션, 온새미로 5섹션 등) 포함 전 섹션 대조에서 변형 0건 —
섹션별 `page_def` 전달 경로(0.1 확정)가 실측으로 입증됨.

## 4. 다음 단계

3단계 — `diff_documents`에 `SectionPageDef` 비교 동승 + 게이트 검출/실샘플 테스트.

승인 요청드립니다.
