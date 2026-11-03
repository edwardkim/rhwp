# #8539 쪽을 넘는 RowBreak 표 칸의 한 줄 이른 분할 재현 입력

## 국민건강보험법 시행규칙 [별표 1]

피부양자 자격의 인정기준 중 부양요건(제2조제1항제1호 관련) (개정 2018. 3. 6.)

- 파일: `nhis-dependent-requirements.hwp` (HWP 5.0.3.4, 3쪽)
- 출처: 국가법령정보센터 공개 빈 서식(개인정보 없음),
  `https://www.law.go.kr/LSW/flDownload.do?flSeq=169566169`.
  [#8198](https://github.com/edwardkim/rhwp/issues/8198) 본문의 재현 링크이며, 남은 차이를 #8539로 분리했다.
- 받은 날짜: 2026-11-02. 내용 변경·재저장 없이 바이트 그대로 보존했다.
- 크기: 17,920바이트.
- SHA-256: `53d7763a0e5c9860e62c8dc3b20cfb515aac6f6eceeeef01ff0fb1abbdfe8fef`.
- 재현 위치: 구역0 문단1의 18행×3열 RowBreak 표.
  - 9행 「5. 손ㆍ외손」: 1→2쪽 경계. 셀[24]·셀[25]의 `ls[2]`가 vpos 0 재시작이다.
  - 16행 「10. 형제ㆍ자매」: 2→3쪽 경계. 셀[44] `p[4] ls[0]`과 셀[46] `ls[9]`가 vpos 0 재시작이다.

## 기준 PDF

- 파일: `nhis-dependent-requirements-2020.pdf` (3쪽, A4 595×841pt)
- 출처: 국가법령정보센터가 같은 서식과 함께 공개한 PDF,
  `https://www.law.go.kr/LSW/flDownload.do?flSeq=169566171`. #8198 본문의 비교 기준이다.
- `Creator: Hwp 2020 1.0.0.0`, `Producer: Hancom PDF 1.3.0.546`, PDF 1.4.
  메타데이터만으로 Print/저장 경로는 확정하지 않는다.
- 크기: 52,093바이트.
- SHA-256: `a54a96205d1916b44d98a71caf6114d47ddb34ebfa9f34263228f875e90e5da6`.
- SHA-1: `a56218fe63a526b37ab0ad5980192da6d6d1c7f2`.
- 대응: 원본 3쪽과 같은 서식·내용·쪽수다. 내장 글꼴은 HY신명조(H2mjsM)·HY헤드라인M(H2hdrM)이다.
- 관측: 9행은 칸별 1쪽 줄 수 2 / 3 / 3, 16행은 2쪽 줄 수 10 / 8 / 10이다.

이 입력의 보존은 시각 검증 완료 판정과 별개다. 전쪽 TSV·직접 판독 결과는
[#8539 계획](../../mydocs/plans/task_m100_8539.md)과 결과 보고서에 연결한다.
