# PR #1232 처리 보고서

- PR: `#1232`
- 제목: `task 1209: task 1139 후속 미주·그림 흐름 보정`
- 기여자: `jangster77`
- 관련 이슈: `#1209`
- 처리일: 2026-06-02

## 1. 처리 결론

PR #1232는 현재 `local/devel` 기준 검증 브랜치에서 병합 시뮬레이션과 자동 검증을 통과했다.

수용 후보 범위:

```text
1. 미주 사이 간격/구분선 아래 여백/compact endnote 흐름 보정
2. Square/어울림 그림의 문단 흐름 anchor 보정
3. internal rewind 수식 및 미주 tail 배치 보정
4. HWP5 object anchor 진단 보강
5. rhwp-studio modal dialog drag 공통화
6. 관련 샘플/문서/회귀 테스트 보강
```

현재 단계의 결론은 **자동 검증 및 메인테이너 시각 판정 통과**이다.

## 2. 로컬 반영

```text
base: local/devel @ f6ffe9d6
PR head: pr/1232 @ ef0330a4
verify branch: local/pr1232-verify
merge commit: 82ae3375 Merge PR 1232 verification
```

충돌:

```text
mydocs/orders/20260602.md
```

충돌 원인:

```text
현재 devel의 #1237 작업 기록과 PR #1232의 #1209 작업 기록이 같은 날짜 주문서에 add/add 형태로 충돌했다.
```

해결:

```text
#1237 항목은 완료 상태로 보존하고, PR #1232의 Task #1209 체크리스트를 같은 문서 아래에 병합했다.
```

코드 충돌은 없었다.

## 3. 검증 결과

통과:

```text
git diff --check HEAD
cargo fmt --all --check
cargo test --test issue_1082_endnote_multicolumn_drift -- --nocapture
cargo test --test issue_1139_inline_picture_duplicate -- --nocapture
cargo test --tests
cargo test --lib renderer::equation -- --nocapture
docker compose --env-file .env.docker run --rm wasm
cd rhwp-studio && npm run build
```

`cargo test --tests` 안에서 다음 PR 본문 가드도 함께 통과했다.

```text
cargo test --test issue_1219_equation_line_hangul_advance -- --nocapture
cargo test --test issue_301 -- --nocapture
```

주요 결과:

```text
issue_1082_endnote_multicolumn_drift: 4 passed
issue_1139_inline_picture_duplicate: 41 passed
cargo test --tests: 전체 통과
renderer::equation: 132 passed
WASM build: success
rhwp-studio build: success
```

비고:

```text
rhwp-studio build는 기존과 동일한 chunk size warning만 출력했다.
일부 렌더링 테스트는 기존 진단용 LAYOUT_OVERFLOW 로그를 출력하지만 테스트 판정은 통과했다.
```

## 4. 확인된 리스크

이번 PR은 `typeset.rs`, `layout.rs`, `height_cursor.rs` 등 페이지네이션/렌더링 공통 경로를 변경한다.

자동 테스트 기준으로는 #1082, #1139, #1189, #1209, 수식, 최근 페이지네이션 가드가 통과했지만, 실제 수용 전에는 메인테이너 시각 판정이 필요하다.

`rhwp-studio` modal drag 공통화는 mouse event 중심이다. 이번 PR의 blocker는 아니지만, touch/pointer 대응은 후속 개선 항목으로 둘 수 있다.

## 5. 메인테이너 시각 판정 대상

권장 판정 대상:

```text
1. #1209 대상 시험지 샘플의 미주 간격/구분선 아래 여백
2. `samples/test-image.hwp`
3. `samples/test-image.hwpx`
4. Square/어울림 그림 이후 문단 흐름
5. rhwp-studio 주요 modal dialog drag 동작
```

판정표:

| 항목 | 기대 동작 | 판정 | 비고 |
|---|---|---|---|
| 미주 사이 간격 | 한컴/PDF 기준과 유사 | 통과 |  |
| 미주 구분선 아래 여백 | 과소/과대 배치 없음 | 통과 |  |
| Square/어울림 그림 흐름 | 그림 주변 텍스트 흐름 정상 | 통과 |  |
| internal rewind 수식 tail | 다음 줄/다음 단 배치 정상 | 통과 |  |
| `samples/test-image.hwp` | 그림 anchor/흐름 정상 | 통과 |  |
| `samples/test-image.hwpx` | 그림 anchor/흐름 정상 | 통과 |  |
| modal drag | 제목줄 drag로 위치 이동 가능 | 통과 |  |

메인테이너 시각 판정:

```text
2026-06-02 통과
```

추가 관찰:

```text
samples/3-09월_교육_통합_2023.hwp 시각 판정 중 별도 페이지네이션 문제가 발견되었다.
이번 PR #1232 때문에 발생한 회귀는 아닌 것으로 판정한다.
```

발견된 별도 문제:

```text
1. 17 페이지 왼쪽 단 끝부분 페이지네이션 실패
2. 18 페이지 문26) 이 문25) 문항과 겹침
3. 19 페이지 오른쪽 단 끝부분 페이지네이션 실패
```

이 문제는 PR #1232 수용 blocker가 아니며 별도 이슈 #1243으로 등록해 추적한다.

## 6. 다음 절차

```text
1. 별도 페이지네이션 이슈 #1243 등록 완료
2. 보고서 승인
3. 검증 브랜치 변경을 local/devel에 반영
4. 필요 시 devel push 및 PR #1232 종료 처리
```
