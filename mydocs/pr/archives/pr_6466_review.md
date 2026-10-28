---
kind: pr-review
status: active
pr: 6466
issue: 6374
---

# PR #6466 검토 - Oracle PDF 형식·엔진 fail-closed 선택

## 결론 - 수용 후보, CI 대기

[PR #6466](https://github.com/edwardkim/rhwp/pull/6466)는 Oracle PDF의 출처를 원본 상대 경로,
원본 형식(HWP/HWPX), 출력 엔진(2020/2024)까지 확인 가능한 canonical 파일로 한정한다. 구현 후보
head는 `630a59867293e065a1285d3de6515fe25d87fd04`이며, required CI 성공 전에는 병합하지 않는다.

## 변경 판단

- `oracle_pair_index.py --args`는 형식·엔진을 확인할 수 없는 legacy PDF를 자동 기준으로 선택하지
  않는다. canonical 후보가 없거나 2020과 2024 후보가 함께 있으면 비교 인자를 출력하지 않고
  fail-closed로 종료하며, 이 경우 `--engine`을 명시해야 한다.
- canonical PDF 이름은 `samples/` 상대 경로와 source stem, HWP/HWPX 형식, engine을 모두 보존한다.
  서로 다른 source 경로나 저장 버전의 동명 파일을 같은 기준 PDF로 합치지 않는다.
- MCP 재산출은 `rhwp info --json`의 `lastSavedWith.product`로 engine을 선택하고, 시작·상태 조회·다운로드를
  동일 endpoint에 고정한다. endpoint별 worker는 하나만 사용해 원격 한컴 worker를 과도하게 병렬화하지
  않는다.
- 이 PR은 Oracle PDF provenance와 선택 안전성을 보정할 뿐 renderer를 변경하지 않는다. 따라서
  `samples/2025 행정업무운영 편람(최종).hwpx`의 rhwp 382쪽과 최신 한컴 384쪽 차이를 수용하거나
  해결하는 변경이 아니다.

## 검증 기록

- `/opt/homebrew/bin/python3 tools/test_oracle_pdf_selection.py` - 6 passed
- `cargo fmt --all`, `cargo fmt --all -- --check`, `git diff --check` - passed
- MCP Oracle PDF 재산출 - 대상 319, 기존 canonical PDF 제외 234, 실패 0
- 이번 PR 범위 PDF 547개는 `pdfinfo`에서 모두 양의 페이지 수를 반환했다.
- `pdf/basic/Hyper(hwp2010)-hwp-2020.pdf`와 `pdf/basic/request-hwp-2020.pdf`는 hyperlink annotation
  destination 경고가 있었지만, `pdfinfo`는 성공했고 페이지 수 검증을 통과했다. 사용자가 Acrobat에서
  정상적으로 열리는 것을 확인했다.

## 후속 상태

- PR #6466은 tools와 Oracle PDF asset을 함께 변경하므로 review-only fast-pass 대상이 아니다. 최신
  head의 full required CI와 CodeQL 결과, mergeability를 확인한 뒤 병합 여부를 재판정한다.
- PR 본문의 `Closes #6374`는 병합 후에만 Issue #6374를 자동 종료한다. 그 전에는 issue를 열어 둔다.
