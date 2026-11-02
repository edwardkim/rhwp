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
| `--media-dir` | 아니오 | 현재는 디렉터리 존재만 확인. 이미지 바이트를 읽지 않음 |
| `--json` | 아니오 | 기계 봉투 (questionCount 등). 매뉴얼 지식 지도 참고 |

이 명령은 PDF/HWP를 분석하지 않는다. JSON만 조립한다.

현재 `build-from-ingest`는 이미지 파일을 읽어 Picture/BinData로 연결하지 않고
`[이미지: …]` placeholder 텍스트를 만든다. `--media-dir`은 디렉터리 존재만 확인한다.
**그림이 필수이면 F18로 중단한다.** 경로·크기·배치 변경만으로 해결되지 않으며,
placeholder 결과를 이미지 포함 시험지로 제공하지 않는다.


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
- media가 있으면 이미지 파일과 Picture 참조를 모두 확인한다.
  현재 빌더에는 삽입 경로가 없으므로 그림 필수이면 F18로 중단한다.
  올바른 경로·크기·배치로 다시 실행해도 해결되지 않는다.

`export-svg` 는 smoke 다. 원본 PDF 와 픽셀 일치를 증명하지 않는다.

## 실패

| 증상 | 원인 | 다음 |
| --- | --- | --- |
| `-o` 누락 메시지 | 필수 인자 | F15 |
| unknown field / deny | 스키마 밖 키 | F11. JSON 수정 |
| boxed text 필드 | #3358 | F12 |
| 그림이 placeholder로 나옴 | ingest 이미지 삽입 미구현 | F18. 그림 필수이면 중단. 경로·배치 재시도로 해결되지 않음 |
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
