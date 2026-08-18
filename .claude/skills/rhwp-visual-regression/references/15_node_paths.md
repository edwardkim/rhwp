# 15 — 노드 경로를 읽는 법

경로는 render tree 의 안정 문자열이다. 예:

`Page/Body2/Column0/TextLine10/TextRun0`

- `Page` — 쪽 루트. 보통 0.00px 이면 전체 틀은 유지
- `PageBg0` — 쪽 배경
- `Body` / `Body2` — 본문 흐름 (마스터/본문 층)
- `ColumnN` — 단. N 이 바뀌면 다른 단
- `TextLineN` — 줄. 0 부터
- `TextRunN` — 같은 줄의 런
- `Table` / `Cell` — 표와 칸
- `Header` / `Footer` — 머리말·꼬리말. 본문 편집과 무관하면 회귀
- `Image` — 그림. 로고가 여기로 잡힌다

페이지 번호는 명령 `-p` 와 출력 `page 0` 모두 0 부터다.

대조 절차:

1. 편집 명령이 건드린 쪽·칸·필드 이름을 적는다.
2. STRUCT/OVER 가 가리키는 경로를 적는다.
3. 둘의 쪽·단·줄이 같은가.
4. 상위 `Page`/`PageBg0` 가 0px 인가.

카탈로그: `fixtures/node_paths.json`.
