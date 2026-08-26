# 12 — rhwp build-from-ingest

기존 CLI 한 줄이 이 스킬의 조립 전부다. 새 플래그를 만들지 않는다.

```
rhwp build-from-ingest <ingest.json> [--media-dir <dir>] -o <out.hwpx>
```

정본: `mydocs/manual/cli_commands.md` §build-from-ingest.

## 인자

| 인자 | 필수 | 의미 |
| --- | --- | --- |
| `<ingest.json>` | 예 | schema v1 문서 |
| `-o` / `--output` | 예 | 산출 HWPX. 빠지면 사용법 오류 |
| `--media-dir` | 이미지 있으면 | `media[].id` · `stem_blocks[].ref` 해석 루트 |
| `--json` | 아니오 | 기계 봉투 (questionCount 등). 매뉴얼 지식 지도 참고 |

이 명령은 PDF/HWP 를 분석하지 않는다. JSON 만 조립한다.

## 레시피

```bash
rhwp build-from-ingest tools/rhwp-ingest/schema/sample_minimal.json \
    -o output/poc/ingest/sample_minimal.hwpx

rhwp build-from-ingest "$TMP/ingest.json" \
    --media-dir "$MEDIA_DIR" \
    -o output/exam/2024_국어.hwpx
```

산출물은 `output/` 아래 (gitignore). 원본 PDF 는 읽기만 한다.

## 성공 후 게이트 (이 스킬의 일부)

```bash
rhwp export-text output/exam/2024_국어.hwpx -o "$TMP/txt"
rhwp dump output/exam/2024_국어.hwpx > "$TMP/dump.txt"
unzip -l output/exam/2024_국어.hwpx | head
```

- export-text 에 각 `stem` 과 `choices[].text` 가 보이는지.
- `"N. N."` 중복 번호가 없는지 (`auto_number` 사고).
- 공유 지문이 한 번만 나오는지.
- media 가 있으면 `BinData/` 또는 dump 의 Picture 흔적. 없어도
  #182 한계일 수 있다. 텍스트 게이트를 우선한다.

`export-svg` 는 smoke 다. 원본 PDF 와 픽셀 일치를 증명하지 않는다.

## 실패

| 증상 | 원인 | 다음 |
| --- | --- | --- |
| `-o` 누락 메시지 | 필수 인자 | F15 |
| unknown field / deny | 스키마 밖 키 | F11. JSON 수정 |
| boxed text 필드 | #3358 | F12 |
| 이미지 파일을 못 찾음 | `--media-dir` 또는 id | 경로 확인 |
| version ≠ `"1"` | 오타 | `"1"` |

빌더 패닉을 이 스킬에서 "고치기" 위해 `exam_paper.rs` 를 열지 않는다.
재현 JSON 을 이슈로 남긴다.

## 발명 금지

```
# 없음
rhwp build-from-ingest --from-pdf exam.pdf
rhwp build-from-ingest --ocr
rhwp exam ingest
```

PDF 는 helper + Vision + JSON 이다.

픽스처: `fixtures/envelopes/build_missing_o.json`,
`fixtures/transcripts/pdf_to_hwpx.json`.
