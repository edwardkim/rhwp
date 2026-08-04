# PR #2675 검토 기록

| 항목 | 내용 |
|---|---|
| PR | [#2675](https://github.com/edwardkim/rhwp/pull/2675) |
| 작성자 / base | postmelee (collaborator self-merge 후보) / `devel` |
| 관련 | merged [#2510](https://github.com/edwardkim/rhwp/pull/2510), [`pr_2510_review.md`](pr_2510_review.md), [#2430](https://github.com/edwardkim/rhwp/issues/2430) |
| 범위 | `tools/task2430/` 증적 표현 정정 (EVIDENCE SHA-256 LF, preflight identity 누적) + 오늘할일 |
| 규모 | 문서 작성 시점 참고값 +61/-13, 4 files (`tools/**`·`mydocs/**` 전용) |
| CI 스냅샷 | 문서 작성 시점 참고값. merge 전 최신 head 기준 GitHub Actions 통과 재확인 필요 |
| 처리 경로 | workflow §8 collaborator self-merge 후보. §8.3에 따라 head 브랜치를 `upstream`(`pr2510-evidence-preflight-fix`)에 직접 생성 |

## 배경

[`pr_2510_review.md`](pr_2510_review.md)(메인테이너 작성)의 **비차단 후속 보완** 2건은
`tools/task2430/EVIDENCE.md`의 증적 표현 정정이다. 두 항목 모두 COM 실행 결과와 정적 테이블의
`--verify` 정합을 바꾸지 않는 문서/도구 표현 문제이며, #2510 merge의 차단 사유가 아니었다.
본 PR은 그중 Windows+한컴 COM이 불필요한 부분을 정정한다.

## 변경과 판단

**항목 1 — ladder TSV SHA-256이 LF 체크아웃과 불일치**

`EVIDENCE.md` §4의 5종 SHA-256이 Windows CRLF 바이트로 기록돼, 저장소가 LF로 저장하는 실제 TSV의
해시(`git show HEAD:<path> | shasum -a 256`)와 어긋났다. LF 기준 값으로 재기록하고 줄바꿈 규칙과
재현식을 명시했다. 예: `ladder_한양신명조.tsv` LF `35e546ea…` vs 최초 커밋 CRLF `5eaec37c…`.

**항목 2 — preflight 5종 identity 아티팩트**

`preflight_report.tsv`가 per-face 프로세스 분할 실행(권장 경로)에서 매 실행마다 덮어써져 마지막
face(휴먼명조) 1행만 잔존했는데, `EVIDENCE.md` §2는 5종 커밋 증적이라 설명했다. `hy_ascii_ladder.py`
preflight를 `requested_face` 기준 누적 병합(`_merge_preflight_report`)으로 바꿔 per-face 실행에서도
5종 identity 행이 보존되게 하고, §2 라벨을 실제 파일 내용(콘솔 stdout vs 파일, 휴먼명조 1행만 보존)에
맞게 정정했다. `--verify`가 TSV↔배열 일치만 보증하고 HFT vs fallback identity는 preflight 아티팩트로만
입증됨을 유의로 기록했다.

한양 4종의 5행 실측 재보존은 COM 재실행이 필요해 본 PR 범위 밖이며, 원저자에게 별도 이슈로 안내하고
[#2430](https://github.com/edwardkim/rhwp/issues/2430)에서 추적한다. 본 PR의 harness 개선이 그 재실행 시
5행 보존을 보장한다.

## 검증

- `python3 tools/task2430/gen_metrics.py --ladder-dir tools/task2430/measured --verify`: 5종
  `95/95 exact match`, exit 0 (TSV 무변경, COM 불필요 — macOS 실행).
- 기록한 5종 LF SHA-256이 워크트리 실제 파일 `shasum -a 256`과 일치.
- `_merge_preflight_report` per-face 3회 누적 + 동일 face 재실행 dedup 단위 검증(헤더 포함 4행 유지).
- 변경 범위는 `tools/task2430/**`와 `mydocs/**`뿐. 실측 데이터(ladder TSV·정적 배열) 무변경.

## 권고

변경은 [`pr_2510_review.md`](pr_2510_review.md)가 명시한 비차단 후속 보완 2건 중 COM 불필요 부분을
정확히 대응하고, 실측 데이터를 바꾸지 않는다. 최신 PR head 기준 GitHub Actions가 통과하고 작업지시자
merge 승인이 있으면 collaborator self-merge 대상이다. 한양 4종 실측 재보존 후속은 별도 이슈와
[#2430](https://github.com/edwardkim/rhwp/issues/2430)으로 이관한다.
