---
kind: decision
status: active
canonical: mydocs/tech/layout_anomaly_detection.md
last_verified: 2026-08-16
---

# 레이아웃 이상탐지(layout anomaly detection) — 세 번째 층

CLI `rhwp layout-anomaly`, 코어 `src/diagnostics/layout_anomaly.rs`.

## 왜 필요한가 — `render-diff` 가 못 보는 것

`render_geom_diff`(CLI `render-diff`)는 두 렌더(왕복 전/후, 또는 두 파일)의 페이지별
`RenderNode` bbox를 구조 경로로 대응시켜 **변위**(`maxDisp`, px)를 잰다. 이건 "라운드트립이
원본과 얼마나 같은가"를 답한다 — 결정론적이고 폰트 래스터화에 의존하지 않아 1차 시각 정합성
게이트로 쓴다.

그런데 두 렌더가 **똑같이** 망가져 있으면 변위는 0인데 결과물은 여전히 깨져 있다. 예를 들어
표가 항상 페이지 밖으로 넘치는 문서는, 원본 IR을 렌더링하든 라운드트립 IR을 렌더링하든 **같은
자리로** 넘친다 — `render-diff` 는 두 렌더가 서로 일치한다고(변위 0) 보고하고 끝난다.
"일치한다"와 "정상이다"는 다른 질문이다.

`layout-anomaly` 는 렌더 **한 장**만 입력받아 그 자체가 말이 되는 기하인지 본다. `render-diff`
가 "달라졌는가"(두 렌더 사이)를 묻는다면, 이 도구는 "이상해 보이는가"(렌더 한 장 안에서)를
묻는다. 같은 렌더 기하 축 위의 서로 다른 질문이고, 한쪽이 다른 쪽을 대신하지 않는다 — 둘 다
필요하면 함께 돌린다(예: 라운드트립 전/후 각각에 `layout-anomaly` 를 돌려 "원본부터 이미
깨져 있었는가"까지 가른다).

```
render_geom_diff  (두 렌더 사이 비교)         layout_anomaly  (렌더 한 장 안의 판정)
  ─ "A 와 B 가 같은가"                          ─ "이 렌더가 정상적인 문서로 보이는가"
  ─ maxDisp, structureMismatch                  ─ overflow / overlap / empty_page
  ─ 라운드트립·두 파일 비교 전용                  ─ 임의의 단일 문서에 바로 적용
```

## 이미 있는 인접 신호와의 관계 — 왜 새로 만들었는가

이 저장소에는 비슷해 보이지만 다른 질문을 답하는 신호가 이미 둘 있다. 셋 다 렌더 기하를
다루지만 겹치지 않는다.

1. **`LayoutOverflow`**(`src/renderer/layout.rs`, `layout_engine.take_overflows()`) — 레이아웃
   엔진이 페이지네이션 **도중** 단 하단을 넘는 항목을 내부적으로 기록하는 구조체다.
   `build_page_render_tree`/`build_page_layer_tree` 는 이 값을 받아서 그 자리에서 버린다
   (`let _overflows = ...`) — 외부에 노출되는 명령이 없다. 페이지네이션 **결정**(다음 쪽으로
   넘길지)에 쓰는 내부 신호이지, 렌더 결과물을 다시 읽어 판정하는 신호가 아니다.
2. **`hidden_text::off_page_paragraphs`**(`src/document_core/queries/hidden_text.rs`,
   `inspect hidden-text --include-offpage`) — 보안 판정이다. 문단이 쪽 경계 **완전히** 밖에
   있는지(부분 겹침이 아니라 전부 밖)만, TextRun만, "숨겨서 프롬프트 인젝션에 쓸 수 있는가"
   관점에서 본다. 표·이미지·부분 초과는 범위 밖이다.

`layout-anomaly` 는 셋째 층이다 — 렌더러가 이미 만들어 낸 `RenderNode` 트리를 **사후에** 읽어,
표·이미지·문단 줄 단위로 "레이아웃 품질"(페이지 밖으로 새는가, 서로 겹치는가, 쪽이 텅 비었는가)
을 판정한다. 페이지네이션 내부 상태에 접근하지 않고, 보안 판정도 아니다 — 순수하게 "이 결과물이
사람이 보기에 정상적인 문서 레이아웃인가"만 본다.

## 판정 3종

입력은 `DocumentCore::build_page_render_tree` 가 페이지마다 만들어 내는 `RenderNode` 트리
하나뿐이다(`render_geom_diff::diff_render_geometry` 와 같은 배선). 렌더러·레이아웃 엔진 코드는
전혀 건드리지 않는다 — 이미 있는 출력을 읽기만 한다.

### overflow

`Body` 노드의 선언 bbox(레이아웃이 여백으로 확정한 콘텐츠 영역)를 기준선(boundary)으로 삼는다.
표·이미지·글상자·수식·묶음·도형류·문단 줄(TextLine) 중 이 경계를 허용치(기본 1.0px, `render-diff`
의 기본 변위 임계와 같은 자릿수) 넘게 벗어난 노드를 보고한다.

`Body::bbox` 를 쓰는 이유: 같은 구조체의 `RenderNodeType::Body::clip_rect` 는 레이아웃이 콘텐츠
배치 **후** 넘친 자손을 반영해 사후 확장된다(`src/renderer/layout.rs` 의 "clip 하방 확장" 로직
— 표 외곽선이 잘리지 않도록 하는 의도적 설계). `clip_rect` 를 기준으로 삼으면 넘친 콘텐츠가 기준
자체를 넓혀 버려 넘침을 놓친다. `Body::bbox`(`RenderNode` 의 `.bbox` 필드, `clip_rect` 확장과는
별개로 레이아웃이 확정한 원래 여백 상자)는 그 확장의 영향을 받지 않는다.

표·이미지 등 "컨테이너" 노드가 검사 대상에 걸리면 그 자손(표 셀 내부 줄 등)은 더 내려가 다시
검사하지 않는다 — 표 하나가 넘치면 그 표에 딸린 모든 줄을 중복 보고하는 대신 표 자체를 한 번만
보고한다.

### overlap

"겹치면 안 되는" 후보를 두 부류로 가른다.

- **흐름 콘텐츠**(표, 문단 줄) — 정상 조판은 절대 두 문단 줄이나 표를 같은 자리에 겹쳐 놓지
  않는다. 단 TextLine 은 **보이는 글자가 있을 때만** 후보다. 표·묶음 개체를 문단에 앵커링하는
  "운반용" 줄은 빈 `TextRun`(`text: ""`) 하나만 자식으로 두고 그 개체와 정확히 같은 좌상단에
  찍힌다 — 실측(`samples/2025 행정업무운영 편람(최종).hwpx`, 383쪽)에서 표-줄 겹침 오탐 43건이
  전부 이 패턴이었다. 실제 텍스트가 없는 줄은 화면에 아무 잉크도 안 남기므로 겹침이 아니다.
- **배치(floating) 개체** — 이미지·도형류는 `text_wrap` 이 "겹침을 배제하는" 종류
  (`Square`/`Tight`/`TopAndBottom` — 텍스트를 밀어내는 wrap)일 때만 후보다. `BehindText`/
  `InFrontOfText` 는 애초에 다른 콘텐츠와 겹치라고 있는 wrap 이라 후보에서 뺀다. 바탕쪽
  (master page) 유래 개체도 항상 배경에 깔릴 뿐이라 제외한다.

후보는 같은 단(Column) 안에서만 짝짓는다 — 서로 다른 단은 x축이 나뉘어 있어 정상 조판에서도
나란히 배치된다. 두 요소의 겹침 폭·높이가 **둘 다** 허용치(기본 2.0px)를 넘어야 보고한다 —
모서리가 살짝 스치는 것(안티앨리어싱·반올림)은 정상 조판에서도 흔하다.

### empty_page

콘텐츠(보이는 텍스트, 또는 표·이미지·도형류)가 전혀 없는 페이지가 **문서 중간**(첫 쪽도 마지막
쪽도 아님)에 있으면 신호를 낸다. 표지 뒷면·장 구분지처럼 의도된 빈 쪽과 회귀로 생긴 빈 쪽을
기하만으로 구분할 수 없으므로, `empty_page` 는 항상 "가능성 신호"다 — 별도 severity 필드를
두지 않는다. 존재 자체가 이미 낮은 신뢰도를 뜻하고, `--strict` 로도 절대 실패를 유발하지 않는다
(아래 "판정은 데이터" 참고).

## 판정은 데이터, 차단은 소비자 몫

이 저장소의 다른 진단 명령(`render-diff`, `inspect hidden-text` 등)과 같은 철학이다. 탐지
건수가 0이 아니어도 기본 종료 코드는 0이다 — anomaly 발견은 도구의 정상 동작이지 실패가 아니다.
`--json` 은 항상 전체 판정을 봉투로 낸다. 소비자가 실패로 취급하고 싶으면 `--strict` 를 명시
한다 — 이때도 `overflow`·`overlap`(확정 신호)만 종료 코드 3(`render-diff` 의 `EXIT_REGRESSION`
과 같은 값·같은 의미론)을 유발하고, `empty_page`(가능성 신호)는 `--strict` 로도 절대 실패를
유발하지 않는다.

## 실측 — 380쪽 실제 문서에서의 신호 품질

`samples/2025 행정업무운영 편람(최종).hwpx`(정부 업무편람, 383쪽)로 개발 중 실측했다. 초기
버전은 overlap 89건을 냈는데, 그중 43건이 위에서 설명한 "운반용 줄" 오탐이었다 — 고친 뒤
4건으로 줄었다(모두 실제 표-표 겹침, 폭 529px 전단 겹침). overflow 161건은 렌더러 자체의
내부 디버그 로그(`LAYOUT_OVERFLOW`/`LAYOUT_OVERFLOW_CELL`, `export-render-tree` stderr)가
독립적으로 같은 페이지·같은 방향의 넘침을 보고해 교차 검증됐다. `samples/
table_giant_cell_overfill.hwpx`(overflow 4577px, `--strict` 종료 코드 3)와 `samples/
issue1549_multipositive_float_tables.hwpx`(표 3개가 서로 겹치는 표본, 겹침 3건 — 같은 문서에서
렌더러 자체의 `LAYOUT_TABLE_OVERLAP` 디버그 로그와 교차 검증됨)는 이미 저장소에 있던 회귀
표본이라 그대로 재사용했다. 정상 문서(`samples/종이기준.hwpx` 등)는 0건.

## CLI

```
rhwp layout-anomaly <파일.hwp|파일.hwpx> [-p <페이지>] [--json] [--strict]
                     [--overflow-tolerance <px>] [--overlap-tolerance <px>]
```

- `--json`: 판정 봉투 한 줄(`schemaVersion`, `pages[].overflow/overlap/emptyPage`, `hasSignal` 등).
- `--strict`: `hasSignal`(overflow·overlap 확정 신호)이 있으면 종료 코드 3.
- `-p`: 사람 모드 출력만 해당 페이지로 좁힌다(스캔 자체는 항상 전 페이지).

## 의도적으로 미룬 것

- MCP 도구 등록(`mcp-serve` 노출)과 `capabilities --mcp`/`export-agent-manifest`/
  `export-ontology` 자기서술 확장은 이번 범위에서 뺐다. `render-diff` 도 MCP 도구
  (`hwp_render_diff`)를 갖고 있어 대칭을 맞추면 좋지만, 사람용 CLI + `capabilities`(사람이 읽는
  `commands[]`) 계약만으로 소비자가 이미 자동화할 수 있다.
- `--batch`(폴더 일괄 스캔, `render-diff --batch` 와 같은 형태)도 미뤘다 — 단일 파일 계약을
  먼저 굳힌 뒤 배치 축을 얹는 편이 안전하다.
- overflow·overlap 판정 대상 타입을 `--types` 로 좁히는 옵션은 아직 없다. 실측에서 TextLine
  overflow(단락이 쪽 하단 밖으로 밀린 경우)가 표·이미지 overflow보다 잦았는데, 이것도 유의미한
  신호(내부 `LAYOUT_OVERFLOW` 로그와 일치)라 기본에서 빼지 않았다 — 표·이미지만 보고 싶은
  소비자는 `pages[].overflow[].nodeType` 으로 후처리 필터링한다.
