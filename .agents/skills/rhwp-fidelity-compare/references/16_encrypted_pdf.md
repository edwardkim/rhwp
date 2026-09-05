# 16 — 예외: 암호화 PDF (F13)

기준 PDF 가 암호로 잠겨 있으면 pypdf / pypdfium2 가 텍스트와 래스터를
읽지 못한다. 이 스킬은 **정지** 한다. 암호를 우회하거나 제거하는
CLI 를 발명하지 않는다.

## 증상

구현은 버전마다 예외 이름이 다르다.

- `pypdf.errors.FileNotDecryptedError`
- `PdfReadError: file has not been decrypted`
- pypdfium2 가 빈 문서 / 예외
- 사용자 말: "열 때 암호를 물어봅니다"

하네스가 암호 프롬프트를 제공하지 않는다. 빈 텍스트층이 전량 소실로
보이면 **추출 실패** 인지 암호인지 먼저 가른다. `PdfReader` 의
`is_encrypted` 가 참이면 F13 이다.

## 처방

1. 비교를 멈춘다. text-report 를 결함으로 읽지 않는다.
2. 사용자에게 **잠금 해제된 공식 PDF** 를 다시 달라고 한다.
   한컴에서 암호 없이 다시 내보내면 그게 새 오라클이다.
3. 새 파일의 provenance 에 `encryption=none`, 도구·버전·메뉴를 적는다.
4. 암호를 에이전트에게 보내 달라고 하지 않는다. 로그에 암호가
   남지 않게 한다.
5. `qpdf --decrypt`, `pdftk`, `mutool` 래퍼를 이 저장소에 추가하지
   않는다. 새 rhwp 하위명령도 없다 (F06).

## 원본 HWP 암호

이 장은 **기준 PDF** 의 암호 다. 원본 HWP 가 암호면 `rhwp` 가
별도 경로(`--password` 등, 이미 있는 CLI) 를 쓴다. 그 계약은
`rhwp-cli` 스킬이다. 이 스킬이 암호 입력을 재작성하지 않는다.
PDF 오라클이 잠겨 있으면 HWP 를 열 수 있어도 비교는 정지다.

## 레시피 — 탐지만

```python
from pypdf import PdfReader
r = PdfReader("oracle.pdf")
print("encrypted", r.is_encrypted, "pages", len(r.pages) if not r.is_encrypted else "?")
```

`is_encrypted` 가 참이면 여기서 끝. 빈 비밀번호로
`decrypt("")` 를 시도하는 자동화는 하지 않는다. 일부 생성기가
빈 암호 플래그만 켜 두지만, 그걸 푸는 도구를 이 스킬에 넣지 않는다.
사용자에게 "빈 암호인지, 잠금 해제본을 주실 수 있는지" 물어본다.

## 에이전트 금지

- 암호 크랙, 사전 공격, 온라인 해제 사이트
- 암호를 SKILL / 이슈 / PR 본문에 붙여 넣기
- `fidelity_compare.py` 에 `--password` 를 이 이슈에서 추가
- 암호화된 채 추출된 빈 텍스트를 "전량 소실 버그" 로 승격
