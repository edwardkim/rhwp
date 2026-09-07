# samples/issue6782 — 칸 앵커 그림을 칸 밖으로 밀어내는 음수 오프셋

[#6782](https://github.com/edwardkim/rhwp/issues/6782)의 정식 회귀 fixture다.

## 재현물

| | |
| --- | --- |
| 원본 이름 | `1480000-201900042_D0150004-1-001_(최종보고서) 안전확인대상생활화학제품 표시기준 제정 연구(부록 제외).hwp` |
| 출처 | PRISM 정책연구관리시스템 공개 문서 |
| 원본 크기 | 6,521,856 bytes (SHA-256 `398d03a5…`) |
| **fixture 크기** | **193,536 bytes** (SHA-256 은 [MANIFEST.json](MANIFEST.json)) |
| 쪽 수 | 104 |
| 저장 제품 | `hancom-office-2010` `8.5.8.1677` |

### 어떻게 34배로 줄였나

원본은 BinData(그림)가 **6.2MB 로 파일의 95%** 이고 나머지 본문·서식은 204KB 뿐이다.
`extract-pages` 로 쪽을 잘라도 BinData 가 걷히지 않아 5.1MB 였다.

그래서 **BinData 204개 스트림을 같은 확장자의 1×1 stub 이미지**(bmp 178 · jpg 11 ·
png 15)로 치환하고 `rhwp convert` 로 재저장했다. 그림 상자의 치수와 위치는
컨트롤 레코드가 정하므로 **조판은 바뀌지 않는다.**

동일성은 좌표로 확인했다.

```text
              쪽수   77쪽 image   대상 그림 bbox
  원본        104        11       x=624.4 y=-233.4 w=81.1 h=65.8
  fixture     104        11       x=624.4 y=-233.4 w=81.1 h=65.8   ← 전부 동일
```

11장 전부가 좌표·치수까지 같고 `lastSavedWith`·`printMethod` 도 같다.
한/글 2020 도 fixture 를 정상으로 열어 103쪽 PDF 를 낸다.

## 형상

77쪽(0-based `76`)의 표 `pi=118 ci=0`(14행×4열)에서 **`row=4 col=3` 칸**에 매달린
그림 하나가 대상이다.

```text
  Cell  row=4 col=3   x=608.0 y=230.2 w=110.1 h=82.9   content_top=232.1
  Image                x=624.4 y=?     w= 81.1 h=65.8
```

저장 세로 오프셋이 `-70,819HU = -944.25px`, 칸 `valign=Center` 라 그대로 실으면

```text
  232.1 + (79.05 - 65.8 - 944.25) / 2 = -233.4      ← 용지 위쪽 밖, 인쇄에서 소실
```

## 계약 — 좁게 적는다

한/글은 **음수 오프셋이 그림을 칸 내용 영역 위로 통째로 밀어낼 때** 그 오프셋을
쓰지 않는다. 구현이 판정하는 것도 정확히 그 조건 하나다.

```text
  v_off < 0  그리고  배치결과 y + 그림높이 <= content_top   →  오프셋을 버리고 0 으로 놓는다
```

⚠ **이것은 "칸과 겹치는가"라는 일반 계약이 아니다.** Top/Center 의 *아래쪽* 이탈,
Bottom 정렬에서 양수 오프셋이 만드는 위쪽 이탈, 모든 정렬의 아래쪽 완전 이탈은
이 갈래가 다루지 않는다. 오라클이 증명하는 것이 **Center + 음수 + 위쪽 이탈** 하나뿐이라
이름·주석·시험을 그 범위에 맞췄다.

판정 기준선이 실제 칸 상단(`230.2`)이 아니라 padding 뒤 `content_top`(`232.1`)인 것도
의도한 것이다 — 그림 좌표를 만드는 `place()` 가 `content_top` 을 기준점으로 쓰므로
**같은 기준으로 판정**해야 부호가 뒤집히지 않는다. 그 1.9px 띠에 바닥이 놓이는 그림은
이 fixture 에 없다.

⚠ **음수라고 무조건 버리면 안 된다.** `#5734`(156684746 9쪽 왼쪽 칸)의 첫 그림도
저장 vpos 가 0이라 같은 폴백 갈래로 오는데, 거기서는 `-1,079HU`(14.4px)가
**적용되는 것이 정답**이다.

```text
  #6782  y + h = -167.6 <= content_top 232.1   → 칸 밖   → 오프셋 무시
  #5734  y + h =  701.1 >  content_top 631.0   → 겹침    → 오프셋 적용
```

## 기준 엔진

**`2020`** — `lastSavedWith.product` 가 `hancom-office-2010` 이므로 저장소 정책
[§3.5.1](../../mydocs/manual/pr_review/visual_fixture_evidence.md) 의 2022 이하 버킷이다.

기준 PDF: [`pdf/1480000-201900042-chemical-labeling-standards-2020.pdf`](../../pdf/1480000-201900042-chemical-labeling-standards-2020.pdf)
— 1,119,717 bytes, SHA-256 `32e0e6d41d53b755b3dc4bcc31937e8b4f0921b282c2e5d3633a3f3617761912`.

| MCP 산출 증적 | 값 |
| --- | --- |
| job id | `38f90d35-4bec-4da8-bcb8-cc8746b3abde` |
| status | `succeeded` → `success` (phase `completed`) |
| 요청 engine / 응답 engine | `2020` / `2020` |
| `engine_profile` / `hancom_version` | `2020` / `12.0.0.4605` |
| backend | `hwp-managed-direct-dll-host` |
| blob 검증 | client·server byte 수·SHA-256 일치 (`verified`) |
| `pdf_page_count` | 103 |

### 정본 대조 — 대상 그림 (77쪽, idx 76)

정본에서 `w=81.1 h=65.7` 그림은 **쪽 인덱스 76** 에 있고, rhwp 와 같은 쪽이다.

```text
              y (px)      판정
  한/글 2020   235.9      칸 안
  rhwp 수정 전 -233.4     용지 밖 — 인쇄에서 소실
  rhwp 수정 후  238.7      칸 안 (한/글과 2.8px)
```

나머지 그림 10장은 수정 전후 좌표가 **한 픽셀도 움직이지 않는다** — 가드가 대상 형상만
잡는다는 증거다(`the_other_ten_images_keep_their_offsets` 가 잠근다).

```text
  rhwp layout-anomaly <fixture> --json

                offCanvas  overflow  overlap  textOverlap  쪽수
  수정 전            1         30        1          2       104
  수정 후            0         30        1          2       104
```

⚠ 문서 전체 쪽수는 정본 103 · rhwp 104 로 한 쪽 다르다. 이 축(#6782)과는 다른 문제이며
대상 그림이 있는 쪽은 두 쪽 모두 인덱스 76 으로 같다.

측정: 정본은 PyMuPDF `get_image_info()['bbox']`(px = pt × 4/3),
rhwp 는 `rhwp export-render-tree -p 76` 의 `Image` 노드 bbox.
