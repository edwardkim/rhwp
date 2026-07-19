# PR #1260 처리 보고서 - trailing 모델 조사 문서 수용

- **작성일**: 2026-06-03
- **PR**: #1260
- **연결 이슈**: #1248
- **컨트리뷰터**: @planet6897
- **검증 브랜치**: `local/pr1260-current`
- **PR head**: `1c50f23e0b3cc141a16e728b447f210d858c7a48`
- **검증 적용 커밋**: `97939dca`, `cfee533d`, `252932f0`
- **메인테이너 보강 커밋**: `71a438f9`
- **devel 병합 커밋**: `ee6f6630`
- **기준 devel**: `59bb0e99`

## 1. 처리 요약

PR #1260은 #1248 trailing 모델 조사 문서를 추가하는 문서 전용 PR이다. 현재 `devel` 기준 검증 브랜치에 PR 커밋 3건을 체리픽했고, 충돌은 없었다.

수용 범위는 다음이다.

- `typeset`, `pagination`, `render/height_cursor`가 trailing `line_spacing`을 해석하는 방식 조사
- `vpos_adjust` 특례 8종과 핀 고정 테스트 정리
- 전면 통일 불가, A 영역 정규화 가능, D/E 영역 게이트 유지라는 결론 기록
- 후속 기술부채로 A 정규화 이슈 골격 제안

## 2. 메인테이너 보강

PR 문서는 #1247/#1259 반영 전 `stream/devel` 기준으로 작성된 조사 스냅샷이다. 현재 `devel`에는 #1247과 #1259가 이미 반영되어 있으므로, 다음 문서에 현행화 메모를 추가했다.

| 파일 | 보강 |
|---|---|
| `mydocs/tech/investigations/issue-1248/trailing_model_render_vs_pagination_1248.md` | #1247/#1259 반영 전 기준 조사 스냅샷임을 명시 |
| `mydocs/report/task_m100_1248_report.md` | 현재 `devel` 기준 재검증 결과와 당시 권장 행동의 해석 기준 명시 |
| `mydocs/pr/pr_1260_review.md` | 수용 판단, 위험, 검증 결과, 권장 절차 기록 |

## 3. 자동 검증 결과

검증 브랜치 `local/pr1260-current`에서 수행:

| 항목 | 명령 | 결과 |
|---|---|---|
| whitespace | `git diff --check devel..HEAD` | 통과 |
| height_cursor 테스트 | `cargo test --lib height_cursor` | 통과, 31 passed |
| 미주 다단 회귀 테스트 | `cargo test --test issue_1082_endnote_multicolumn_drift` | 통과, 4 passed |

`devel` 병합 후 수행:

| 항목 | 명령 | 결과 |
|---|---|---|
| whitespace | `git diff --check HEAD~1..HEAD` | 통과 |
| height_cursor 테스트 | `cargo test --lib height_cursor` | 통과, 31 passed |
| 미주 다단 회귀 테스트 | `cargo test --test issue_1082_endnote_multicolumn_drift` | 통과, 4 passed |

코드 변경이 없는 문서 전용 PR이므로 WASM 빌드와 시각 판정은 필수 게이트로 적용하지 않았다.

## 4. devel 병합

`local/pr1260-current`를 `devel`에 `--no-ff`로 병합했다.

```text
ee6f6630 Merge PR 1260 trailing model investigation
```

## 5. 최종 판단

자동 검증이 모두 통과했고, 조사 문서의 기준 시점 차이도 메인테이너 보강으로 명시했으므로 PR #1260을 수용한다.

남은 절차:

1. 본 처리 보고서 커밋
2. `origin/devel` push
3. PR #1260에 메인테이너 처리 코멘트 추가
4. PR #1260 종료
5. 이슈 #1248 종료
