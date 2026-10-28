# Task M100 #4090 Stage 1 - HWP 2020 기준 쪽 경계 복원

- 기준 commit: `e689ef0412344e683bf65163e4f23033d3b1b390`
- 작업일: 2026-08-07 KST
- 상태: 완료

## 기준과 분석

이 작업의 PDF 기준본은 HWP 2020 MCP가 실제 변환한 아래 파일 하나만 사용한다.

`pdf/issue4090/156492236_규제샌드박스_min-hancom2020-production-verify.pdf`

기준 PDF는 17쪽이다. `rhwp export-pdf` 산출물은 기준이나 증적으로 유지하지 않고 제거했다.
`rhwp` 자체의 조판 결과는 PDF가 아닌 `dump-pages --json`의 내부 페이지 수로만 확인한다.

저장 HWPX를 분석한 결과, 다음 세 본문 문단은 마지막 줄만 `vpos=0`으로 다음 물리 쪽에
저장돼 있고 바로 다음 문단은 명시적 쪽나누기 표제였다.

| 문단 | 저장 줄 경계 | HWP 2020 쪽 경계 |
| --- | --- | --- |
| 59 | 1번째 줄 뒤 `vpos=0` | 5쪽 → 6쪽 |
| 74 | 2번째 줄 뒤 `vpos=0` | 7쪽 → 8쪽 |
| 183 | 2번째 줄 뒤 `vpos=0` | 15쪽 → 16쪽 |

기존 일반 HWPX 경로는 이 세 tail 줄을 현재 쪽에 함께 배치해 총 14쪽이 됐다. 기존
`internal_vpos_page_break_line`은 HWP3와 특정 sample16 형상만 처리하므로 이 HWPX 저장
계약에는 적용되지 않았다.

## 구현

1. 빈 호스트 문단의 우측 Square 표에서 좌측 본문 띠를 복원했다.
2. 표 옆 일부 줄과 표 아래 전폭 줄이 공존하는 문단은 `WrapAroundPara`의 줄 범위로 분리했다.
3. HWPX stored layout에서만, 본문 마지막 줄이 `vpos=0`이고 직전 줄이 본문 하단에 있으며
   다음 문단이 명시적 쪽나누기인 경우 마지막 줄을 `PartialParagraph`로 다음 쪽에 보냈다.

HWP3와 일반 HWP5에는 이 규칙을 적용하지 않는다. 표·그림·각주가 있는 문단, 중간 줄 reset,
명시적 다음 쪽이 없는 문단도 제외한다.

## 실행 결과

| 검증 | 결과 |
| --- | --- |
| HWP 2020 MCP 기준 PDF | 17쪽. `PrintToPDFEx`, `PrintMethod=0`, 서버 validation `ok` 확인. |
| `cargo fmt --check` | 통과 |
| `CARGO_TARGET_DIR=target/issue4090-pdf-17pages CARGO_INCREMENTAL=0 cargo check --profile release-test --bin rhwp` | 통과 |
| `rhwp dump-pages ... --json` | `pageCount: 17` |
| 문단 59 | 5쪽 `0..1`, 6쪽 `1..2`로 분할 |
| 문단 74 | 7쪽 `0..2`, 8쪽 `2..3`으로 분할 |
| 문단 183 | 15쪽 `0..2`, 16쪽 `2..3`으로 분할 |
| `git diff --check` | 통과 |

## 다음 Stage

이 저장 계약을 순수 단위 테스트와 fixture 기반 `dump-pages` 회귀로 고정한다. 다음 Stage는
분석 문서를 먼저 커밋한 뒤 코드와 실행 결과를 같은 후속 커밋으로 남긴다.
