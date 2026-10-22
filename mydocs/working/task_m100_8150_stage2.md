# #8150 — 2단계 완료보고: 전쪽 독립 복원과 터미널 표시

- Issue: [#8150](https://github.com/edwardkim/rhwp/issues/8150)
- [수행 계획](../plans/task_m100_8150.md), [구현 절편](../plans/task_m100_8150_impl.md),
  [1단계](task_m100_8150_stage1.md)
- 승인: 2026-10-22 사용자의 「다음 단계 진행을 승인합니다」.
- 제품 코드: `2d2d9b1921eaef5f586937739920dbee4689b75d`. 이번 단계에서 Rust·Cargo·CI는 변경하지 않았다.
- 기준 devel: `2e087a5b4bc1515ad877e8020b30114639a59392`.
- 실행 Native SHA-256: `9cd196a82f728271a51099b5a9721a92589575c8e4eea784c60d70a2d574101e`.
- 기준 Native SHA-256: `1cae3dd3a9b2c28af1fed9634072c42b82311b106386a2ace2f1d77e07eb7207`.
- 중간 증거: ignored `output/issue8150/stage2/`.

## 독립 복원과 기존 출력 보존

`scripts/verify-kitty-output.py`를 추가했다. 제품의 Kitty writer를 호출하지 않고
APC 헤더·계속/종료·청크 길이·Base64 패딩·PNG signature를 검사한 뒤 Pillow로 픽셀을 복원한다.
같은 Print 프로필·배율의 기존 PNG를 흰색에 합성한 결과와 바이트 단위로 대조한다.
이전/현재 CLI의 기본·축소 PNG 픽셀과 Sixel 전송 바이트도 대조한다.
이 도구는 진단용이며 새 실물 회귀 fixture·좌표 pin·golden을 추가하지 않았다.

| 공개 입력 | 전쪽 | 너비별 검사 |
| --- | ---: | ---: |
| `samples/hwp_table_test.hwp` | 3 | 6 |
| `samples/footnote-01.hwp` | 6 | 12 |
| `samples/pic-crop-01.hwp` | 1 | 2 |
| `samples/issue7688/arrow-controls/control-compound.hwpx` | 1 | 2 |
| 합계 | 11 | 22 |

360/720px의 **22건 모두** Kitty 복원 픽셀과 흰 배경 PNG가 같았다.
전쪽 기본 PNG 11쪽과 축소 PNG 22건의 변경 전후 픽셀이 같았고,
Sixel 22건도 변경 전후 바이트가 같았다. 파일 모드 stdout은 비어 있었으며,
전쪽 파일·쪽별 stdout·마지막 쪽 선택 파일의 바이트가 같았다.
한 이미지의 청크 수는 2~53개였다(`documents-print/results.json`, `commands.jsonl`).

검사 도구의 최초 실행에서는 기본 screen SVG의 논리 크기를 Print 래스터의 크기로 사용해
표 첫 쪽의 509px 높이를 510px로 예상했다. 원인은 검사 조건의 프로필 혼합이었다.
`paint/paper_viewport.rs`의 기존 Print 종이 계약과 소비 경로를 확인하고,
비교용 SVG에도 `--backend layer --profile print`를 명시했다. 실제 출력·기준값은 바꾸지 않았다.
같은 입력을 새 출력 폴더에서 전쪽 재실행해 통과했다(`documents.log`, `documents-print.log`).

```bash
venv/bin/python scripts/verify-kitty-output.py \
  --rhwp-bin output/issue8150/candidate/rhwp \
  --baseline-bin output/issue8150/base/rhwp \
  --input samples/hwp_table_test.hwp --input samples/footnote-01.hwp \
  --input samples/pic-crop-01.hwp \
  --input samples/issue7688/arrow-controls/control-compound.hwpx \
  --out output/issue8150/independent
venv/bin/python -m unittest discover -s scripts/tests -p test_verify_kitty_output.py
```

전체 CLI 수집 실행에는 기존 Python/Pillow를 사용했고, 저장된 22건의 독립 복원을
저장소 `venv/bin/python`에서도 재검사해 같은 픽셀 일치를 확인했다.
검사기의 양성·손상 입력 검사는 2개 test method에서 13개 스트림을 검사해 통과했다.
로그 혼입·중복 키·잘못된 quiet/format·미종료 청크·손상 Base64·PNG 아님·과대 청크를 거부했다.

## kitty의 실제 표시

공식 배포의 kitty 0.49.2를 임시 경로에 풀어 Linux WSLg/X11에서 실행했다.
배포 tarball SHA-256은 GitHub release의 digest와 같은
`d573618b911e9c461bd421b96c13c74c7f1cb2f1ac9c327818d4ff84366cf5c6`이다.
사용자 설정·PATH는 바꾸지 않았고 별도 테스트 창과 로컬 제어 소켓만 사용했다.
창은 110열×65줄이며 한 줄은 18px였다. SSH는 Linux x86_64/glibc 2.44 호스트에서 실행했다.

| 경로 | 문서·너비 | 직접 관측 |
| --- | --- | --- |
| 로컬 직접 출력 | 표 360px | 한글·표·외곽 표시, 이미지 아래 프롬프트 |
| 로컬 직접 출력 | 각주 720px | 한글 본문·각주선·각주 본문 표시, 아래 프롬프트 |
| 로컬 파일 재표시 | crop 그림 720px | 그림 색·crop·종횡비 유지, 아래 프롬프트 |
| SSH 직접 출력 | 각주 720px | 원격 글꼴 공급 후 재출력·재캡처, 아래 프롬프트 |
| SSH 파일 재표시 | 복합 개체 720px | 색·표·화살표 표시, 아래 프롬프트 |

다섯 화면의 PNG를 직접 열어 원격/로컬의 해당 복원 PNG와 대조했다.
이미지는 (0, 18)에서 시작했고 마지막 내용 아래에 `EXIT=0`과 실제 셸 프롬프트가 있었다.
표·crop·복합 개체 화면의 이미지 영역 픽셀은 복원 PNG와 같았다.
각주 화면 두 건에서는 한 픽셀에 최대 1/255의 채널 차이가 있었다. PNG 전송 데이터의 차이는 없으며,
터미널 캡처까지 완전한 픽셀 동일성이라고 판정하지 않았다.
PNG 무손실 검사와 터미널 화면의 직접 판독·프롬프트 위치 판정은 구분했다.
근거는 `terminal-results.json`, `{local,ssh}-{direct,file}-*.png`, `terminal-with-fonts.log`다.

## Windows 11 WezTerm과 원격 한글 글꼴

설치된 Windows WezTerm은 `20240203-110809-5046fc22`다.
WSL을 거치는 SSH에서는 이미지가 보이지 않았으며, 사용자는 Windows 11 호스트에서
`wezterm ssh`로 직접 접속해야 한다고 접속 경로를 확정했다.
[WezTerm 자체 SSH](https://wezterm.org/ssh.html)는 내장 SSH client를 사용하는 별도 경로다.
WSL 경유 결과를 이 경로의 판정에 사용하지 않았다.

원격 검증 폴더에 같은 SHA의 CLI와 공개 샘플 4종을 준비했다. 기존 PATH의 CLI는 변경하지 않았다.
사용자는 직접 SSH에서 먼저 「보입니다」로 그래픽 표시를 확인했고,
이어 「한글 폰트가 없어 글자가 보이지 않는군요」라고 지적했다.
실제 Windows 화면에서도 이미지와 아래 프롬프트는 보이나 한글만 빈 것을 확인했다
(`wezterm-native-direct-360.png`).

원격의 전송 전 PNG에 이미 한글이 없었으므로 터미널의 글꼴 설정을 변경하지 않았다.
작업 폴더에 저장소의 `ttfs/opensource` 공개 폰트 4개와 README·OFL 라이선스를 공급했다.
6개 파일의 업로드 전후 SHA-256이 같았다. 시스템 폰트 설치·캐시 변경은 하지 않았다.
이후 원격 문서 4종의 파일·stdout을 다시 생성하고 같은 호스트의 흰 배경 PNG와 대조해
4건 모두 픽셀이 같음을 확인했다(`remote-with-fonts/results.json`).
한글이 들어간 표 PNG와 kitty의 새 SSH 화면을 직접 판독했다.
사용자도 Windows 11 직접 SSH에서 「네 이제 한글도 보여집니다」로 최종 표시를 확인했다.
글꼴 보정 뒤 Windows 화면의 별도 캡처 대신 이 사용자 판정을 기록한다.

로컬 fontconfig 파일 190개, 추가 번들 4개, 원격 fontconfig 파일 714개의 SHA 목록을
각각 `font-manifest.json`, `additional-font-manifest.json`, `remote-font-manifest.json`에 기록했다.
공급한 번들은 `remote-bundle-sha256.txt`로 연결한다. 로컬/원격 글꼴 집합과 fallback 결과는 다르며,
글꼴 차이를 문서 조판의 완전한 일치나 Kitty 어댑터의 개선으로 주장하지 않는다.
다른 작업 디렉터리에서 실행할 때는 `--font-path`의 절대 경로를 지정하도록 원격 README와
[사용 가이드](../manual/export_kitty_command.md)에 기록했다. 저장 스트림은 폰트 공급 뒤 재생성해야 한다.

## 단계 판정과 다음 절편

변경 문서 6개의 내부 링크, 매뉴얼 3개의 메타데이터, Python 구문과 공백 검사는 통과했다.
1단계에서 해시를 기록한 제품·CI 파일 12개의 바이트가 그대로임도 확인했다.

2단계의 독립 복원·기존 출력 보존·kitty 로컬/SSH 표시·문서화를 충족했다.
추가로 사용자가 지정한 Windows 11 WezTerm 직접 SSH의 그래픽·한글 표시도 확인했다.
한컴 Print PDF와의 전쪽 일치, 전체 회귀·lint·fresh WASM 등 제출 전 게이트를
이번 입출력 검사나 사용자 표시 판정으로 대신하지 않는다. 해당 항목은 승인 후 3단계에서 수행한다.
PR 제출·push·GitHub comment·merge·이슈 close는 수행하지 않았다.

[단계별 완료보고 규칙](../manual/hyper_waterfall_docs_guide.md#working--단계별-완료보고서)에 따라
검사 도구·가이드·보고서를 같은 단계 커밋에 기록하고 다음 절편 승인을 받는다.
