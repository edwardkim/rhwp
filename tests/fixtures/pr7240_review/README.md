# PR #7240 단 나눔 편집 산출물

기존 `samples/issue7218/outline_headings.hwpx`를 통합 후보
`75a48488676d79a0357ba1cae6c863ac2120b668`의 다음 CLI 명령으로 편집했다.

```sh
rhwp edit insert-column-break samples/issue7218/outline_headings.hwpx \
  --para 3 --offset 0 -o tests/fixtures/pr7240_review/column_start.hwpx --json
```

기존 5개 문단을 보존하고 문단 3에 단 나눔 속성을 추가한다. 기존 원본은 중복 보관하지 않는다.
한컴 2020 MCP가 같은 파일에서 생성한 기준은 `pdf/pr7240/column-start-2020.pdf`이며
변환 provenance·SHA256·Native/fresh WASM overlay와 남은 차이는
`mydocs/pr/archives/pr_7240_review.md`를 따른다. UI Ctrl+Shift+Enter 분할 명령의 산출물과 구분한다.
