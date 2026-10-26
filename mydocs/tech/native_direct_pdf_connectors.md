---
kind: reference
status: active
canonical: mydocs/tech/native_direct_pdf_connectors.md
last_verified: 2026-10-26
---

# Native Direct PDF 연결선과 화살촉

## 기존 거부와 공통 replay

[#7501](https://github.com/edwardkim/rhwp/issues/7501)은 `samples/2025 행정업무운영 편람(최종).hwp`의
63번째 쪽에서 Direct PDF가 `connector line style`로 중단되는 문제다. 같은 요청의 `--page` 값은
0부터 시작하는 `62`다. 공개 원본은 10,687,488 bytes이며 SHA-256은
`40d6d05eac4d55bdc4b0c62c42d93af104d5123b447581246f36fd15de7bd46f`다.

이후 [#7697](https://github.com/edwardkim/rhwp/pull/7697)의 #7688 보정으로
`renderer::arrow`가 끝점·방향·모양·크기를 공통 path로 생산하고, Native Skia의 Line과 Path가
이를 실제로 그리게 됐다. `21ed64071f81f35f37618717693bd30f727392c9`에 통합됐고,
`496e5a0755500be979ebf053036bfa7d6bdecb91`은 선형 화살촉 `OpenArrow` 보존을 보완했다.
Direct PDF도 동일한 Skia canvas replay를 사용한다.

[#7964](https://github.com/edwardkim/rhwp/pull/7964)는 이미지 효과와 편람 조판을 통합하면서
63쪽 연결선 및 289쪽 Line 화살표의 엄격한 거부를 의도적으로 남겼다. 이 변경은 해당 거부를
공통 replay가 지원하는 항목에 한해 해제한다. Native PNG와 화살촉 geometry 생산은 바꾸지 않는다.

## Direct PDF 허용 범위

- 단일 Line의 시작·끝 화살촉과, 유한한 `connector_endpoints`를 가진 Path의 화살촉을 허용한다.
  화살촉이 없는 연결선의 선택용 끝점도 허용한다.
- 열린/채운 삼각형·오목 화살·마름모·원·사각형·선형 화살촉은 기존 공통 path를 그대로 쓴다.
  양끝의 크기 코드는 각각 `0..=8`이며 서로 다를 수 있다.
- 꺾은선과 cubic 연결선의 방향은 공통 생산자가 결정한 끝점의 접선을 따른다. PDF에서 방향이나
  크기를 별도로 추측하지 않는다. 선 본체와 화살촉은 vector path로 기록한다.
- 화살촉이 있는 연결선은 첫 MoveTo 하나와 하나 이상의 LineTo·CurveTo로 이뤄진 단일 열린
  subpath여야 한다. 실제 첫 점·마지막 점과 `connector_endpoints`가 정확히 일치해야 하며,
  빈 몸통·MoveTo만 있는 몸통·분리된 subpath·선 본체와 다른 끝점 metadata는 거부한다.
- 사전 검증은 요청한 화살촉 수와 실제 공통 path 수가 같은지, 확장된 좌표가 Skia의 유한한
  scalar 범위 안인지 확인한다. 끝점 누락·퇴화한 방향·비유한 좌표·잘못된 크기·확장 overflow는
  전체 선택 문서의 PDF를 기록하기 전에 오류로 반환한다.
- 화살촉이 요청된 쪽의 첫/마지막 segment는 유효한 접선을 직접 제공해야 한다. 특히 cubic의
  첫 control이 시작점과 같거나 마지막 control이 끝점과 같은 경우, 공통 생산자는 높은 차수의
  접선 대신 chord를 사용할 수 있으므로 현재 PDF admission에서는 거부한다. 이 대조는 Skia가
  사용하는 f32 좌표에서도 수행해, f64에서는 서로 다르지만 변환 뒤 같은 점으로 축소되는 control과
  짧은 끝 segment도 거부한다. 같은 이유로 화살촉이 있는 Line의 두 끝점도 f32에서 달라야 한다.
  해당 방향의 화살촉이 없으면 그 control의 퇴화는 허용하며, 화살촉 없는 기존 zero Line도 허용한다.
  퇴화 control의 일반화는 후속 범위다.
- 공통 생산자가 접선을 보장하지 않는 ArcTo·닫힌 Path의 화살촉은 계속 거부한다.
  화살촉 없는 일반 arc·닫힌 도형의 기존 지원은 유지한다.
- `LineRenderType::Single` 이외의 다중선과 shadow는 계속 거부한다. pattern, 비선형 gradient,
  해독할 수 없는 이미지 등 다른 기존 Direct PDF 제한도 유지한다.

이 범위는 공통 replay의 표현을 PDF에 전달하는 계약이다. 모든 화살촉 모양의 최종 한컴 일치나
편람 전체 383쪽의 Direct PDF 지원 완료를 뜻하지 않는다.

## 회귀 실행 경로

기존 `tests/render_p37_direct_pdf_export.rs`의 `connector_arrow_regressions`에 다음 검사를 둔다.
이 source는 기존 Native Skia job의 release-test와 release 경로에서 실행하므로 별도 CI 호출은
추가하지 않는다. default feature에서만 수집됐다는 사실을 Native 실행 근거로 사용하지 않는다.

- 시작·끝 triangle의 PDF content stream을 풀어 vector 꼭짓점을 확인한다. #7688의 독립
  한컴 Print 관측인 medium 길이·폭 15.36 CSS px와 원본 끝점 소유를 기준으로 하고,
  표준 PDF CTM과 72/96 단위 변환을 적용한다. 동일 arrow helper의 출력을 기대값으로 부르지 않는다.
- cubic 연결선의 수직 접선, 열린/채운 모양, 서로 다른 양끝 크기 및 비표시 metadata를 확인한다.
- 몸통/끝점 metadata의 불일치, 분리된 subpath와 퇴화 cubic control의 잘못된 tangent를 거부한다.
  화살촉을 요청하지 않은 쪽의 control 퇴화는 정상 대조군으로 허용한다.
- 무효 geometry·크기·overflow와 기존 다중선·shadow 거부를 확인한다. 두 번째 쪽이 잘못된 경우
  선택 문서 전체의 사전 검증이 오류를 내는지 확인한다.
- 단순 화살촉 PDF에 image XObject가 생기지 않는지, Native PNG에서도 선 본체 바깥의 화살촉
  픽셀이 보존되는지 확인한다.

```bash
node scripts/run-rust-test.mjs --cargo-test render_p37_direct_pdf_export -- --profile release-test --features native-skia
node scripts/run-rust-test.mjs --cargo-test issue_536_direct_pdf_linear_gradient -- --profile release-test --features native-skia
```

공개 원본의 focused smoke는 다음과 같다. 임시 PDF는 Git에서 제외된 `output/` 아래에 둔다.

```bash
rhwp export-pdf "samples/2025 행정업무운영 편람(최종).hwp" \
  --backend direct --profile print --page 62 -o output/issue7501-page63.direct.pdf
```

최종 한컴 자료·전체 코퍼스·실물 OS/GPU 대조는 사용자 지시에 따른 후속 검증이다.
실행한 검사와 현재 HEAD의 결과는 PR 본문에 별도로 기록한다.
