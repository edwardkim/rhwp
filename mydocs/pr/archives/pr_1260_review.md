# PR #1260 검토 — trailing 모델 조사 문서 수용 검토

- **작성일**: 2026-06-03
- **PR**: #1260
- **제목**: `조사: render/pagination trailing 모델 — 통일 가능성 분석 (#1248)`
- **컨트리뷰터**: @planet6897
- **연결 이슈**: #1248
- **base/head**: `devel` <- `feature/issue-1248-trailing-model-investigation`
- **PR head**: `1c50f23e0b3cc141a16e728b447f210d858c7a48`
- **PR 기준 base**: `5137c07f`
- **현재 devel**: `59bb0e99`
- **검증 브랜치**: `local/pr1260-current`
- **검증 적용 커밋**: `97939dca`, `cfee533d`, `252932f0`
- **규모**: 7 files, +559 / -0
- **GitHub mergeable**: true
- **PR 댓글**: 없음

## 1. PR 요약

PR #1260은 코드 변경 없이 #1248 trailing 모델 조사 문서를 추가한다.

조사 대상은 `typeset`, `pagination`, `render/height_cursor`가 trailing `line_spacing`을 서로 다르게 해석하는 구조이며, 결론은 다음과 같다.

- trailing은 현재 단일 SSOT가 아니라 typeset, pagination, render에 분산되어 있다.
- 전면 통일은 권장하지 않는다.
- `typeset`의 base-flow 1984HU 가정과 `render`의 trailing 재구성 사이 좁은 정규화는 가능하다.
- backward/rewind 계열 게이트는 한컴 `LINE_SEG` 입력의 모호성에 따른 비가역 영역이므로 유지해야 한다.
- #1247 min-gap 특례는 후속 A 정규화가 성공하면 흡수 가능한 기술부채로 분류한다.

## 2. 주요 변경 범위

| 파일 | 변경 |
|---|---|
| `mydocs/tech/investigations/issue-1248/trailing_model_render_vs_pagination_1248.md` | trailing 처리 현황, vpos_adjust 특례, 통일 가능/불가 영역 분석 |
| `mydocs/plans/task_m100_1248.md` | 작업 계획 |
| `mydocs/plans/task_m100_1248_impl.md` | 구현/조사 계획 |
| `mydocs/working/task_m100_1248_stage1.md` | Stage 1 현황 맵 |
| `mydocs/working/task_m100_1248_stage2.md` | Stage 2 특례 분석 |
| `mydocs/working/task_m100_1248_stage3.md` | Stage 3 결론 |
| `mydocs/report/task_m100_1248_report.md` | 최종 보고서 |

## 3. 타당한 부분

### 3.1 코드 변경 없이 조사 결과만 추가한다

PR은 `height_cursor`나 pagination 코드를 수정하지 않는다. 따라서 런타임 회귀 위험은 없고, 검토 대상은 조사 문서의 정확도와 향후 의사결정에 주는 유용성이다.

### 3.2 #1247/#1259 계열 결정을 설명하는 배경 문서로 가치가 있다

최근 미주 trailing 관련 수정은 단일 규칙으로 정리하기 어렵고, 특례가 왜 필요한지 설명하기가 점점 어려워졌다. 이 문서는 다음 구분을 명시해 향후 작업 기준점으로 쓸 수 있다.

- 통일 가능한 영역: typeset base-flow 가정과 render 재구성 간 불일치
- 유지해야 할 영역: backward/rewind/invalid-lazy-base 계열 게이트
- 후속 기술부채: min-gap 특례의 A 정규화 흡수 가능성

### 3.3 후속 이슈 범위를 과도하게 넓히지 않는다

문서는 전면 통일을 권장하지 않고, 좁은 A 정규화만 별도 후속으로 제안한다. `height_cursor`가 회귀 위험이 높은 파일임을 고려하면 범위 관리가 현실적이다.

## 4. 위험 및 주의 사항

### 4.1 문서 기준 시점이 현재 devel과 다르다

PR 문서에는 `stream/devel`, `PR #1247 미머지`, `PR #1247 그대로 머지` 같은 표현이 남아 있다.

현재 `devel`은 이미 #1247과 #1259를 포함한다. 따라서 PR을 그대로 수용하면 독자가 이 문서를 현재 코드 상태의 실시간 인벤토리로 오해할 수 있다.

권장 보완:

```text
문서 상단에 "본 조사는 #1247/#1259 반영 전 기준의 조사 기록이며,
현재 devel에서는 관련 PR이 이미 반영되어 테스트 개수와 일부 상태 표현이 다를 수 있다"는
현행화 메모를 추가한다.
```

### 4.2 검증 개수가 현재 devel에서 달라졌다

PR 문서의 검증 로그는 `cargo test --lib height_cursor = 26 passed`로 기록되어 있다.

현재 `devel` 위에 체리픽한 검증에서는 같은 명령이 31개 테스트 통과로 실행된다. 이는 #1247/#1259 이후 테스트가 늘어난 영향으로 보이며 실패는 아니다. 다만 문서가 historical snapshot임을 명시해야 한다.

### 4.3 메모리 키워드가 문서에 직접 등장한다

`tech_lazy_base_trailing_ls_gate` 같은 내부 메모리 키워드가 기술 문서에 등장한다. 조사 맥락을 보존하는 데는 도움이 되지만, 외부 독자에게는 경로가 불명확하다.

수용 시에는 "기존 작업 메모/회귀 기준" 정도의 설명을 덧붙이면 더 읽기 쉽다.

## 5. 자동 검증 결과

현재 `devel` 위에 PR 커밋 3개를 체리픽해 검증했다.

| 항목 | 명령 | 결과 |
|---|---|---|
| cherry-pick | PR commits 3건 -> `local/pr1260-current` | 통과, 충돌 없음 |
| whitespace | `git diff --check devel..HEAD` | 통과 |
| height_cursor 테스트 | `cargo test --lib height_cursor` | 통과, 31 passed |
| 미주 다단 회귀 테스트 | `cargo test --test issue_1082_endnote_multicolumn_drift` | 통과, 4 passed |

코드 변경이 없으므로 WASM 빌드와 시각 판정은 필수 게이트로 보지 않는다.

## 6. 권장 처리

권장안: **현행화 메모를 추가한 뒤 수용**한다.

근거:

- PR은 문서 전용이라 런타임 회귀 위험이 없다.
- trailing 모델의 통일 가능/불가 영역을 정리한 문서로 유지 가치가 있다.
- 현재 `devel` 기준 자동 검증이 통과했다.
- 다만 문서의 기준 시점이 현재 `devel`과 달라, 상단 현행화 메모 없이 수용하면 혼동 가능성이 있다.

## 7. 다음 승인 요청

권장 절차:

```text
1. 조사 문서와 완료 보고서에 현재 devel 기준 현행화 메모 추가
2. 문서 diff 재확인
3. 검증 커밋 작성
4. devel 병합
5. devel 기준 간단 검증 후 push
6. PR #1260 및 이슈 #1248 종료 처리
```
