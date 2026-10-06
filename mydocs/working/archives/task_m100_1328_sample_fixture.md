# Task M100-1328 샘플 fixture 작성 절차 — local-font-nanumsquare-bold.hwpx

- 이슈: https://github.com/edwardkim/rhwp/issues/1328
- 작성일: 2026-06-21
- 샘플 파일: `samples/hwpx/local-font-nanumsquare-bold.hwpx`
- 기반 파일: `samples/hwpx/issue_241.hwpx`
- 목적: rhwp-studio의 로컬 글꼴 감지 동의 UX와 감지 후 실제 렌더링 변경을 검증한다.

이번 작업에서는 AI가 검증용 HWPX 샘플을 직접 만들고, rhwp-studio에서 로컬 글꼴 감지 승인 전/후 PNG 비교까지 수행했다. 이 문서는 그 구체 절차와 판단 근거를 남기는 working 기록이며, 반복 가능한 원칙은 `mydocs/manual/ai_sample_document_authoring_guide.md`로 일반화했다.

## 1. 샘플 요구사항

이번 샘플은 단순히 로컬 글꼴 감지 모달만 띄우는 파일이 아니라, 사용자가 감지를 승인했을 때 실제 렌더링이 달라지는지 확인할 수 있어야 한다.

필수 조건은 다음과 같다.

1. 문서에 rhwp 기본 웹폰트로 등록되지 않은 글꼴이 있어야 한다.
2. 그 글꼴은 사용자가 로컬에 설치할 수 있는 공개 글꼴이어야 한다.
3. 문서 본문에 해당 글꼴을 사용하는 텍스트가 눈에 띄게 포함되어야 한다.
4. 문서 자체에는 글꼴 파일을 임베드하지 않아야 한다.
5. 기존 parser/renderer 결함과 섞이지 않도록 HWPX 구조는 이미 검증된 샘플을 기반으로 해야 한다.
6. 감지 대상 글꼴 수가 과도하게 많지 않아 모달의 상태 요약을 해석하기 쉬워야 한다.

## 2. 판단 흐름

처음에는 두 가지 선택지가 있었다.

1. rhwp-studio에서 새 HWPX를 직접 작성한다.
2. 기존 샘플에 포함된 글꼴을 새로 설치하거나, 기존 HWPX fixture를 수정해 검증용 문서를 만든다.

새 HWPX를 rhwp-studio에서 직접 만드는 방식은 사용자가 절차를 이해하기 쉽지만, 현재 저장 경로가 이번 이슈의 구현 변경과 섞일 수 있다. 저장 결과가 에디터 구현 상태에 영향을 받으면, 로컬 글꼴 감지 UX 검증용 샘플인지 저장 기능 검증용 샘플인지 경계가 흐려진다.

기존 큰 샘플에 필요한 글꼴을 설치하는 방식은 실제 문서에 가깝지만, 문서에 필요한 글꼴이 많아 모달 요약이 복잡해진다. 또한 사용자의 로컬 환경에 이미 설치된 글꼴 상태에 따라 재현성이 떨어진다.

따라서 기존에 rhwp가 정상적으로 읽는 작은 HWPX fixture를 기반으로 두고, 글꼴 선언과 일부 표시 문구만 바꾸는 방식을 채택했다. 이 방식은 HWPX 컨테이너, 이미지, 표, lineSeg 구조를 유지하면서 로컬 글꼴 감지 케이스만 좁게 만들 수 있다.

## 3. 기반 파일 선택

기반 파일은 `samples/hwpx/issue_241.hwpx`로 선택했다.

선택 이유는 다음과 같다.

1. 1쪽짜리 작은 HWPX라 rhwp-studio에서 빠르게 열 수 있다.
2. 기존 문서가 표, 이미지, 일반 문단을 포함하고 있어 실제 문서 구조를 유지한다.
3. `Contents/header.xml`의 글꼴 선언이 단순하다.
4. 기존 외부 글꼴이 `Pretendard ExtraBold`, `Pretendard Medium` 두 계열로만 제한되어 있어, 검증용 글꼴로 치환하기 쉽다.
5. `BinData/image1.png`와 `Preview/PrvImage.png`를 그대로 재사용할 수 있어 샘플 생성 범위를 글꼴과 텍스트에 집중할 수 있다.

기반 파일의 글꼴 목록은 다음처럼 확인했다.

```bash
unzip -p samples/hwpx/issue_241.hwpx Contents/header.xml \
  | rg -o 'face="[^"]+"' \
  | sort -u
```

기반 파일의 주요 글꼴은 다음이다.

```text
face="Pretendard ExtraBold"
face="Pretendard Medium"
face="함초롬돋움"
face="함초롬바탕"
```

## 4. 대상 글꼴 선택

검증 대상 글꼴은 `나눔스퀘어 Bold`로 선택했다.

선택 이유는 다음과 같다.

1. rhwp-studio의 기본 번들 웹폰트 목록에는 `나눔고딕`, `나눔명조`, `나눔고딕코딩`은 있지만 `나눔스퀘어 Bold`는 없다.
2. 공개 배포되는 한글 글꼴이라 검증자가 로컬에 설치하거나 상태를 통제하기 쉽다.
3. 본문 고딕 계열이므로 로컬 적용 전후의 획 굵기와 폭 차이를 눈으로 비교하기 쉽다.
4. 문서에서 필요한 외부 글꼴을 하나로 좁힐 수 있어 모달에서 `로컬 확인 필요: 1개` 형태로 검증하기 좋다.

중요한 점은 `나눔스퀘어 Bold`를 HWPX에 임베드하지 않았다는 것이다. 문서는 글꼴명만 참조한다. 따라서 rhwp-studio는 사용자가 승인하기 전까지 로컬 글꼴 목록을 읽지 않고, 승인 후에만 해당 글꼴이 로컬에 있는지 확인해야 한다.

## 5. HWPX 수정 절차

HWPX는 zip 컨테이너이므로 임시 디렉터리에 풀어서 XML과 미리보기 텍스트를 수정한 뒤 다시 패키징했다.

### 5.1 압축 해제

```bash
rm -rf /tmp/rhwp-local-font-sample
mkdir -p /tmp/rhwp-local-font-sample
unzip -q samples/hwpx/issue_241.hwpx -d /tmp/rhwp-local-font-sample
```

### 5.2 글꼴 선언 변경

`Contents/header.xml`에서 기존 외부 글꼴 두 개를 모두 `나눔스퀘어 Bold`로 변경했다.

변경 전:

```text
Pretendard ExtraBold
Pretendard Medium
```

변경 후:

```text
나눔스퀘어 Bold
나눔스퀘어 Bold
```

두 font id를 하나로 합치지 않은 이유는 charPr 참조 구조를 건드리지 않기 위해서다. 기존 문서의 `charPrIDRef`와 `fontRef` 관계를 유지하면 lineSeg, 문단, 표 구조를 불필요하게 바꾸지 않아도 된다. 이 fixture의 목적은 HWPX 구조 정리가 아니라 로컬 글꼴 감지 흐름 검증이므로, 참조 id는 보존하고 face 이름만 바꿨다.

### 5.3 표시 문구 변경

`Contents/section0.xml`에서 사용자가 바로 볼 수 있는 문구만 검증 목적에 맞게 바꿨다.

변경 내용:

```text
교육 이수 확인서
-> 로컬 글꼴 테스트

샘플 테스트 입니다
-> 나눔스퀘어 Bold 렌더 확인 문장입니다
```

표 본문과 이미지 배치는 그대로 유지했다. 문서 전체를 새로 설계하지 않은 이유는 기존 fixture의 안정적인 레이아웃을 재사용하기 위해서다.

### 5.4 미리보기 텍스트 변경

`Preview/PrvText.txt`는 샘플 목적을 알 수 있도록 짧게 교체했다.

최종 내용:

```text
로컬 글꼴 테스트

나눔스퀘어 Bold 렌더 확인 문장입니다
이 문서는 rhwp-studio 로컬 글꼴 감지 동의 UX 검증용 샘플입니다.
```

`Preview/PrvImage.png`는 변경하지 않았다. rhwp-studio의 실제 렌더 검증은 `Contents/header.xml`과 `Contents/section0.xml`을 기준으로 이루어지고, 이번 fixture의 목적도 브라우저 렌더링 경로 확인이기 때문이다.

### 5.5 재패키징

HWPX는 zip 기반이지만 `mimetype` 파일은 첫 번째 엔트리로 두고 압축하지 않는 것이 안전하다.

재패키징 절차는 다음 형태를 사용한다.

```bash
cd /tmp/rhwp-local-font-sample
rm -f /tmp/local-font-nanumsquare-bold.hwpx
zip -X0 /tmp/local-font-nanumsquare-bold.hwpx mimetype
zip -Xr9 /tmp/local-font-nanumsquare-bold.hwpx \
  BinData Contents META-INF Preview Scripts settings.xml version.xml
cp /tmp/local-font-nanumsquare-bold.hwpx \
  /Users/melee/Documents/projects/forks/rhwp/samples/hwpx/local-font-nanumsquare-bold.hwpx
```

최종 archive는 다음 구조를 가진다.

```bash
unzip -l samples/hwpx/local-font-nanumsquare-bold.hwpx
```

핵심 엔트리:

```text
mimetype
Contents/header.xml
Contents/section0.xml
Preview/PrvText.txt
Preview/PrvImage.png
BinData/image1.png
META-INF/container.xml
META-INF/manifest.xml
```

## 6. 결과 확인

### 6.1 글꼴 목록 확인

```bash
unzip -p samples/hwpx/local-font-nanumsquare-bold.hwpx Contents/header.xml \
  | rg -o 'face="[^"]+"' \
  | sort -u
```

기대 결과:

```text
face="나눔스퀘어 Bold"
face="함초롬돋움"
face="함초롬바탕"
```

이 결과가 중요한 이유는 문서에 남은 외부 검증 대상이 `나눔스퀘어 Bold` 하나임을 보여주기 때문이다. `함초롬돋움`, `함초롬바탕`은 rhwp 기본 지원 글꼴로 처리된다.

### 6.2 본문 문구 확인

```bash
unzip -p samples/hwpx/local-font-nanumsquare-bold.hwpx Contents/section0.xml \
  | rg '로컬 글꼴 테스트|나눔스퀘어 Bold 렌더 확인 문장입니다'
```

이 명령으로 실제 표시 문구가 section XML에 들어갔는지 확인한다.

### 6.3 압축 무결성 확인

```bash
unzip -t samples/hwpx/local-font-nanumsquare-bold.hwpx
```

zip 컨테이너가 손상되지 않았는지 확인한다.

### 6.4 rhwp-studio 수동 검증

검증 전 브라우저의 저장된 로컬 글꼴 감지 결과를 지우면 모달이 다시 뜬다.

검증 흐름:

1. rhwp-studio 로컬 서버를 실행한다.
2. `samples/hwpx/local-font-nanumsquare-bold.hwpx`를 연다.
3. 저장된 로컬 글꼴 감지 결과가 없다면 로컬 글꼴 감지 모달이 표시된다.
4. 모달 상세에는 기본 지원 글꼴과 로컬 확인 필요 글꼴이 구분되어야 한다.
5. `로컬 글꼴 감지 (권장)`을 선택한다.
6. 로컬에 `나눔스퀘어 Bold`가 설치되어 있으면 감지 후 렌더링이 다시 수행되고, 해당 글꼴이 웹 대체보다 앞선 font-family chain으로 적용된다.
7. 로컬에 해당 글꼴이 없으면 감지 결과는 누락 또는 웹 대체/fallback 경로로 남는다.

Chrome 계열에서는 Local Font Access API로 전체 로컬 글꼴 목록을 확인한다. Firefox 계열에서는 전체 목록 API가 없으므로 사용자가 승인한 뒤 현재 문서에 필요한 후보 글꼴명만 font presence probe로 확인한다. 두 브라우저 모두 사용자 승인 전에는 새 로컬 글꼴 조회를 하지 않는 것이 핵심이다.

## 7. 기대 동작

로컬 감지 전:

- 문서 글꼴 목록에는 `나눔스퀘어 Bold`가 포함된다.
- 저장된 감지 결과가 없다면 `나눔스퀘어 Bold`는 `로컬 확인 필요`로 분류된다.
- 렌더링은 웹 대체 또는 시스템 fallback을 사용한다.

로컬 감지 승인 후, 사용자의 로컬 환경에 `나눔스퀘어 Bold`가 있는 경우:

- 감지 결과에 `나눔스퀘어 Bold`가 저장된다.
- 표시용 font-family chain에서 `나눔스퀘어 Bold`가 웹 대체 글꼴보다 앞에 온다.
- canvas가 다시 렌더링되며 본문 글꼴 폭과 굵기가 달라진다.
- 이후 같은 브라우저/확장 저장소에서는 저장된 감지 결과를 재사용한다.

로컬에 `나눔스퀘어 Bold`가 없는 경우:

- 감지는 성공해도 해당 family는 사용 가능으로 분류되지 않는다.
- 문서는 웹 대체 또는 generic fallback으로 계속 표시된다.
- 이 경우 샘플은 모달 UX 검증은 가능하지만, 렌더링 변화 검증에는 적합하지 않다.

## 8. 시각 비교 산출물

로컬 적용 전후 비교 산출물은 다음 경로에 남겼다.

```text
output/local-fonts/local-font-nanumsquare-before.png
output/local-fonts/local-font-nanumsquare-after.png
output/local-fonts/local-font-nanumsquare-diff.png
output/local-fonts/local-font-nanumsquare-comparison.png
```

이 산출물은 브라우저에서 실제로 글꼴 적용 전후가 달라졌는지 확인하기 위한 보조 자료다. 최종 판단은 rhwp-studio에서 샘플을 직접 열고, 로컬 글꼴 감지 승인 전후 렌더링을 보는 방식으로 한다.

## 9. 주의사항

1. 브라우저에 저장된 `rhwp-local-fonts` snapshot이 남아 있으면 모달이 다시 뜨지 않을 수 있다. 재검증 시 환경설정의 감지 결과 초기화 또는 브라우저 storage 삭제가 필요하다.
2. `나눔스퀘어 Bold`가 OS에 설치되어 있지 않으면 렌더링 변화는 발생하지 않는다.
3. Firefox는 전체 로컬 글꼴 목록을 제공하지 않으므로 Chrome과 저장 snapshot의 의미가 다르다. Firefox snapshot은 현재 문서 후보 글꼴에 대한 확인 결과로 이해해야 한다.
4. 이 fixture는 로컬 글꼴 감지 UX 검증용이다. HWPX preview image 정합이나 전체 문서 내용의 의미 검증용으로 쓰지 않는다.
