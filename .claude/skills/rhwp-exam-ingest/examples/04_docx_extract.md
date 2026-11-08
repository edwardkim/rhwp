# 예제 04_docx_extract — DOCX 추출

이 워크스루는 gym 과제가 아니다. 기존 helper 와 `rhwp build-from-ingest` 만 쓴다.
교차: `T04`. 동작: extract_docx.py. 이미지 입력은 경로·실제 치수·한도를 검증하고 산출 Picture/BinData와 실제 표시를 확인한다.

## 입력

사용자가 시험지 원본(또는 그 경로)을 준다. 원본은 읽기만 한다.

## 명령

```bash
bash .claude/skills/rhwp-exam-ingest/helpers/check_deps.sh --json
python3 .claude/skills/rhwp-exam-ingest/helpers/extract_docx.py \
    --json --dry-run "$DOCX" "$TMP"
python3 .claude/skills/rhwp-exam-ingest/helpers/extract_docx.py \
    "$DOCX" "$TMP"
rhwp build-from-ingest "$TMP/ingest.json" --media-dir "$MEDIA" -o "$OUT"
rhwp export-text "$OUT" -o "$TMP/txt"
rhwp dump "$OUT" > "$TMP/dump.txt"
```

## ingest 요지

- 스키마 `version: "1"`. 미지 필드 없음.
- 교차 픽스처/정지를 `T04` 로 확인.
- `auto_number` 정책을 10장에 맞게 고정.

## 정지

실패 봉투는 `references/14_failure_envelopes.md`. 성공이 아니면 `-o` 산출을 사용자에게 주지 않는다.

## 한계

build-from-ingest는 로컬 PNG/JPEG/GIF/BMP/TIFF를 Picture/BinData로 삽입한다. 이미지 오류는 INGEST_IMAGE_*로 진단하며 placeholder로 대체하지 않는다. 파일·자원·텍스트 보존과 실제 표시를 확인한다. 수식과 표 이미지는 편집 가능한 수식·표 구조가 아니다.
