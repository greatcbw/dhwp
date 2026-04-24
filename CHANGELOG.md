# Changelog

이 저장소([hwpdocx](https://github.com/greatcbw/dhwp))의 변경 이력입니다.  
원본 프로젝트([golbin/hop](https://github.com/golbin/hop)) 이후 추가된 내용만 기록합니다.

---

## [Unreleased]

### Added

- **DOCX 내보내기** (`file:export-docx` 커맨드)
  - HWP/HWPX 문서를 `.docx` 형식으로 저장
  - 본문 텍스트, 문단 서식(정렬·들여쓰기·여백·줄간격) 변환
  - 글자 서식(굵게·기울임·밑줄·취소선·위/아래첨자·크기·색상·글꼴) 변환
  - 표(colspan·rowspan·셀 세로 정렬) 변환
  - Rust 순수 구현 — 외부 도구 의존 없음 (`docx-rs` 0.4 사용)
  - 원자적 파일 쓰기(임시 파일 → rename) 로 저장 안전성 보장

### Changed

- `apps/desktop/src-tauri/Cargo.toml`: `docx-rs = "0.4"` 의존성 추가
- `apps/studio-host/src/core/tauri-bridge.ts`: `exportDocxFromCommand` 브릿지 구현
- `apps/studio-host/src/command/commands/file.ts`: `file:export-docx` 커맨드 등록

### Files Added

- `apps/desktop/src-tauri/src/docx_export.rs` — DocumentCore → docx-rs 변환 로직
