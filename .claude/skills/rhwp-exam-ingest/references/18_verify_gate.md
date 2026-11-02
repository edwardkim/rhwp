# 18 — 검증 게이트

생성만으로 끝내지 않는다. 기존 CLI 로 텍스트가 들어갔는지 본다.
시각 픽셀 일치는 이 스킬의 통과 조건이 아니다.

`build-from-ingest`는 로컬 PNG·JPEG·GIF·BMP·TIFF를 검증해 Picture/BinData로 삽입한다.
기준 경로는 JSON 디렉터리이며 `--media-dir`로 바꿀 수 있다. 경로 이탈·누락·손상·한도 초과는
`INGEST_IMAGE_*`로 실패한다. placeholder로 대체하지 않으며 실제 이미지 표시도 확인한다.
문장 중간의 위치는 `paragraph.runs`의 text/image 순서로 지정한다.

## 명령

```bash
rhwp export-text "$OUT" -o "$TMP/txt"
rhwp dump "$OUT" > "$TMP/dump.txt"
unzip -l "$OUT"
```

`export-svg` 는 smoke. 원본 PDF 와 비교하지 않는다.

## 텍스트 대조

ingest 의 각 문항에 대해:

1. `stem` (또는 첫 stem_blocks text) 가 export-text 에 부분 문자열로 존재.
2. 각 `choices[].text` 존재.
3. `passages[].blocks[].text` 가 **한 번** 등장 (두 번이면 복제 사고).
4. 정규식 `([0-9]+)\.\s+\1\.` 가 없으면 통과 (auto_number 중복).
   예: export-text 에 `N. N.` / `3. 3. 밑줄` 이 보이면 ingest 를 고친다.

한 항목이라도 없으면 ingest 를 고치고 다시 build. writer 를 의심하기 전에 JSON.

## dump 로 보는 것

- 문단 수 ≈ 지문 블록 + 문항 발문 + 선택지 + 보기
- media를 요청했다면 Picture/컨트롤과 이미지 참조를 확인. 누락되면 성공으로 판정하지 않음
- 폰트 이름에 `함초롬` 또는 지정한 `default_font`

dump 포맷을 이 스킬이 파싱하는 새 도구를 만들지 않는다.
에이전트가 읽고 체크리스트를 틱한다.

## unzip

```
[Content_Types].xml
Contents/section0.xml
BinData/   # 이미지 바이트. 반복 Picture가 같은 자원을 공유할 수 있음
```

이미지를 요청했으면 BinData와 manifest·Picture의 참조 관계를 확인한다.
누락·깨진 참조·원하지 않은 중복이 있으면 실패다. 실제 이미지 표시도 확인한다.

## 통과 선언

사용자에게:

- 출력 경로
- 문항 수 (`questions.length`)
- 공유 지문 수
- media 수 / crop 수
- Picture 개수·고유 이미지 자원 수·실제 표시 결과와 INGEST_IMAGE_* 오류 여부
- 게이트: export-text 대조 결과

"한컴에서 열어 보니 예쁘다" 는 통과 조건이 아니다. 열어 보라고 권할 수는 있다.
