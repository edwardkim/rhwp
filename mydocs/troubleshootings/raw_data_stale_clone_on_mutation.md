---
kind: reference
status: active
canonical: mydocs/troubleshootings/README.md
last_verified: 2026-07-19
---

# raw_data 를 문 clone 뮤테이션 — 화면엔 보이고 저장엔 사라지는 3중 구조 (PR #2416/#2421)

## 증상

편집 결과가 **편집기 화면에는 정상 반영**되는데, 저장 후 다시 열면 되돌아가
있다. 실측 사례 2건 (kevin9327, 2026-07-19):

- HTML 붙여넣기 서식(굵기·색·크기·글꼴 등)이 저장 시 전부 기본 서식으로 회귀
  (PR #2416, html_import 의 `css_to_char_shape_id`/`css_to_para_shape_id`)
- 표 이웃 셀의 공유 변 테두리 갱신이 저장 시 옛 테두리로 회귀
  (PR #2421, table_ops 의 `update_neighbor_borders`)

## 3중 구조 (이 계열이 잘 숨는 이유)

1. **clone 이 raw_data 를 물고 온다** — 파싱된 DocInfo 아이템(CharShape/
   ParaShape/BorderFill 등)은 원본 레코드 바이트를 `raw_data` 로 보존한다
   (라운드트립 계약, `hwpx2hwp-rule.md` 참조). 기존 아이템을 clone 해 필드를
   고치면 stale 바이트가 따라온다.
2. **직렬화기는 raw_data 우선** — `serializer/doc_info.rs` 는
   `raw_data.clone().unwrap_or_else(|| serialize_*(item))` 패턴이라, raw_data
   가 있으면 수정된 필드는 아예 직렬화되지 않는다.
3. **PartialEq/비교 헬퍼가 raw_data 를 제외** — 중복 아이템 검색이 stale
   clone 을 걸러내지 못해 "필드만 같은" 새 레코드가 push 되고, 파일에는
   원본과 바이트 동일한 레코드가 늘어난다.

렌더는 IR 필드를 읽으므로 화면 검증으로는 절대 잡히지 않는다 — **자기 검증
≠ 저장 검증**의 전형이다.

## 관례 (정정 방법)

**기존 아이템을 clone 해 필드를 변경하는 즉시 `raw_data = None`** 으로
무효화한다. 원본 미변경 아이템은 raw_data 를 유지한다 (라운드트립 보존
계약과 충돌하지 않는 경계 — "변경된 clone 만" 무효화).

선행 관례: `html_table_import.rs` 의 `create_border_fill_from_json` 이 이미
이 형태다. 신규 코드는 이 관례를 따르고, 리뷰에서는 "DocInfo 아이템 clone
후 필드 대입" 패턴이 보이면 raw_data 무효화 여부를 확인한다.

## 점검 방법

- 의심 지점 탐색: `grep -n "\.clone()" src/document_core/commands/*.rs` 후
  DocInfo 아이템 계열에서 필드 대입이 이어지는 곳
- 확증: 편집 → 직렬화 → 재파싱 → 필드 비교 통합 테스트 (PR #2416/#2421 의
  테스트가 견본 — raw_data 를 채운 아이템으로 시작해 저장 결과를 검증)

관련: PR #2416 · PR #2421 · `hwpx2hwp-rule.md`(raw_data 보존 계약)
