# PR #1250 검토 — task 1245 어울림 그림 및 미주 수식 배치 보정

- **작성일**: 2026-06-02
- **PR**: #1250 (OPEN)
- **제목**: `task 1245: 어울림 그림 및 미주 수식 배치 보정`
- **컨트리뷰터**: @jangster77
- **연결 이슈**: #1245
- **base/head**: `devel` ← `task_m100_1245`
- **Head SHA**: `c5f5442934c64bf2d9d03cc0de365274383e0693`
- **현재 local/devel**: `f7cd9556`
- **규모**: 11 files, +700 / -22, 6 commits
- **GitHub 상태**: `MERGEABLE`
- **CI**: `Build & Test`, `Render Diff`, `CodeQL` 통과. `WASM Build`는 skip.
- **PR 댓글/리뷰**: 없음

## 1. PR 요약

PR #1250은 #1245에서 보고된 `3-09월_교육_통합_2022.hwp` 7쪽 하단 그림 배치 불일치를 시작점으로, 후속 분석 중 확인된 수식-only TAC 문단과 미주 선두 번호 중복 표시까지 함께 보정한다.

주요 대상:

- `samples/3-09월_교육_통합_2022.hwp`
- `samples/3-09월_교육_통합_2023.hwp`

핵심 변경:

1. `Square/어울림` 그림의 line segment 기반 세로 위치를 raw `vertical_pos`가 아니라 문단 첫 줄 대비 상대 delta로 계산한다.
2. 본문/미주 수식-only TAC 문단에서 TAC가 없는 선행 guide 줄을 수식 배치 후보와 y advance에서 제외한다.
3. 본문/미주 수식-only 문단은 저장된 `LINE_SEG` 흐름을 따르고, 셀 내부 수식-only 문단은 기존처럼 셀 문단 정렬을 따른다.
4. 미주 선두 번호가 prefix `TextRun`으로 이미 그려진 경우 같은 위치의 `FootnoteMarker`를 중복 생성하지 않는다.

## 2. 주요 변경 범위

| 파일 | 변경 |
|---|---|
| `src/renderer/layout.rs` | `Square/어울림` 그림의 첫 좁은 line segment vpos를 첫 줄 대비 delta로 보정 |
| `src/renderer/layout/paragraph_layout.rs` | 수식-only TAC guide 줄 제외, 본문/미주 vs 셀 정렬 기준 분리, 미주 marker 중복 억제 |
| `tests/issue_1139_inline_picture_duplicate.rs` | #1245, #1209, 2023 `문26)` 회귀 테스트 추가/보강 |
| `mydocs/plans/task_m100_1245*.md` | 계획/구현 계획 문서 |
| `mydocs/working/task_m100_1245_stage*.md` | 단계별 분석/검증 문서 |
| `mydocs/report/task_m100_1245_report.md` | 완료 보고서 |

## 3. 타당한 부분

### 3.1 `LINE_SEG.vertical_pos`의 상대/절대 혼입을 보정한다

`Square/어울림` 그림 위치 보정에서 raw `vertical_pos`를 그대로 문단 y에 더하면, 첫 줄 `vertical_pos`가 이미 누적 흐름값인 문서에서는 `para_y + absolute_vpos`가 되어 그림이 페이지 하단 밖으로 밀린다.

이번 변경은 첫 줄 대비 delta만 사용하므로 기존 첫 줄 vpos가 0인 문서에서는 기존 동작을 유지하고, 누적 vpos 문서에서는 중복 계상을 제거한다.

### 3.2 수식-only TAC guide 줄을 실제 배치 줄과 분리한다

`char_start=0, char_end=0`이고 TAC가 없는 선행 guide 줄이 실제 수식 앞 높이로 예약되면, 한컴보다 수식이 아래로 밀린다.

PR은 TAC가 없는 선행 guide 줄을 후보에서 제외하고, 다음 줄의 equation TAC를 실제 배치 기준으로 삼는다. `문12)` 수식 위치 문제의 원인 설명과 맞다.

### 3.3 셀 내부 수식 정렬 회귀를 의식한다

본문/미주 수식-only 문단은 `effective_col_x` 기준으로 두되, 셀 내부 수식-only 문단은 기존 Task #490의 셀 문단 정렬 동작을 유지한다.

이 분리는 큰 방향에서 맞다. 셀 내부 수식 정렬 회귀를 막기 위한 별도 테스트가 이미 있는지 검증 브랜치에서 함께 확인해야 한다.

### 3.4 미주 선두 번호 중복 원인을 좁혀 처리한다

미주 선두 번호가 prefix `TextRun`으로 이미 렌더된 경우 같은 위치의 `FootnoteMarker`를 생략한다. 각주 전체를 건드리지 않고 Endnote 선두 marker 중복만 막는 점은 범위가 적절하다.

## 4. 위험 및 주의 사항

### 4.1 `paragraph_layout.rs`의 공통 흐름을 건드린다

TAC 수식 배정, line advance, baseline 배치, 미주 marker 생성은 여러 문서의 조판에 영향을 줄 수 있다.

따라서 PR 자체 테스트 외에 다음 회귀군을 검증해야 한다.

- `issue_1139_inline_picture_duplicate`
- `issue_1082_endnote_multicolumn_drift`
- `issue_1209` 계열이 포함된 테스트
- 수식/셀 정렬 관련 테스트
- 전체 integration 테스트

### 4.2 대체 시각 판정이 필요하다

PR은 `3-09월_교육_통합_2022.hwp`와 `3-09월_교육_통합_2023.hwp`의 특정 페이지를 대상으로 한다.

자동 테스트만으로는 실제 한컴/PDF 기준의 하단 그림 배치, 수식 배치, `문26)` 중복 표시가 충분히 검증되지 않는다.

### 4.3 열린 PR 이후의 devel 기준은 현재 충돌 없음

`git merge-tree HEAD pr/1250` 기준 현재 `local/devel`과 충돌은 없다.

## 5. 권장 검증

현재 `local/devel` 기준 검증 브랜치를 만들고 PR #1250을 병합한 뒤 다음을 실행한다.

```text
git diff --check HEAD
cargo fmt --all --check
cargo test --test issue_1139_inline_picture_duplicate -- --nocapture
cargo test --test issue_1082_endnote_multicolumn_drift
cargo test --lib renderer::layout
cargo test --tests
docker compose --env-file .env.docker run --rm wasm
cd rhwp-studio && npm run build
```

시각 판정 후보:

| file | page | 확인 항목 |
|---|---:|---|
| `samples/3-09월_교육_통합_2022.hwp` | 7 | `문25)`/`문28)` 어울림 그림 위치 |
| `samples/3-09월_교육_통합_2022.hwp` | 10 | `문12)` 우측 단 수식 위치 |
| `samples/3-09월_교육_통합_2023.hwp` | 4 | `문26)` 중복 표시 제거 |

## 6. 권장 처리

권장안: **수용 후보로 진행한다.**

근거:

- PR head CI가 통과했다.
- 현재 `local/devel` 기준 병합 충돌이 없다.
- 변경 원인이 한컴/PDF 기준 증상과 연결되어 있다.
- 변경 범위는 크지만 회귀 테스트가 함께 추가되었다.
- PR #1241, #1247 이후 rebase와 회귀 확인이 PR 본문에 기록되어 있다.

다만 `paragraph_layout.rs` 공통 경로를 건드리므로, 검증 브랜치에서 전체 테스트와 WASM/Studio 빌드, 메인테이너 시각 판정을 게이트로 둔다.

## 7. 다음 승인 요청

다음 단계로 진행하려면 작업지시자 승인이 필요하다.

권장 절차:

```text
1. `local/pr1250-verify` 브랜치를 현재 `local/devel`에서 생성
2. PR #1250을 병합
3. 회귀 테스트 및 전체 테스트 실행
4. WASM/Studio 빌드
5. 메인테이너 시각 판정
6. 판정 통과 후 local/devel 반영
```
