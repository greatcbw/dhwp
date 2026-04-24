# hwpdocx

**hwpdocx — HWP/HWPX to DOCX Converter**

hwpdocx는 HWP/HWPX 문서를 열고 편집하며 DOCX(Word) 형식으로 내보낼 수 있는 오픈소스 macOS, Windows, Linux용 데스크탑 앱입니다.

[golbin/hop](https://github.com/golbin/hop)을 포크하여 DOCX 내보내기 기능을 추가했습니다. 문서 파싱과 렌더링의 기반은 [rhwp](https://github.com/edwardkim/rhwp)를 사용합니다.

![hwpdocx editor](assets/screenshots/hop-editor.webp)

## 할 수 있는 일

hwpdocx는 다음 흐름을 지원합니다.

* HWP/HWPX 문서 열기
* HWP 문서 저장, 다른 이름으로 저장
* **DOCX(Word)로 내보내기** ← 이 저장소에서 추가
* PDF로 내보내기
* 인쇄 다이얼로그 열기
* 파일 드래그 앤 드롭으로 열기
* `.hwp`, `.hwpx` 파일 연결
* 여러 창에서 문서 열기

## DOCX 내보내기 지원 범위

| 요소 | 지원 여부 |
|------|----------|
| 본문 텍스트 | ✅ |
| 문단 정렬 (좌/우/가운데/양쪽) | ✅ |
| 문단 들여쓰기 / 내어쓰기 | ✅ |
| 줄 간격 / 문단 앞뒤 여백 | ✅ |
| 굵게 / 기울임 / 밑줄 / 취소선 | ✅ |
| 위첨자 / 아래첨자 | ✅ |
| 글자 크기 / 색상 | ✅ |
| 한글 / 영문 글꼴 | ✅ |
| 표 (colspan / rowspan) | ✅ |
| 이미지 / 도형 | ❌ (향후 계획) |

## 개발하기

개발 환경 준비, 실행 명령, 프로젝트 구조는 [개발 문서](docs/DEVELOPMENT.md)에 정리되어 있습니다.

### 빌드 요구사항

* Rust 1.75+
* Node.js 18+
* pnpm 10+

```bash
# 의존성 설치
pnpm install

# 개발 서버 실행
pnpm tauri dev

# 릴리즈 빌드
pnpm tauri build
```

## Credits

| 프로젝트 | 역할 | 라이선스 |
|---------|------|---------|
| [rhwp](https://github.com/edwardkim/rhwp) by Edward Kim | HWP/HWPX 파싱·렌더링 엔진 | MIT |
| [golbin/hop](https://github.com/golbin/hop) | 데스크탑 앱 원본 (이 저장소의 포크 베이스) | MIT |
| [docx-rs](https://github.com/bokuweb/docx-rs) | Rust DOCX 생성 라이브러리 | MIT |

이 프로젝트는 위 오픈소스들을 기반으로 합니다. 각 프로젝트 개발자분들께 감사드립니다.

## License

MIT License — 자유롭게 사용, 수정, 배포할 수 있습니다. 자세한 내용은 [LICENSE](LICENSE) 파일을 참조하세요.

Copyright (c) 2025-2026 Edward Kim  
Copyright (c) 2026 Billyboy (hwpdocx fork)
