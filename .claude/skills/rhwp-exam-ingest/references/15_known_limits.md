# 15 — 알려진 한계

과거 #182의 Picture writer와 #665의 ingest 연결은 별개다. 현재 CLI는 두 경로를 연결한다.

## 1. 이미지 입력과 표시

`build-from-ingest`는 PNG·JPEG·GIF·BMP·TIFF를 실제 디코딩해 검증하고
Picture/BinData로 삽입한다. GIF·TIFF는 첫 프레임/페이지를 사용한다.
기본 기준 경로는 JSON 디렉터리이며 `--media-dir`로 바꾼다. 절대 경로·`..`·기준 경로
밖의 심볼릭 링크는 거부한다. SVG·WebP 등은 PNG로 변환해 공급한다.

- `natural_w`·`natural_h`를 실제 이미지 픽셀 치수와 맞춘다.
- `target_w_mm`는 양의 폭이다. 종횡비를 보존하며 본문 폭·한 쪽 높이보다 클 수 없다.
- 손상·경로·형식·치수·한도 오류는 `INGEST_IMAGE_*`로 실패한다. F18에서 오류에 맞게 수정한다.
- 기본값과 조정 옵션은 [CLI 매뉴얼](../../../../mydocs/manual/cli_commands.md)을 따른다.
- CLI 종료 코드 0, export-text 통과, crop 파일 생성만으로 이미지 표시를 판정하지 않는다.
  ZIP의 BinData·manifest·Picture 참조와 실제 한컴/렌더 출력을 확인한다.

Rust의 기존 `build_exam_paper`는 텍스트/placeholder projection API다.
실제 이미지는 파일 시스템에 의존하지 않는 `IngestImage::from_bytes`와
`build_exam_paper_with_images`를 사용한다. CLI는 이 이미지 API를 연결한다.

## 2. 수식과 수식 이미지

ingest에는 편집 가능한 수식 필드가 없다. `latex`, `omml`, `equation` 키를 발명하지 않는다.
편집 불가를 고지하고 사용자가 동의하면 수식 crop을 image 블록으로 삽입한다.
수식 편집이 필수이면 F16으로 중단한다. 평문으로 충족할 수 있는 `E=mc^2` 등은 text로 쓴다.

## 3. 표와 표 이미지

ingest에는 편집 가능한 표 구조 필드가 없다. `rows`/`cells`를 추가하지 않는다.
사용자가 동의하면 표 bbox crop을 이미지로 삽입한다. 표 편집이 필수이면 F17로 중단한다.
원본에 이미 편집 가능한 HWP 표가 있을 때만 `rhwp-table-exchange`의 CSV 왕복 대상이다.

## 그 밖의 한계

| 한계 | 행동 |
| --- | --- |
| 손글씨 답·낙서 | 요청 범위에 포함되면 이미지 보존 여부를 확인 |
| 2단 조판의 정확한 단 위치 | 읽는 순서로 문항을 재구성하며 원 단 위치는 재현하지 않음 |
| 페이지 머리글 장식 | `header_text` 문자열만 |
| 없는 글꼴 | 실제 fallback을 확인하고 글꼴 차이를 고지 |
| 듣기 평가 QR/음원 | 이미지는 보존할 수 있으나 음원 실행 기능은 없음 |
| OMR 답안 마킹 | 만들지 않음 |

픽스처: `fixtures/matrices/known_limits.json`.
예제: `examples/17_picture_serialization_limit.md`,
`examples/18_equation_as_image.md`, `examples/19_table_as_picture.md`.
