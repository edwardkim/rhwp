# planet6897 PR #6586--#6649 통합 후보 시각 증적

- 검토일: 2026-09-02
- 기준: `upstream/devel` `2edbe62e5dc74db58c33df2c006ae587f86a1a71`
- 기능 통합 후보: `review/planet6897-open-batch-20260902`의 문서 기록 전 HEAD `208a18b8d7cd86568a3b1c15e026f202454631a9`
- 출력기: `target/pr-review/release-test/rhwp export-png --profile high-quality`; HWPX는 저장 버전에 맞춰 `--compat 2022` 또는 `2024`를 지정했다.
- 무결성: 모든 보존 PNG의 SHA-256은 [manifest](../assets/pr_6586_6649_planet6897_integration_20260902/manifest.sha256)에 기록했다.

## 직접 확인 범위

| PR | 원문 또는 재현물 | 후보 관찰 | 자산 |
| --- | --- | --- | --- |
| #6586 | `36399617`, HWPX 2020, p1 | 페이지 고정 블록을 포함한 1쪽 구조 | [PNG](../assets/pr_6586_6649_planet6897_integration_20260902/review_6586_issue6535_page_001.png) |
| #6604, #6610 | `36331407`, HWPX 2020, p1 | 기준의 세로 밀림/`359.4px` overflow가 후보에서 같은 줄 배치로 해소됨 | [before](../assets/pr_6586_6649_planet6897_integration_20260902/review_6604_6610_issue6601_before_page_001.png) / [after](../assets/pr_6586_6649_planet6897_integration_20260902/review_6604_6610_issue6601_after_page_001.png) |
| #6613 | `18098267`, HWP5 2020, p3 | 마지막 쪽 표와 각주가 물리 페이지 안에서 끝남 | [PNG](../assets/pr_6586_6649_planet6897_integration_20260902/review_6613_issue4915_page_003.png) |
| #6615--#6625 | `156627451`, HWPX p2 | EMF 도해 아이콘이 본문 카드 안에 유지됨. 최종 누적 스택 관찰임 | [PNG](../assets/pr_6586_6649_planet6897_integration_20260902/review_6615_6625_issue6577_page_002.png) |
| #6627 | 재현 HWPX 및 `156745900` p2/p31 | synthetic border는 직접 확인. 원문은 font fallback 때문에 테두리 구조만 확인 | [synthetic](../assets/pr_6586_6649_planet6897_integration_20260902/review_6627_synthetic_cellzone_border.png), [p2](../assets/pr_6586_6649_planet6897_integration_20260902/review_6627_source_page_002_font_fallback.png), [p31](../assets/pr_6586_6649_planet6897_integration_20260902/review_6627_source_page_031_font_fallback.png) |
| #6633 | `156658611`, HWPX 2018, p1 | 표가 문단 상단에 겹치지 않는 배치 구조. font fallback 한계 있음 | [PNG](../assets/pr_6586_6649_planet6897_integration_20260902/review_6633_issue6614_page_001_font_fallback.png) |
| #6636 | `30307`, HWP5 p3/p5 | 후보 페이지 출력. U+2007 advance 자체는 focused test가 비율로 검증 | [p3](../assets/pr_6586_6649_planet6897_integration_20260902/review_6636_issue6597_page_003.png), [p5](../assets/pr_6586_6649_planet6897_integration_20260902/review_6636_issue6597_page_005.png) |
| #6649 | `2744465`, HWP5 2020, p1 | anchor와 표가 페이지 테두리 안에 유지됨 | [PNG](../assets/pr_6586_6649_planet6897_integration_20260902/review_6649_issue6598_page_001.png) |

## 기준 대조와 한계

- #6604/#6610만은 같은 원문을 기준 `upstream/devel`과 후보에서 각각 직접 출력해 전후를 비교했다. 후보의 3쪽 `2.7px` overflow는 기준에도 동일하므로 이 통합 후보가 새로 만든 회귀가 아니다.
- 나머지는 원문 저장본 또는 재현물을 후보에서 직접 렌더했다. 현 세션에는 HWP 2024 converter MCP 실행 경로가 없어 새 Hancom PDF를 생성하거나 픽셀 대조하지 않았다.
- `156745900`과 `156658611`은 `한컴바탕`/`함초롬` 등 원문 글꼴이 현재 Mac에서 해석되지 않아 텍스트가 fallback 상자로 출력된다. 따라서 두 원문 PNG는 구조·테두리 관찰만 기록하며, 글자 모양·색·두께·좌표의 Hancom 동등성 증거가 아니다.
- 기여자의 PR 본문에 있던 Hancom 캡처는 기여자 증거로만 취급했고, 이 파일의 stable PNG에는 포함하지 않았다.
