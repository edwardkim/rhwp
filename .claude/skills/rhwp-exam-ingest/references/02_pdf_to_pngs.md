# 02 — pdf_to_pngs.sh

PDF 시험지를 페이지 PNG 로 바꾼다. Vision 의 입력이다.
이 스크립트는 poppler `pdftoppm` 을 우선하고, 없으면 ImageMagick 으로 넘긴다.

## 계약

```
pdf_to_pngs.sh [--json] [--dry-run] <input.pdf> <out_dir> [<dpi>]
```

| 항목 | 값 |
| --- | --- |
| 기본 DPI | 300 |
| DPI 범위 | 72–600 정수 |
| 페이지 이름 | `page_001.png`, `page_002.png`, … |
| 텍스트 보조 | `pdftotext -layout` → `out_dir/text.txt` (실패해도 본 변환은 성공) |
| stdout `--json` | `{schemaVersion,helper,ok,code,pages,engine,dpi,textLayer}` |
| dry-run | 변환하지 않음. 엔진·planned 명령만 |

## 엔진 선택

1. `pdftoppm` — `pdftoppm -r $DPI -png "$INPUT" "$OUTDIR/page" -f 1`
   산출은 `page-1.png`. helper 가 `page_001.png` 로 rename.
   `10#$n` 로 8진수 함정 (`08`, `09`) 을 피한다.
2. `magick` — `magick -density $DPI "$INPUT" "$OUTDIR/page_%03d.png"`
   ImageMagick 페이지 번호는 0부터일 수 있다. 그 경우 Vision 루프는
   존재하는 파일을 정렬해 읽고, `page_000.png` 가 있으면 첫 페이지로 취급한다.
   rename 은 pdftoppm 분기에만 있다.
3. `convert` — ImageMagick 6 동일.

셋 다 없으면 `PDF_MISS_TOOLS` exit 2. 봉투:

```json
{
  "schemaVersion": "1.0",
  "helper": "pdf_to_pngs.sh",
  "ok": false,
  "code": "PDF_MISS_TOOLS",
  "message": "오류: pdftoppm / magick / convert 중 하나가 필요합니다 (poppler-utils 또는 ImageMagick)"
}
```

## 종료 코드

| exit | code | 의미 |
| --- | --- | --- |
| 0 | `PDF_OK` | 변환 또는 dry-run 통과 |
| 1 | `PDF_ARGS` | 인자 부족 |
| 1 | `PDF_SRC_MISSING` | 파일 없음 |
| 2 | `PDF_MISS_TOOLS` | poppler/ImageMagick 없음 |
| 4 | `PDF_DPI_RANGE` | DPI 가 72–600 밖이거나 비정수 |

## 레시피 — 수능 국어 PDF

```bash
TMP=$(mktemp -d /tmp/rhwp-ingest.XXXXXX)
bash .claude/skills/rhwp-exam-ingest/helpers/pdf_to_pngs.sh \
    "downloads/2024_수능_국어.pdf" "$TMP" 300
# 완료: 20 페이지 → /tmp/rhwp-ingest.XXXXXX/page_NNN.png
# 텍스트 layer: /tmp/rhwp-ingest.XXXXXX/text.txt
```

Vision 은 `page_001.png` 부터 읽는다. 표지·주의사항 페이지도 읽되,
문항이 없으면 ingest 에 빈 문제를 넣지 않는다. 머리말 후보
(`국어 영역`, `홀수형`) 는 `header_text` / `form_label` 로 옮긴다.

## 레시피 — 흐린 스캔 PDF

300 에서 번호가 뭉개지면 DPI 를 올린다. 600 을 넘기지 않는다.

```bash
bash .claude/skills/rhwp-exam-ingest/helpers/pdf_to_pngs.sh \
    scan_학력평가.pdf "$TMP" 400
```

그래도 안 되면 F10. 사용자에게 원본 또는 재스캔을 요청한다.
`pdftotext` 가 빈 파일이면 스캔본이다. Vision 만으로 간다.

## 레시피 — dry-run 게이트

CI 나 계약 시험은 실제 PDF 없이 스크립트 존재와 플래그만 검사한다.
라이브 환경에서 파일이 있으면:

```bash
bash .claude/skills/rhwp-exam-ingest/helpers/pdf_to_pngs.sh \
    --json --dry-run "$PDF" /tmp/out 300
# {"ok":true,"code":"PDF_OK","dryRun":true,"engine":"pdftoppm",...}
```

없는 파일:

```bash
bash .claude/skills/rhwp-exam-ingest/helpers/pdf_to_pngs.sh \
    --json --dry-run /no/such.pdf /tmp/out
# exit 1, code PDF_SRC_MISSING
```

dry-run 도 파일 존재를 검사한다. "명령만 조립" 이 아니라
**계약을 같은 코드 경로로 검증** 하는 것이 목적이다.

## 하지 말 것

- `pdftoppm` 출력을 그대로 ingest media 경로로 쓰기 (`page-1.png` ≠ `page_001.png`).
- 텍스트 layer 를 stem 에 통째로 복사. 두 단 시험지는 왼쪽·오른쪽이 섞인다.
- Ghostscript 전용 경로를 helper 에 추가. 엔진은 위 세 개로 고정.
- PDF 를 직접 `build-from-ingest` 에 넘기기. 그 명령은 JSON 만 받는다.
