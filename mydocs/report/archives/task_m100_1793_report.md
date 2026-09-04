# 최종 결과 보고서 — Task M100 #1793

## 이슈

HWPX→HWP 라운드트립: 특수문자(하이픈/묶음빈칸) 렌더 문자 손상 — '-'→NUL, NBSP→'-' 오매핑

## 결론

두 개의 독립 버그를 각각 수정했다. 손상은 본문 텍스트가 아니라 **doc_info 직렬화 결함**
(글머리표)과 **본문 특수문자 코드 오기**(NBSP)였다. 이전 세션의 control_mask 차이 가설은
무관한 것으로 판명 (HWPX 로드 경로만 mask 를 재계산하는 기존 정상 동작).

### 원인 1 — '-'→NUL: BULLET 레코드 직렬화 레이아웃 비대칭

손상된 '-' 는 본문 문자가 아니라 **글머리표(Bullet) 문자**였다. admrul_0072 의 셀 문단
(ps_id=34, HeadType::Bullet)은 본문 text 에 '-' 가 없고 렌더 시 bullet[0].bullet_char 를
그린다. `diag` 확인: HWPX 원본 bullet[0]='-'(U+002D), HWP 저장본 bullet[0]=U+0000.

- `parse_bullet`(src/parser/doc_info.rs)은 문단 머리 정보 **12바이트**(attr 4 +
  width_adjust 2 + text_distance 2 + char_shape_id 4) 뒤 offset 12 에서 bullet_char 를
  읽는다. 한컴 원본 HWP5 코퍼스(big_hwp)에서 ○/-/PUA 문자가 올바로 파싱됨 — 파서가 정답.
- `serialize_bullet`(src/serializer/doc_info.rs)은 char_shape_id 4바이트를 **누락**하고
  offset 8 에 bullet_char 를 기록했다. 재파싱 시 bullet_char 자리에서 image_bullet
  하위 2바이트(0)를 읽어 '\0' 이 된다.
- HWP5→HWP5 는 raw_data 보존 경로여서 무증상. **HWPX→HWP 저장(raw_data=None)에서만 발현.**
- 이전 소거 조사에서 doc_info 는 char_shapes/font_faces 만 비교해 bullets 가 누락됐었다.

### 원인 2 — NBSP→'-': 본문 직렬화 코드 오기

`serialize_para_text`(src/serializer/body_text.rs)가 U+00A0(묶음 빈칸)을 코드
**24(0x18, 하이픈)** 로 기록 (초기 커밋부터의 오기). 파서 역매핑은 24→'-', 30(0x1E)→NBSP.
올바른 코드 **30(0x1E)** 으로 수정.

## 수정 내역

| 파일 | 수정 |
|------|------|
| `src/model/style.rs` | `Bullet.char_shape_id: u32` 필드 추가 (레코드 24바이트 정합) |
| `src/parser/doc_info.rs` | `parse_bullet` 이 char_shape_id 보존 (기존: 읽고 버림) |
| `src/serializer/doc_info.rs` | `serialize_bullet` 이 char_shape_id 4바이트 기록 |
| `src/serializer/body_text.rs` | NBSP → 0x001E (코드 30) |

회귀 테스트 2개 추가:
- `serializer::doc_info::tests::test_serialize_bullet_layout_and_roundtrip` — 레이아웃(offset 12) + doc_info 전체 라운드트립
- `serializer::body_text::tests::test_nbsp_serializes_as_code_30`

## 검증 결과

1. **cargo test --release 전수**: 2,053 lib + 통합 스위트 전부 통과. 유일 실패
   `form_01_keeps_nine_cfb_streams` 은 경로 구분자(`/BodyText/Section0` vs
   `\BodyText\Section0`) Windows 기대 실패 (#1775, 본 수정과 무관).
2. **admrul_0072** ('-'→NUL 재현물): HWPX→HWP 재변환 후 4페이지 export-text 가
   HWPX 원본과 **완전 일치**, NUL 0개 (수정 전: 페이지 2·3 에서 NUL 4개).
3. **seoul_0505** (NBSP→'-' 재현물): 재변환 후 2페이지 **완전 일치**
   (수정 전 변환본은 p1 에서 8자 차이).
4. **코퍼스 스윕**: sample_hwpx 300 중 bullet 보유 3건, big_hwpx 2,500 중 30건 —
   변환→재파싱 bullet 문자 **전부 보존, 불일치 0**.

## 후속

- #1795 (글상자 줄바꿈, seoul_0043): 본 수정 후 재검 예정 (같은 원인 가능성).
- #1794 (표 앵커 95px, seoul_0765): 미조사.
