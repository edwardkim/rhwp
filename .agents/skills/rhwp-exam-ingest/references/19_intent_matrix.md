# 19 — 발화 → 동작 행렬

에이전트가 사용자 문장을 기존 helper/CLI 로만 매핑한다.
새 동사를 만들지 않는다. gym 발화가 아니다.

행 수: 85. 기계본: `fixtures/matrices/intent_matrix.json`.

| ID | 발화 | 동작 | 장 | 정지 |
| --- | --- | --- | --- | --- |
| I001 | 이 PDF를 HWPX로 만들어줘 | pdf_to_pngs.sh → Vision → build-from-ingest | 02_pdf_to_pngs.md | F05 |
| I002 | 수능 국어 PDF 한글 문서로 | pdf_to_pngs.sh → passages → build-from-ingest | 07_passages_questions.md | F05 |
| I003 | /rhwp-exam-ingest exam.pdf | 같은 사다리 | 00_tree.md | F01 |
| I004 | 이 스캔 사진 한 장 변환 | image passthrough | 04_image_passthrough.md | F06 |
| I005 | JPG 여러 장 페이지 순서대로 | numeric sort + confirm | 04_image_passthrough.md | F06 |
| I006 | quiz.md 를 시험지로 | MD + ![alt](path) | 05_md_image_refs.md | F07 |
| I007 | 학원.docx 변환 | extract_docx.py | 03_extract_docx.md | F08 |
| I008 | 의존성 있니 | check_deps.sh --json | 13_check_deps.md | F01 |
| I009 | poppler 없이 PDF 가능하냐 | magick fallback | 02_pdf_to_pngs.md | F03 |
| I010 | python-docx 없는데 DOCX | zip fallback, 중단 금지 | 03_extract_docx.md | F04 |
| I011 | 그래프를 지문과 선택지 사이에 | placement between | 09_media_placement.md | F14 |
| I012 | 그림이 발문보다 위 | placement above + 블록 순서 | 09_media_placement.md | F14 |
| I013 | 그림이 선택지 다음 | placement below | 09_media_placement.md | F14 |
| I014 | 문장 가운데 작은 도형 | placement inline, #182 고지 | 09_media_placement.md | F18 |
| I015 | 공유 지문 1~3 | passages + passage_ref | 07_passages_questions.md | F19 |
| I016 | <보기> 상자 살려줘 | stem_blocks boxed | 08_stem_blocks_boxed.md | F12 |
| I017 | 번호가 이미 지문에 있어 | auto_number false | 10_auto_number.md | F13 |
| I018 | 번호 자동으로 붙여 | auto_number true, stem 에 번호 없음 | 10_auto_number.md | F13 |
| I019 | 이 그래프만 잘라서 넣어 | crop_image.sh bbox | 11_crop_bbox.md | F14 |
| I020 | 수식도 한글로 편집 가능하게 | 거절, 이미지로. Equation IR 없음 | 15_known_limits.md | F16 |
| I021 | 표를 한글 표로 | 거절, Picture. Table IR 없음 | 15_known_limits.md | F17 |
| I022 | OCR 엔진 깔아줘 | 거절, Vision 사용 | 16_pitfalls.md | F19 |
| I023 | exam-from-pdf 명령 있어? | 없다. 발명 금지 | 00_tree.md | F19 |
| I024 | -o 빼도 돼? | 안 됨. -o 필수 | 12_build_from_ingest.md | F15 |
| I025 | media-dir 없이 그림 넣기 | 불가. --media-dir | 12_build_from_ingest.md | F15 |
| I026 | 정답도 JSON 에 넣어 | answer 키 금지 deny_unknown | 06_ingest_schema_v1.md | F11 |
| I027 | 흐린 스캔인데 | DPI 400 또는 원본 재요청 | 02_pdf_to_pngs.md | F10 |
| I028 | 한 페이지에 문제 40개 | 사분면 분할 Vision | 00_tree.md | F09 |
| I029 | 산출 확인은? | export-text / dump / unzip -l | 18_verify_gate.md | F19 |
| I030 | 한컴에서 그림이 안 보여 | #182 한계 고지. writer 수정 금지 | 15_known_limits.md | F18 |
| I031 | 국어 지문 HWPX로 | 사다리 동일, 과목은 Vision 힌트일 뿐 | 07_passages_questions.md | F19 |
| I032 | 국어 지문 한글 시험지로 | 사다리 동일, 과목은 Vision 힌트일 뿐 | 07_passages_questions.md | F19 |
| I033 | 국어 지문 ingest 해줘 | 사다리 동일, 과목은 Vision 힌트일 뿐 | 07_passages_questions.md | F19 |
| I034 | 국어 지문 변환해 줘 | 사다리 동일, 과목은 Vision 힌트일 뿐 | 07_passages_questions.md | F19 |
| I035 | 국어 지문 만들어 줘 | 사다리 동일, 과목은 Vision 힌트일 뿐 | 07_passages_questions.md | F19 |
| I036 | 영어 장문 HWPX로 | 사다리 동일, 과목은 Vision 힌트일 뿐 | 07_passages_questions.md | F19 |
| I037 | 영어 장문 한글 시험지로 | 사다리 동일, 과목은 Vision 힌트일 뿐 | 07_passages_questions.md | F19 |
| I038 | 영어 장문 ingest 해줘 | 사다리 동일, 과목은 Vision 힌트일 뿐 | 07_passages_questions.md | F19 |
| I039 | 영어 장문 변환해 줘 | 사다리 동일, 과목은 Vision 힌트일 뿐 | 07_passages_questions.md | F19 |
| I040 | 영어 장문 만들어 줘 | 사다리 동일, 과목은 Vision 힌트일 뿐 | 07_passages_questions.md | F19 |
| I041 | 수학 적분 HWPX로 | 사다리 동일, 과목은 Vision 힌트일 뿐 | 15_known_limits.md | F19 |
| I042 | 수학 적분 한글 시험지로 | 사다리 동일, 과목은 Vision 힌트일 뿐 | 15_known_limits.md | F19 |
| I043 | 수학 적분 ingest 해줘 | 사다리 동일, 과목은 Vision 힌트일 뿐 | 15_known_limits.md | F19 |
| I044 | 수학 적분 변환해 줘 | 사다리 동일, 과목은 Vision 힌트일 뿐 | 15_known_limits.md | F19 |
| I045 | 수학 적분 만들어 줘 | 사다리 동일, 과목은 Vision 힌트일 뿐 | 15_known_limits.md | F19 |
| I046 | 과학 그래프 HWPX로 | 사다리 동일, 과목은 Vision 힌트일 뿐 | 09_media_placement.md | F19 |
| I047 | 과학 그래프 한글 시험지로 | 사다리 동일, 과목은 Vision 힌트일 뿐 | 09_media_placement.md | F19 |
| I048 | 과학 그래프 ingest 해줘 | 사다리 동일, 과목은 Vision 힌트일 뿐 | 09_media_placement.md | F19 |
| I049 | 과학 그래프 변환해 줘 | 사다리 동일, 과목은 Vision 힌트일 뿐 | 09_media_placement.md | F19 |
| I050 | 과학 그래프 만들어 줘 | 사다리 동일, 과목은 Vision 힌트일 뿐 | 09_media_placement.md | F19 |
| I051 | 사회 통계 표 HWPX로 | 사다리 동일, 과목은 Vision 힌트일 뿐 | 15_known_limits.md | F19 |
| I052 | 사회 통계 표 한글 시험지로 | 사다리 동일, 과목은 Vision 힌트일 뿐 | 15_known_limits.md | F19 |
| I053 | 사회 통계 표 ingest 해줘 | 사다리 동일, 과목은 Vision 힌트일 뿐 | 15_known_limits.md | F19 |
| I054 | 사회 통계 표 변환해 줘 | 사다리 동일, 과목은 Vision 힌트일 뿐 | 15_known_limits.md | F19 |
| I055 | 사회 통계 표 만들어 줘 | 사다리 동일, 과목은 Vision 힌트일 뿐 | 15_known_limits.md | F19 |
| I056 | 한국사 연표 HWPX로 | 사다리 동일, 과목은 Vision 힌트일 뿐 | 04_image_passthrough.md | F19 |
| I057 | 한국사 연표 한글 시험지로 | 사다리 동일, 과목은 Vision 힌트일 뿐 | 04_image_passthrough.md | F19 |
| I058 | 한국사 연표 ingest 해줘 | 사다리 동일, 과목은 Vision 힌트일 뿐 | 04_image_passthrough.md | F19 |
| I059 | 한국사 연표 변환해 줘 | 사다리 동일, 과목은 Vision 힌트일 뿐 | 04_image_passthrough.md | F19 |
| I060 | 한국사 연표 만들어 줘 | 사다리 동일, 과목은 Vision 힌트일 뿐 | 04_image_passthrough.md | F19 |
| I061 | 한문 구결 HWPX로 | 사다리 동일, 과목은 Vision 힌트일 뿐 | 10_auto_number.md | F19 |
| I062 | 한문 구결 한글 시험지로 | 사다리 동일, 과목은 Vision 힌트일 뿐 | 10_auto_number.md | F19 |
| I063 | 한문 구결 ingest 해줘 | 사다리 동일, 과목은 Vision 힌트일 뿐 | 10_auto_number.md | F19 |
| I064 | 한문 구결 변환해 줘 | 사다리 동일, 과목은 Vision 힌트일 뿐 | 10_auto_number.md | F19 |
| I065 | 한문 구결 만들어 줘 | 사다리 동일, 과목은 Vision 힌트일 뿐 | 10_auto_number.md | F19 |
| I066 | 제2외국어 대화문 HWPX로 | 사다리 동일, 과목은 Vision 힌트일 뿐 | 05_md_image_refs.md | F19 |
| I067 | 제2외국어 대화문 한글 시험지로 | 사다리 동일, 과목은 Vision 힌트일 뿐 | 05_md_image_refs.md | F19 |
| I068 | 제2외국어 대화문 ingest 해줘 | 사다리 동일, 과목은 Vision 힌트일 뿐 | 05_md_image_refs.md | F19 |
| I069 | 제2외국어 대화문 변환해 줘 | 사다리 동일, 과목은 Vision 힌트일 뿐 | 05_md_image_refs.md | F19 |
| I070 | 제2외국어 대화문 만들어 줘 | 사다리 동일, 과목은 Vision 힌트일 뿐 | 05_md_image_refs.md | F19 |
| I071 | PDF 경로가 틀려 | PDF_SRC_MISSING | 02_pdf_to_pngs.md | F05 |
| I072 | 이미지 경로가 틀려 | CROP_SRC_MISSING | 11_crop_bbox.md | F14 |
| I073 | DOCX 가 없어 | DOCX_SRC_MISSING | 03_extract_docx.md | F08 |
| I074 | DPI 30으로 | PDF_DPI_RANGE | 02_pdf_to_pngs.md | F05 |
| I075 | bbox 에 12.7 썼어 | CROP_BBOX_NOT_UINT | 11_crop_bbox.md | F14 |
| I076 | 폭 0으로 crop | CROP_BBOX_EMPTY | 11_crop_bbox.md | F14 |
| I077 | ImageMagick 없는데 crop | CROP_MISS_IMAGEMAGICK | 13_check_deps.md | F02 |
| I078 | rhwp 바이너리 없는데 | DEP_MISS_RHWP | 13_check_deps.md | F01 |
| I079 | boxed 에 text 필드 | F12 rebuild | 08_stem_blocks_boxed.md | F12 |
| I080 | version 2 로 | 스키마 const 1 | 06_ingest_schema_v1.md | F11 |
| I081 | MD 에서 상대 경로 이미지 | 경로 확인 | 05_md_image_refs.md | F07 |
| I082 | MD 에서 img 태그 | 경로 확인 | 05_md_image_refs.md | F07 |
| I083 | MD 에서 참조 링크 이미지 | 경로 확인 | 05_md_image_refs.md | F07 |
| I084 | MD 에서 원격 URL 이미지 | 05장 규약. URL 은 다운로드 금지 | 05_md_image_refs.md | F07 |
| I085 | MD 에서 깨진 이미지 경로 | 경로 확인 | 05_md_image_refs.md | F07 |

## 읽는 법

- `command` 열에 `rhwp exam-from-pdf` 가 있으면 발명된 금지 명령이다. 이 표가 틀린 것이다.
- 정지 열의 Fxx 는 SKILL.md 정지 표와 같아야 한다.
- 과목명(국어/수학)은 Vision 힌트일 뿐 다른 파이프라인이 아니다.
