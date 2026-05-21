# PR #1039 최종 보고서 — HWPX slash/backSlash 형태 enum 파싱 분리

- PR: [#1039](https://github.com/edwardkim/rhwp/pull/1039)
- 제목: HWPX slash/backSlash 형태 enum 파싱 분리 (#1038)
- 작성자: planet6897 (Jaeuk Ryu) — 누적 컨트리뷰터 (PR #221, #587 등 굵직한 기여)
- base ← head: `devel` ← `planet6897:task1038-hwpx-slash-diagonal`
- 결정: **merge (수용)** — 시각 판정 면제, 정량 게이트로 충족
- 일자: 2026-05-21

## 1. 결정

**merge 수용.** PR 의 책임 분리 설계 (slash/backSlash = 방향/형태 enum
vs `<hh:diagonal>` = 선 종류·굵기·색) 가 HWPX spec 1:1 정합. 모든
결정적 게이트 통과 + 회귀 가드 테스트 동봉.

**시각 판정 면제** — 작업지시자 결정. 본 PR 의 검증 근거가 결정적
측정 ("검정 대각선 3→0") + 신규 단위 테스트 4건 (#1038 회귀 가드 +
비회귀 보장) 으로 자가검증 패턴과 결이 다름. 평소 적용하던 메인테이너
hands-on 게이트를 본 PR 에 한해 생략하고 정량 게이트로 충족.

이슈 #1038 은 OPEN, planet6897 본인 작성, assignee 비어있음. PR merge
시 `closes #1038` 자동 close 처리.

## 2. 검증 결과

| 게이트 | 결과 |
|--------|------|
| CI: Build & Test | ✅ pass |
| CI: Analyze rust/js/py | ✅ pass |
| CI: CodeQL | ✅ pass |
| CI: WASM | skip (parser-only PR) |
| PR 본문 검증 — `cargo test` 전체 통과 + 신규 단위/회귀 4건 | ✅ |
| PR 본문 검증 — 문제 샘플 p4 헤딩: 검정 대각선 3→0 (한컴 PDF 정합) | ✅ 정량 측정 |
| PR 본문 검증 — `tac-img-02.hwpx`: 대각선 정상 유지 (비회귀) | ✅ |
| 본 환경 PR 검증 (cherry-pick 후) | (cherry-pick 단계에서 cargo test/fmt 확인 예정) |
| **작업지시자 시각 판정** | **면제 (작업지시자 결정)** |

> 본 환경 시각 검증 보조 시 발견 사실 (참고):
> - PR 본문이 가리키는 `samples/2. 인공지능(AI) ...` 파일은 컨트리뷰터
>   로컬의 비공개 파일로 본 저장소 `samples/` 에 없음
> - 본 환경 `samples/` 전체 sweep 결과 #1038 본질 (slash CENTER +
>   diagonal 없음) 재현 샘플 없음
> - `tac-img-02.hwpx` 는 slash CENTER + diagonal SOLID 보유 (비회귀
>   샘플) — rhwp-studio 에서 p5 대각선 정상 표시 확인 (devel 기준,
>   메인테이너 hands-on)
> - PR 의 신규 단위 테스트 `test_slash_center_without_diagonal_no_line`
>   가 #1038 본질을 결정적으로 검증 → 정량 게이트로 시각 판정 대체
>   판단의 핵심 근거

## 3. 변경 내용

`src/parser/hwpx/header.rs` 단일 파일:

| 영역 | 변경 |
|------|------|
| slash 핸들러 | `type` → `parse_slash_shape_code()` → 방향 비트(attr)만 설정. `diagonal_type`/`width`/`color` 분기 제거 |
| backSlash 핸들러 | 동일 (shift=5) |
| 신규 헬퍼 `parse_slash_shape_code` | enum (NONE/CENTER/CENTER_BELOW/CENTER_ABOVE/기타) → 3비트 방향 코드 매핑 |
| `set_diagonal_attr_bits` | `code & 0x07` 그대로 기록 (기존 nonzero→0b010 축소 제거) |
| 신규 테스트 4건 | enum 매핑 + 비트 설정 + #1038 회귀 가드 + 비회귀 보장 |

전체: +128/-29 (테스트 +93 포함, 실 변경 +35/-29).

## 4. 설계 평가

- **`feedback_hancom_compat_specific_over_general` 정합**: 책임 분리
  (방향/형태 vs 선 종류) = 구조 가드. 측정 의존 분기 없음. HWPX spec
  의 두 요소 1:1 매핑.
- **`feedback_small_batch_release_strategy` 정합**: 단일 commit + 단일
  파일 (소스). 파서 책임 분리에 한정.
- **scope 정직**: PR 본문 "렌더러/모델/HWP5·HWP3 경로 무수정" 명시,
  실제 변경 파일 검증 일치.
- **회귀 가드 동봉**: #1038 정합 테스트 + 비회귀 (slash NONE + diagonal
  SOLID 정상) 테스트 2건 신규 추가.
- **정보 보존**: 기존 `set_diagonal_attr_bits` 가 3비트 → 1비트 축소
  (`line_type != 0 ? 0b010 : 0`) 하던 것을, 본 PR 이 3비트 정보 완전
  보존. backSlash 분리 + CENTER_BELOW/ABOVE 구분 가능.

## 5. cherry-pick 처리

PR 본질 commit:
- `3d5c0ead` HWPX slash/backSlash 형태 enum 파싱 분리 (closes #1038)

처리: 단일 commit author (planet6897) 보존 cherry-pick. 본 환경 정합
clean-up commit 없음 (코드 품질 지적 사항 없음).

## 6. 잔존 / 후속

### 본 PR scope 외

- 이슈 #1038 assignee 누락 — PR #1031/#950 과 동일 패턴 (본인 작성
  + 본인 PR). 메모리 룰 `feedback_assign_issue_before_work` 안내 후보,
  merge blocker 아님.
- PR 본문 검증 샘플이 컨트리뷰터 로컬 비공개 파일 — 본 환경에 공개
  fixture 추가 없음. 향후 #1038 패턴 회귀 방지 위해 공개 fixture 추가
  task 후보 (별도, 본 PR scope 외).

### 분리 보존 — 본 PR scope 외

- 다른 OPEN PR (#1051 postmelee Task #986, #1048/#1045/#1044
  planet6897 등) — 본 PR 처리와 독립

## 7. 산출물

- `mydocs/pr/pr_1039_review.md` (검토 문서)
- 본 보고서
- 소스: PR `header.rs` 책임 분리 + 신규 테스트 4건

## 8. 메모리 룰 갱신 검토

- `project_external_contributors`: planet6897 = 등재된 누적 기여자.
  갱신 불요.
- **신규 룰 후보 없음** — 본 PR 처리는 기존 룰
  (`feedback_hancom_compat_specific_over_general`,
  `feedback_small_batch_release_strategy`,
  `feedback_visual_judgment_authority` — 정량 게이트 충족 시 면제 가능
  사례) 의 권위 사례.
- **신규 룰 후보 (별도 정리 task)** — "정량 게이트 충족 시 시각 판정
  면제 가능" 패턴: 결정적 측정 (before→after 정량) + 회귀 가드 단위
  테스트 동봉 + parser-only scope 의 3 조건 동시 만족 시 메인테이너
  hands-on 게이트 면제 가능. 본 PR 이 첫 권위 사례. (별도 task 후보,
  본 처리와 독립)
