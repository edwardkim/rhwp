# PR #1213 검토 — 그림/표/도형 textFlow 속성 roundtrip 보존 (BOTH_SIDES 초기화 버그)

- **작성일**: 2026-06-01
- **PR**: #1213 (OPEN)
- **컨트리뷰터**: @Martinel2 (**rhwp 첫 기여**)
- **연결 이슈**: #1188 (`closes #1188`)
- **base/head**: `devel` ← `fix/issue-1188` (cross-repo)
- **규모**: 7 파일, +200/−21 (모두 소스 — parser/serializer/model/document_core)
- **mergeable**: **MERGEABLE / BEHIND**
- **CI**: **미실행** ("no checks reported") → 로컬 검증 필수 수행
- **라벨**: bug / enhancement / 마일스톤 v1.0.0

## 1. 문제 (코드 확인 완료)

그림/표/도형의 `textFlow`(텍스트가 개체 좌우 중 어느 쪽으로 흐르는지) 속성이 roundtrip 시
항상 `BOTH_SIDES` 로 초기화됨:
- HWPX 직렬화(`serializer/hwpx/{shape,picture,table}.rs`)가 `("textFlow","BOTH_SIDES")` **하드코딩**.
- HWP5 바이너리 파싱(`parser/control/shape.rs`)이 attr **bit 24-25 미파싱**.
- 모델 `CommonObjAttr` 에 text_flow 필드 부재.

→ 원본이 LEFT_ONLY/RIGHT_ONLY/LARGEST_ONLY 여도 보존 안 됨.

## 2. 수정 내용 검토

| 파일 | 변경 |
|------|------|
| `model/shape.rs` | `TextFlow` enum 신규(BothSides/LeftOnly/RightOnly/LargestOnly, Default=BothSides) + `CommonObjAttr.text_flow` 필드 |
| `parser/control/shape.rs` | HWP5 attr `(attr>>24)&0x03` → TextFlow (0/1/2/3) |
| `parser/hwpx/section.rs` | HWPX `textFlow` 속성 파싱 (표/그림/도형/차트/OLE 5개 위치) |
| `serializer/hwpx/{shape,picture,table}.rs` | 하드코딩 제거 → 실제 text_flow 출력 |
| `document_core/converters/common_obj_attr_writer.rs` | 비트 패킹 `<<24` + roundtrip 테스트 4건 |

설계 평가:
- 파싱 `(attr>>24)&0x03` ↔ 직렬화 `<<24` **정확히 대칭**. 비트 위치/매핑 검증 테스트 포함.
- 기존 비트 레이아웃(devel)은 bit 0~23 사용(treat_as_char 0, …, text_wrap 21-23) — **bit 24-25 비어있음**.
  text_wrap(개체 배치, 21-23) 바로 다음에 text_flow(텍스트 방향, 24-25) 추가 → 충돌 없음.

## 3. 권위 자료 교차검증 (한컴 스펙 — feedback_no_inference_authoritative_spec)

hwp2hwpx (`ForShapeObject.java` + `TextHorzArrange.java`) 로 비트값/문자열 확정:
- `textFlow` ← `getTextHorzArrange()` (≠ textWrap ← getTextFlowMethod, 이름 혼동 주의).
- `TextHorzArrange`: **BothSides(0) / LeftOnly(1) / RightOnly(2) / LargestOnly(3)**.
- `TextFlowSide` 문자열: BOTH_SIDES / LEFT_ONLY / RIGHT_ONLY / LARGEST_ONLY.

→ PR 의 비트값(0/1/2/3)·HWPX 문자열 매핑이 **권위 자료와 정확히 일치**. (추정 아님.)

## 4. 검증 결과 (로컬 머지 시뮬 `pr1213-verify`)

| 단계 | 결과 |
|------|------|
| merge | ✅ CLEAN (section.rs 자동 머지, 충돌 0) |
| fmt | ✅ clean |
| build | ✅ Finished |
| **clippy `--lib`** | ✅ **clean** (PR 코드 무경고) |
| clippy examples(inspect_574 등) | ⚠️ 에러 — **devel 선존재 기존 이슈**(char_shapes.get(0) 등), PR 무관 |
| 전체 테스트 `cargo test --tests` | ✅ **1916 passed, 0 failed** (textFlow 9건 추가) |
| textFlow roundtrip 테스트 | ✅ left/right/largest + bit position + default |
| **실샘플 roundtrip** | ✅ `hwp3-sample19-hwpx.hwpx` `LARGEST_ONLY` → IR `LargestOnly` 파싱 확인(devel 은 BothSides 로 소실) |

## 5. 판단 — 머지 권고 (시각 판정 게이트)

- 진단 정확, 파싱↔직렬화 대칭, 권위 자료(hwp2hwpx)로 비트값 확정, 비트 충돌 없음, 회귀 0.
- 첫 기여자 PR 이나 코드 품질 견고(테스트 9건, 비트 검증).
- **CI 미실행** → 로컬 fmt/build/clippy(lib)/test 1916 직접 검증 완료.
- HWPX 직렬화 변경이므로 자체 검증 ≠ 한컴 호환(`feedback_self_verification_not_hancom`):
  textFlow 가 시각에 영향(개체 좌우 텍스트 흐름)하므로 **작업지시자 시각 판정** 게이트 권고
  (textFlow≠BOTH_SIDES 개체의 렌더 + 가능 시 한컴 비교).
- 승인 + 시각 판정 통과 시 메인테이너 로컬 `--no-ff` 머지 + push. **첫 기여 → 따뜻한 환영**.

## 6. 시각 영향 — 없음 (무회귀 실증)

렌더러/레이아웃은 `text_flow` 를 **전혀 참조하지 않음**(grep 0 hit). 따라서 SVG 시각 변화 없음:
- `hwp3-sample19-hwpx.hwpx`(textFlow=LARGEST_ONLY rect 포함) 보정 전(devel)/후 SVG **byte 단위 완전 동일**(쪽 001·002).
- 즉 본 PR 은 기존 렌더 출력에 영향 0 = **무회귀 확정**. 변화는 IR 보존/직렬화 값뿐.

→ 판정축은 시각 대조가 아니라 **roundtrip 보존 + 권위 정합 + 무회귀**. 작업지시자 결정:
roundtrip+권위 검증으로 머지(시각 변화 없음이 정상).

## 7. 비고

- examples clippy 에러는 별개 정리 이슈(이 PR 범위 밖).
- textFlow 의 실제 좌우 흐름 렌더 반영은 본 PR 범위 밖(후속 가능). 현 단계는 "BOTH_SIDES 로
  소실되던 값"의 보존이 본질. 향후 렌더 반영 시 시각 판정 필요.
