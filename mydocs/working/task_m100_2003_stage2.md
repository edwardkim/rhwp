# 단계 완료 보고 — Task M100 #2003 2단계: 추출 1 (② GSO 개체 디스패치)

- 작성일: 2026-07-06 / 브랜치: `local/task2003`

## 수행 내용

`parse_object_control_char`의 개체 처리부를 `parse_hwp3_object_dispatch`로 추출 (동작 불변).

- **경계 정정(착수 중 발견)**: 계획의 "ch==10 블록 780줄"은 실제로는 **ch==10/11/14~17/29/
  5~8을 아우르는 if-else 체인 전체**였다(‘블록 1’ brace 매칭이 else-if 연쇄를 한 덩어리로
  측정). 체인을 분절하지 않고 **통째 이동** — 개체 디스패치라는 단일 책임 단위로 오히려
  정합. 후속(인터루드·tail 170줄)은 캐리 소비자로 caller 잔류.
- **`Hwp3DrawingCarry` struct** (9필드: nested_paragraphs/parsed_table/equation/picture/
  line/drawing_object/obj_type/is_hypertext/info_buf) — 디스패치가 채우고 후속이 소비하는
  캐리오버 묶음. 동명 destructure로 본문 이동, 직접 대입 11곳만 `**` deref.
- 조기 return 15곳: `Ok((i, utf16_len, X))` → `Ok(Some(X))` (i/utf16_len은 체인 내 대입 0
  실측 — 읽기 전용이라 반환 불요). caller가 `Some` 수신 시 그대로 전파.
- 컴파일러 검출 보정 2건: `header_val1` 파라미터(1차 스캔 누락), 이중 참조 deref
  (`read_exact`는 clippy 지적에 따라 `as_mut_slice()`로 정리 — 의미 동일, HWP3 경로
  표적 재테스트 통과).

## 게이트 결과 (전수 통과)

| 게이트 | 결과 |
|---|---|
| cargo fmt --check / clippy | 통과 / **경고 0** (deref 정리 후) |
| cargo test --profile release-test --tests | **2,912 통과 / 실패 0** |
| OVR baseline 5샘플 | **추가 변동 0** (기지 #1936발 3건 시그니처 동일) |

## 계측

| 함수 | 이전 | 이후 |
|---|---|---|
| `parse_object_control_char` | 1,039줄 · 분기 215 | **289줄 · 57** |
| `parse_hwp3_object_dispatch` (신규) | — | 812줄 · 159 |

## 다음 단계

3단계 — 추출 2: ① B 블록 (`RunEmitState` 도입 커밋 + 본 추출 커밋). 승인 후 착수.
