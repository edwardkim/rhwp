# 단계 완료 보고 — Task M100 #2001 2단계: 추출 1 (GSO/개체 catch-all arm)

- 작성일: 2026-07-06 / 브랜치: `local/task2001`

## 수행 내용

`parse_paragraph_list`의 컨트롤 코드 `match ch` catch-all(`_`) arm(GSO/개체: 표·글상자·
수식·버튼, 1,116줄)을 `parse_object_control_char`로 추출 (동작 불변, 코드 이동).

- **`Hwp3CharScan` 공유 상태 struct** 도입 (5필드: text_string/char_offsets/
  hwp3_char_to_utf16_pos/controls/ctrl_data_records) — 함수 진입에서 동명 destructure로
  본문 무변경. **스칼라 2종(i, utf16_len)은 값 전달 + 반환**으로 처리 — 구현계획서의
  7필드 안 대비 축소(본문 deref 재작성 위험 회피, arm 내부 `for i` shadow 2곳과의
  간섭 원천 차단).
- **제어 흐름 치환(기계적, 전수 분류 근거)**: 중첩 추적 스캔으로 arm 내 break 17곳이
  전부 문자 루프 대상(내부 루프 대상 0)임을 확인 → `return Ok((i, utf16_len, true))`로
  치환, caller가 반환값으로 break. continue 2곳은 내부 루프 대상이라 무변경. `?` 6곳은
  동일 에러 타입으로 전파.
- byteorder import를 함수 스코프에 추가(원함수도 함수-내부 use 관례).

## 게이트 결과 (전수 통과)

| 게이트 | 결과 |
|---|---|
| cargo fmt --check | 통과 |
| cargo clippy --profile release-test --all-targets | **경고 0** |
| cargo test --profile release-test --tests | **2,912 통과 / 실패 0** |
| hwp5_roundtrip_baseline (HWP3 변환 경유) | 3/3 |
| OVR baseline 5샘플 | **추가 변동 0** — rowbreak-problem-pages의 기지 3건(#1936발, 시각 판정 대기)과 시그니처 동일, 나머지 4샘플 회귀 0 |

## 계측

| 함수 | 이전 | 이후 |
|---|---|---|
| `parse_paragraph_list` | 2,270줄 · 분기 지표 459 | **1,183줄 · 245** (−48%) |
| `parse_object_control_char` (신규) | — | 1,039줄 · 215 |

## 다음 단계

3단계 — 추출 2: 소형 arm 10개(~330줄) 헬퍼화, `Hwp3CharScan` 재사용. 승인 후 착수.
