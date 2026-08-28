---
kind: working
status: active
issue: 6318
---

# layout-anomaly 글자 겹침 후보를 본문 밖까지 넓힌다 (#6318)

## 무엇을

`layout_anomaly::scan_page` 가 `Body` 서브트리만 순회해서, 본문 글자가 바탕쪽·머리말·
꼬리말·각주 영역의 글자를 덮어도 신호가 0 이었다. **글자 겹침(text-overlap) 후보 수집
범위만** 본문 밖까지 넓힌다.

- `src/diagnostics/layout_anomaly.rs`
  - `collect_text_outside_body` — `MasterPage`·`Header`·`Footer`·`FootnoteArea` 에서
    `TextRun` 후보만 모은다
  - `text_columns_can_overlap` — "다른 단"과 "단 밖"을 구분한다
  - 단위 테스트 3종
- `tests/fixtures/text_overlap_baseline.tsv` — 넓힌 범위로 재측정

## 왜

페이지 트리의 직계 자식은 `Body` 하나가 아니다.

```
Page
├─ PageBackground
├─ MasterPage      <- 바탕쪽 (사이드바 탭·장식 상자·배경)
├─ Header
├─ Body            <- 종전에는 이 노드만 스캔
└─ Footer
```

편람 69쪽(`devel` b1485e0a14)에서 본문 줄이 오른쪽으로 넘쳐 바탕쪽 사이드바의 "제1절"
라벨 위에 그대로 얹힌다. 그런데 명령은 0 을 냈다.

```
w=8.0  A='붙임파일에 비밀번호를 설정할 경우, 해당 비밀번호를…'  B='제'
w=7.0  A= 〃                                                    B='1'
w=8.0  A= 〃                                                    B='절'
```

#5952 의 "상자 오른쪽을 넘어 사이드바와 겹친다" 가 이 형상이고, PR #6083 검토에서
검토자가 `devel` 렌더 PNG 로 확인한 증상과 같다. 판정기가 그 쪽을 스캔하고도 0 을 낸
이유는 순전히 시작 노드가 `Body` 하나였기 때문이다.

### 이슈의 4 건을 3 건으로 정정

이슈 #6318 에는 4 건으로 적었다. 그때는 렌더 트리의 `TextRun` 을 전부 러프하게 셌고,
실제 판정기는 `is_text_overlap_candidate` 의 가시 글자 요건을 건다. 4 번째 짝의 바탕쪽
런은 텍스트가 빈 문자열이라 **제외되는 것이 맞다**. 판정기가 어림셈보다 엄격한 쪽이다.

## 설계 — 폭발 반경을 좁힌 세 가지

### 1. 본문 밖에서는 `TextRun` 후보만 모은다

바탕쪽은 전면 배경 이미지를 갖는 일이 흔하다(편람 `Image x=0..740.8 y=0..1014.4`).
컨테이너 overlap 후보(`flow`)에 넣으면 그 배경이 본문 요소 전부와 겹치는 오탐이 된다.
단위 테스트 `master_page_background_image_does_not_create_overlap_noise` 가 이를 고정한다.

### 2. overflow·off-canvas·overlap 의 기준 상자는 그대로

이 셋은 "본문 여백을 넘었나"(`Body::bbox`), "페이지 상자 밖인가"(Page bbox) 라는 뜻이
분명하다. 판정 기준은 손대지 않고 **겹침 후보의 수집 범위만** 넓힌다.

### 3. "다른 단"과 "단 밖"을 구분한다

종전 짝짓기는 `if a.column != b.column { continue; }` 였다. 본문 런은 `Some(0)`,
쪽 고정 런은 `None` 이라 이 규칙이 본문↔바탕쪽 짝을 통째로 버렸다.

- 서로 다른 단은 x 축이 나뉘어 있어 정상 조판에서도 나란히 놓인다 → 종전대로 제외
- **단 밖**은 다른 단이 아니라 단 개념이 없는 자리다 → 어느 단과도 짝이 된다

흐름 요소(`flow`)는 종전 규칙(`flow_columns_can_overlap`)을 그대로 쓴다. 컨테이너
overlap 수치는 바뀌지 않는다. 단위 테스트 `different_columns_still_do_not_pair` 가
"다른 단은 여전히 제외" 를 고정한다.

## 실측 — 증가분 898 건의 성격

samples 945 건 전수(`layout-anomaly --batch samples --json`) 분류다.

| 영역 조합 | 건수 | 깊은 겹침(줄 높이 50% 이상) | 폭 중앙값 | 문서 수 |
| --- | ---: | ---: | ---: | ---: |
| Body x Body | 4,371 | (기존) | — | 152 |
| Body x FootnoteArea | 568 | 312 (55%) | 25.0px | 8 |
| Body x Footer | 185 | 120 (65%) | 38.0px | 30 |
| Body x MasterPage | 128 | 98 (77%) | 11.0px | 8 |
| Header x Header | 8 | 8 (100%) | 135.0px | 2 |
| MasterPage x MasterPage | 7 | 7 (100%) | 18.0px | 3 |
| Footer x FootnoteArea | 2 | 2 (100%) | 38.7px | 1 |

**`Body x Body` 가 정확히 4,371 이다** — #6317 이 고정한 값과 한 건도 다르지 않다.
기존 판정이 전혀 바뀌지 않았다는 기계적 증거다.

문서 총계는 152 -> 173 종, 4,371 -> 5,269 건이다.

### 각주 영역을 빼지 않은 이유

각주는 본문 바로 아래가 정상 배치라, 본문 마지막 줄과 각주 첫 줄의 줄상자가 몇 px
스치는 것은 구조적 인접일 수 있다고 의심했다. 데이터가 그 의심을 기각한다.

- 568 건이 **문서 8 종에만** 몰려 있다. 구조적 인접이면 각주를 쓰는 문서 전반에 고루
  퍼져야 한다.
- 폭 중앙값 25.0px 로 한글 글자 두 개 폭이다. 여백 근접의 크기가 아니다.
- 55% 가 줄 높이 절반 이상 겹친다.
- 최다 문서가 `issue1937_rowbreak_footnote_overpagination.hwp` (136 건) 로, 이름
  그대로 각주 페이지네이션 회귀 샘플이다.

## 부수 발견 — 한 쪽에 바탕쪽이 여럿 그려진다

영역 내부 겹침 15 건을 보면 이런 형상이다.

```
A: Page/MasterPage1/Table1/Cell0/TextLine0/TextRun0
B: Page/MasterPage5/Table1/Cell0/TextLine0/TextRun0    겹침 18.0 x 15.3px
```

같은 쪽에 `MasterPage` 노드가 둘 이상 있고 서로 덮는다(`exam-kor-3p.hwp`,
`exam_science.hwp`, `exam_social.hwp`). 한글 바탕쪽은 양쪽/홀수/짝수 종류가 있어 한 쪽에
둘 이상이 동시에 그려지는 것이 맞는지 확인이 필요하다. **이 PR 의 범위가 아니라
별건으로 남긴다** — 이 PR 은 판정 범위만 넓히고, 그 결과 드러난 형상은 baseline 에
기록해 둔다.

## 검증

| 명령 | 결과 |
| --- | --- |
| `cargo fmt --all -- --check` | 통과 |
| `cargo clippy --profile release-test --all-targets -- -D warnings` | 통과 (2분 12초, 경고 0) |
| `cargo test --lib diagnostics::layout_anomaly` | 16 통과 (신규 3 포함) |
| 래칫 테스트 (갱신 baseline) | 통과 (207.49s, 945 건 스캔 / 스킵 3) |

## 적층

이 PR 은 #6317(`gate/text-overlap-ratchet`) 위에 쌓았다. baseline 픽스처가 거기서
처음 들어오므로 순서가 바뀌면 baseline 이 두 번 흔들린다. #6317 이 병합되면
`devel` 로 리베이스한다.

## 실측

### 편람 69쪽 — 0 건에서 3 건으로

```
$ rhwp layout-anomaly "samples/2025 행정업무운영 편람(최종).hwp" -p 68 --json
page68 textOverlap: 3 | overlap: 0 | overflow: 2
  w=8.0 h=13.3  A: Body/Column0/Table0/Cell1/TextLine9/TextRun1
                B: MasterPage1/Group2/Rect0/TextBox0/TextLine0/TextRun0   ('제')
  w=7.0 h=13.3  B: ... TextRun1  ('1')
  w=8.0 h=13.3  B: ... TextRun2  ('절')
```

`devel` 은 같은 명령에서 `textOverlap: 0` 이었다. `overlap`(컨테이너) 은 `devel` 과 같은
0, `overflow` 도 같은 2 다 — 컨테이너 판정이 안 바뀌었다는 실물 확인이다.

### 전수 래칫 — 갱신 baseline 으로 통과

```
text-overlap 스윕: 샘플 945건(스킵 3) / 0 아닌 문서 173종 / 총 5269건
test text_overlap_baseline::text_overlaps_do_not_grow ... ok
finished in 207.49s
```

### 커밋 대상

```
src/diagnostics/layout_anomaly.rs
tests/fixtures/text_overlap_baseline.tsv
mydocs/working/anomaly_outside_body_scope.md
```

`tests/generated/`, `tests/suites/manifest.json` 은 `.gitignore` 대상이라 들어가지 않는다.
