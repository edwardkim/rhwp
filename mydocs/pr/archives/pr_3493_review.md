# PR #3493 검토 — HWP3 개요번호 마커를 소비 직후 IR 에서 걷어냄

Issue: #3492 / author: planet6897 / reviewer: edwardkim / milestone: v1.0.0
연작: planet6897 7건 누적 검토(2026-07-28), 체리픽 6순위

## metadata (작성 시점 참고값)

| 항목 | 값 |
|---|---|
| 기능 커밋 | `2f8121a67` → 누적 `a7f0bbeda` |
| 규모 | +187 -0 (`src/parser/hwp3/mod.rs` +32, 테스트 +155) |
| CI | 원 PR head SUCCESS, mergeable CLEAN |

## 변경

HWP3 개요번호 마커가 소비된 뒤에도 IR 문단 텍스트에 잔류해 두 갈래 증상을 만들었다 —
① 저장(HWP5/HWPX 변환) 시 마커 바이트가 본문으로 새어 저장 손실, ② 미주 앵커 위치가
마커 폭만큼 이탈. 수정은 파서가 마커를 소비한 직후 IR 에서 제거해 공통 뿌리를 닫는다.

**경계 준수 확인**: 변경이 `src/parser/hwp3/` 안에서 끝난다. 렌더러·레이아웃·문서 코어에
HWP3 분기를 추가하지 않는다 — CLAUDE.md/parser_architecture 의 HWP3 격리 규칙 그대로다.

## 검증

- focused 4건 통과 (마커 미잔류, 저장 왕복 무손실, 미주 위치, 무개요 문서 무회귀)
- 누적 branch 전체 게이트: release-test **4253 passed / 0 failed** — IR field sweep
  baseline 포함 통과. 새 fixture 없음(기존 샘플 사용)이라 4.3.1 baseline 등록 대상 아님.
- fmt·clippy 클린

## 시각 판정

불필요 — IR 정합 수정이며 svg_snapshot golden 무변화가 렌더 무파급을 교차 확인. 마커
잔류가 그려지던 문서는 잔류 텍스트가 사라지는 방향의 의도 변화이고, 저장 왕복 테스트가
그 의도를 고정한다.

## 권고

**merge (통합 PR 경유).** 두 증상의 공통 뿌리를 파서 단일 지점에서 닫은 정확한 귀속이다.
