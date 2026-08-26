# 11 — crop_image.sh bbox 계약

페이지 PNG 에서 그래프·표·수식만 잘라 `media[].id` 경로에 저장한다.
좌표계는 **소스 이미지 픽셀, 좌상단 원점, x 오른쪽, y 아래**.

## 계약

```
crop_image.sh [--json] [--dry-run] <source.png> <x> <y> <w> <h> <out.png>
```

| 항목 | 규칙 |
| --- | --- |
| x, y | 10진 정수 ≥ 0 |
| w, h | 10진 정수 ≥ 1 |
| 소수 | 거부 (`CROP_BBOX_NOT_UINT` exit 4) |
| 음수 | 거부 (정규식 `^[0-9]+$`) |
| 빈 값 | 인자 누락 exit 1 |
| 소스 없음 | `CROP_SRC_MISSING` exit 1 |
| ImageMagick 없음 | `CROP_MISS_IMAGEMAGICK` exit 2 |
| 출력 미생성 | `CROP_NO_OUTPUT` exit 3 |
| dry-run | 자르지 않음. planned magick 명령만 |

엔진:

```
magick "$SRC" -crop "${W}x${H}+${X}+${Y}" +repage "$OUT"
```

`+repage` 가 잘린 캔버스를 리셋한다. 빼지 않는다.

## Vision → 숫자

모델이 "그래프는 페이지 중상단" 이라고만 하면 crop 하지 않는다.
숫자를 내라.

대략 변환 (A4, 300 DPI, 2480×3508 가정):

- 상단 여백 ~150 px
- 본문 좌 ~180 px
- 한 문항 블록 높이 400–900 px (밀도에 따라)

정확한 값은 Read 한 이미지를 보고 센다. 위 숫자는 시작점일 뿐.

bbox 가 페이지 밖으로 나가면 ImageMagick 이 빈 그림을 만들 수 있다.
dry-run 은 범위 검사는 하지 않는다 (소스 픽셀을 읽지 않음).
에이전트가 `x+w ≤ natural_w`, `y+h ≤ natural_h` 를 지킨다.

## 레시피

```bash
bash .claude/skills/rhwp-exam-ingest/helpers/crop_image.sh \
    --json --dry-run \
    "$TMP/page_004.png" 180 620 2100 880 \
    "$MEDIA_DIR/img/q11_graph.png"
```

통과 봉투:

```json
{
  "schemaVersion": "1.0",
  "helper": "crop_image.sh",
  "ok": true,
  "code": "CROP_OK",
  "dryRun": true,
  "engine": "magick",
  "bbox": {"x": 180, "y": 620, "w": 2100, "h": 880}
}
```

소수 거부:

```bash
bash helpers/crop_image.sh --json --dry-run page.png 10.5 20 100 80 out.png
# exit 4 CROP_BBOX_NOT_UINT
```

## media 와 짝

crop 출력 경로의 `--media-dir` 상대 부분이 `media[].id` 와 같아야 한다.

```
MEDIA_DIR=/tmp/media
out=/tmp/media/img/q11_graph.png
id=img/q11_graph.png
natural_w=2100
natural_h=880
```

`natural_*` 는 crop 결과 픽셀 (w, h) 과 같게 두는 것이 정직하다.
원본 페이지 크기를 넣지 않는다.

## 하지 말 것

- 페이지 전체를 문항 media 로 넣기.
- 선택지 텍스트를 그림으로 자르기. 선택지는 `choices[]`.
- Python PIL crop 을 새로 짜기. 엔진은 ImageMagick.
- bbox 를 ingest JSON 에 넣기 (미지 필드).

픽스처: `fixtures/helpers/crop_bbox_contract.json`,
`fixtures/envelopes/crop_*.json`.
