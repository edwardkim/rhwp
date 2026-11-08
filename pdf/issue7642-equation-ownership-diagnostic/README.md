# 수식 항 소속 독립 대조군

Issue: #7642

한컴 정상 문서의 스타일과 구역을 재사용한 구조 진단 입력이다. 정식 회귀 fixture가 아니다.
`ownership-input.hwpx`의 크기·기준선·LineSeg를 비우고 한컴 Office 2020에서 HWP로 정상 저장한 뒤
그 HWP를 Print로 출력했다. PDF는 RHWP 출력이나 재저장본이 아니다.

- 정상 저장 job: `a892ec0a-ffbd-4c47-81f0-8be60c6e2545`
- Print job: `348a6d88-a1cf-470f-9529-17e68a82b464`
- 엔진: Hancom Office 2020, `11.0.0.9136`, `PrintMethod=0` (1-up).
- 1–15쪽: EQALIGN의 그룹·항 종류·칸 경계.
- 16쪽: 내용 없는 PILE의 호스트 문단과 후속 본문.
- 17–20·27–30쪽: 공식 중위 LSUB/LSUP과 그룹·혼재·연쇄.
- 21–26·31–32쪽: prefix 구문 오류 대조. 정상 중위 문법과 구분하며 원본을 보존한다.

같은 단어 EQALIGN의 equation 레코드는 이전 구조 대조군과 같지만 두 문서의 Print 정렬이 다르다.
이전 HWP를 그대로 두 번 재출력해도 그 문서의 기존 정렬은 재현된다. 원인이 아직 확인되지 않아
타입별 정렬 예외나 새 회귀 기대값의 근거로 사용하지 않는다. 모든 검증 완료를 뜻하지 않는다.

## 파일 SHA-256

- `ownership-hancom-2020.hwp`: `c8131c4551489cdb2b93a0ca3f52d666984c2c078b738b466f2a19fcd36bacb5`
- `ownership-hancom-2020.pdf`: `5356dfa18a1a139096a5ca25e1b3bb5c40bcf4c05f8ccfdc64a56d120a6f23b0`
- `ownership-input.hwpx`: `32ff253e5fdae35c83ed322b5439862524bc63abe86d0556676b9f7a0d1cc861`

## 항의 순서 대조 5쪽

별도 `alignment-order-*` 입력은 단어 EQALIGN을 첫 식으로 두고, 그룹 식·동일 단어 식·분수 식·동일 단어 식 순서로 Print했다. 다섯 쪽 모두 & 열을 공유했다. 이전 HWP의 로컬 행 정렬과 달라 원인을 아직 특정하지 못했다.
정상 저장 job `4cc5dd96-0560-47eb-a676-144874dec0b5`, Print job `9b77a5c3-ff5b-455c-a347-5247e7b8341c`. 엔진/인쇄 방식은 위 32쪽 대조군과 같다.

- `alignment-order-hancom-2020.hwp`: `7ed07ee13d97c09823c78fd35241da1709258edec3b36519a0b03088884fc1cd`
- `alignment-order-hancom-2020.pdf`: `dbaf9a1e8126986666543289d3be7f870f429db848ad126e7ab6df28e89f24c3`
- `alignment-order-input.hwpx`: `e6937fa7a0b596af83d4c8d5ed972b28f28259a1c4cc871b08daecb95d008bbc`
