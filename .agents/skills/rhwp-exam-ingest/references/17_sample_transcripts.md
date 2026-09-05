# 17 — 표본 트랜스크립트

아래는 실 에이전트가 따라야 할 **대화+명령** 골격이다.
픽스처 JSON 은 `fixtures/transcripts/`.

## T01 PDF 수능 국어 (성공)

사용자: `이 시험지 PDF를 HWPX로 변환해줘 — downloads/2024_수능_국어.pdf`

1. `check_deps.sh --json` → ok
2. `pdf_to_pngs.sh downloads/2024_수능_국어.pdf $TMP 300` → 20 pages, textLayer true
3. Read `page_001.png` … 표지에서 `header_text=국어 영역`, `form_label=홀수형`
4. Read 본문 페이지. `[1~3]` 공유 지문, 문항 1–3, 보기 없음, 그림 없음
5. Write `$TMP/ingest.json` (`valid_shared_passage` 형태)
6. `rhwp build-from-ingest $TMP/ingest.json -o output/exam/2024_국어.hwpx`
7. `rhwp export-text` 에서 지문 1회, 문항 번호 1. 2. 3. 중복 없음
8. 보고: 문항 수, 경로, Picture 없음, 한계 해당 없음

## T02 PNG 한 장 + 그래프

사용자: `사진 한 장이야. 그래프 문제 한글 문서로.`

1. 패스스루. Read `desk.jpg`
2. 문항 1, 발문, 그래프 bbox 240,410,980,620, 선택지 5
3. `crop_image.sh desk.jpg 240 410 980 620 $MEDIA/img/q1.png`
4. ingest: placement between, auto_number true
5. `build-from-ingest --media-dir $MEDIA -o output/exam/graph.hwpx`
6. 고지: #182 로 한컴에서 그래프가 비어 보일 수 있음

## T03 MD + 이미지 ref

사용자: `notes/quiz.md 를 시험지로`

1. MD 읽기. `![plot](figures/pm10.png)` 존재 확인
2. 복사 `figures/pm10.png` → `$MEDIA/img/pm10.png`
3. `## 2.` → auto_number false
4. build --media-dir
5. export-text 에 `"2. "` 한 번

## T04 DOCX fallback

사용자: `학원.docx`

1. check_deps: python-docx 없음, python3 있음
2. `extract_docx.py` engine zip-regex-fallback, exit 0
3. text.txt 토큰이 붙음. Vision 은 img/ PNG 로 구조 복구
4. 표 문항 1개는 텍스트가 비어 사용자에게 PDF 경로 제안
5. 나머지 12문항 build

## T05 poppler 없음

사용자: `exam.pdf 변환`

1. check_deps: `DEP_MISS_POPPLER`, magick OK
2. `pdf_to_pngs.sh` engine magick, 성공
3. 이후 T01 과 동일

## T06 poppler·magick 둘 다 없음 (PDF)

1. check_deps: `DEP_MISS_IMAGEMAGICK` + `DEP_MISS_POPPLER`
2. `pdf_to_pngs.sh` → `PDF_MISS_TOOLS` exit 2
3. 정지 F03. 설치 힌트. ingest 를 추측으로 쓰지 않음

## T07 python-docx 없음 (DOCX, 성공)

1. `DEP_MISS_PYTHON_DOCX` 만. ok true
2. fallback 추출. 중단하지 않음

## T08 bbox 소수

Vision 이 x=120.4 를 냄.

1. `crop_image.sh … 120.4 …` → `CROP_BBOX_NOT_UINT` exit 4
2. 반올림 120 으로 재시도 dry-run → CROP_OK
3. 실제 crop

## T09 auto_number 중복 발견

export-text 에 `3. 3. 밑줄 친`

1. ingest 의 문항 3 `auto_number` 를 false 로 고치거나 stem 에서 `"3. "` 제거
2. 다시 build
3. 게이트 통과

## T10 수식 페이지

적분 3문항.

1. 각 수식 crop → image
2. 사용자에게 Equation IR 없음 (F16) 고지
3. 텍스트 발문·선택지는 살림

기계 가독 본문: `fixtures/transcripts/*.json`.
