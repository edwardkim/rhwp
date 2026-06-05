# PR #1091 검토 문서

- PR: <https://github.com/edwardkim/rhwp/pull/1091>
- 제목: Task #1082: 다단 미주 vpos-absolute 정합 — 본문 하단 overflow 해소
- 관련 이슈: <https://github.com/edwardkim/rhwp/issues/1082>
- 작성일: 2026-05-26
- 작성자: Codex

## 1. PR 상태

| 항목 | 값 |
|---|---|
| 상태 | open |
| base | `devel` |
| head | `pr/task1082-endnote-multicolumn-drift` |
| head sha | `e8a3a9fc60ffc27e7db0b0c971fbbc213688d5cc` |
| mergeable | true |
| 변경 파일 | 11개 |
| 증감 | +542 / -39 |
| 리뷰/댓글 | 없음 |
| 요청 리뷰어 | `edwardkim` |

CI 확인:

| check | status |
|---|---|
| Analyze (javascript-typescript) | pass |
| Analyze (python) | pass |
| Analyze (rust) | pass |
| Build & Test | pass |
| Canvas visual diff | pass |
| CodeQL | pass |
| WASM Build | skipped |

## 2. 이슈 요약

이슈 #1082는 시험지류 문서에서 페이지 하단 overflow가 수백 px까지 누적되는 문제다.
이슈 본문은 처음에는 빈 문단 누적 vpos 드리프트로 정리되어 있었지만, PR 분석에서는 실제 영향
축을 **2단 미주(endnote) 영역의 vpos 간격 누락**으로 재정의한다.

재현/권위 자료는 로컬에 존재한다.

```text
samples/3-09월_교육_통합_2022.hwp
samples/3-09월_교육_통합_2022.hwpx
samples/3-09월_교육_통합_2023.hwp
samples/3-09월_교육_통합_2023.hwpx
samples/3-10월_교육_통합_2022.hwp
samples/3-10월_교육_통합_2022.hwpx
samples/3-11월_실전_통합_2022.hwp
samples/3-11월_실전_통합_2022.hwpx
pdf/3-09월_교육_통합_2022.pdf
pdf/3-09월_교육_통합_2023.pdf
pdf/3-10월_교육_통합_2022.pdf
pdf/3-11월_실전_통합_2022.pdf
```

## 3. 변경 요약

PR #1091의 핵심 판단은 다음과 같다.

```text
렌더러는 미주 문단을 파일 vpos 기준으로 정규화해서 배치한다.
하지만 typeset의 다단 미주 누적은 미주 문단 내부 span만 더해 미주 사이 vpos 간격을 누락한다.
그 결과 단 높이 회계상으로는 들어간 것처럼 보이지만, 실제 렌더에서는 페이지 하단을 초과한다.
```

수정 방향:

```text
다단 미주 누적을 직전 배치 아이템 bottom vpos 기준 delta(px)로 계산한다.
본문에서 미주로 넘어가는 첫 미주의 base는 직전 본문 FullParagraph bottom vpos로 시드한다.
단 advance가 발생하면 base를 None으로 리셋해 새 단 첫 미주는 자체 높이 기준으로 계산한다.
```

주요 코드 변경:

| 파일 | 내용 |
|---|---|
| `src/renderer/typeset.rs` | `TypesetState.prev_body_bottom_vpos` 추가, 다단 미주 누적을 vpos delta 기반으로 정합 |
| `src/document_core/queries/rendering.rs` | dump에서 본문+미주 paragraphs를 합쳐 미주 `para_index` 디버깅 정합 |
| `tests/issue_1082_endnote_multicolumn_drift.rs` | 4개 샘플 기반 미주 overflow 회귀 가드 추가 |

## 4. 현재 코드 반영 여부

현재 `local/devel`에는 PR #1091의 핵심 구현이 아직 없다.

확인 결과:

```text
prev_body_bottom_vpos 없음
issue_1082_endnote_multicolumn_drift 테스트 없음
dump_page_items의 FullParagraph[미주] 마킹 없음
```

현재 `local/devel`은 PR #1084 반영 후 상태이며, PR #1091 브랜치의 merge-base는 `27441c84`이다.
따라서 PR #1091 브랜치는 현재 `devel`보다 한 단계 이전 기준이다.

## 5. 검토 포인트

### 5.1 수용 가능성이 높은 점

- PR의 타깃 문서와 이슈 #1082의 overflow 증상이 일치한다.
- 변경 범위가 typeset의 다단 미주 누적과 dump 디버깅 보강으로 좁혀져 있다.
- PR CI에서 Build & Test, Rust analyze, Canvas visual diff가 통과했다.
- 공개 샘플 기반 회귀 테스트가 추가되어 있다.
- PR 본문에 잔여 한계가 명시되어 있다.

### 5.2 주의해야 할 점

이 변경은 typeset의 다단 미주 배치 누적 경로를 건드린다.

특히 다음 부분은 현재 `devel` 기준으로 반드시 다시 확인해야 한다.

```text
1. 새 테스트 issue_1082_endnote_multicolumn_drift가 현재 devel에서 통과하는지
2. 기존 #1062 안전 floor가 유지되어 미주가 과밀 배치되지 않는지
3. 단단(col_count == 1) 미주 경로가 기존 동작을 유지하는지
4. PR #1084에서 막 반영한 그림 pushdown 변경과 typeset.rs 충돌/회귀가 없는지
```

PR의 변경은 `typeset.rs`를 수정한다. 현재 `devel`에는 PR #1084의 `typeset.rs` 변경도 들어와
있으므로, 수용 시에는 현재 브랜치 기준 체리픽 후 충돌 여부와 테스트를 다시 봐야 한다.

### 5.3 잔여 한계

PR 본문은 다음 known limitation을 남긴다.

```text
약 25px 잔여 overflow는 본문 formatter 누적의 trailing line spacing overcount가 미주로 전파되는
작은 base drift로 본다.
단단 본문 formatter 누적의 vpos 정합은 별도 과제다.
```

이는 종전 수백 px overflow를 대부분 해소하는 변경으로는 수용 가능하지만, 완전 정합 이슈는 별도로
남을 수 있다.

## 6. 권장 처리 방향

권장안:

```text
1. PR #1091 단일 커밋을 현재 local/devel에 -x cherry-pick 한다.
2. 현재 devel 기준으로 fmt/check/test를 다시 수행한다.
3. wasm build 후 메인테이너 시각 판정을 받는다.
4. 통과하면 devel에 반영하고 PR #1091 및 이슈 #1082를 체리픽 수용으로 close 한다.
```

바로 merge commit으로 받기보다는 `-x` cherry-pick이 적합하다.

이유:

```text
- PR base가 현재 devel보다 오래되었다.
- 변경 범위가 단일 커밋으로 정리되어 있다.
- 외부 기여자 출처를 커밋 메시지에 남길 수 있다.
- 현재 devel 기준으로 #1084와 함께 재검증하기 쉽다.
```

## 7. 승인 요청

다음 절차로 진행해도 되는지 승인 요청한다.

```text
git cherry-pick -x e8a3a9fc60ffc27e7db0b0c971fbbc213688d5cc
cargo fmt --check
cargo check
cargo test --test issue_1082_endnote_multicolumn_drift
cargo test --lib
docker compose --env-file .env.docker run --rm wasm
```
