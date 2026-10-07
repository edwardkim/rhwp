# #7491 검증 입력의 출처

- `center_align_first_line_indent_missing_column.hwp`: 기존 축소 샘플의 바이트 그대로다. 단 정의가 누락되어 MCP는 기본 여백으로 출력하며 기존 실패와73.57805% 비교를 보존한다.
- `imo-award-full-original.hwp`: 기상청 공개 보도자료 원문이다. 축소 샘플 내용은3쪽이며 원문 첫 `cold` 레코드가 single-column 정의를 갖는다. 원문 MCP PDF는 `pdf/semanticist21-20261005/pr7491/mcp/issue6190-full-original-2020.pdf`다.
- `samples/issue6190/center_align_first_line_indent.hwp`: 누락 단 정의를 공개 API로 복원한 작업 입력이다. 여백25mm, 본문과 글자모양·문단모양의 수치를 임의 변경하지 않았다. 과거 실패본과 같은 파일이라고 보고하지 않는다.
- `center_align_first_line_indent_edited.hwp`: 복원본 문단4 offset7 및 문단7 offset0에 각각 `가`를 삽입한 공개 API 출력이다. Native와 fresh WASM export 바이트가 동일하다.

생성은 저장소 루트에서 `node mydocs/pr/assets/semanticist21-20261005/pr7491-fixture-replay.mjs`를 실행한다. 이 스크립트는 ignored output에만 출력한다. 입력/MCP job/파일 SHA는 `mydocs/pr/assets/semanticist21-20261005/pr7491-mcp-references.json`, 중간90% 선행 증거는 `pr7491-interim-evidence.json`, 해석·회귀·최종 검증은 `mydocs/pr/archives/pr_7491_review_impl.md`에 연결한다. 복원본의100%와 편집본90.06833%를 과거 누락 입력의 개선율로 바꾸지 않는다.
