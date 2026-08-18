# 14 — 실패 봉투

실패는 예외가 아니라 데이터다. helper 와 CLI 의 종료 코드·code 문자열을
읽고 다음 행동을 고른다. 새 봉투 스키마를 rhwp 에 추가하지 않는다.

## 공통 모양 (helper --json)

```json
{
  "schemaVersion": "1.0",
  "helper": "crop_image.sh",
  "ok": false,
  "code": "CROP_BBOX_NOT_UINT",
  "message": "오류: bbox 는 10진 정수여야 합니다 …"
}
```

성공도 같은 껍데기, `ok: true`.

## helper 코드 목록

| helper | code | exit | 의미 |
| --- | --- | --- | --- |
| pdf_to_pngs | `PDF_OK` | 0 | 변환/dry-run |
| pdf_to_pngs | `PDF_ARGS` | 1 | 인자 |
| pdf_to_pngs | `PDF_SRC_MISSING` | 1 | 파일 없음 |
| pdf_to_pngs | `PDF_MISS_TOOLS` | 2 | poppler/magick 없음 |
| pdf_to_pngs | `PDF_DPI_RANGE` | 4 | DPI 72–600 |
| crop_image | `CROP_OK` | 0 | 자르기/dry-run |
| crop_image | `CROP_ARGS` | 1 | 인자 |
| crop_image | `CROP_SRC_MISSING` | 1 | 소스 없음 |
| crop_image | `CROP_MISS_IMAGEMAGICK` | 2 | magick 없음 |
| crop_image | `CROP_NO_OUTPUT` | 3 | 출력 미생성 |
| crop_image | `CROP_BBOX_NOT_UINT` | 4 | 비정수 bbox |
| crop_image | `CROP_BBOX_EMPTY` | 4 | w/h < 1 |
| extract_docx | `DOCX_OK` | 0 | 추출/dry-run/fallback |
| extract_docx | `DOCX_ARGS` | 1 또는 2 | 인자/플래그 |
| extract_docx | `DOCX_SRC_MISSING` | 1 | 파일 없음 |
| check_deps | `DEP_MISS_*` | 0 또는 1 | 13장 |

## build-from-ingest

이 명령의 실패는 helper JSON 이 아니다. CLI 종료 코드 #2707 계열:

| 상황 | 대략 | 에이전트 |
| --- | --- | --- |
| `-o` 없음 | 사용법, exit 2 | F15 |
| JSON 파싱 실패 | 런타임, exit 1 | ingest 수정 |
| unknown field | 런타임, stderr 힌트 | F11 |
| boxed.text | 런타임, #3358 힌트 | F12 |

stdout 을 파싱할 수 없으면 stderr 를 사용자에게 그대로 보여 준다.
exit 를 삼키고 "성공한 척" 하지 않는다.

## 정지 ID 연결

| 봉투 | 정지 |
| --- | --- |
| `DEP_MISS_RHWP` | F01 |
| `DEP_MISS_IMAGEMAGICK` | F02 |
| `PDF_MISS_TOOLS` | F03 |
| `DEP_MISS_PYTHON3` | F04 |
| `PDF_SRC_MISSING` | F05 |
| `DOCX_SRC_MISSING` | F08 |
| `CROP_BBOX_*` | F14 |
| CLI `-o` 누락 | F15 |

## DATA 로 읽을 것

- `ok: false` 인데 산출 HWPX 가 있으면 쓰지 않는다.
- dry-run `ok: true` 는 파일이 생겼다는 뜻이 아니다.
- `DEP_MISS_PYTHON_DOCX` 는 실패가 아니다. fallback 경로.
- poppler 누락 + magick 존재 = PDF 가능. `missingOptional` 만 보고 멈추지 않는다.

픽스처 디렉터리 `fixtures/envelopes/` 의 모든 JSON 은 위 `code` 중 하나를
가지거나, CLI 사용법 표본(`BUILD_MISSING_O`)이다.
