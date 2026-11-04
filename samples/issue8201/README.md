# #8201 서명·날인 기호 U+F012B 재현 입력

## 인감증명법 시행령 [별지 제13호서식]

인감증명서 발급 위임장 또는 미성년자의 법정대리인ㆍ한정후견인 및 성년후견인 동의서 (개정 2020. 2. 18.)

- 파일: `seal-delegation-form.hwp` (HWP 5.x, 2쪽)
- 출처: 국가법령정보센터 공개 빈 서식(개인정보 없음),
  `https://www.law.go.kr/LSW/flDownload.do?flSeq=145805913`.
  [#8201](https://github.com/edwardkim/rhwp/issues/8201) 본문의 재현 링크다.
- 받은 날짜: 2026-10-24. 내용 변경·재저장 없이 바이트 그대로 보존했다.
- 크기: 19,456바이트.
- SHA-256: `6fac28bd4e08219e909a538c297cbd193af10961c6b851337c893365717ac695`.
- 재현 위치: 1쪽 아래 동의 칸 `인감 \u{F012B} 또는 서명` 줄. PARA_TEXT에 U+F012B 한 글자로
  저장되어 있고 글자겹침(tcps) 조판 부호가 아니다. 문단 글자모양은 10pt·장평 100·자간 −31%이다.

## 기준 PDF

- 파일: `seal-delegation-form-2020.pdf` (2쪽, A4 595×841pt)
- 출처: 국가법령정보센터가 같은 서식과 함께 공개한 PDF,
  `https://www.law.go.kr/LSW/flDownload.do?flSeq=145805915`. 이슈 본문의 비교 기준이다.
- `Creator: Hwp 2020 1.0.0.0`, `Producer: Hancom PDF 1.3.0.546`, PDF 1.4.
  메타데이터만으로 Print/저장 경로는 확정하지 않는다.
- 크기: 72,007바이트.
- SHA-256: `43ed1a9c58c0b221c391703fb2f76f658f520e75d64e7b213b3c9b2fa477a5a8`.
- SHA-1: `8081e84e8aad93c60e07ed16b69b69e36ed673ea`.
- 대응: 원본 2쪽과 같은 서식·내용·쪽수다. 기호는 내장 Haansoft Batang GID 3(`/W` 1000)으로 그려진다.

이 입력의 보존은 시각 검증 완료 판정과 별개다. 전쪽 TSV·직접 판독 결과는
[#8201 계획](../../mydocs/plans/task_m100_8201.md)과 결과 보고서에 연결한다.
