# 04 — 페이지 시트 `cmp-pNNN.png`

시트 모드는 기준 PDF 래스터와 rhwp SVG 를 Chrome 으로 잡은 PNG 를
한 장에 나란히 붙인다. 파일 이름은 `cmp-pNNN.png` (1-based, 3자리).

이 장은 **사람이 볼 자료** 를 만드는 법을 다룬다. 숫자 랭킹은 05장,
Chrome 부재는 13장.

## 왜 시트인가

픽셀 diff% 만 보면 자간 프린지와 표 붕괴가 같은 숫자가 된다. 시트는
왼쪽(한컴 PDF) 과 오른쪽(rhwp) 을 같은 스케일로 보여 주어, 유지자가
"글자만 두껍다"와 "행이 한 쪽 밀렸다"를 구분하게 한다.

거버넌스의 OVL 패널(R=오라클, G=B=rhwp) 은 별도 산출이다. 이 하네스의
기본 시트는 side-by-side 비교 PNG 다. OVL 을 이 스킬이 재구현하지 않는다.

## 호출

```bash
# Chrome 이 PATH 또는 기본 경로에 있어야 한다
venv/bin/python tools/fidelity_compare/fidelity_compare.py plan 0 9 \
  --out-dir /tmp/rhwp-fidelity-plan
ls /tmp/rhwp-fidelity-plan/cmp-p00*.png
```

`--text-only` 를 주면 시트를 만들지 않는다. `report.tsv` 의 `diff%` 열은
`not-run` 이 된다. 글자 후보만 필요하면 그게 맞다 (F02).

## 창 크기

초기 버전은 고정 창으로 SVG 를 캡처해 A3(`korexam`) 를 잘랐고, 잘린
여백이 가짜 고 diff% 가 됐다. 현재 하네스는 **SVG 판형을 읽어 창을
맞춘다.** 에이전트가 `--window-size=1920,1080` 을 새로 붙이지 않는다.
그런 플래그는 이 도구에 없다.

A3 2단 시험지를 볼 때:

```bash
venv/bin/python tools/fidelity_compare/fidelity_compare.py korexam 0 2 \
  --out-dir /tmp/rhwp-fidelity-korexam
# cmp-p001.png 가 잘리지 않았는지 먼저 본다
```

잘렸으면 도구 버그 후보다. 문서 회귀로 승격하지 말고 README 함정
노트를 인용해 이슈를 연다.

## Chrome 캡처

`capture_with_chrome` 은 `--headless=new --disable-gpu --screenshot=`
을 쓰고, 실패하면 **한 번 재시도** 한 뒤 stderr 와 exit 를 표면화한다.
에이전트는 재시도 루프를 바깥에 또 돌리지 않는다. 두 번 실패하면
그 쪽은 `비교 시트 PNG 실패` 노트로 남고, `run-state` 가 incomplete 가
될 수 있다 (F12).

캐시: 같은 `--out-dir` 에 이미 크기 > 0 인 PNG 가 있으면 재캡처하지
않는다. 글꼴을 고친 뒤에는 `--out-dir` 을 바꾸거나 해당 PNG 를 지운다.

## 시트와 원본

시트는 산출이다. 원본 HWP/PDF 를 덮어쓰지 않는다 (F18).
`cmp-pNNN.png` 를 `mydocs/pr/assets/` 에 올리려면 거버넌스 파일 이름
규약(`pr{번호}_{주제}_review_p{페이지}.png`) 을 따르고, 이 스킬이 그
복사를 자동화하지 않는다.

## 보는 순서

1. `run-state.tsv` 가 complete 인가. 아니면 누락 쪽부터 (F12)
2. 시트가 □ 투성이인가. 이면 F14, 09·17장
3. `report.tsv` 상위 쪽의 시트를 연다 (F03)
4. 왼쪽·오른쪽에서 **구조**(표 격자, 단, 각주 영역) 가 같은지
5. 글자 프린지는 캡션에 "폰트 메트릭" 이라고 적고 넘어갈 수 있다
6. 구조가 다르면 후보를 이슈 초안으로만 남기고 유지자 판정을 기다린다 (F05)

## 레시피 — 세 쪽만 시트

긴 문서는 전수 시트가 비싸다. 텍스트 전수로 쪽을 고른 뒤 그 창만
시트로 올린다.

```bash
# 1) 전수 텍스트
venv/bin/python tools/fidelity_compare/fidelity_compare.py plan 0 34 \
  --text-only --export-all-svg --out-dir /tmp/rhwp-fidelity-plan
# 2) text-report 상위 쪽만 시트. 같은 out-dir 이면 SVG 캐시 재사용
venv/bin/python tools/fidelity_compare/fidelity_compare.py plan 11 12 \
  --out-dir /tmp/rhwp-fidelity-plan
```

`--export-all-svg` 가 채운 `svg/` 를 두 번째 호출이 재사용한다.
`RHWP_BIN` 이 바뀌었으면 캐시가 옛 바이너리일 수 있다. 그때는
`--out-dir` 을 새로 잡는다.

## 에이전트 금지

- 고정 해상도 캡처 스크립트를 새로 쓰기
- 시트를 자동 OVL 로 "통과" 처리
- 시트 없이 diff% 만으로 이슈를 닫기
- Chrome 없는 환경에서 시트 모드를 반복 재시도
