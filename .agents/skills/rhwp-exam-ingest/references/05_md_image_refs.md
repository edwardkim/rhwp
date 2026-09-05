# 05 — Markdown 이미지 참조

MD 시험지는 텍스트가 이미 있다. Vision 은 첨부된 그림에만 필요하다.
`![alt](path)` 를 `media[].id` 로 옮기는 것이 이 장의 전부다.

## 인식하는 문법

```markdown
![그래프1](images/q3_plot.png)
![표](./img/table-2.jpg)
<img src="media/figA.png" alt="실험 장치">
```

- 상대 경로는 **MD 파일이 있는 디렉터리** 기준.
- URL (`https://…`) 은 다운로드하지 않는다. 사용자에게 로컬 파일을 요청.
- 빈 alt 는 허용. `media[].id` 는 파일명으로 만든다.
- 참조 링크 `![alt][id]` / `[id]: path` 도 따라간다.

## 절차

1. MD 를 UTF-8 로 읽는다. `\r\n` 은 논리적으로 `\n` 과 같다.
2. 이미지 경로를 모아 존재 여부를 검사한다. 하나라도 없으면 F07.
   추측으로 `images/` 와 `img/` 를 바꿔 가며 찾지 않는다. 후보를 사용자에게 보여 준다.
3. 존재하는 파일을 `$MEDIA_DIR/img/<basename>` 으로 복사한다.
   basename 충돌이면 `img/q{n}_{basename}`.
4. 문항 분할:
   - `## 1.` `### 3.` 또는 줄 시작 `1.` `2.`
   - `①` 로 시작하는 연속 줄은 그 문항의 choices
   - `[1~3]` 블록은 passage 후보
   - `<보기>` 코드블록 또는 인용(`>`) 은 boxed 후보
5. 이미지 위치:
   - 질문 문장과 ① 사이에 있으면 `between`
   - 질문보다 위면 `above`
   - 선택지 다음이면 `below`
   - 문장 한가운데면 `inline`

## 예

```markdown
# 국어 영역

[1~2] 다음 글을 읽고 물음에 답하시오.

환경 오염은 …

## 1. 윗글의 주제로 가장 적절한 것은?

① 환경 보호의 중요성
② 도시 생활의 편리
③ 전통 음식의 역사
④ 기술 발전 동향
⑤ 진로 탐색의 필요

## 2. 다음 그래프에서 알 수 있는 사실은?

![미세먼지](figures/pm10.png)

① 2010년 이후 증가
② 2015년이 최고
③ 2020년은 2010년의 두 배
④ 2018년부터 둔화
⑤ 2022년 감소
```

에이전트가 쓰는 ingest 조각:

```json
{
  "number": 2,
  "stem": "다음 그래프에서 알 수 있는 사실은?",
  "auto_number": false,
  "stem_blocks": [
    {"type": "text", "text": "2. 다음 그래프에서 알 수 있는 사실은?"},
    {"type": "image", "ref": "img/pm10.png", "placement": "between"}
  ],
  "media": [
    {"id": "img/pm10.png", "natural_w": 900, "natural_h": 520,
     "target_w_mm": 90, "placement": "between"}
  ]
}
```

헤더가 `## 2.` 이면 stem 에 번호가 이미 있다 → `auto_number: false`.
헤더가 `## 주제` 이고 본문이 번호 없으면 `auto_number: true`.

## natural_w / natural_h

MD 만 보면 픽셀을 모른다. 가능한 경우:

```bash
magick identify -format "%w %h" "$MEDIA_DIR/img/pm10.png"
```

Identify 가 없으면 Vision 이 본 대략값 (예: 800×600) 을 쓰되,
`target_w_mm` 만 본문폭의 70% 근처(80–90)로 둔다. 두 값이 1 미만이면
스키마가 거부한다 (`minimum: 1`).

## 하지 말 것

- MD 를 그대로 HWP 로 보내는 가상의 `rhwp import-md` (발명 금지).
- 원격 이미지를 침묵 다운로드.
- alt 텍스트를 stem 으로 대체. alt 는 캡션 후보일 뿐.
- 이미지 없는 MD 를 Vision 단계로 보냄. 텍스트만으로 ingest 를 쓴다.
