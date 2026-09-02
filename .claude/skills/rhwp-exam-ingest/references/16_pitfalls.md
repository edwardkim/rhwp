# 16 — 함정

실 에이전트가 반복해서 빠지는 곳만 적는다.

## 번호 중복

stem 에 `"1. "` 가 있는데 `auto_number` 기본 true. 산출 `"1. 1. …"`.
MD 헤더 `## 1.` 을 그대로 옮길 때 특히 자주 난다. → 10장.

## boxed.text

보기 본문을 `{"type":"boxed","text":"…"}` 로 쓴다. #3358 이 거절한다.
`blocks:[{type:text,text:…}]`.

## 미지 필드

`answer: 3` 을 나중에 쓰려고 넣는다. deny_unknown_fields.
정답은 ingest 밖 (작업 로그, 별도 파일).

## page-1.png

pdftoppm 원본 이름을 media id 로 쓴다. helper 가 `page_001.png` 로
바꿔 두었는데 Vision 메모는 `page-1` 로 남아 있다. crop 소스가 없다.

## bbox 를 mm 로

A4 210 mm 를 x 로 넣는다. 계약은 픽셀 정수. 210×297 crop 은
페이지 왼쪽 위 작은 조각만 자른다.

## 공유 지문 복제

문항마다 지문을 붙여 시험지가 세 배가 된다. `passages` + `passage_ref`.

## passage_ref 오타

`p1-3` vs `1-3`. 스키마는 통과, 지문은 실종. id 를 복사한다.

## 선택지를 stem 에

①–⑤ 를 한 text 블록에 줄바꿈으로 넣는다. 번호/들여쓰기가 무너진다.
반드시 `choices[]`.

## PDF 를 build-from-ingest 에

`rhwp build-from-ingest exam.pdf -o out.hwpx`. JSON 파서가 터진다.

## --media-dir 없이 media[]

id 를 해석할 루트가 없다. 빈 그림 또는 오류.
media 가 있으면 `--media-dir` 을 붙인다.

## python-docx 없음을 실패로

fallback 이 있다. DOCX 를 거절하지 않는다. 단, 표 문항은 빠질 수 있다고 고지.

## 외부 OCR

Tesseract 를 깔라고 한다. 이 스킬은 Vision. 설치 안내를 하지 않는다.

## 원본 덮어쓰기

PDF 옆에 같은 이름으로 저장. `-o` 를 `output/` 아래로.

## 이웃 스킬 재작성

시험지 변환 중에 form-fill 문서를 고친다. 범위 밖.

## gym 과제

"이 파이프라인을 gym pack 으로" — 이 이슈는 gym 금지.

## ImageMagick `convert` 와 Windows convert

Windows 는 `convert` 가 파일시스템 변환일 수 있다. helper 는 `magick` 을
먼저 찾는다. Windows 에서는 ImageMagick 7 (`magick`) 을 권장한다고 고지한다.

## 8진수 페이지

`page-08.png` 를 `printf %03d $n` 으로 돌리면 bash 가 8진수 오류.
helper 는 `10#$n` 을 쓴다. 에이전트가 직접 rename 할 때도 10진 강제.

## 두 단을 한 stem 에

왼쪽 단 끝과 오른쪽 단 처음이 한 문장이 된다. Vision 이 단을 나눠 문항을
재구성한다. pdftotext 를 그대로 믿지 않는다.
