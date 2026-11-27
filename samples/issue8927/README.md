# #8927 삽입 수식 저장 프레임 표본

| 파일 | 출처 | SHA-256 앞 12자리 |
|---|---|---|
| `base.hwp` | 이슈 첨부 원본(220×280mm, 본문 문단에 저장 줄 없음) | `6a23e7a84fe8` |
| `rhwp-HYhwpEQ.hancom-relayout.hwp` | 이슈 첨부. 같은 수식 7개를 한글 2020(11.0.0.9166)이 다시 조판해 저장 | `3f74388395da` |
| `rhwp-HancomEQN.hancom-relayout.hwp` | 이슈 첨부. 위와 같고 수식 글꼴만 HancomEQN | `50af1f33b658` |
| `base-a4.hwp` | `base.hwp`를 한글 2022(12.0.0.4645)에서 열어 편집 용지만 A4로 바꿔 저장 | `66d2351a7768` |
| `a4-inserted-HYhwpEQ.hwp` | `base-a4.hwp`에 `rhwp edit insert-equation`으로 이슈의 수식 7개를 넣음(fd008abb42) | `d51e2042e767` |
| `a4-inserted-HancomEQN.hwp` | 위 파일의 수식 7개를 `rhwp edit set-equation-properties --props '{"fontName":"HancomEQN"}'`로 바꿈 | `71088a5cb141` |

`pdf/issue-8927/a4-inserted-*-2022.pdf`는 위 두 `a4-inserted-*.hwp`를 한글 2022 12.0.0.4645에서
열어 Hancom PDF(1.3.0.550)로 A4·100% Print한 것이다. 본문 글꼴 `KoPubWorld바탕체 Light`를 설치한
환경이며, 그 밖의 한컴바탕 글리프는 수식 `⌊`·`⌋` 두 개뿐이다.

수식 삽입 위치(문단, 글자 위치)는 이슈의 재현 순서를 따른다:
(0,0) `a ^{2} + b ^{2} = c ^{2}`, (1,4) `alpha + beta LEQ gamma`, (2,4) `a times b`,
(3,4) `e ^{i pi} + 1 = 0`, (4,4) `LFLOOR a RFLOOR`, (5,4) `1 over {2 over 3}`, (6,4) `prod _{i} x`.
글자 크기는 모두 `--font-size 1000`이다.
