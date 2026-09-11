# 변경 이력

이 프로젝트의 주요 변경 사항을 기록합니다.
형식은 [Keep a Changelog](https://keepachangelog.com/ko/1.1.0/)를 따릅니다.

## [Unreleased]

### Added — Config (Viper 패리티)

- 5계층 우선순위: 기본값 → 설정 파일 → 환경변수 → CLI 플래그 → `set` 명시 값.
- `set`, `is_set`, `register_alias`, `set_key_delimiter`, `set_env_key_replacer`,
  `allow_empty_env`.
- 타입 getter: `get_int64`, `get_uint`, `get_duration`, `get_time`,
  `get_size_in_bytes`, `get_string_slice`.
- 열거·하위 트리: `all_keys`, `all_settings`, `SettingsMap`/`SettingsEntry`,
  `get_string_map`/`get_string_map_string`/`get_string_map_string_slice`, `sub`.
- 런타임 읽기·병합: `read_config`, `merge_in_config`, `merge_config_map`.
- 쓰기: `write_config_as`, `safe_write_config_as`.
- 신규 포맷(피처): `ini-config`, `dotenv-config`, `properties-config` (읽기+쓰기).
- 구조체 역직렬화(피처 `serde`): `unmarshal`, `unmarshal_key`.
- 설정 파일 감시: `on_config_change`, `set_watch_interval`, `watch_config`,
  `ConfigWatcher` (폴링 기반, 기본 1초).
- 신규 에러: `ConfigTypeNotSet`, `ConfigFileExists`, `ConfigDeserializeError`,
  `ConfigWatchNotReady`.

### Fixed

- bash/zsh completion 스크립트 생성 오류 (stray quote, `#compdef` 중복,
  `_arguments` continuation 누락, fish short 플래그 누락).
- `Panel`이 `padding != 1`일 때 제목 테두리 폭이 본문과 어긋나던 문제.
- `--help` 컬럼 정렬/줄바꿈이 CJK·ANSI 출력에서 깨지던 문제 (`display_width` 적용).

### Changed

- Config getter 6종에 중복되던 4계층 조회 로직을 `resolve()`로 통합.
- 설정 탐색 경로 재계산, 플래그 시드, `Style::apply`의 실행당 할당 감소.
- 문서: 가이드를 `docs/`로 통합하고 `README.md`/`docs/GUIDE.md`/`AGENTS.md`를 갱신,
  `CHANGELOG.md` 추가.

## [0.1.0]

- 최초 릴리스: 커맨드 트리, 타입 플래그, 4계층 설정, completion, 터미널 스타일링.
