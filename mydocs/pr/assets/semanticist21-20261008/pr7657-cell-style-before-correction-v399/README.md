---
kind: evidence
status: active
canonical: mydocs/manual/pr_review/visual_fixture_evidence.md
last_verified: 2026-10-12
---

# 원 PR #7657 스타일 삭제의 저장 줄 원점 보정 전 증거

초기18개 중 [#7657](https://github.com/edwardkim/rhwp/pull/7657)의 기존
`tests/cases/deleted_style_keeps_nested_paragraph_styles.rs`와 같은 편집·저장 순서다.
정상 회귀 fixture/golden이나 해결 완료가 아닌 진단 증거이며 정확한 기존3개 의미 검사는 유지한다.
원본 samples를 수정하거나 삭제하지 않았다. 덧말 입력의 마지막 스타일 참조 설정은 기존 검사에
명시된 수동 모델 변경이며 정상 한컴 생성본으로 분류하지 않는다.

`generation.rs.txt`는 동일 순서를 저장한 독립 진단 helper다. 현재 생산 소스와 같은 cached
release-test library를 읽어 standalone rustc로 실행했으며 Cargo target을 변경하지 않았다.
6개 입력의 같은 바이트를 실제 한컴2020 runtime11.0.0.9136에서 PrintMethod0/1-up/
`hancom2020_pdf_driver_one_up`/input_preprocess none으로 출력했다.
원래 실물 문서는 SO-SUEOP.hwpx, 143E433F503322BD33.hwp,
36389301_결재문서본문_직장훈련계획_덧말.hwpx이며 최초 셀 사례는 API의 빈 문서 생성이다.
Print6개는 각각1/1/2/1/2/46쪽이며 현재 전쪽 Native/fresh 검증은 v398에서 진행 중이다.

후속 확인: v398은 Native/fresh 모두 실제 종료했고 같은 미달을 확인했다.
v406에서 저장 첫 줄 원점을 보정했지만, 원점 변경만으로 셀 높이와 뒤 문단 흐름이
복구되지는 않았다. 아래 한컴 저장 대조군은 첫 줄 원점 외에 표 Common 높이도
1282→3082HWPUNIT으로 바뀌었다. 초기 기록에서 빠진 이 차이가 100% 대조군의
측정 결과를 가렸으므로, 이를 renderer가 앞 간격을 이미 올바르게 측정한다는 근거로
사용하지 않는다. [후속 보정·반례·남은 측정 경로](../pr7657-batched-correction-v406/README.md)를 따른다.

| 편집 저장본 | 입력 SHA256 | 같은 입력의 독립 Print | PDF SHA256 | job |
| --- | --- | --- | --- | --- |
| [original-143e-after-delete.hwp](original-143e-after-delete.hwp) | `d4ce355e04edf942628a82d1722fad387f74afa70f855d025d470b9d295c90a6` | [original-143e-after-delete-d4ce355e04ed-2020.pdf](../../../../../pdf/semanticist21-20261008/original-pr7657-style-delete-v397/original-143e-after-delete-d4ce355e04ed-2020.pdf) | `90a0b8fb03a4f2389c65cd5e03acb8c9b8ff9a4d3127200dd5dfb6c258c8c174` | `6ad90c0a-3908-4f40-8eea-ac8ebfa148a9` |
| [original-cell-style-after-delete.hwp](original-cell-style-after-delete.hwp) | `a7b954842ef693b99f72eba020c450bb5554974fc1bea7131a758335bba1906d` | [original-cell-style-after-delete-a7b954842ef6-2020.pdf](../../../../../pdf/semanticist21-20261008/original-pr7657-style-delete-v397/original-cell-style-after-delete-a7b954842ef6-2020.pdf) | `df70f217fd2242aedb9e4d58f36be20cbadf9ff4366729c969852e43f73a4d73` | `972f84ba-5467-442b-8c51-fcbeb016012b` |
| [original-ruby-style-after-delete.hwp](original-ruby-style-after-delete.hwp) | `0b8e4afed62e11a71a67488e7ad8ad7e956a14d15078e26ba6d18c3e7778c95a` | [original-ruby-style-after-delete-0b8e4afed62e-2020.pdf](../../../../../pdf/semanticist21-20261008/original-pr7657-style-delete-v397/original-ruby-style-after-delete-0b8e4afed62e-2020.pdf) | `5b59276665ad9c9db2450432cf7260be6c3a593ace9030c2c213ba5d3029c074` | `7eeb7394-5704-4776-8cd7-3cce4367cb28` |
| [original-cell-style-after-delete.hwpx](original-cell-style-after-delete.hwpx) | `cd7c8fc9fde5fca72482408a216b70bb771590d2de4d3b50ae157298cb0559bd` | [original-cell-style-after-delete-cd7c8fc9fde5-2020.pdf](../../../../../pdf/semanticist21-20261008/original-pr7657-style-delete-v397/original-cell-style-after-delete-cd7c8fc9fde5-2020.pdf) | `c1f2b9ddd8a4bb1ebf193498f8625fcbac24fc9bf8e1c868324303d3a04beaf4` | `74deccc7-bbf9-438e-beef-e85aef255f68` |
| [original-ruby-style-after-delete.hwpx](original-ruby-style-after-delete.hwpx) | `1a2b454f56dc12c7fd3a4b2dc70b49f94d868fa3e29d932d8ca6733e793ab505` | [original-ruby-style-after-delete-1a2b454f56dc-2020.pdf](../../../../../pdf/semanticist21-20261008/original-pr7657-style-delete-v397/original-ruby-style-after-delete-1a2b454f56dc-2020.pdf) | `91e2e7f2a121ed6ac3067013a9e971c8e8b56d29ce0c8b6b1402ec95768fdaff` | `60f5897b-bebe-4fdf-ae47-d60eb20b8171` |
| [original-sosueop-after-delete.hwpx](original-sosueop-after-delete.hwpx) | `c43961693982d75b1803c8c0c96b20f8ccd87ce02d65f7af8aa408b4a94cf85c` | [original-sosueop-after-delete-c43961693982-2020.pdf](../../../../../pdf/semanticist21-20261008/original-pr7657-style-delete-v397/original-sosueop-after-delete-c43961693982-2020.pdf) | `4727e7a1c9aee21760b0e0b5548a27b8cea9b8a988d1736c368488e2178a5c25` | `3abc4929-9768-440e-aaaa-0c04c60e068f` |

## 직접 발견한 미충족과 원인 계층

최초 셀 사례의 HWP/HWPX Native 전체1쪽은 모두41.80704%다.
[실제 review](native-review_001.png)와 [overlay](native-overlay_001.png)를 직접 읽었다.
표 상단·너비와 앞 글자는 대응하지만 Native 셀 높이는25.1px, 한컴은41.1px다.
뒤 문단의 물리 흐름도 Native에서16px 앞당겨진다. 점수만의 문제가 아니며 보정 대상이다.

같은 생성 HWP를 실제 한컴2020으로 HWPX 저장한 [독립 정상 저장 대조군](hancom-saved-cell-style.hwpx)은
최초 셀 첫 LineSeg의 `vertpos=1200`, rhwp 생성본은0이다. 둘 다 `vertsize/textheight=1600`,
`baseline=1360`, `spacing=960`, `horzsize=40932`, `flags=393216`이며 셀 높이 선언282와
안여백141도 동일하다. 문단 앞 간격은1200HWPUNIT(96dpi에서16px)이다.
한컴은 문단ID1→0, 문단 모양12→11 등도 정규화했으므로 바이트 동일 입력으로 보고하지 않는다.

이 한컴 저장 대조군을 기존 Print와 현재 Native로 대조한 결과는100%다.
실제 Cell 높이는41.1px, 내부 빈 TextLine 위는175.3px이며 원 생성본은25.1/159.3px다.
뒤 문단 위도186.2→202.2px로16px 차이가 난다. 원 생성본41.80704%를 대조군100%로
대체하지 않으며 원 생성본의 보정과 fresh 검증이 남는다.

호출 경로는 `apply_cell_style_native`(formatting.rs:2196)의 문단/글자 모양 변경 →
`reflow_cell_paragraph` → `reflow_line_segs_impl`의 기존 원점 보존(빈 문단4478/4490,
일반 문단4827 이후) → 저장 첫 LineSeg → `cell_paragraph_spacing_before`(renderer/mod.rs:1588)의
저장 원점에 앞 간격이 포함된다는 소비 계약 → 높이 측정/실제 빈 줄 배치/뒤 문단 흐름이다.
현재 API는 앞 간격을 바꾼 뒤에도 옛0 원점을 보존한다. 한컴 정상 저장이1200을 생산하므로
생산 경로의 서식 변경·줄 원점 정합을 먼저 보정한다. renderer의 저장 정보 수용 조건을
합성 입력만으로 완화하거나 출력 높이를 clamp하는 수정은 하지 않는다.

실제 Native v315/fresh v323은 생산 커밋940ea1322617ccff5a7b47d6920ae96e94cfffad와
1111개 소스 해시가 같다. 원래 검사 실행·현재 입력 생성·실제 Print·Native 계측과
독립 저장 대조군을 구분한다. 같은16px 차이를 독립 기대값으로 검증한 뒤 보정하고,
원래 모든 관련 페이지의 Native/fresh90% 이상·확대 판독 후 필요한 관계 검사를 추가한다.

ignored 실행 기록: `original-pr7657-style-delete-v397-generate.json`,
`original-pr7657-style-delete-print-v397-result.json`, `original-pr7657-style-delete-visual-v398-result.json`,
`original-pr7657-style-delete-review-v399/result.json`, `original-pr7657-cell-style-hancom-save-v400-result.json`,
`original-pr7657-cell-origin-control-v401-result.json`. 중간 JSON/TSV/로그/SVG는 Git에 추가하지 않는다.
