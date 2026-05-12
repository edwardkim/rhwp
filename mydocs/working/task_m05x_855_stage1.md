# Stage 1 완료 보고서 — Task #855

## 작업 내용

어울림(Square wrap) 표 옆 문단 흡수 판정 보정.

### 계획 대비 변경점

구현 계획서에는 수정 대상을 `src/renderer/pagination/engine.rs` 로 적었으나, 실제 활성 페이지네이션 경로는 `TypesetEngine`(`src/renderer/typeset.rs`) 임을 조사 중 확인했다 (`RHWP_USE_PAGINATOR=1` 일 때만 `engine.rs` 의 `paginate_with_measured_opts` 가 fallback 으로 사용됨; 기본값은 `TypesetEngine::typeset_section`). 따라서 수정은 `src/renderer/typeset.rs` 에 적용했다. `engine.rs` 의 동일 로직(주석에 "engine.rs:288-320 동일 시멘틱" 명시)은 fallback 경로이며 본 타스크 범위에서는 손대지 않았다 (후속 정합 항목으로 보고서에 기록).

## 수정 (`src/renderer/typeset.rs`)

어울림 표 anchor 의 wrap zone (cs, sw) 과 후속 문단 매칭 시, Table anchor 흡수 분기(`current_column_wrap_around_paras` 등록 + `continue`)에서:

- 기존: 후속 문단의 **첫 LINE_SEG** 만 wrap zone 과 일치하면 문단 전체를 0-높이로 흡수 → 첫 줄만 표 옆이고 나머지 줄이 본문 전체 폭으로 흐르는 문단(`pi=300`)이 통째로 페이지 흐름에서 누락.
- 수정: **마지막 LINE_SEG 도** wrap zone (cs, sw) 과 일치할 때(또는 빈 문단일 때)만 흡수. 불일치 시 wrap zone 을 종료하고 일반 텍스트 배치로 폴백 — LINE_SEG 의 cs/sw 가 이미 wrap 형상(첫 줄 들여쓰기, 나머지 전폭)을 인코딩하므로 layout 이 첫 줄을 표 옆에, 나머지를 표 아래에 정상 렌더한다.

Picture anchor 분기(`wrap_anchors` 등록 + FullParagraph 통과)는 변경하지 않았다.

## 검증 (1차)

- `cargo build --release` 성공.
- `rhwp dump-pages samples/21_언어_기출_편집가능본.hwp -p 13`:
  - 수정 전: `단 1` items=8, `pi=300` 누락, diff=-300.3px
  - 수정 후: `단 1` items=10, `pi=300` (`FullParagraph h=180.2`) 정상 등장, diff=-15.1px

## 다음 단계

Stage 2 — SVG/PDF 시각 검증.
