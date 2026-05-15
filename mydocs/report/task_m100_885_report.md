# Task #885 최종 결과 보고서

작성일: 2026-05-15
브랜치: `local/task885` (from `stream/devel`)
이슈: [#885 한국어 표준 폰트(HY계열) 미설치 환경 폴백 메트릭 보정](https://github.com/edwardkim/rhwp/issues/885)
Ref: #696

## 1. 작업 요약

한컴 전용 폰트 (HY 계열) 가 미설치된 Linux/macOS 환경에서, 한국어 폰트 face 이름이 메트릭 DB 에 매칭되지 않아 시각 정합성이 떨어지는 문제 해결.

`resolve_metric_alias` 에 **HY 계열 한국어 별칭 8개** 추가:
- HY수평선B/M, HY울릉도B/M, HY태백B, HY동녘B/M, HY각헤드라인M

## 2. 산출물

| 산출물 | 위치 |
|--------|------|
| 수행 계획서 | `mydocs/plans/task_m100_885.md` |
| 구현 계획서 | `mydocs/plans/task_m100_885_impl.md` |
| Stage 1 — 누락 폰트 식별 | `mydocs/tech/task885_missing_aliases.md` |
| Stage 1 보고서 | `mydocs/working/task_m100_885_stage1.md` |
| Stage 2 — 구현 보고서 | `mydocs/working/task_m100_885_stage2.md` |
| Stage 3 — 시각 정합 측정 | `mydocs/working/task_m100_885_stage3.md` |
| Stage 4 — 측정 방법론 | `mydocs/tech/font_diff_rmse_normalization.md` |
| 최종 보고서 (본 문서) | `mydocs/report/task_m100_885_report.md` |

## 3. 코드 변경

| 파일 | 변경 |
|------|------|
| `src/renderer/font_metrics_data.rs` | `resolve_metric_alias` 별칭 8개 추가 + 회귀 테스트 `task885_hy_extra_aliases_resolve` |

## 4. 검증

| 검증 | 결과 |
|------|------|
| `cargo test --release --lib -- task885` | 1 passed |
| `cargo test --release --lib -- font` | 33 passed, 0 failed |
| `cargo clippy --release --lib -- -D warnings` | 통과 |
| `rhwp dump` IR 무영향 | ✓ 변경 전/후 출력 완전 동일 |
| 회귀 샘플 (`samples/hwpx/2024년 1분기 ...ff.hwpx`) | hashmap 정렬 외 변경 없음 |

## 5. 시각 정합 측정 결과

`samples/table-in-tbox.hwp` p1 vs `pdf/table-in-tbox-2022.pdf` (150 DPI)

| 항목 | RMSE |
|------|------|
| PDF vs Before | 26.863% |
| PDF vs After | 26.797% |
| **개선폭** | **+0.066 %p** |
| Before vs After [SVG 변화량] | 3.904% |

### 해석

- 메트릭 별칭은 **텍스트 좌표/폭 계산을 보정**한다 (SVG 자체 변화 3.9%).
- 그러나 PDF 와의 픽셀 RMSE 개선은 미미 (+0.07%p).
- 근본 원인: `rsvg-convert` 가 시스템 폴백 글리프로 렌더하므로, **글리프 모양 차이가 RMSE 가산의 대부분**을 차지하며 옵션 1 로는 보정 불가.

## 6. 이슈 본문 3개 옵션 평가 결과

| 옵션 | 본 타스크 결과 |
|------|--------------|
| 1) 폴백 메트릭 보정 | **본 타스크에서 적용 완료**. 텍스트 메트릭 정합은 개선되었으나 픽셀 RMSE 개선은 미미. |
| 2) 자동 임베딩 + OFL 폰트 번들 | **본 타스크 범위 외**. 픽셀 RMSE 의 본질적 해결책으로 측정 확인. 별도 후속 이슈 권장. |
| 3) RMSE 측정 방법론 보정 | **본 타스크에서 문서화 완료** (`mydocs/tech/font_diff_rmse_normalization.md`). 구현은 별도 이슈 권장. |

## 7. 후속 이슈 권장

본 타스크에서 분리한 후속 작업:

1. **(옵션 2) 자동 폰트 임베딩 + OFL 호환 번들** — 픽셀 RMSE 의 본질적 개선
2. **(옵션 3 구현) 분리 측정 도구** — MaskedRMSE / TextBoxIoU / GlyphFreeRMSE 측정 바이너리 또는 스크립트
3. **HY 계열 패턴 확장** — HY바다/간기/산/나무/백송/해서 등 (Stage 1 §2.2 보류)
4. **함초롬·Pretendard weight 분기** — 현재 Regular/Bold 만 지원
5. **기타 한글 폰트 메트릭 DB 추가** — KoPub돋움체, 한컴 윤고딕/소망/쿨재즈, 양재튼튼체, DX새고딕, HCI Hollyhock 등 (Stage 1 §2.3)
6. **DTP 폰트 별칭** — `-윤명조120/150/320`, `Yoon가변 윤고딕 310_TT`, `한컴 윤고딕 230/240/250/760` 등

## 8. 결론

이슈 #885 의 직접 목표 (HY 계열 폴백 메트릭 보정) 는 달성. 메트릭 별칭 적용으로 텍스트 좌표/폭 계산이 정합되었고, IR 회귀 없음을 확인했다.

다만 측정 과정에서 **픽셀 RMSE 개선폭이 매우 작다**는 사실을 확인했고, 본질적 시각 정합 개선은 옵션 2 (자동 임베딩) 가 필요함을 정량으로 입증했다. 후속 이슈 6건으로 분리 권장.

## 9. 이슈 클로즈

작업지시자 승인 후:
```
gh issue close 885 --repo edwardkim/rhwp
```

후속 이슈 6건 생성도 별도 승인 후 진행.
