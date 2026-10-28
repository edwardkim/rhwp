---
kind: investigation
status: active
canonical: mydocs/manual/codex/docs_and_git_workflow.md
last_verified: 2026-07-29
---

# Task #3486 Stage 4 — 동일 원본 4형식 정합과 HWP3 표 채움 결함

- 이슈: [#3486](https://github.com/edwardkim/rhwp/issues/3486)
- 브랜치: `task_m100_3486_hwp3_render_fidelity_v2`
- 기준 오라클: `pdf/HWP3-password-123456.pdf`
- 입력: `HWP3-password-123456.hwp`, `HWP5-nopassword-123456.hwp`,
  `HWP5-nopassword-123456.hwpx`, `HWP5-password-123456.hwpx`

## 판정 범위 정정

네 입력은 같은 한글 97 안내문을 암호화·포맷만 달리 보존한 계열이다. 따라서 이번 작업의 수용 기준은
"HWP3 PDF 1쪽의 중앙 그림 색조" 하나가 아니라 **동일 문서의 HWP3·HWP5·평문 HWPX·암호 HWPX가
텍스트, 표, 배경, 페이지 흐름에서 의미 있게 수렴하는가**다.

현재 직접 CLI 기준으로 HWP3/HWP5는 24쪽, HWPX 두 입력은 23쪽이다. 이 차이를 무시하고 같은 쪽
번호를 픽셀 비교하지 않는다. 대신 제목, 4×2 시스템 사양 표, 머리말, 중앙 그림처럼 같은 의미 앵커를
대조하며 HWPX용 한컴 PDF가 확보되기 전에는 HWP3 PDF를 HWPX의 페이지별 오라클로 주장하지 않는다.

## 재현 결과: HWP3 표가 검은 채움으로 붕괴

3쪽의 4×2 시스템 사양 표는 HWP5/HWPX에서 흰 우측 셀과 본문을 보이지만, HWP3만 우측 네 셀이
검정 사각형이 되어 텍스트를 가린다. 같은 HWP3 문서의 `dump`에는 해당 셀 텍스트가 정상 파싱되어
있으므로, 복호화·텍스트 해독 실패가 아니라 표 셀 채움 변환 결함이다.

원시 HWP3 표 42 셀 레코드를 확인했다.

| 셀 | 원시 `셀의 색깔` | 원시 `음영 비율` | 기대 표시 | 기존 rhwp |
| --- | ---: | ---: | --- | --- |
| 우측 1·2·3·4행 | `0x0007` (흰색 팔레트) | `100` | 흰 바탕, 셀 본문 표시 | 검정 채움, 본문 가림 |
| 좌측 라벨 셀 | `0x0006` (노랑 팔레트) | `0` | 채움 없음 | 채움 없음 |

기존 HWP3 parser는 색상 word를 버리고 `255 - shade × 255 / 100`으로 회색만 만들었다. 즉
`흰색·100%`도 `검정`으로 계산했다. 수정은 표 42의 색상 팔레트와 음영 비율을 흰 바탕 위에서 합성한다.
HWP5/HWPX의 BorderFill parser나 renderer는 변경하지 않는다.

## 1쪽과 남은 항목

- HWP3 제목의 `ᄒᆞᆫ` 조합 자모 및 PUA 표시는 Studio Canvas 폰트·paint 경로에서 HWP5/HWPX와도
  다르게 보인다. 표 채움처럼 원문이 사라진 경우와 폰트/조합 렌더링 차이를 분리해 후속 추적한다.
- 중앙 쪽 배경 BMP는 네 파일에서 형식별로 명도·opacity가 다르다. 이전의 "brightness/contrast 순서를
  바꾸면 해결"이라는 가설은 네 형식 대조 전의 가설일 뿐 수용 결론이 아니다. 특히 `alpha="0"`와
  watermark opacity의 의미를 오라클로 검증하기 전에는 변경하지 않는다.
- HWP5와 HWPX의 IR diff에는 line segment, control 수, vpos 등 363개 차이가 남는다. parser IR의
  바이트 동일성은 목표가 아니며, 실제 화면 앵커를 기준으로 원인을 각각 좁힌다.

## 이번 단계의 회귀 계약

1. HWP3 색상 팔레트 7과 음영 100%는 `0x00FFFFFF`으로 합성한다.
2. 실제 암호 HWP3 fixture의 첫 4×2 표 우측 셀은 모두 흰 단색 채움으로 읽힌다.
3. HWP3 p3 SVG/PNG에서 이전의 검정 우측 셀 사각형이 사라지고 셀 본문이 보이는지를 확인한다.
4. HWP5·평문/암호 HWPX의 parser·renderer 산출은 이 HWP3 한정 수정으로 바뀌지 않아야 한다.

## Stage 4 시작 전 확인한 검증

검토 전용 target에서 Stage 3의 이전 parser/CLI 계약을 실행했다.

```text
CARGO_TARGET_DIR=target/task_3486_render_v2 CARGO_INCREMENTAL=0 \
  cargo test --profile release-test --test hwpx_password_fixture
3 passed; 0 failed

CARGO_TARGET_DIR=target/task_3486_render_v2 CARGO_INCREMENTAL=0 \
  cargo test --profile release-test --lib hwpunitchar_spacing_keeps_hwp3_lineage_storage_scale
1 passed; 0 failed
```

이 문서는 **코드 변경 전에** 원시 셀 값과 교차 포맷 판정 기준을 고정한 Stage 4 분석 기록이다. 이후 구현·focused test·수정 후 p3 시각 산출은 다음 Stage 기록에 분리하며, 이 단계는 최종 시각 수용 판정을 주장하지 않는다.

