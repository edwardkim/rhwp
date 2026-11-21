# PR #9268 검토 — 한컴 Print PDF와 실루엣 TSV의 생성 근거

**작성자 self-review · 승인.** 검토한 코드 head는 `e331f849e5988d96c38b639fe0a8222545a5fc31`이다.

### 한컴 Print 영수증 (#9263)

`refresh_oracle_pdfs.py --receipt-dir`에서 시작·상태·다운로드를 따라가 실제로 제출한 사본, 같은 job의 한컴 Print 조건, 내려받은 바이트가 연결되는지 검토했다. PDF와 영수증은 각 목적지에서 완성한 뒤 게시한다. 두 게시 사이에 중단된 경우의 복구 절차는 [변환 매뉴얼](../../manual/mcp_hwp2024Convert_usage.md)에 있다.

- [보존된 영수증](../assets/hyunmyungjanelee_20261112/mcp-print-reference/receipts.json)의 MCP Print 33건을 당시 원본·PDF로 오프라인 재생했다. 당시 기록으로 상태·다운로드 응답을 구성해 새 작성기와 기존 감사기에 전달했고, 두 해시가 이전 기록과 같고 33건 모두 `verified`였다. 수동 Print 1건은 범위 밖이다. 새 MCP 인쇄는 실행하지 않았다.
- oracle_public 검사 129건 및 PDF 선택 검사 16건이 통과했다. 원본 교체는 `mismatch`로 보고됐고 다른 job·출력 모드·해시·크기·preflight 거부와 작업 중 원본 변경에는 영수증이 남지 않았다.

### 페이지별 TSV 식별 (#9264)

TSV 작성이 끝난 시점의 SHA-256과 렌더링 호출에서 확인한 DPI가 분할 및 전체 manifest에 전달되는지 검토했다. 이미지만 제공하는 경로에서는 원래 DPI를 추정하지 않는다. 검증 방법은 [시각 증거 절차](../../manual/pr_review/visual_fixture_evidence.md)에 있다.

- 한컴 Print 2쪽짜리 `empty-host-split-table.hwp`를 동일한 Native Skia 실행물과 96 DPI로 새 실행·재개한 뒤 PNG 쌍만 재채점했다. 세 TSV의 SHA-256은 모두 `75a9bab7c2e3c98ae402a8caf9e39fecf6b6081c5a8aa222f99b01cf9da24690`였다. 분할 TSV도 각각의 바이트 해시와 일치했다. 문서 측정 DPI는 96, PNG 재채점의 값은 `null`이었다.
- 97 DPI로 바꿔 요청한 재개는 거부됐다. Visual Sweep 도구 검사 269건이 통과했다. 해당 변경은 파일 식별 정보를 추가하며 점수 계산과 판정 기준에는 영향을 주지 않는다.
- [코드 CI](https://github.com/edwardkim/rhwp/actions/runs/44778238621)가 통과했다. 검토 기록까지 포함한 head의 검사를 확인한 뒤 병합한다.
