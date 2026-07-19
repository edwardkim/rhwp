---
kind: guide
status: active
canonical: mydocs/manual/ai_sample_document_authoring_guide.md
last_verified: 2026-07-17
---

# AI 샘플 문서 작성 가이드

이 문서는 AI가 rhwp 프로젝트에서 샘플 문서, 회귀 fixture, 첨부 템플릿 기반 문서를 만들 때 따라야 할 절차를 정리한다.

목표는 빠르게 문서를 만드는 것이 아니라, 나중에 다시 검증하고 설명할 수 있는 문서를 만드는 것이다. 샘플 문서는 코드만큼 오래 남는 테스트 자산이므로, 생성 이유와 변경 범위를 함께 남겨야 한다.

이 가이드는 #1328 로컬 글꼴 감지 작업에서 AI가 `local-font-nanumsquare-bold.hwpx` 샘플을 직접 만들고, rhwp-studio에서 승인 전/후 렌더링 차이까지 검증한 절차를 바탕으로 일반화했다. 해당 작업의 구체 기록은 `mydocs/working/task_m100_1328_sample_fixture.md`에 남긴다.

## 1. 기본 원칙

1. 먼저 목적을 한 문장으로 고정한다.
2. 새 문서를 처음부터 만들기보다 기존에 정상 동작하는 샘플이나 사용자가 제공한 템플릿을 우선 검토한다.
3. 문서 구조, 테스트 대상, 검증 방법을 분리해서 생각한다.
4. 템플릿의 레이아웃과 스타일은 가능한 유지하고, 검증에 필요한 최소 내용만 바꾼다.
5. 사람이 첨부한 템플릿의 원본 의미와 개인정보를 임의로 확장하지 않는다.
6. 생성 과정은 내부 사고가 아니라 재현 가능한 판단 근거와 명령으로 기록한다.
7. 문서 파일만 추가하지 말고, 어떤 조건에서 어떤 동작을 기대하는지도 함께 문서화한다.

## 2. 먼저 확정할 것

작업 시작 전에 다음을 확인한다.

| 항목 | 확인 질문 |
|---|---|
| 목적 | 이 문서가 어떤 버그, 기능, UX, 렌더링 차이를 검증하는가? |
| 형식 | HWP, HWPX, PDF 비교본, 이미지 템플릿 중 무엇이 필요한가? |
| 기준 | 한컴 원본, 기존 fixture, 사용자 첨부 템플릿, rhwp-studio 출력 중 무엇이 기준인가? |
| 변경 범위 | 내용만 바꾸는가, 스타일도 바꾸는가, 레이아웃 구조도 바꾸는가? |
| 검증 환경 | CLI, rhwp-studio, Chrome, Firefox, 한컴, 시각 비교 중 무엇으로 검증하는가? |
| 재현성 | 사용자의 로컬 폰트, OS, 브라우저 권한, 외부 파일에 의존하는가? |
| 저장 위치 | `samples/`, `samples/hwpx/`, `output/`, `mydocs/working/` 중 어디에 둘 것인가? |

목적이 불명확하면 샘플이 과해진다. 예를 들어 “로컬 글꼴 감지 UX”를 검증하는 샘플에는 복잡한 표 편집이나 페이지네이션 요소를 새로 넣지 않는다.

## 3. 입력 자료 분류

AI가 받을 수 있는 입력은 보통 다음 네 가지다.

### 3.1 기존 repository 샘플

가장 안정적인 기반이다.

사용 조건:

1. 이미 rhwp가 읽을 수 있다.
2. 테스트하려는 기능과 무관한 결함이 적다.
3. 문서 크기가 작고 구조가 이해 가능하다.
4. 필요한 레이아웃 요소가 이미 들어 있다.

예: 기존 1쪽 HWPX의 글꼴명만 바꿔 로컬 글꼴 감지 fixture를 만드는 방식.

### 3.2 사용자가 첨부한 템플릿

템플릿 자체가 기준이다. 임의로 재구성하지 말고, 템플릿이 가진 구조를 먼저 관찰한다.

확인할 것:

1. 파일 형식과 열람 가능 여부
2. 페이지 수와 주요 구성
3. 사용 글꼴
4. 이미지/표/도형/머리말/꼬리말/쪽 설정
5. 개인정보 또는 외부 공개가 부적절한 내용
6. 템플릿에서 유지해야 할 부분과 바꿔도 되는 부분

첨부가 이미지나 PDF라면, 그것을 HWPX로 직접 변환했다고 주장하지 않는다. 이미지/PDF는 시각 기준으로 두고, 실제 HWPX는 별도로 만들거나 기존 샘플을 수정해야 한다.

### 3.3 새로 작성해야 하는 단순 문서

테스트 대상이 매우 작고 문서 구조가 단순할 때만 선택한다.

적합한 경우:

1. 특정 글꼴, 문단, 표 하나만 검증한다.
2. 기존 샘플이 테스트 대상보다 너무 복잡하다.
3. rhwp-studio 저장 결과를 검증하는 것이 목적이다.

주의할 점:

새 문서를 rhwp-studio로 만들면 rhwp-studio의 현재 저장 구현 상태가 fixture에 섞인다. 저장 기능을 검증하는 작업이 아니라면, 기존 검증된 HWPX를 최소 수정하는 쪽이 더 안정적이다.

### 3.4 외부 자료 기반 문서

라이선스와 개인정보를 먼저 확인한다. 공개 저장소에 넣을 fixture는 재배포 가능한 자료만 사용한다. 유료 글꼴 파일, 비공개 문서, 개인정보가 포함된 원본은 그대로 커밋하지 않는다.

## 4. 작업 전략 선택

샘플 문서 작성 전략은 크게 네 가지다.

### 4.1 Clone and Narrow

기존 샘플을 복제하고 검증 대상만 좁힌다.

사용할 때:

1. 기존 문서 구조가 안정적이다.
2. 특정 글꼴, 특정 문구, 특정 속성만 바꾸면 된다.
3. parser/renderer의 다른 결함과 섞이지 않게 하고 싶다.

장점:

1. lineSeg, section, 이미지, 표 구조를 새로 만들지 않아도 된다.
2. 검증 실패 원인을 수정 범위로 좁히기 쉽다.
3. 새 fixture의 의도가 명확하다.

주의:

복제한 문서의 기존 결함도 함께 가져올 수 있다. 기반 파일을 고른 이유를 문서에 남겨야 한다.

### 4.2 Template Fill

사용자 템플릿의 레이아웃을 유지하고 텍스트/값만 채운다.

사용할 때:

1. 사용자가 특정 양식이나 출력 형태를 요구한다.
2. 문서의 레이아웃 자체가 중요한 기준이다.
3. 새 샘플보다 “템플릿을 따라 작성한 결과물”이 목적이다.

주의:

템플릿의 스타일, 쪽 설정, 표 구조를 보존한다. AI가 보기 좋게 다시 디자인하면 검증 기준이 사라진다.

### 4.3 Style Swap

문서 구조는 그대로 두고 글꼴, charPr, paraPr, 색상 같은 스타일만 바꾼다.

사용할 때:

1. 글꼴 fallback, bold, underline, spacing, line height를 검증한다.
2. 렌더링 전후 차이가 눈에 보여야 한다.
3. 내용 자체는 중요하지 않다.

주의:

HWPX의 `fontRef` id와 `charPrIDRef`를 함부로 재번호화하지 않는다. 참조 구조를 유지하고 face 이름이나 속성만 최소 변경하는 쪽이 안전하다.

### 4.4 Content Swap

문서의 구조와 스타일은 유지하고 표시 텍스트만 바꾼다.

사용할 때:

1. 스크린샷에서 테스트 대상이 바로 보이게 해야 한다.
2. 미리보기 텍스트를 샘플 목적에 맞게 정리해야 한다.
3. 기존 문서의 본문 의미가 샘플 목적과 맞지 않는다.

주의:

텍스트 길이가 달라지면 lineSeg와 페이지 흐름이 달라질 수 있다. 줄바꿈이나 페이지네이션이 테스트 대상이 아니라면 짧은 문구로 유지한다.

## 5. HWPX 작성 절차

HWPX는 zip 컨테이너다. 가능한 한 XML을 구조적으로 이해하고 수정한다.

### 5.1 구조 확인

```bash
unzip -l samples/hwpx/example.hwpx
```

핵심 파일:

```text
mimetype
Contents/header.xml
Contents/section0.xml
Contents/content.hpf
Preview/PrvText.txt
Preview/PrvImage.png
BinData/*
META-INF/*
settings.xml
version.xml
```

섹션이 여러 개라면 `Contents/section1.xml`, `Contents/section2.xml`도 확인한다.

### 5.2 글꼴 목록 확인

```bash
unzip -p samples/hwpx/example.hwpx Contents/header.xml \
  | rg -o 'face="[^"]+"' \
  | sort -u
```

글꼴 fixture라면 이 목록이 검증 조건의 핵심이다. 기본 지원 글꼴, 웹 대체 가능 글꼴, 로컬 확인이 필요한 글꼴을 구분한다.

### 5.3 본문 텍스트 확인

```bash
unzip -p samples/hwpx/example.hwpx Preview/PrvText.txt
unzip -p samples/hwpx/example.hwpx Contents/section0.xml \
  | rg '검증할 문구|테스트 문장'
```

미리보기 텍스트는 문서 검색과 사람의 이해를 돕는다. 실제 렌더링 기준은 section XML이다.

### 5.4 임시 작업 디렉터리 사용

```bash
rm -rf /tmp/rhwp-sample-work
mkdir -p /tmp/rhwp-sample-work
unzip -q samples/hwpx/base.hwpx -d /tmp/rhwp-sample-work
```

원본 파일을 직접 수정하지 않는다. 먼저 임시 디렉터리에서 변경하고, 검증 후 target path로 복사한다.

### 5.5 수정

권장 순서:

1. `Contents/header.xml`: 글꼴, charPr, paraPr, style 정의
2. `Contents/section*.xml`: 실제 문단, 표, 그림, 컨트롤
3. `Preview/PrvText.txt`: 샘플 설명용 텍스트
4. `Contents/content.hpf` 또는 manifest: 파일 추가/삭제가 있을 때만
5. `BinData/*`: 이미지나 임베드 리소스가 바뀔 때만

주의:

1. 참조 id를 바꾸면 연결된 모든 참조를 함께 바꿔야 한다.
2. 필요하지 않으면 id 재번호화는 하지 않는다.
3. XML namespace를 지우지 않는다.
4. `mimetype`은 보존한다.
5. 임베드하지 않는 글꼴은 `isEmbedded="0"` 상태로 둔다.

### 5.6 재패키징

```bash
cd /tmp/rhwp-sample-work
rm -f /tmp/new-sample.hwpx
zip -X0 /tmp/new-sample.hwpx mimetype
zip -Xr9 /tmp/new-sample.hwpx \
  BinData Contents META-INF Preview Scripts settings.xml version.xml
cp /tmp/new-sample.hwpx /path/to/rhwp/samples/hwpx/new-sample.hwpx
```

`mimetype`을 첫 번째 엔트리로 압축 없이 넣는 방식을 기본으로 한다.

## 6. HWP 작성 시 주의

HWP 바이너리는 HWPX보다 손수 수정하기 어렵다. AI가 임의의 바이너리 편집으로 HWP를 만들면 안 된다.

권장 방식:

1. 한컴 또는 rhwp의 안정된 변환/저장 경로로 만든다.
2. 사용자가 한컴에서 저장한 HWP를 기준으로 받는다.
3. HWPX fixture가 충분하면 HWP는 만들지 않는다.
4. HWP가 꼭 필요하면 생성 도구, 기준 프로그램, 저장 버전을 문서에 남긴다.

검증:

```bash
cargo run --bin rhwp -- dump samples/example.hwp > /tmp/example-dump.txt
cargo run --bin rhwp -- export-svg samples/example.hwp -o output/example/
```

HWP를 공개 fixture로 넣기 전에는 라이선스와 개인정보를 확인한다.

## 7. 첨부 템플릿 기반 작성 절차

사용자가 템플릿을 첨부하면 다음 순서로 작업한다.

### 7.1 템플릿 인벤토리

템플릿에서 확인할 항목:

1. 파일명과 형식
2. 페이지 수
3. 주요 레이아웃: 표, 이미지, 머리말, 꼬리말, 쪽 번호, 배경
4. 사용 글꼴
5. 입력해야 할 필드
6. 유지해야 할 스타일
7. 제거해야 할 개인정보
8. rhwp에서 현재 지원하지 않는 요소

### 7.2 작업 방식 결정

| 상황 | 권장 방식 |
|---|---|
| 템플릿이 HWPX이고 구조가 안정적 | HWPX 압축 해제 후 필요한 XML만 수정 |
| 템플릿이 HWP이고 수정이 작음 | 한컴/사용자 저장본 기준 유지, 가능하면 HWPX 변환본도 확보 |
| 템플릿이 PDF 또는 이미지 | 시각 기준으로만 사용하고 HWPX는 별도 생성 |
| 템플릿이 복잡하고 테스트 대상은 작음 | 기존 작은 fixture에 대상 속성만 이식 |
| 템플릿 자체 재현이 목적 | 레이아웃 보존 우선, 내용 변경 최소화 |

### 7.3 내용 채우기

내용을 채울 때는 다음을 지킨다.

1. 기존 필드의 위치와 스타일을 보존한다.
2. 긴 문구를 넣어 레이아웃을 깨뜨리지 않는다.
3. 개인정보가 필요한 경우 더미 값을 사용한다.
4. 날짜, 금액, 이름 같은 값은 문서 목적에 맞게 일관되게 쓴다.
5. 자동 줄바꿈이 테스트 대상이 아니라면 문구 길이를 원본과 비슷하게 유지한다.

### 7.4 결과 설명

템플릿 기반 문서를 만들었다면 별도 문서나 작업 보고서에 다음을 남긴다.

```text
- 입력 템플릿:
- 생성 파일:
- 유지한 요소:
- 변경한 요소:
- 제거/마스킹한 요소:
- 검증 명령:
- 사람이 확인해야 할 부분:
```

## 8. 검증 체크리스트

### 8.1 컨테이너 검증

```bash
unzip -t samples/hwpx/new-sample.hwpx
unzip -l samples/hwpx/new-sample.hwpx
```

### 8.2 XML 내용 검증

```bash
unzip -p samples/hwpx/new-sample.hwpx Contents/header.xml \
  | rg -o 'face="[^"]+"' \
  | sort -u

unzip -p samples/hwpx/new-sample.hwpx Contents/section0.xml \
  | rg '기대 문구'
```

### 8.3 rhwp CLI 검증

가능한 경우:

```bash
cargo run --bin rhwp -- dump samples/hwpx/new-sample.hwpx > /tmp/new-sample-dump.txt
cargo run --bin rhwp -- export-svg samples/hwpx/new-sample.hwpx -o output/new-sample/
```

작업 범위가 rhwp-studio라면 CLI보다 브라우저 검증이 더 중요할 수 있다. 어떤 검증이 authoritative인지 문서에 적는다.

### 8.4 rhwp-studio 검증

확인할 항목:

1. 파일이 열린다.
2. 페이지 수가 예상과 맞다.
3. 테스트 대상 요소가 화면에 보인다.
4. 모달, 권한, fallback 등 UX가 의도대로 동작한다.
5. 변경 전후 차이가 시각적으로 확인된다.
6. 콘솔에 치명 오류가 없다.

브라우저별 동작이 다르면 Chrome, Firefox, Safari 등 기준을 분리해서 적는다.

### 8.5 시각 비교

필요하면 `output/<task>/` 아래에 before/after/diff 이미지를 둔다.

```text
output/<task>/<sample>-before.png
output/<task>/<sample>-after.png
output/<task>/<sample>-diff.png
output/<task>/<sample>-comparison.png
```

시각 비교 산출물은 보조 자료다. 최종 판단 기준이 한컴인지, rhwp-studio인지, 브라우저별 렌더링인지 명확히 적는다.

## 9. 문서화 형식

샘플을 추가하면 함께 남겨야 할 최소 기록은 다음이다.

```markdown
# 샘플 작성 절차 — <파일명>

- 목적:
- 생성 파일:
- 기반 파일 또는 템플릿:
- 작성일:

## 요구사항

## 기반 파일/템플릿 선택 이유

## 변경 내용

## 생성 절차

## 검증 방법

## 기대 동작

## 주의사항
```

판단 흐름은 다음처럼 쓰면 된다.

```text
처음 검토한 선택지는 A/B/C였다.
A는 재현성이 낮아 제외했다.
B는 테스트 대상과 무관한 변수가 많아 제외했다.
C는 기존 검증된 구조를 유지하면서 대상 속성만 바꿀 수 있어 채택했다.
```

이것은 재현 가능한 의사결정 기록이다. AI의 내부 사고를 그대로 적을 필요는 없다.

## 10. 금지 사항

1. 사용자 동의 없이 첨부 문서의 개인정보를 공개 fixture로 커밋하지 않는다.
2. 재배포 불가 글꼴, 이미지, 문서를 repository에 추가하지 않는다.
3. HWP 바이너리를 임의 byte patch로 수정하지 않는다.
4. 템플릿을 “비슷하게” 새로 디자인하고 원본 기반이라고 쓰지 않는다.
5. 검증하지 않은 파일을 “정상 동작”한다고 기록하지 않는다.
6. 생성 경로와 검증 명령 없이 샘플만 추가하지 않는다.
7. 테스트 목적과 무관한 대규모 레이아웃 변경을 하지 않는다.
8. rhwp-studio 저장 기능이 테스트 대상이 아닌데 저장 결과를 무비판적으로 fixture 기준으로 삼지 않는다.

## 11. 좋은 샘플의 조건

좋은 샘플은 작고, 의도가 분명하고, 실패했을 때 원인을 좁힐 수 있다.

좋은 샘플:

1. 파일명이 목적을 드러낸다.
2. 문서 안의 표시 문구가 목적을 드러낸다.
3. 필요한 외부 조건이 명시되어 있다.
4. 기존 구조를 가능한 보존한다.
5. 검증 명령이 짧고 반복 가능하다.
6. 한 기능 또는 한 결함을 중심으로 한다.

나쁜 샘플:

1. 여러 기능을 한꺼번에 검증한다.
2. 외부 폰트나 이미지 의존성이 설명되어 있지 않다.
3. 어떤 파일을 기반으로 했는지 알 수 없다.
4. 결과가 환경에 따라 달라지는데 조건이 기록되어 있지 않다.
5. 사람이 봐야 할 부분과 자동 검증 가능한 부분이 분리되어 있지 않다.

## 12. 예시 요약

로컬 글꼴 감지 fixture를 만들 때 적용한 일반 패턴은 다음과 같다.

1. 목적: 로컬 글꼴 감지 동의 UX와 감지 후 렌더링 변경 확인
2. 전략: 기존 정상 HWPX를 복제한 뒤 글꼴명과 표시 문구만 변경
3. 기반: 작은 1쪽 HWPX fixture
4. 변경: `Pretendard` 계열을 `나눔스퀘어 Bold`로 치환
5. 보존: 표, 이미지, lineSeg, section 구조
6. 검증: zip 무결성, 글꼴 목록, 본문 문구, rhwp-studio 수동 확인
7. 문서화: 별도 working 문서에 선택 이유와 재현 절차 기록

이 패턴은 글꼴뿐 아니라 문단 스타일, 표 속성, 이미지 배치, 브라우저별 UX 검증 fixture에도 적용할 수 있다.
