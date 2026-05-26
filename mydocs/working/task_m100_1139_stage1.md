# Stage 1 진단 보고서 — Task #1139

## 대상

- 파일: `samples/3-09월_교육_통합_2022.hwp`
- 페이지: 5쪽, export 옵션 기준 `-p 4`
- 비교 기준: 작업지시자가 제공한 한컴오피스 2022 화면

## 수행

```bash
./target/release/rhwp dump-pages samples/3-09월_교육_통합_2022.hwp -p 4
./target/release/rhwp dump samples/3-09월_교육_통합_2022.hwp -s 0 -p 278
./target/release/rhwp dump samples/3-09월_교육_통합_2022.hwp -s 0 -p 288
./target/release/rhwp dump samples/3-09월_교육_통합_2022.hwp -s 0 -p 320
./target/release/rhwp dump samples/3-09월_교육_통합_2022.hwp -s 0 -p 321
./target/release/rhwp export-svg samples/3-09월_교육_통합_2022.hwp -p 4 -o output/diag_1139_plain
```

## 확인 결과

- 문23 `lim _{x\` -> \`0} ...` 수식은 `lim`, `→`, 분수 구조가 SVG에 정상 렌더된다.
- 문24 `int _{0} ^{pi } {} x\`cos LEFT ( {pi } over {2} -x RIGHT ) dx` 수식은 `LEFT/RIGHT` 명령이 문자로 새지 않고 `LayoutKind::Paren`으로 렌더된다.
- 다만 문24의 큰 둥근 괄호가 `Q` 단일 곡선과 얇은 선폭으로 출력되어 한컴 대비 세로 막대처럼 보일 수 있다.
- 문27 오른쪽 문단의 작은 `△△`처럼 보이는 항목은 Equation 명령 누출이 아니라 `Control::Picture` TAC 그림이다. SVG에서도 `<image>`로 출력되므로 이번 이상 문자 원인에서 제외한다.
- SVG에서 `>LEFT<`, `>RIGHT<`, `>it<`, `>ANGLE<` 문자열 누출은 확인되지 않았다.

## 결론

이번 수정 대상은 parser/tokenizer가 아니라 수식 렌더러의 stretched round parenthesis path이다. SVG와 rhwp-studio WASM Canvas 경로를 함께 보정해야 한다.

