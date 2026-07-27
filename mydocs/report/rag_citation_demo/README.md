---
kind: guide
status: active
canonical: mydocs/report/rag_citation_demo/README.md
last_verified: 2026-07-27
---

# 실제 CLI 작동 사례 — 검색으로 근거 '쪽'을 답하고 그 쪽만 렌더해 인용 (RAG citation)

> 여정: [버그 헌팅 playbook](../../manual/bug_hunting_playbook.md) "공고 검색 → 근거 조항 위치 → 해당 쪽만 렌더".
> 실행: `rhwp` CLI만으로 처음부터 끝까지. 정답지 = 실제 `export-svg` 렌더.

## 무엇을 증명하나

에이전트가 긴 공문서에서 어떤 조항의 **근거를 인용**하려면 세 가지가 순서대로 돼야 한다.

1. 문서를 검색해 매치를 찾는다 (`search`)
2. 그 매치가 **몇 쪽**인지 안다 (search 의 `page`)
3. 그 쪽만 렌더해 근거로 제시한다 (`export-svg -p <page>` / `export-png -p`)

평문만 뽑아 외부에서 찾으면 "몇 쪽"의 주소가 소멸해 근거 제시가 불가능하다. rhwp 는 조판 엔진이 있어 검색 결과에 페이지를 실어주고, 그 페이지만 렌더해 루프를 닫는다.

## 실측 흐름

현재 `devel` 기준 release 바이너리로 같은 명령을 다시 실행하면 검색 결과는 **2건**이며,
두 결과 모두 `page: 3`(0기준)을 가리킨다. `export-svg -p 3`은 실제 4번째 쪽 파일
`…_004.svg` 하나를 만들고, 같은 쪽의 render tree에는 검색 문구가 `TextRun`으로 존재한다.

![search가 가리킨 실제 4번째 쪽 렌더](cited-page.png)

## 재현 (실제 35쪽 정부문서)

대상: `samples/2022년 국립국어원 업무계획.hwp` (35쪽).

```bash
# 1) 검색 — 매치마다 구역·문단·페이지·문자 오프셋
rhwp search "samples/2022년 국립국어원 업무계획.hwp" "한국어교육 정책" --json
# → matchCount:2, 두 match 모두 page = 3 (0기준),
#   context: "□ 한국어교육 정책 민관 협의 강화 및 교육과정‧교재 활용성 확대"

# 2) search 가 답한 그 쪽만 렌더 (page 와 -p 는 둘 다 0기준으로 일관)
rhwp export-svg "samples/2022년 국립국어원 업무계획.hwp" -p 3 -o out
# → out/…_004.svg 하나만 생성

# 3) 같은 쪽 render tree에서 인용문이 실재하는지 기계 확인
rhwp export-render-tree "samples/2022년 국립국어원 업무계획.hwp" -p 3 -o tree
rg "한국어교육 정책 민관 협의 강화" tree/render_tree_004.json
# → TextRun에서 일치 ✅ → 인용 루프 닫힘
```

위 그림이 2단계에서 렌더된 페이지이며, search가 돌려준 `context`와 페이지 표제
"□ 한국어교육 정책 민관 협의 강화 …"가 일치한다.

## 인덱싱 계약 (한 번에 헷갈리기 쉬운 지점)

`search` 의 `page` 와 `export-svg`/`export-png` 의 `-p` 는 **모두 0기준**이다([CLI 매뉴얼](../../manual/cli_commands.md) §search·§export). 따라서 `search … | jq '.matches[0].page'` 값을 그대로 `-p` 에 넘기면 정확히 그 쪽이 렌더된다. (사람에게 보일 때만 `.page + 1` 로 1기준 표기.) `export-svg` 의 **파일명**은 `_001` 부터의 1기준이라 파일명과 `-p` 인덱스를 혼동하지 않도록 주의한다.

## 검증 범위와 관련 격차

- 이 데모가 검증하는 범위는 위 검색어의 **2개 match가 모두 page 3을 가리키고**, 실제
  `render_tree_004.json`에 해당 표제가 존재한다는 인용 루프다.
- 페이지 분할 표의 검색 주소 정확성은 별도 이슈
  [#3403](https://github.com/edwardkim/rhwp/issues/3403)의 범위다. 이 데모의 2개 match로
  광역 정확도나 분할 표 건수를 추정하지 않는다.

## 관련

- 방법론: [버그 헌팅 playbook](../../manual/bug_hunting_playbook.md)
- 명령 계약: [CLI 명령어 매뉴얼](../../manual/cli_commands.md)
- 발견 격차: [#3403](https://github.com/edwardkim/rhwp/issues/3403)
