---
kind: pr-review
status: active
---

# PR #3639 review — 자리차지 개체 문단의 절대 vpos 기준

| 항목 | 값 |
| --- | --- |
| 작성자 / base | planet6897 / `devel` |
| head 참고값 | `06c7b7a00ec44dae1035874eb32b852bab842936` |
| 관련 이슈 | Fixes #3637 |
| 신규 fixture | `samples/issue3637/press_release_topbottom_float.hwpx` |
| 권고 | 통합 PR로 반영, #3637 close |

`LINE_SEG.vertical_pos`를 단 기준 절대값으로 사용할 때 흐름 커서에 다시 더하지 않고 단 상단을
기준으로 삼는 수정이 실제 HWPX 재현 문서의 본문 소실을 막는다. focused regression 1건과 전체
release-test을 통과했다.

한글 2020 `PrintToPDFEx` 기준 PDF는 `pdf/issue3637/press_release_topbottom_float-2020.pdf`
(`80f282…e4ae`, 2쪽, MCP job `b9e530fa-1bbc-44ab-9c90-687481772419`, run_status 0,
validation ok)로 보존했다. rhwp는 3쪽이어서 전체 sweep은 페이지 수 불일치로 중단됐다. 공통 1–2쪽
sweep에서는 1쪽 본문이 페이지 밖으로 사라지지 않은 것을 사람이 확인했다. page 2는 rhwp의
후속 페이지와 PDF poster가 대응하지 않아 pixel/ink 지표를 merge 통과 근거로 쓰지 않는다.

대표 검토 asset: `mydocs/pr/assets/pr_3639_planet6897_visual_p001.png`
(page 1 proxy 8.08583%; 폰트·전반 layout 차이가 커 자동 지표는 보조값일 뿐이다).
