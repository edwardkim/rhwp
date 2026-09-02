# 03 — extract_docx.py

DOCX 시험지에서 본문과 임베디드 이미지를 꺼낸다.
한컴 DOCX 가 아니라 OOXML (`word/document.xml` + `word/media/`) 이다.

## 계약

```
extract_docx.py [--json] [--dry-run] <input.docx> <out_dir>
```

| 산출 | 내용 |
| --- | --- |
| `out_dir/text.txt` | UTF-8. python-docx 면 단락마다 개행. fallback 이면 `w:t` 토큰 개행 |
| `out_dir/img/<name>` | `word/media/` 원본 바이트. 확장자 유지 (png/jpeg/emf) |

EMF/WMF 는 Vision 이 못 읽을 수 있다. 그 경우 사용자에게 PNG 로
다시 보내 달라고 하거나, DOCX→PDF 경로를 제안한다. helper 가
벡터를 래스터로 바꾸지 않는다 (새 의존성 금지).

## 엔진

1. `python-docx` (`import docx`) — 단락 단위. 표 셀 텍스트는 기본
   `Document.paragraphs` 에 안 들어간다. 표 안의 문항은 빠질 수 있다.
2. zip + `re.findall(r"<w:t[^>]*>([^<]*)</w:t>", xml)` — 토큰이 붙는다.
   실패가 아니다. `DEP_MISS_PYTHON_DOCX` 봉투, exit 0.

python3 자체가 없으면 helper 를 실행할 수 없다. 그때는
`check_deps.sh` 의 `DEP_MISS_PYTHON3` (DOCX 입력일 때만 정지 F04).

## 종료 코드

| exit | code | 의미 |
| --- | --- | --- |
| 0 | `DOCX_OK` | 추출 또는 dry-run 통과 (fallback 포함) |
| 1 | `DOCX_ARGS` | 인자 부족 |
| 1 | `DOCX_SRC_MISSING` | 파일 없음 |
| 2 | `DOCX_ARGS` | 알 수 없는 플래그 |

## 레시피

```bash
python3 .claude/skills/rhwp-exam-ingest/helpers/extract_docx.py \
    --json --dry-run 고2_모의고사.docx /tmp/docx-out
```

성공 dry-run 봉투:

```json
{
  "schemaVersion": "1.0",
  "helper": "extract_docx.py",
  "ok": true,
  "code": "DOCX_OK",
  "dryRun": true,
  "engine": "python-docx",
  "planned": ["/tmp/docx-out/text.txt", "/tmp/docx-out/img/*"],
  "pythonDocx": true,
  "fallback": null
}
```

fallback 이면 `"engine": "zip-regex-fallback"`, `"pythonDocx": false`.

실제 추출 후 Vision 은 `img/` 의 PNG/JPEG 를 Read 하고, `text.txt` 로
번호를 교차 확인한다. 둘 이 어긋나면 Vision 을 따른다.

## 표·수식·텍스트박스

| DOCX 안의 것 | helper 가 하는 일 | 에이전트가 하는 일 |
| --- | --- | --- |
| 본문 단락 | text.txt | stem / choices |
| 임베디드 PNG | img/ | media + crop 불필요 (이미 잘림) |
| 표 | 단락 추출에서 누락 가능 | PDF 경로 제안 또는 표 스크린샷을 사용자에게 요청 |
| OMML 수식 | 텍스트로 깨짐 | 수식은 이미지로. F16 |
| 텍스트박스 | 빠질 수 있음 | Vision 이 렌더된 페이지를 봐야 함 → PDF 권장 |

## 하지 말 것

- `pip install python-docx` 를 사용자 동의 없이 실행.
- fallback 을 실패로 보고 중단.
- DOCX 를 직접 `build-from-ingest` 에 전달.
- `word/media` 파일명을 임의로 `q1.png` 로 바꾸지 말 것. 원본 basename 을
  `media[].id` 에 쓰거나, 복사할 때 매핑 표를 ingest 주석이 아니라
  작업 로그에만 남긴다. ingest JSON 에 매핑 필드를 추가하지 않는다
  (`deny_unknown_fields`).
