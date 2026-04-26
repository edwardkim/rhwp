# PR #323 처리 결과 보고서

## PR 정보

| 항목 | 내용 |
|------|------|
| PR 번호 | [#323](https://github.com/edwardkim/rhwp/pull/323) |
| 작성자 | [@planet6897](https://github.com/planet6897) (Jaeuk Ryu) |
| 이슈 | [#321](https://github.com/edwardkim/rhwp/issues/321) (페이지네이션 LINE_SEG vpos-reset) |
| 처리 | **Closed (not merged)** — 작성자가 자체 close 후 PR #343 으로 통합 |
| 처리일 | 2026-04-26 (작성자 자체 close) |

## 처리 경위

1. **2026-04-25**: 작성자 PR #323 제출 (Task #321 vpos-reset 강제 분할). 메인테이너 검증 결과 **21_언어 1페이지 우측단 col 1 시작 y 회귀** 발견 — body-wide TopAndBottom 표 reserve 처리 (v2/v3) 부작용.
2. **2026-04-25**: 메인테이너가 회귀 통보 코멘트 게시 ([comment-4318132231](https://github.com/edwardkim/rhwp/pull/323#issuecomment-4318132231)) — Option A (가드 정밀화) 권장.
3. **2026-04-26 01:12**: 작성자 PR #323 자체 close. 후속 작업으로 v4/v5/v6 정밀화 + Task #322~#332 누적을 [PR #343](https://github.com/edwardkim/rhwp/pull/343) 으로 통합 제출.
4. **2026-04-26**: PR #343 머지로 본 영역 결과 통합 반영 — Task #279 결과는 작성자 자체 수정 (`782a5a7`) 으로 정확히 복원.

## 검증 결과 (close 시점)

PR #323 자동 검증 + 시각 검증 결과:

| 항목 | 결과 |
|------|------|
| `cargo build --release` | ✅ |
| `cargo test --lib` | ✅ 992 passed |
| svg_snapshot 6/6 | ✅ |
| issue_301 z-table 가드 | ✅ |
| clippy + wasm32 | ✅ |
| **21_언어 page 1 col 1 시작 y** | ⚠️ **회귀 발견** (devel 의 1184px reserve → PR 의 reserve=0) |

회귀가 머지 차단 사유 → close 결정 (작성자 자체 close + PR #343 통합).

## 변경 본질 (PR #323 시점)

- vpos-reset 강제 분할 (`typeset.rs::typeset_section`): `cv == 0 && pv > 5000 HU && !current_items.is_empty()` 가드
- `pending_body_wide_top_reserve` 추적 (v2)
- Paper 도형 가드 (v3)

이 중 v3 의 `VertRelTo::Paper` 제외 가드가 21_언어 4×5 표 (body 와 수직 겹침) 케이스를 잘못 처리 → col 1 reserve 무력화.

## 후속 (PR #343 으로 통합)

작성자가 본 PR close 후 다음과 같이 정리:
- v4/v5/v6 정밀화 (Paper 도형 + body 수직 겹침 케이스 분리)
- Task #322~#332 누적 추가
- 회귀 핀포인트 revert (#13, #332 stage3a/3b)
- 메인테이너 우려 사항 (Q1 KTX TOC) 자체 수정

[PR #343](https://github.com/edwardkim/rhwp/pull/343) 로 통합 머지 (commit `bed276c`, 2026-04-26).

## 외부 기여 가치

- 빠른 회귀 인지 + 자체 close — 메인테이너 검증 결과를 즉시 수용
- 후속 정밀화 (v4/v5/v6) + 누적 통합 (PR #343) 으로 책임감 있는 처리

## 참고 링크

- [PR #323 (closed)](https://github.com/edwardkim/rhwp/pull/323)
- [메인테이너 회귀 통보 코멘트](https://github.com/edwardkim/rhwp/pull/323#issuecomment-4318132231)
- 통합 후속 PR: [#343 (merged)](https://github.com/edwardkim/rhwp/pull/343)
- 이슈: [#321](https://github.com/edwardkim/rhwp/issues/321)
- 트러블슈팅: `mydocs/troubleshootings/typeset_layout_drift_analysis.md` (PR #343 통합 시 등록)
