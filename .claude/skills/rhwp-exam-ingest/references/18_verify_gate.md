# 18 — 검증 게이트

생성만으로 끝내지 않는다. 기존 CLI 로 텍스트가 들어갔는지 본다.
시각 픽셀 일치는 이 스킬의 통과 조건이 아니다.

현재 `build-from-ingest`는 이미지 파일을 읽어 Picture/BinData로 연결하지 않고
`[이미지: …]` placeholder 텍스트를 만든다. `--media-dir`은 디렉터리 존재만 확인한다.
**그림이 필수이면 F18로 중단한다.** 경로·크기·배치 변경만으로 해결되지 않으며,
placeholder 결과를 이미지 포함 시험지로 제공하지 않는다.

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
BinData/   # 실제 이미지 삽입이라면 필요. 현재 ingest 빌더는 생성하지 않음
```

현재 빌더에서 media를 넣어도 BinData가 없는 것은 이미지 삽입 미구현 때문이다.
텍스트 검사가 통과해도 그림 필수 입력은 실패다. 경로·배치 변경으로 반복하지 않는다.

## 통과 선언

사용자에게:

- 출력 경로
- 문항 수 (`questions.length`)
- 공유 지문 수
- media 수 / crop 수
- 그림이 필수인지와 F18 중단 여부. placeholder를 완성된 이미지로 보고하지 않음
- 게이트: export-text 대조 결과

"한컴에서 열어 보니 예쁘다" 는 통과 조건이 아니다. 열어 보라고 권할 수는 있다.
