# Task #311 3단계 완료 보고서: 부정적 발견 문서화 + 다음 sub-issue 제안

상위: 구현 계획서, Epic #309

## 변경

당초 계획(default on 전환)은 보류. 3단계는 다음으로 변경 수행:

- `mydocs/tech/line_seg_vpos_analysis.md` 부록 A 추가 — Task #310 가설의 검증 결과
- `mydocs/report/task_m100_311_report.md` 최종 보고서 — 부정적 결과 + 다음 작업 후보
- `mydocs/orders/20260425.md` 갱신 (다음 커밋)
- Epic #309 코멘트 게시 (다음 커밋)

## 핵심 결과

| 샘플 | OFF | ON | 평가 |
|------|-----|----|----|
| 21_언어 | 19 | **20** | ❌ 가설 부정 |
| 다른 3샘플 | (각각) | (각각) | 무변화 |

vpos-reset 강제 분리만으로는 21_언어 +4쪽 문제 해결 불가. 진짜 원인은 column 가용 공간 계산 차이로 재추정.

## 다음 단계

- Sub-issue #311 클로즈 (작업지시자 승인 후)
- Epic #309에 신규 sub-issue 제안: column 가용 공간 정확도 조사
