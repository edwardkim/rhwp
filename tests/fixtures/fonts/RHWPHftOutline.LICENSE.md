# HFT 윤곽 paint 계약 글꼴

`scripts/generate_hft_outline_fixture.py`가 생성한 독창적인 사각형이며 저장소의 MIT
라이선스로 배포한다. U+AC00 하나를 1000 UPM의 (100,0)–(400,700) 사각형에 매핑한다.
독립 한컴 출력이나 실제 문서의 배치 정답지가 아닌 font-only 계약 입력이다.

한컴 문서와의 실제 일치는 #7902의 원본·독립 Print 및 Native/fresh WASM 전쪽 비교로 확인한다.

재생성: 저장소 루트에서 `venv/bin/python scripts/generate_hft_outline_fixture.py`.

`RHWPHftOutlineJamo.otf`는 같은 독창적인 사각형에 U+A960 매핑만 추가한다. 원래 HFT
프로그램에도 옛자모가 있을 때 `RHWP Old Hangul` 우선 선택을 보존하는 font-only 계약이다.
재생성은 같은 script이며 `--jamo-only`는 원래 파일을 다시 쓰지 않는다.
