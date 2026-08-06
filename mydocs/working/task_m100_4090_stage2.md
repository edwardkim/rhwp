# #4090 Stage 2 - 저장 HWPX tail 쪽 경계 회귀 계약

## 목적

Stage 1의 `vpos=0` tail 줄 분할 보정이 다시 사라져도 즉시 검출하도록, 실제 문제
fixture의 페이지 수와 문단별 줄 범위를 CLI `dump-pages --json` 계약으로 고정한다.

## 기준

- 대상: `samples/issue4090/156492236_규제샌드박스_min.hwpx`
- PDF 기준: HWP 2020 MCP `PrintToPDFEx`, `PrintMethod=0` 결과 17쪽
- PDF 파일: `pdf/issue4090/156492236_규제샌드박스_min-hancom2020-production-verify.pdf`
- rhwp 검증 대상: PDF가 아닌 `dump-pages --json`의 페이지네이션 결과

HWP 2020 MCP PDF만 기준 산출물로 유지한다. rhwp 자체 PDF는 이 단계에서 생성하지
않는다.

## 분석

Stage 1에서 확인한 저장 HWPX 고유의 쪽 경계는 아래 세 곳이다.

| 문단 | 앞쪽 줄 범위 | 다음 쪽 tail 줄 범위 | 기대 쪽 |
| --- | --- | --- | --- |
| `pi=59` | `[0, 1)` | `[1, 2)` | 5쪽, 6쪽 |
| `pi=74` | `[0, 2)` | `[2, 3)` | 7쪽, 8쪽 |
| `pi=183` | `[0, 2)` | `[2, 3)` | 15쪽, 16쪽 |

세 문단 모두 마지막 저장 줄 `vpos=0` 뒤에 명시적 쪽 나눔 문단이 온다. 전체 17쪽만
고정하면 다른 문단을 밀어도 통과할 수 있으므로, 각 `partialParagraph`의
`startLine`/`endLine`과 페이지까지 함께 고정해야 한다.

## 구현 계획

1. `tests/issue_4090_hwpx_tail_page_break.rs`를 추가한다.
2. 실제 CLI의 JSON 봉투를 파싱하여 전체 17쪽과 세 tail 분할의 페이지·줄 범위를
   검증한다.
3. fixture가 실제로 HWPX이고 `pi=59, 74, 183`을 모두 포함하는지 별도 확인한다.
4. 포맷, focused integration test, `dump-pages --json` 재실행 결과를 기록한 뒤 테스트와
   이 문서를 하나의 일반 커밋으로 고정한다.

## 결과

### 구현

- `tests/issue_4090_hwpx_tail_page_break.rs`를 추가했다.
- 테스트는 실제 `rhwp dump-pages <fixture> --json`을 실행하고 JSON을 파싱한다.
- `pageCount=17`과 `pi=59`, `pi=74`, `pi=183`의 현재 쪽/다음 쪽
  `partialParagraph` 줄 범위를 함께 검증한다.

### 실행 결과

```text
CARGO_TARGET_DIR=target/issue4090-pdf-17pages \
CARGO_INCREMENTAL=0 \
cargo test --profile release-test --test issue_4090_hwpx_tail_page_break -- --nocapture

running 1 test
test issue_4090_hwpx_tail_lines_follow_the_explicit_page_break ... ok

test result: ok. 1 passed; 0 failed
```

동일한 CLI 재확인 결과는 아래와 같았다.

```json
{
  "pageCount": 17,
  "splits": [
    { "page": 5, "paraIndex": 59, "startLine": 0, "endLine": 1 },
    { "page": 6, "paraIndex": 59, "startLine": 1, "endLine": 2 },
    { "page": 7, "paraIndex": 74, "startLine": 0, "endLine": 2 },
    { "page": 8, "paraIndex": 74, "startLine": 2, "endLine": 3 },
    { "page": 15, "paraIndex": 183, "startLine": 0, "endLine": 2 },
    { "page": 16, "paraIndex": 183, "startLine": 2, "endLine": 3 }
  ]
}
```

`cargo fmt --check`와 `git diff --check`도 통과했다. 이 단계에서는 rhwp PDF를
생성하지 않았고, `pdf/issue4090/`에는 HWP 2020 MCP 기준 PDF만 유지했다.

## 다음 단계

Stage 3에서 이번 저장 HWPX 전용 가드의 적용 조건을 단위 수준으로 분해해, 일반 HWP/HWPX의
명시 쪽 나눔과 충돌하지 않는지 확인한다. 해당 단계도 계획 문서 작성 후 코드·검증 결과를
같은 커밋으로 고정한다.
