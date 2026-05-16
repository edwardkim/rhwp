---
issue: 929
milestone: v1.0.0 (M100)
branch: local/task929
status: 구현 계획 — 승인 대기
parent: task_m100_929.md
---

# Task #929 구현 계획서

수행 계획서(`task_m100_929.md`) 승인 후 단계별 구현 계획을 확정한다.

## 사전 조사 요약 (사실)

`parse_hwp3` 진입 → `Hwp3DocInfo` (128B) → `Hwp3DocSummary` (1008B) → 정보 블록 → flate2 압축 해제 (옵션) → 글꼴/스타일 → **`parse_paragraph_list`** → 추가 정보 블록 → 부가 데이터(이미지 등).

`parse_paragraph_list` (`src/parser/hwp3/mod.rs:244`) 가 핵심 후보:

- 각 문단마다 `Hwp3ParaInfo::read(body_cursor)?` → `char_count==0` 이면 종료.
- 그 다음 `Hwp3LineInfo::read` × `line_count` → `?` 전파.
- `include_char_shape != 0` 이면 char 별 `Hwp3CharShape::read` → `?` 전파.
- 본문 char loop 에서 `read_u16` 의 ch 값에 따라 분기:
  - 특수 컨트롤 7|8, 18..=21, 24|25, 30|31: `read_exact` 가 EOF 면 `break` (흡수)
  - 그 외 컨트롤(`special_char.rs`, `drawing.rs`, `ole.rs`, `records.rs` 의 다른 record): `?` 전파

후보 EOF 지점은 **`?` 가 붙은 read 들**이다. `break` 흡수 지점은 용의자가 아니다.

## 단계 구성

### Stage 1 — 실패 지점 특정 (진단)

**목표**: 어떤 컨트롤·문단·바이트 오프셋에서 `read_exact` 가 EOF 를 만나는지 정확히 식별.

**작업**:
1. `parse_hwp3` 가 `Hwp3Error` 를 반환할 때 **오프셋·문맥** 을 함께 담을 수 있도록 임시 진단 출력 추가 (디버그 로그 또는 임시 `eprintln!`).
   - `parse_paragraph_list` 의 각 `?` 전파 지점 직전에 `body_cursor.position()` + 문맥 키 출력
   - 마지막 직전까지 진행한 `(para_idx, char_idx, ch, sub-record name)` 를 표시
2. `cargo run --release --bin rhwp -- dump samples/hwp3-sample19.hwp 2>&1 | head -40` 실행하여 출력 분석.
3. 실패 직전까지의 흐름을 `mydocs/working/task_m100_929_stage1.md` 에 기록 (마지막 성공 문단 인덱스, 마지막 read 위치, 의심 컨트롤·레코드 식별).
4. **진단 출력은 본 단계 종료 시 임시 커밋만 하고 Stage 3 마무리 시 제거** (디버깅 흔적 잔존 금지).

**산출물**: `mydocs/working/task_m100_929_stage1.md` (마지막 성공 위치 + 실패 위치 + 의심 가설).

**완료 기준**: 어느 read 호출이 EOF 를 만나는지 1행 명시 가능.

**예상 분량**: 코드 변경 ≈ 20–50 라인 (임시 진단 출력만).

---

### Stage 2 — 수정 설계 + 단위 테스트

**목표**: Stage 1 에서 식별된 결함의 수정 방향을 확정하고, 회귀를 막을 테스트를 작성.

**작업**:
1. Stage 1 의 가설에 따른 **수정 후보 분류**:
   - (a) 레코드 길이 필드 해석 오프바이원
   - (b) 미지원 컨트롤 (예: 특정 OLE 변형 / 도형 변형) — 해당 컨트롤만 안전하게 skip
   - (c) 파일 말미 트레일링 패딩 또는 종결 마커 누락 처리
   - (d) `include_char_shape` 또는 `line_count` 같은 헤더 필드 잘못 읽음
2. 수정 방향을 `mydocs/working/task_m100_929_stage2.md` 로 명시 + 코드 변경 범위·인터페이스 변경 여부 기술.
3. 회귀 가드 테스트 작성:
   - `tests/` 또는 `src/parser/hwp3/` 의 기존 통합 테스트 패턴 따름
   - hwp3-sample19 + 기존 hwp3-sample, sample10, sample11, sample13, sample14, sample16, sample17 (있다면) 의 `parse_hwp3` 성공 보장
   - 테스트는 **본 단계에서는 의도적으로 실패 상태**로 커밋 (Stage 3 의 코드 수정으로 통과)

**산출물**: `mydocs/working/task_m100_929_stage2.md` + 테스트 코드 (실패 상태).

**완료 기준**: 수정 방향 작업지시자 승인 + 테스트가 의도된 실패를 보여줌.

**예상 분량**: 테스트 ≈ 30–80 라인.

---

### Stage 3 — 구현 + 회귀 검증

**목표**: Stage 2 의 설계대로 코드 수정 + Stage 1 의 진단 출력 제거 + 전수 검증.

**작업**:
1. `src/parser/hwp3/` 내 결함 수정 (Stage 2 설계대로).
2. Stage 1 의 진단 출력 코드 제거 (또는 `log::debug!` 로 정리).
3. 검증:
   - `cargo test --release` 전체 통과
   - `cargo clippy --all-targets -- -D warnings` 통과
   - `rhwp dump samples/hwp3-sample19.hwp` 정상 출력
   - `rhwp export-svg samples/hwp3-sample19.hwp -o output/svg/task929/` 정상 SVG 생성
   - 기존 hwp3 샘플 (sample, sample10, sample11, sample13, sample14, sample16) `rhwp dump` 회귀 없음
   - `rhwp ir-diff samples/hwp3-sample19-hwpx.hwpx samples/hwp3-sample19.hwp --summary` 결과의 카테고리별 차이 검토 (큰 차이 없을 것)
4. `mydocs/working/task_m100_929_stage3.md` 에 검증 결과 첨부.

**산출물**: 코드 수정 + `task_m100_929_stage3.md` (테스트·dump·diff 결과 캡처).

**완료 기준**:
- hwp3-sample19 파싱 성공
- 모든 검증 명령 통과
- `src/parser/hwp3/` 외 공통 모듈 무변경

**예상 분량**: 코드 변경 ≈ 10–100 라인 (결함 크기에 따라).

---

### Stage 4 (조건부) — 시각 검증

**조건**: Stage 3 까지 끝나도 SVG 출력이 hwp3-sample19-hwp5 / pdf 권위 자료와 시각적으로 명백히 다를 경우 진입.

**목표**: HWP3 원본의 SVG 가 HWP5 변환본·PDF 와 핵심 시각 정합.

**작업**:
1. `rhwp export-svg samples/hwp3-sample19.hwp` + `samples/hwp3-sample19-hwp5.hwp` 두 SVG 생성.
2. `pdf/hwp3-sample19-hwp5-2022.pdf` 와 SVG 셀프 비교 (rsvg-convert + 이미지 확인).
3. 차이가 있다면 Stage 3 추가 보완.
4. `mydocs/working/task_m100_929_stage4.md` 에 비교 결과 + 스크린샷 경로 기재.

**완료 기준**: 시각적 동등성 확보 (또는 잔존 차이가 task929 범위 밖임을 명시하고 별 이슈로 분리).

---

### Stage 5 — 최종 보고서

**작업**:
1. `mydocs/report/task_m100_929_report.md` 작성:
   - 원인 분석 (어떤 결함이었는지)
   - 수정 내용 요약
   - 검증 결과 (테스트·dump·시각)
   - 변경 파일 목록
   - 후속 과제 (있다면)
2. 오늘할일(`mydocs/orders/20260516.md`) 의 #929 상태 갱신 (없으면 추가).
3. 모든 미커밋 파일 `git status` 로 확인.

**완료 기준**: 최종 보고서 작성 완료 + 작업지시자 승인 → 이슈 클로즈 + `local/devel` 머지 절차로 이행.

---

## 단계 간 승인 게이트

CLAUDE.md 절차상 각 Stage 완료 후 단계별 완료보고서(`task_m100_929_stage{N}.md`) 작성 → 승인 → 다음 단계 진행.

## 리스크 & 완화

| 리스크 | 완화 |
|--------|------|
| Stage 1 진단으로도 EOF 위치를 못 잡음 (e.g. flate2 단계 실패) | 진단 범위를 `parse_paragraph_list` 외에도 확대 (압축 해제 직후 cursor 길이 출력) |
| 수정 후 다른 hwp3 샘플에서 회귀 | Stage 2 테스트에 기존 6+ 샘플 전부 포함 → 회귀 즉시 검출 |
| 한컴 사양 외 변형이라 정답 모름 | hwp3-sample19-hwp5/hwpx IR 을 권위 자료로 사용 (`ir-diff`) |
| 결함이 둘 이상 — 한 번 고치니 다음이 드러남 | Stage 3 검증 시 발견되면 task929 범위 안에서 계속 추적 (작업지시자 판단으로 별 이슈 분리 가능) |

---

## 승인 요청

본 구현 계획대로 진행해도 될지 확인 부탁드립니다.

특히:
- **Stage 단계 수 / 단계 경계**: 3 단계 기본 + Stage 4 조건부 + Stage 5 최종 보고서. 적절한지 검토 부탁드립니다.
- **Stage 1 의 진단 출력 임시 커밋 후 Stage 3 에서 제거** 방식 (또는 처음부터 별도 PR-out feature flag/log::debug 로 처리할지) 확인 부탁드립니다.

승인 후 Stage 1 진단부터 시작합니다.
