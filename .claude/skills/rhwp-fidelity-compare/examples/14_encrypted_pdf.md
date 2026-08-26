# 예제 — 암호화 PDF 정지

이슈 #5329. 실 에이전트 경로. gym 아님.

## 상황

기준 PDF 가 열릴 때 암호를 묻는다. 또는 `PdfReader.is_encrypted`.

## 행동

비교를 멈춘다. 잠금 해제된 공식 PDF 를 다시 받는다.
`qpdf --decrypt` 래퍼를 만들지 않는다. 새 CLI 를 만들지 않는다.
암호를 이슈에 붙이지 않는다.

text-only 여도 정지다. 빈 텍스트층을 전량 소실로 읽지 않는다.

관련: `references/16_encrypted_pdf.md`.
전사: `fixtures/transcripts/encrypted_pdf.txt`.
정지 F13.
