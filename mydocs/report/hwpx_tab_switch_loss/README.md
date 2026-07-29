# HWPX 저장에서 tabPr 의 hp:switch 구조가 소실된다 (#3551)

## 현상

한컴이 저장한 HWPX 를 `export-hwpx` 로 다시 쓰면 `hh:tabPr` 안의
`<hp:switch>`(HwpUnitChar `case` + `default`) 구조가 사라지고 `default` 쪽만 남아
**`tabItem` 개수가 정확히 절반**이 된다.

`--verify`(IR 대조)·`--verify-pages` 는 통과한다 — IR 값에는 차이가 없어 기존 오라클로는
잡히지 않는다. ZIP 엔트리 바이트를 직접 대조해야 보인다.

## 재현 — 인터넷 배포 실물 문서

서울시 정보소통광장 공개 결재문서
[보고서 작성 서식 안내](https://opengov.seoul.go.kr/sanction/11678326) 첨부 4건(전부 한컴 저장
HWPX)으로 실측했다. 이 폴더의 [`seoul_brief_report_form.hwpx`](seoul_brief_report_form.hwpx)
는 그중 붙임2(약식보고서 서식)다.

```
rhwp export-hwpx seoul_brief_report_form.hwpx out.hwpx --verify --verify-pages
```

| 문서 | tabItem 원본 | 수정 전 | 수정 후 | hp:switch 원본 | 수정 전 | 수정 후 |
|---|---|---|---|---|---|---|
| 결재문서본문 | 480 | 240 | **480** | 273 | 33 | **273** |
| 겉표지보고서서식 | 66 | 33 | **66** | 73 | 40 | **73** |
| 약식보고서서식 | 66 | 33 | **66** | 59 | 26 | **59** |
| 붙임서식 | 66 | 33 | **66** | 52 | 19 | **52** |

수정 전 `hp:switch` 감소분은 정확히 탭 switch 쌍 수와 일치했다(결재문서 273−33 = 240 = 탭쌍 240).
**남아 있던 switch 는 전부 paraPr 것** — paraPr 은 보존되고 tabPr 만 버려지는 비대칭이었다.

## 원인

- 파서는 switch 를 제대로 읽는다 — `src/parser/hwpx/header.rs` `parse_tab_def`.
  HwpUnitChar `case` 를 우선하고 `item.position *= 2` 로 2× 스케일에 맞춰 적재한다.
- 직렬화기는 switch 를 만들지 않았다 — `src/serializer/hwpx/header.rs` `write_tab_pr` 가
  `hh:tabItem` 을 그대로 방출.

같은 파일의 paraPr 은 `write_para_margin_switch` 로 대칭 처리돼 있다. tabPr 만 빠져 있었다.

## 값 손실은 없었다

수집한 switch 쌍 **339개 전수에서 `default == case × 2`** 가 성립한다. 파서가 `case × 2` 를
적재하므로 IR 값은 원본과 같고 렌더·페이지 수도 변하지 않는다. 이 건은 **한컴 원본의 구조
보존** 문제이지 데이터 손실이 아니다.

## 수정

1. 직렬화기: `write_para_margin_switch` 와 동형으로 tabItem 을 `<hp:switch>` 로 감싼다
   (`case` = 저장값/2 + `unit="HWPUNIT"`, `default` = 저장값).
2. 파서: `case` 와 `default` 가 짝으로 있고 어긋나면 `default` 원값을 채택한다.

2번이 없으면 **새 결함이 생긴다** — 파서가 `case` 를 우선하므로 홀수 저장값이
`101 → case 50 → 재파싱 100` 으로 1 잘린다. #3368 이 paraPr 여백에서 지적한 것과 같은 계약이다.

회귀 테스트: `write_tab_pr_emits_tab_item_switch_case_default`(직렬화기),
`issue3551_odd_tab_position_prefers_exact_default`(파서).
