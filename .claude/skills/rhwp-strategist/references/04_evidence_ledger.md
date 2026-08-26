# 04 Phase B — 근거 대장 (evidence.json)

정본: playbook §4, engagement.py `build_ledger` / `copy_coords`.

## 엔트리 두 종류

### kind: search

질문 키워드 × 문서마다 `rhwp search <doc> --json -- <keyword>`.
매치 하나 = EV 하나.

필수에 가까운 필드: `id`, `kind`, `question`, `keyword`, `file`, `quote`,
`context`, `command`. 좌표 키는 봉투에 있는 것만.

### kind: data

`extract-data` 가 광고되면 `--kind date` 와 `--kind amount` 를 문서마다.
`dataKind`, `quote`(raw), `normalized`, 있으면 `currency`/`unit`.

## 좌표 복사 — 발명의 금지

```python
COORD_KEYS = ("section", "paragraph", "page", "charOffset", "length", "cell", "textbox")
# copy_coords: {k: src[k] for k in COORD_KEYS if k in src}
```

봉투에 `page` 가 없으면 대장 엔트리에도 `page` 가 없다. `null` 로
채우거나 `0` 으로 추정하거나 1-based 로 고치지 않는다.

조판에 아직 안 올라간 문단, 숨은 누름틀, 일부 헤더/푸터 경로는
`page` 없이 나오는 것이 정상이다. 그 상태를 정직하게 보존하는 것이
좌표 계약이다.

## 절단과 실패

- `search --limit` 으로 잘리면 `truncatedSearches[]` 에
  `file`, `keyword`, `totalMatchCount`, `omittedCount`.
- search/extract-data 실패는 `failures[]` 에 `phase`, `file`, `reason`.
- search 호출이 **전부** 실패하면 대장을 만들지 않고 RuntimeError (exit 1).
- 일부만 실패하면 성공분으로 대장을 만들고 실패는 배열에 남긴다.

## command 필드

그 근거를 제3자가 재현하는 실행 명령이다. 예:

```
/path/rhwp search corpus/rfp/과업지시서.hwp --json -- 필수기능
```

고객이 "이 문장 어디서 왔나"를 물으면 EV id → file·좌표 → `command`
순으로 답한다.

## 0건 매치

어떤 질문의 키워드가 전 문서에서 0건이면 그 질문 id 로 묶인 search
엔트리가 없다. 골격은 그 절에 CLAIM 을 만들지 않고 "근거 없음"을 적는다.
0건은 오류가 아니다. 에이전트가 전망으로 빈 칸을 메우면 ST-FORECAST.

## 스키마 가드 (이 스킬의 계약 시험이 본다)

- `schemaVersion` == `"1"`
- `generatedBy` == `tools/strategist/engagement.py`
- `entryCount` == `len(entries)`
- 모든 `id` 가 `EV-n` 이고 중복 없음
- search 엔트리는 `question` 이 `Q` 로 시작
- 좌표 키 중 봉투에 없던 키가 생기지 않음 (픽스처 `missing_page`)

다음: [05_claim_gate.md](05_claim_gate.md).
