# 최종 결과 보고서 — Task M100 #1794

## 이슈

HWPX→HWP 라운드트립: 표 앵커 세로 위치 95px 차이 (중첩 1×1 쪽 기준 표, seoul_0765)

## 결론

원인은 표 앵커 계산이 아니라 **그 앞 문단의 float exclusion 처리 비대칭**이었다.
자리차지(TopAndBottom) float 표의 exclusion zone에 문단 첫 줄 잉크가 겹치는지 검사하는
프로브(`item_probe_height`, layout.rs)가 `is_hwpx_source` 게이트로 HWPX 소스에만 적용되어,
HWP5 재파스 경로에서는 " 끝." 문단이 자리차지 표 위에 겹쳐 그려지고 후속 표 앵커가
95.16px 위로 어긋났다. 게이트를 제거하여 소스 포맷과 무관하게 동일 적용했다.

## 원인 상세

- 재현 문서 페이지 구성: 자리차지 표 2개(pi=0, pi=1 v_off=75.7px) + 문단 pi=2~4 +
  쪽 기준(재배치 후 rel=Para) 1×1 표(내부 12×41 중첩표) pi=5
- 문단 pi=4 " 끝."의 줄 잉크(400.8~418.1px)가 표 pi=1의 exclusion zone[402.7~496.0]과 겹침
- A(HWPX): 잉크-겹침 프로브 발화 → zone 하단으로 push → 후속 표 앵커 569.1
- B(HWP5): 프로브가 hwpx 게이트에 막혀 미발화 → 텍스트가 표에 겹침 → 표 앵커 473.9
- 판정: 저장 lineseg vpos는 이 페이지 전체가 float 밀림 미반영(stale)이라 기준 불가.
  자리차지 의미상 텍스트-표 겹침은 시각 결함이므로 A(프로브 적용)가 정당.
- 소거 기록: ir-diff 유일 차이 cc 1건(무관), dump-pages 배치 동일 → 렌더 단계 국소화,
  `compute_table_y_position`/`RHWP_VPOS_DEBUG`/shift 계측으로 exclusion_jump 발화 차이 특정.

## 수정 내역

| 파일 | 수정 |
|------|------|
| `src/renderer/layout.rs` | 잉크-겹침 프로브의 `is_hwpx_source` 게이트 제거 (HWP5 동일 적용). #1789 line_spacing 제외 규칙 유지 |

## 검증 결과

1. **seoul_0765**: render-diff --via hwp OVER 95.16px → **PASS 0.00px**
2. **big_hwpx 2,500 배치** (--via hwp, 기준선 rd_big_v3fix 대비): 변화 21건 전부 개선,
   **회귀 0건**. OVER 12건 해소 — seoul_0765(95.2→0), admrul_0288(80→0),
   seoul_0505(9→0, #1793 효과 포함), admrul_0227/0440/0605/0722/0786/0800/1006/1218,
   seoul_0889/0892. 새 분포: PASS 2448 / OVER 12 / STRUCT 32 / PAGE 4 / LOAD_FAIL 4.
3. **big_hwp 2,500 배치** (--via hwp, 기준선 rd_big_hwp 대비): 변화 1건 = 개선
   (admrul_0593 OVER 9.0→6.7), **회귀 0건**. 분포 동일 (PASS 2494 / OVER 4 / STRUCT 2).
4. **cargo test --release** 전수: 통과 (유일 실패 = #1775 Windows 경로 구분자 기대 실패).

※ 중간에 big_hwp를 `--via hwpx`로 잰 비교(416건 차이)는 기준선 생성 조건(--via hwp)과
불일치한 측정 오류로 폐기 — 기준선과 동일 조건 재실행으로 회귀 0 확인.

## 잔여 확인 권고

- identity 라운드트립 게이트는 A/B가 같이 움직이는 변화를 감지하지 못한다. 네이티브
  HWP5에서 프로브가 새로 발화하는 케이스의 한글 편집기 시각 대조
  (tools/verify_pi_page_vs_hangul.py)는 작업지시자 시각 판정 권장.
- HWPX→HWP 변환 시 셀 field_name("틀", "발신명의") 소실 확인 — 레이아웃 무관이나
  충실도 이슈로 별도 등록 검토.
- 이슈에 언급된 동족 후보 admrul_0296/1066/0556(1.9~6.1px)은 본 수정 후에도 잔존
  (admrul_0296 OVER 3.867 유지) — 별개 원인.
