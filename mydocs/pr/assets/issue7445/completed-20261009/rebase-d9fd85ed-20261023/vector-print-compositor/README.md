# Print 벡터 합성 소비 검증

source hash·검증 범위·PNG 출처는 [최종 증적](validation.json)에 있다.
부모 `f1f52d57bdf5cb42b4cea2a2515a9985e2dc67b7`, base `d9fd85ed5bc3c4d9ce6b2981422e48bf9bb90867`.
이 자료는 Studio 소비 경계 검증이며 전체 50조건의 전쪽 수용이 아니다.

| 범위 | 건수 | 최저 raw | 90% 미만 |
| --- | ---: | ---: | ---: |
| [독립 Print 비교](independent-print-review.tsv) | 31 | 93.15185% | 0 |
| [확대·DPR·복원](zoom-independent-review.tsv) | 20 | 90.00026% | 0 |

확대 자료는 같은 source로 성공한 3개 입력 15건의 hash 검증 재사용과 새 로드맵 5건을 합쳤다.
Print 31건은 새 캡처다. caret을 제품 메서드로 숨긴 전후 Canvas 바이트 불변을 확인했다.

## 저장 줄 결측 대조군, 확대 1.25 / DPR 1.5

정확한 이전 Git 메서드와 현재 메서드를 같은 완성 Canvas에 적용한 **정책 대조**다.
전체 base 앱의 전후 검증은 아니다. 내용 소속을 보존하며 괘선·글자의 미세 래스터 차이는 남는다.

| 정책 | raw | review | overlay |
| --- | ---: | --- | --- |
| 이전 | 86.78369% | [review](missing-lineseg-zoom-before-policy-review.png) | [overlay](missing-lineseg-zoom-before-policy-overlay.png) |
| 현재 | 90.00026% | [review](missing-lineseg-zoom-current-policy-review.png) | [overlay](missing-lineseg-zoom-current-policy-overlay.png) |

## 기존 입력 대표

| 입력 | raw | review | overlay |
| --- | ---: | --- | --- |
| sample11 원본 50쪽 Print | 93.15185% | [review](sample11-p050-print-review.png) | [overlay](sample11-p050-print-overlay.png) |
| 로드맵 153쪽 DPR 1.5 | 96.10150% | [review](roadmap-p153-dpr15-review.png) | [overlay](roadmap-p153-dpr15-overlay.png) |

sample11의 독립 Print와 편집기 그림 면·색상 차이가 이번 CSS 보정으로 해소됐다는 뜻은 아니다.
로그와 관측기 등 임시 파일은 `output/pr-review/7445-completed-20261009/rebase-d9fd85ed-20261023/`
아래에 보존한다. 이 자료의 PNG는 실제 캡처·정본 비교 도구의 산출물을 바이트 그대로 복사했다.
