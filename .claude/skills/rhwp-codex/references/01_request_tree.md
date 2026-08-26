# 요청 → 명령 판단 트리

정본: `mydocs/manual/agent_codex/01_판단트리.md`.
모르는 문서는 언제나 ① 파악에서 시작한다.

```text
요청
 ├─ 이 문서 뭐야?              → ① 파악   → 10장
 ├─ 데이터 뽑아 줘             → ② 수확   → 20장
 ├─ 고쳐 줘 / 채워 줘          → ③ 편집   → 30장  (-o, --dry-run)
 ├─ 다른 형식으로              → ④ 변환   → 40장
 ├─ 증명해 줘 / 믿어도 돼?     → ⑤ 검증   → 50장
 ├─ 이 문서 안전해?            → ⑥ 보안   → 60장
 ├─ 수백 개야 / 세션           → ⑦ 대량   → 80장
 ├─ 명령을 못 찾겠다           → capabilities --search → 70장
 └─ 렌더 버그 조사             → 85장 (개발자 전용, 통상 금지)
```

## 갈래 표

| 갈래 | 장 | 대표 명령 | 인계 |
|---|---|---|---|
| 파악 | 10 · 10_조회.md | `info`, `explain`, `digest`, `search` | `rhwp-doc-triage` |
| 수확 | 20 · 20_표와_데이터.md | `export-tables`, `table-to-csv`, `csv-to-table`, `extract-data` | `rhwp-table-exchange` |
| 편집 | 30 · 30_편집과_계획.md | `edit replace-text`, `edit set-cell`, `edit fill-fields`, `edit insert-image` | `rhwp-safe-edit` |
| 변환 | 40 · 40_변환과_렌더.md | `convert`, `export-hwpx`, `export-markdown`, `export-pdf` | `rhwp-visual-regression` |
| 검증 | 50 · 50_검증_사다리.md | `verify`, `ir-diff`, `replay`, `audit` | `rhwp-work-receipt` |
| 보안 | 60 · 60_보안.md | `inspect injection`, `inspect hidden-text`, `inspect unicode`, `armor` | `rhwp-security-sweep` |
| 대량 | 80 · 80_대량과_상주.md | `batch`, `mcp-serve` | `rhwp-bulk-pipeline` |

## 갈래를 못 정하면

1. `rhwp capabilities --search <키워드>`
2. 히트의 `name` 으로 이 표의 장을 연다
3. 0건이면 표면 밖(X03). 명령을 발명하지 않는다

자기서술 검색은 `--json` 이 아니라 `--search` 다. `capabilities` 단독은 전체 목록.
