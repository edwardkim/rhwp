# 01 — 입력 정규화

네 가지 입력을 **페이지 PNG + (있으면) 텍스트 + 임베디드 이미지** 로 맞춘 뒤에야
Vision 과 `ingest.json` 이 같은 좌표계를 쓴다.

## 목표 산출

임시 디렉터리 하나 (`TMP=$(mktemp -d /tmp/rhwp-ingest.XXXXXX)`):

```
$TMP/
  page_001.png      # PDF·이미지. MD/DOCX 는 없을 수 있음
  page_002.png
  text.txt          # pdftotext 또는 DOCX 본문 또는 MD 원문 복사
  img/              # DOCX 임베디드, MD 가 가리키는 파일 복사본
  ingest.json       # Step 3 에서 작성
```

미디어 디렉터리는 따로 둔다. crop 결과가 여기로 들어간다.

```
$MEDIA_DIR/
  img/q1_graph.png
  img/q4_table.png
```

`build-from-ingest --media-dir "$MEDIA_DIR"` 가 `media[].id` 를 이 루트에서 찾는다.

## 형식별 정규화

### PDF

1. `check_deps.sh --json` — poppler 또는 ImageMagick.
2. `pdf_to_pngs.sh` — 기본 300 DPI. 스캔이 흐리면 400, 600 은 최댓값.
3. `page_001.png` 부터 번호가 **1부터, 3자리, 밑줄**. `page-1.png` 로 남기지 않는다.
   helper 가 pdftoppm 출력을 rename 한다.
4. `text.txt` 가 생기면 Vision 보조로만 쓴다. 텍스트 layer 가 틀린 줄바꿈을
   가지고 있어도 Vision 이 우선이다.

```bash
bash .claude/skills/rhwp-exam-ingest/helpers/pdf_to_pngs.sh \
    samples/2010-exam_kor.pdf "$TMP"
bash .claude/skills/rhwp-exam-ingest/helpers/pdf_to_pngs.sh \
    --json --dry-run samples/2010-exam_kor.pdf "$TMP" 300
```

### 이미지

변환하지 않는다. 사용자가 준 PNG/JPG 경로를 `page_001.png` 로 **복사하거나
그대로 Read** 한다. 여러 장이면 파일명 정렬 순서를 페이지 순서로 삼고,
사용자에게 순서를 확인한다. EXIF 회전은 ImageMagick `magick -auto-orient`
를 쓸 수 있으나, 이 helper 는 자동 회전하지 않는다. 회전이 필요하면
사용자에게 묻고, 동의하면 기존 `magick` 한 줄로 정규화한다. 새 helper 를
만들지 않는다.

### Markdown

1. MD 파일을 UTF-8 로 읽는다. BOM 이 있으면 제거한다.
2. `![alt](path)` 와 `<img src="path">` 를 모두 모은다.
3. 상대 경로는 MD 파일 위치 기준. 없으면 F07.
4. 이미지 파일을 `$MEDIA_DIR/img/` 로 복사하고, `media[].id` 는
   `img/<basename>` 으로 통일한다.
5. ATX 헤더 (`# 1.`, `## 3.`) 또는 줄 시작 `1.` 을 문항 후보로 삼는다.

MD 를 HWPX 로 직접 변환하는 CLI 는 없다. 반드시 ingest.json 을 경유한다.

### DOCX

```bash
python3 .claude/skills/rhwp-exam-ingest/helpers/extract_docx.py \
    학원모의고사.docx "$TMP"
```

- `text.txt` — 단락 단위.
- `img/` — `word/media/` 원본 바이트.
- python-docx 가 있으면 단락 경계가 산다. 없으면 `<w:t>` 토큰이 붙는다.
  fallback 이어도 Vision 이 이미지를 보면 문항 구조는 복구된다.

표 안의 문항은 `text.txt` 만으로 깨질 수 있다. 그 경우 표를 이미지로
내리지 말고, DOCX 를 PDF 로 보낸 뒤 PDF 경로를 타라고 사용자에게 제안한다.
이 스킬에 `docx_to_pdf` helper 는 없다.

## 정규화가 아닌 것

- OCR. Vision 이 페이지 PNG 를 읽는다.
- PDF 텍스트를 그대로 stem 에 붙여 넣기. 줄바꿈·두 단 조판이 섞인다.
- HWP/HWPX 입력. 이미 한글 파일이면 이 스킬이 아니다.
- 폴더 재귀 변환 CLI. 에이전트가 파일 목록을 돌린다.

## dry-run

세 helper 모두 `--dry-run` 을 받는다. 파일을 만들지 않고 엔진·경로·계약을
검사한다. 계약 시험은 이 경로를 바이너리 없이 문서·픽스처로 고정하고,
환경에 bash 가 있으면 실제로 한 번 호출해도 된다.

```bash
bash helpers/pdf_to_pngs.sh --json --dry-run in.pdf /tmp/out
bash helpers/crop_image.sh --json --dry-run page.png 10 20 100 80 out.png
python3 helpers/extract_docx.py --json --dry-run in.docx /tmp/out
```

DPI 범위(72–600)와 bbox 정수 계약은 dry-run 에서도 같은 종료 코드를 낸다.

## 실패를 정규화로 숨기지 말 것

| 증상 | 코드 | 다음 |
| --- | --- | --- |
| PDF 없음 | `PDF_SRC_MISSING` | 경로 확인 |
| poppler·magick 없음 | `PDF_MISS_TOOLS` | 설치 안내 |
| DPI 30 또는 1200 | `PDF_DPI_RANGE` | 72–600 |
| DOCX 없음 | `DOCX_SRC_MISSING` | 경로 확인 |
| python-docx 없음 | (exit 0, fallback) | 경고만. 중단하지 않음 |

픽스처: `fixtures/helpers/input_kind.json`, `fixtures/envelopes/`.
