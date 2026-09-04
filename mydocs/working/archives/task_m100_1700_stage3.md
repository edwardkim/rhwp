# Stage 3 완료보고 — Task #1700 검증

## 1. 고정 18건 (어울림 표 직후 빈 문단)
수정 전: PARA_COUNT 12 + PI_MISMATCH(rhwp_pNone) 6.
수정 후: **PARA_COUNT 0**, MATCH 6, PI_MISMATCH 12.
- **PARA_COUNT off-by-one 12건 전부 해소**(문단수 정합).
- 잔여 12건은 #1700(빈 문단 흡수)이 아닌 별개 원인으로 재분류:
  - **표 행분할 페이지네이션 차이 6건**: rhwp 가 작은 표를 2쪽으로 분할, 한글은 1쪽에 적재
    (예 14504219: 9×4 표 rows 8–9 가 p2 로 넘침). 빈 문단은 표 끝쪽을 충실히 추종.
  - **제3 경로 미배치 6건**: block/tac 표 사이·뒤 빈 문단(`rhwp_pNone`). wrap_around 도
    hidden_empty_paras 도 아닌 별도 흡수 경로(예 2957879 pi3, 36400586 pi17).

## 2. 전체 1,000건 (seed 42)
| 판정 | 수정 전 | 수정 후 |
|------|--------|--------|
| MATCH | 932 | **949** (+17) |
| PI_MISMATCH | 20 | 27 |
| PAGE_DELTA | 35 | 24 |
| **PARA_COUNT** | **13** | **0** |
| ERR | 0 | 0 |

> 주의: hwpdocs 에 수집기가 실시간 추가 → 같은 seed 라도 모집단이 달라져 전/후 표본이
> 부분만 겹친다. 따라서 회귀 판정은 아래 격리 A/B 로 수행.

## 3. 회귀 격리 A/B (동일 바이너리·동일 한글데이터)
변경이 **순수 가산**(dump-pages 에 `WrapAroundPara`/`HiddenEmptyPara` 라인만 추가, 기존
item 출력 불변)임을 이용. 동결 251건(불일치 51 + MATCH 200)에서 출력의 추가 라인을 제거한
"수정 전 등가"와 전체 출력을 같은 한글 데이터로 비교:

- **REGRESSION (before MATCH → after non-MATCH): 0** ✅
- IMPROVED (before non-MATCH → after MATCH): +1 (해당 부분집합)
- PARA_COUNT 4 → 0
- (51 ERRH 는 한글 재시작 후 하네스 플레이크 — 전체 1000 실행 ERR=0 으로 제품 무관)

**구조적 보장**: MATCH 문서는 흡수된 빈 문단이 0개여야만 성립(있으면 한글 대비 문단수
부족으로 이미 불일치)하므로, 가산 변경으로 MATCH→불일치 회귀는 원리상 불가능. A/B 가 이를 실증.

## 4. 표준 회귀 게이트
- `cargo test --release --test hwpx_roundtrip_baseline`: **4 passed, 0 failed** (구조 보존).
- `cargo test --release` 전체: 1건 실패 = `form_01_keeps_nine_cfb_streams`
  (`tests/issue_852_hwpx_to_hwp_contract_streams.rs`).
  - 원인: 단언이 `/BodyText/Section0`(슬래시) 기대, Windows 실제 스트림은 `\BodyText\Section0`.
  - **사전 존재 Windows 경로구분자 테스트 버그.** 이 테스트는 dump/rendering 을 참조하지 않으며
    (grep 0), 본 변경은 `rendering.rs` 단일 파일 +100줄 가산(삭제 0)으로 CFB 변환 경로와 무관.
  - → #1700 회귀 아님. 별도 처리 대상(플랫폼 테스트 정정).

## 5. 결론
- #1700 목표(어울림 표 직후 빈 문단 누락)의 **PARA_COUNT off-by-one 전수 해소**, 회귀 0.
- 잔여 PI_MISMATCH 는 별개 원인(표 행분할 / 제3 흡수경로) → 후속 이슈 권고.
