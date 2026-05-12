# 최종 결과 보고서 — Task #855

## 이슈

[#855] 21_언어_기출_편집가능본.hwp 14p 우측 단: Square-wrap 표 뒤 문단(pi=300) 렌더링 누락

## 증상

`samples/21_언어_기출_편집가능본.hwp` 14페이지 오른쪽 단에서, `[A]` 묶음 박스(어울림 배치 3×2 표) **아래쪽 본문**("최근에는 기존의 법학방법론적 논의와 …" — `pi=300`)이 렌더링되지 않음. 페이지 흐름에서 문단 전체가 소실 (다음 페이지로 넘어간 것도 아님).

## 원인

`src/renderer/typeset.rs` 의 어울림(Square wrap) 표 옆 문단 흡수 로직.

어울림 표 anchor 의 wrap zone(`column_start`, `segment_width`)을 등록한 뒤, 후속 문단이 그 zone 과 일치하는지 검사할 때 **첫 LINE_SEG 만** 비교했다. `pi=300` 은 12개 LINE_SEG 중 첫 줄(`ls[0]`)만 표를 피해 들여쓰기되어 wrap zone(cs=3455, sw=27581)과 일치하고, 나머지 11줄(`ls[1..11]`)은 표 아래 본문 전체 폭(cs=852, sw=30184)이다. 그런데 첫 LINE_SEG 일치만으로 문단 **전체**를 "표 옆에 나란히 들어가는 0-높이 문단"으로 간주하여 `current_column_wrap_around_paras` 에만 기록하고 `continue` → 페이지 높이를 소비하지 않고 흐름에서 제외 → `pi=300` 누락.

`WrapAroundPara` 흡수 메커니즘은 본래 좁고 긴 어울림 표 옆 공간을 채우는, **전체가 표 옆에 들어가는** 문단(주로 빈 ↵ 표시 문단)을 위한 것이었다.

## 수정

`src/renderer/typeset.rs` — Table anchor 흡수 분기에서, 후속 문단의 **마지막 LINE_SEG 도** wrap zone(cs, sw)과 일치할 때(또는 빈 문단일 때)만 0-높이 흡수하도록 조건을 강화. 불일치 시(= 일부 줄만 표 옆) wrap zone 을 종료하고 일반 텍스트 배치로 폴백한다. LINE_SEG 의 cs/sw 가 이미 wrap 형상을 인코딩하므로, layout 은 첫 줄을 표 옆 들여쓰기로, 나머지 줄을 표 아래 전폭으로 정상 렌더한다.

Picture anchor 흡수 분기(`wrap_anchors` 등록 → FullParagraph 통과)는 영향 없음.

- 수정 파일: `src/renderer/typeset.rs` (1개 분기, 약 +13줄)
- 레이아웃·문서코어·HWP3 파서 변경 없음

### 미처리(후속 정합 항목)

`src/renderer/pagination/engine.rs` 의 `paginate_with_measured_opts` 에도 동일 로직(주석에 "engine.rs:288-320 동일 시멘틱")이 존재한다. 이 경로는 `RHWP_USE_PAGINATOR=1` 일 때만 동작하는 fallback 이며 기본값(TypesetEngine)에는 영향이 없어 본 타스크에서는 수정하지 않았다. 두 구현 정합이 필요하면 별도 타스크로 처리한다.

## 검증

| 항목 | 결과 |
|------|------|
| `cargo build --release` | 성공 |
| `cargo test --release` | 전체 통과 (1232 + 통합 테스트, 0 failed) |
| `cargo clippy --release` | 경고 0건 |
| `dump-pages -p 13` | 수정 전 `단 1` items=8 / `pi=300` 누락 / diff=-300.3px → 수정 후 items=10 / `pi=300` `FullParagraph h=180.2` 등장 / diff=-15.1px |
| `export-svg -p 13` 시각 검증 | `[A]` 박스 아래 `pi=300` 본문 정상 렌더, `pdf/...-2022.pdf` 14p 단 구조와 일치 |
| 샘플 SVG 스팟체크 | `samples/`·`samples/basic/`·`samples/hwpx/` 전부 패닉/오류 없음 |

## 결론

의도한 버그(어울림 표 아래 문단 누락) 해결. 회귀 없음. merge 가능.
