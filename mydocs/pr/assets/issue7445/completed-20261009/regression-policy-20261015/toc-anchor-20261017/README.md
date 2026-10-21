# 목차 숫자 끝점과 리더 소유

수동 한컴 Print64쪽의 숫자26개는 같은 오른쪽 끝에 정렬된다.
같은 런의 탭+숫자만0.25em 왼쪽으로 옮기던 공통 측정/문자 위치 감산을 제거했다.
교차 런·한 런 모두 동일한 정지를 사용하며 리더 여백은 숫자 앵커에 다시 반영하지 않는다.

[검증 신원·전쪽 결과](verification.md), [독립 숫자 끝점](verification.md),
[원인·소비 경로](verification.md), [Native 전64쪽](whole64-native.tsv), [fresh WASM 전64쪽](whole64-wasm.tsv).
수치는 색상 경계 보정 전2px raw다. Native2쪽은98% 미만이므로 새 절대 좌표 검사는 추가하지 않는다.
기존 완료19쪽의 독립 제목 좌표/내용 소유 검사와 양쪽98% 이상은 유지한다.

| 출력 | 수정 전 | 수정 후 |
| --- | --- | --- |
| Native | [review](before-native-review.png) · [overlay](before-native-overlay.png) | [review](after-native-review.png) · [overlay](after-native-overlay.png) |
| fresh WASM | [review](before-wasm-review.png) · [overlay](before-wasm-overlay.png) | [review](after-wasm-review.png) · [overlay](after-wasm-overlay.png) |

실제 Studio Canvas2D Print도 [review](actual-studio-review.png)와
[overlay](actual-studio-overlay.png)를 같은96DPI 독립 Print와 대조했다.
제품 default2022의 관측이며 portable compat2024 전쪽192DPI 검증과 구분한다.
실제 초기화 JS/WASM 응답2개 hash가 동결 package와 같고 임의 paint 변경이나 mask는 없다.

집중 회귀13+16+17 PASS는 최종 전체 회귀 완료가 아니다.
기존 x650 리더 핀은 전쪽 검증 뒤 같은 줄 숫자를 침범하지 않는 관계로 교체했고, 실제13건 모두 통과했다.
upstream/devel 전쪽 비교·전체 nextest·필수 gate·전체 실제 소비 경로는 아직 완료하지 않았다.
로그는 ignored output에 보존하며 이 asset에는 포함하지 않는다.
