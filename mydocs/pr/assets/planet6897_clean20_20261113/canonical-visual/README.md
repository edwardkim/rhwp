# 정본 PDFium 기준 Native/fresh WASM 전쪽 TSV

코드 `676e3900c6490657229f9781e34f89c65c7126da`, 고정 base `31a422575722f634cfc2d9eb97e708da58aea188`의 수치 기록이다.

- 각 backend: 주장 22조건의 후보·base 각 380쪽, 보호 18조건 1,771쪽.
- PDFium 5.13.0 / library `33c98063af28c0b7cbf8227f4422bf5c15942df2455cf7f0a5dce3dc601d52b0`, 96 DPI, 원 조건별 compat, raw 2px 실루엣을 유지했다.
- 보호 18조건 모두 공식 두 자리 표시값 하락 0쪽이다. Native는 원 PNG의 생산자·파일 SHA를 검사해 재사용하고 PDF PNG·TSV를 전쪽 다시 산출했다. WASM은 fresh 로컬 대체 빌드의 실제 Print Canvas PNG다.
- `pr9015` base는 실제 9쪽/PDF 8쪽 불일치를 보존하며 공통 물리 1~8쪽을 진단했다. 후보는 8쪽으로 일치한다. base의 쪽수 통과를 주장하지 않는다.
- 원 PR이 공개한 보건·식량 보고서의 악화는 별도 보정 대상이다. 이 TSV로 반례 악화나 실제 backend 검증·확대 판독을 면제하지 않는다.
- 이후 보정 후보의 새 head 통과 증거로 이 기록을 바꾸어 인용하지 않는다.

세부 점수와 TSV SHA는 [qualification.json](qualification.json)에 연결한다.
