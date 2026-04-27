# PR #366 검토 — Task #353: 쪽번호 처리 NewNumber 매 페이지 재적용 회귀 수정

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#366](https://github.com/edwardkim/rhwp/pull/366) |
| 작성자 | [@planet6897](https://github.com/planet6897) (Jaeuk Ryu) |
| base / head | `devel` ← `task353` |
| state | OPEN |
| mergeable | MERGEABLE |
| mergeStateStatus | **BEHIND** (devel 보다 뒤) |
| 이슈 | [#353](https://github.com/edwardkim/rhwp/issues/353) (NewNumber 컨트롤 매 페이지 재적용 결함) |
| 변경 통계 | +826 / -38, 14 files |

## 결함 요약

`samples/2022년 국립국어원 업무계획.hwp` 본문 모든 페이지가 `page_num=1` 로 고정.

**근본 원인** (`typeset.rs::finalize_pages`):
```rust
for &(nn_pi, nn_num) in new_page_numbers {
    if nn_pi <= fp {           // 매 페이지 매번 참 → 매번 리셋
        page_num = nn_num as u32;
    }
}
```

NewNumber 가 한 번만 적용되어야 하지만 `nn_pi <= fp` 조건이 모든 후속 페이지에서 참이라 매 페이지마다 page_num 재설정.

## **중복 처리 — Task #361 (이미 v0.7.7 에 적용)**

본 결함은 이미 작업지시자 + Claude 가 **Task #361 (PR #366 과 동시기)** 으로 정정하여 v0.7.7 (2026-04-27 배포) 에 포함됨.

| 항목 | PR #366 (planet6897) | Task #361 (이미 머지됨) |
|------|--------------------|----------------------|
| 이슈 | #353 (OPEN) | #361 (CLOSED) |
| 결함 | NewNumber 매 페이지 재적용 | **동일 결함** |
| typeset.rs 정정 | ✅ Assigner 호출로 치환 | ✅ `prev_page_last_para` 추적으로 인라인 정정 |
| pagination/engine.rs 정정 | ✅ 동일 Assigner 적용 | ❌ 손대지 않음 (Paginator 경로는 원래 코드 유지) |
| 신규 모듈 | ✅ `src/renderer/page_number.rs` (PageNumberAssigner) | ❌ 추출 안 함 |
| 회귀 테스트 | ✅ `tests/page_number_propagation.rs` (2 건) | ❌ 별도 추가 안 함 |

## 비교 평가

### PR #366 의 강점
1. **`PageNumberAssigner` 모듈 추출** — TypesetEngine/Paginator 두 경로의 같은 시멘틱을 단일 모듈로 통합 → 코드 중복 제거 + 재발 방지
2. **Paginator 경로도 동일하게 정정** — Task #361 은 typeset.rs 만 수정 (Paginator 의 시멘틱은 이미 거의 정상이지만 신뢰성 낮은 부분 존재 한다고 PR 본문 지적)
3. **회귀 테스트 신설** — `tests/page_number_propagation.rs` 2건 + 단위 테스트 6건
4. **상세 stage 보고서 + 시각 검증** — `2022년 국립국어원 업무계획.hwp` PDF 와 푸터 일치 확인

### PR #366 의 약점
1. **이미 정정된 결함의 재정정** — 시멘틱은 같지만 코드 위치/구조가 다름. 충돌 가능성
2. **mergeStateStatus = BEHIND** — devel rebase 필요. rebase 시 Task #361 의 변경과 충돌 발생 예상

### Task #361 의 비교
1. **단순 인라인 정정** — 모듈 추출 없이 `finalize_pages` 함수 내에서만 수정
2. **Paginator 경로 미수정** — Paginator 는 이미 거의 정상으로 판단되어 변경 안 함
3. **회귀 테스트 미신설** — 작업지시자 시각 판정 + svg_snapshot 으로 검증

## 처리 방향 후보

### 옵션 A: PR #366 close + 일부 가치 흡수

근거: 결함 자체는 이미 v0.7.7 에 정정됨. 그러나 PR #366 의 다음 가치는 흡수 가능:
- **`tests/page_number_propagation.rs` 회귀 테스트** — 별도 task 로 추가 가능
- **`PageNumberAssigner` 모듈 추출** — 코드 정합성 측면에서 가치 있으나 별도 task 로 후속 가능
- **Paginator 경로 정정** — 필요 시 후속 task

### 옵션 B: PR #366 머지 (Task #361 변경 위에)

근거: planet6897 의 작업이 더 정돈됨. devel rebase + 충돌 해결 후 머지.

문제:
- 같은 시멘틱의 두 가지 다른 코드 (Task #361 의 인라인 + PR #366 의 Assigner) 가 충돌
- 충돌 해결 시 Task #361 의 인라인 수정을 제거하고 PR #366 의 Assigner 채택해야 일관성 확보
- 시각/회귀 검증 재수행 필요

### 옵션 C: 메인테이너 흡수 (체리픽 방식)

근거: 본 task #353 처리 (PR #341 / Task #340 시점) 와 같은 방식 — planet6897 의 일부 변경만 주체적으로 흡수.

흡수 후보:
- 회귀 테스트 (`tests/page_number_propagation.rs`)
- `PageNumberAssigner` 모듈 (Task #361 의 인라인 수정을 모듈 호출로 환원)
- Paginator 경로 정정 (필요 시)

문서 (#353 의 plans / report) 는 이미 Task #361 의 문서로 대체되어 흡수 불필요.

## 권장

**옵션 C** — 메인테이너 흡수 (체리픽 방식):
1. PR #366 의 `tests/page_number_propagation.rs` + `src/renderer/page_number.rs` 만 흡수
2. Task #361 의 인라인 수정을 PageNumberAssigner 호출로 환원 (코드 일관성)
3. Paginator 경로도 동일 모듈 사용하도록 정정 (PR #366 의 변경 채택)
4. 작성자 attribution 보존 (Co-Authored-By)
5. PR #366 close + `pr_366_report.md` 작성 — 흡수 + 작성자 감사

이슈 #353 도 클로즈 (Task #361 + 본 PR 흡수 작업 으로 처리됨).

## 다음 단계

작업지시자 결정 (옵션 A / B / C) 부탁드립니다.
선택 옵션에 따라:
- A: 단순 close + 보고서
- B: rebase + 충돌 해결 + 머지 + 검증
- C: 체리픽 + Task #361 의 인라인을 모듈 호출로 환원 + 검증

검토 항목 + Claude 점검:
- [ ] PR #366 의 `PageNumberAssigner` 시멘틱이 Task #361 과 등가인지 확인
- [ ] Paginator 경로 정정의 필요성 검토 (PR 본문 주장 vs 실제 회귀 가능성)
- [ ] `tests/page_number_propagation.rs` 회귀 테스트 흡수 시 Task #361 의 효과 검증 가능 여부

## 참고

- 이슈: [#353](https://github.com/edwardkim/rhwp/issues/353) (OPEN)
- PR: [#366](https://github.com/edwardkim/rhwp/pull/366) (OPEN, BEHIND)
- 관련 task (이미 처리됨): Task #361 (`mydocs/report/task_m100_361_report.md`)
- v0.7.7 릴리즈 (2026-04-27): Task #361 포함
