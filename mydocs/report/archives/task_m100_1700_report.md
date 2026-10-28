# 최종 결과보고서 — Task #1700: 표 직후 빈 문단 페이지 배치 보존

- 마일스톤: M100 (v1.0.0) / 이슈: edwardkim/rhwp#1700 / 브랜치: `local/task1700`
- 작성일: 2026-06-30

## 1. 문제

rhwp 레이아웃이 **어울림(Square wrap) 표 직후의 빈 문단**을 페이지 배치 산출(`dump-pages`)에서
누락. 한글(OLE)은 이 문단을 본문 문단 1개로 카운트하므로 문단→페이지 매핑이 1개씩 어긋남
(PARA_COUNT off-by-one). 한컴 한글 2022 대비 1,000개 표본 검증에서 불일치 68건 중
**18건(26%)**이 표 인접 빈 문단 누락으로 분류됨.

## 2. 원인

`src/document_core/queries/rendering.rs` `dump_page_items()` 가 `ColumnContent.items` 만
순회. 어울림 표 뒤 빈 문단은 `ColumnContent.wrap_around_paras` 에 별도 보관되어
**items 에 없으므로 `pi=` 로 출력되지 않음**. (레이아웃/렌더 트리는 wrap_around_paras 를
받아 표 옆에 정상 배치 → 시각은 정상, 누락은 배치 산출/매핑 표면화 단계에 한정.)

## 3. 해결 (접근법 B — 쿼리 표면화, 순수 가산)

`dump_page_items()` 에 사전 패스 + 출력 추가:
- 어울림 문단: 앵커 표(`table_para_index`)가 놓인 페이지(다중 페이지 표는 끝 페이지)에 귀속 →
  `WrapAroundPara pi=…` 출력.
- 빈 줄 감춤(`hidden_empty_paras`): 직전 item 문단 페이지에 귀속 → `HiddenEmptyPara pi=…`.
- 기존 item 출력은 **불변**(가산만). 레이아웃·페이지네이션·렌더 트리 미변경 →
  기하·페이지 수·시각 불변.

변경 규모: `rendering.rs` 단일 파일 **+100줄, 삭제 0**.

## 4. 검증

| 항목 | 결과 |
|------|------|
| 고정 18건 PARA_COUNT off-by-one | **12 → 0 전수 해소** |
| 전체 1,000건 MATCH | 932 → **949** (PARA_COUNT 13→0) |
| 회귀 격리 A/B(251건, 동일 한글데이터) | **REGRESSION 0**, IMPROVED +1 |
| `hwpx_roundtrip_baseline` | 4 passed, 0 failed |

**회귀 0의 구조적 보장**: MATCH 문서는 흡수된 빈 문단이 0개여야 성립(있으면 한글 대비 문단수
부족으로 이미 불일치). 따라서 가산 변경으로 MATCH→불일치 회귀는 원리상 불가능하며 A/B 가 실증.

검증용 한글 문서 2건을 `samples/task1700/` 에 동봉(메모리 룰 `rhwp-pr-include-hangul-docs`).

## 5. 잔여(별개 원인 — 후속 이슈 권고, #1700 범위 밖)

수정으로 문단수는 정합됐으나 일부는 다른 원인으로 PI_MISMATCH 잔존:
1. **표 행분할 페이지네이션 차이**: rhwp 가 작은 표를 2쪽으로 분할, 한글은 1쪽에 적재
   (빈 문단은 표 끝쪽을 충실히 추종). 페이지 적재량(축 b) 계열.
2. **제3 흡수경로**: block/tac 표 사이·뒤 빈 문단 미배치(`rhwp_pNone`). wrap_around·
   hidden_empty_paras 어디에도 없는 별도 경로.

## 6. 알려진 사항 (회귀 아님)

`cargo test --release` 전체에서 `form_01_keeps_nine_cfb_streams`
(`tests/issue_852_hwpx_to_hwp_contract_streams.rs`) 1건 실패 —
단언이 `/BodyText/Section0`(슬래시)인데 Windows 실제 CFB 스트림은 `\BodyText\Section0`.
**사전 존재 Windows 경로구분자 테스트 버그**로, dump/rendering 미참조·본 가산 변경과 무관.
별도 플랫폼 테스트 정정 대상.

## 7. 결론

#1700 핵심(어울림 표 직후 빈 문단 누락)의 PARA_COUNT off-by-one을 회귀 없이 전수 해소.
잔여 PI_MISMATCH는 별개 원인(표 행분할 / 제3 흡수경로)으로 후속 이슈 권고.
