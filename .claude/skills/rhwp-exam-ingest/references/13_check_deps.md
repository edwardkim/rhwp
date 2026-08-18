# 13 — check_deps.sh 실패 봉투

의존성 점검은 helper 한 개다. 새 CLI 가 아니다.

```
check_deps.sh [--json]
```

사람용 표는 **stderr**. `--json` 봉투는 **stdout**.
필수 누락이면 exit 1. 선택 누락만이면 exit 0.

## 심각도

| code | severity | exit | 막는 것 |
| --- | --- | --- | --- |
| `DEP_MISS_RHWP` | required | 1 | `build-from-ingest` 전부 |
| `DEP_MISS_IMAGEMAGICK` | required | 1 | `crop_image.sh`, PDF magick fallback |
| `DEP_MISS_POPPLER` | pdf_input | 0* | `pdftoppm` 권장. magick 있으면 PDF 가능 |
| `DEP_MISS_PDFTOTEXT` | optional | 0 | 없음. Vision 만으로 진행 |
| `DEP_MISS_PYTHON3` | docx_input | 0* | `extract_docx.py` |
| `DEP_MISS_PYTHON_DOCX` | docx_input_soft | 0 | 없음. zip 정규식 fallback |

\* 필수(rhwp/ImageMagick)가 이미 빠졌으면 전체 exit 는 1.
poppler 만 없어도 필수 통과로 본다. PDF 입력이면 에이전트가
`pdf_to_pngs.sh` 를 치기 전에 magick 이 있는지 다시 본다.
magick 도 없으면 `PDF_MISS_TOOLS`.

## JSON 모양

```json
{
  "schemaVersion": "1.0",
  "helper": "check_deps.sh",
  "ok": false,
  "rhwp": null,
  "imagemagick": null,
  "pythonDocx": false,
  "missingRequired": ["rhwp", "imagemagick"],
  "missingOptional": ["pdftoppm", "pdftotext", "python-docx"],
  "envelopes": [
    {
      "code": "DEP_MISS_RHWP",
      "severity": "required",
      "tool": "rhwp",
      "exit": 1,
      "hint": "cargo build --release 또는 cargo run --bin rhwp",
      "blocks": ["build-from-ingest"]
    }
  ]
}
```

## 입력 종류별 해석

| 입력 | poppler 없음 | magick 없음 | python3 없음 | python-docx 없음 |
| --- | --- | --- | --- | --- |
| PDF | magick fallback. 둘 다 없으면 정지 | crop 불가 + PDF fallback 불가 | 무관 | 무관 |
| PNG | 무관 | crop 필요할 때만 정지 | 무관 | 무관 |
| MD | 무관 | crop 없을 수 있음 | 무관 | 무관 |
| DOCX | 무관 | 이미지 crop 시 | 정지 F04 | fallback, 경고 |

## 설치 힌트 (사용자 동의 후)

- macOS: `brew install poppler imagemagick`
- Debian: `apt install poppler-utils imagemagick`
- python-docx: `pip install python-docx`
- rhwp: `cargo build --release`

에이전트가 brew/apt/pip 를 침묵 실행하지 않는다. 힌트를 보여 준다.

## 픽스처 봉투

기계 시험은 라이브 환경을 가정하지 않는다. 아래 파일은 **예상 모양**이다.

- `fixtures/envelopes/check_deps_ok.json`
- `fixtures/envelopes/check_deps_miss_poppler.json`
- `fixtures/envelopes/check_deps_miss_imagemagick.json`
- `fixtures/envelopes/check_deps_miss_python_docx.json`
- `fixtures/envelopes/check_deps_miss_rhwp.json`

라이브 `check_deps.sh --json` 의 `code` 집합이 이 파일들의
`envelopes[].code` 를 덮는지는 계약 시험이 검사한다.
