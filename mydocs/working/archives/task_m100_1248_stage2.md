# Stage 2 완료 보고서 — Task #1248 특례 해부 + 불일치 지점

- **이슈**: edwardkim/rhwp#1248
- **단계**: Stage 2 / 3
- **산출물**: 조사문서 §2(특례 해부), §3(불일치 지점)
- **코드 변경**: 없음

## 한 일

1. vpos_adjust 특례 **8종 + 공통 게이트 3종**을 트리거 조건·존재 이유·핀 고정 테스트로 매핑 (§2)
2. render gap ≠ typeset/pagination 예약의 **정확한 불일치 경로** 규명 (§3)

## 검증 (인용 전 실제 실행)

- `cargo test --lib height_cursor` → **26 passed, 0 failed**
- `cargo test --test issue_1082_endnote_multicolumn_drift` → **4 passed** (3-09/3-11 실샘플)

## 핵심 발견

1. **특례 폭증의 진원지는 단 하나의 질문**: "미주 제목 앞 gap 을 얼마로?" — 8종 중 6종이
   `compact_endnote_question_title` 조건 의존. 답이 (컬럼 위치·prev 성격·vpos 방향)마다 달라 분기화.

2. **forward 계열(S2/S3/S8) ↔ backward 계열(S5/S6/S7) 대칭**: 저장 vpos gap 의 과소/과대
   인코딩을 양방향으로 메움. 근본은 "저장 vpos gap ≠ 실제 목표 gap".

3. **불일치는 typeset 가정 ↔ render 재구성 사이** (§3.2):
   - typeset: base-flow 1984HU 가 연속 vpos 에 **이미 있다**고 가정 (`typeset.rs:2213/2219`)
   - render: vpos 연속이면 **이미 포함**으로 처리 (`height_cursor.rs:163` trailing_ls_hu=0)
   - **단일줄 prev = 일치 / 다줄 prev = 어긋남** → #1246 gap≈0. S8(min-gap)이 `prev_is_multiline`
     한정인 이유가 이 비대칭.

4. **pagination 은 typeset 편에 정렬** (§3.3): IR 굽힌 trailing 신뢰 + vpos_offset 예약.
   PR #1247 이 별도 overflow 수정 없이 pi=475 해소된 이유 설명됨.

## 핀 고정 테스트 인벤토리 (회귀 위험 정량화)

- height_cursor 단위 26건 (특례별 직접 핀)
- 통합: issue_1082(미주 drift 4샘플), issue_1139(문27 인라인 그림)
- → trailing 모델 통일 시 **최소 30+ 테스트가 현재 동작에 핀 고정**. 단순 통일 = 다수 회귀 위험.

## 다음 단계 (Stage 3)

- §4 판정: 통일 가능 영역 / 게이트 필수(통일 불가) 영역 구분
- 후속 이슈 제안 + 최종 결과보고서

## 승인 요청

Stage 2 승인 후 Stage 3(판정 + 최종 보고) 착수.
