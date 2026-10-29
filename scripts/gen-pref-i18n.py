"""Generate src/i18n/prefLabels.{en,ru}.ts from catalog labels."""
from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
lines = (ROOT / "scripts" / "_pref_labels.txt").read_text(encoding="utf-8").splitlines()

# Curated RU translations for common prefs; rest fall back to EN in the TS merge helper.
RU: dict[str, str] = {
    "use_custom_skin": "Использовать скин",
    "skin_file": "Файл скина",
    "embed_video": "Встраивать видео в интерфейс",
    "resize_to_video": "Изменять размер окна под видео",
    "remember_window_geometry": "Запоминать размер и положение окна",
    "keep_above": "Поверх всех окон",
    "show_decorations": "Показывать рамку окна",
    "show_filename_title": "Показывать имя файла в заголовке",
    "fullscreen_screen": "Экран для полноэкранного режима",
    "start_compact": "Запускать в компактном режиме",
    "autohide_controls": "Автоскрытие элементов управления",
    "autohide_after": "Скрывать через",
    "show_fs_controls": "Показывать управление в полноэкранном режиме",
    "systray_icon": "Значок в системном трее",
    "start_minimized_tray": "Запускать свёрнутым в трей",
    "track_popup": "Всплывающее уведомление о треке",
    "popup_duration": "Длительность уведомления",
    "playlist_tree": "Дерево плейлиста",
    "save_recent": "Сохранять недавние",
    "recent_filter_regex": "Фильтр недавних (regex)",
    "fetch_metadata_net": "Загружать метаданные из сети",
    "lastfm_enabled": "Last.fm",
    "lastfm_user": "Пользователь Last.fm",
    "lastfm_url": "URL Last.fm",
    "hotkey_profile": "Профиль горячих клавиш",
    "skip_very_short": "Очень короткий скачок",
    "skip_short": "Короткий скачок",
    "skip_medium": "Средний скачок",
    "skip_long": "Длинный скачок",
    "mouse_single": "Одинарный клик мыши",
    "mouse_double": "Двойной клик мыши",
    "mouse_wheel": "Колёсико мыши",
    "mouse_hscroll": "Горизонтальная прокрутка",
    "mouse_reverse_scroll": "Инвертировать прокрутку",
    "http_intf": "HTTP-интерфейс",
    "http_port": "HTTP-порт",
    "continue_mode": "Продолжать воспроизведение",
    "autoplay": "Автовоспроизведение",
    "default_rate": "Скорость по умолчанию",
    "pause_on_audio_duck": "Пауза при снижении громкости системой",
    "preferred_quality": "Предпочтительное качество",
    "menu_language": "Язык меню",
    "default_audio_track": "Звуковая дорожка по умолчанию",
    "default_spu_track": "Субтитры по умолчанию",
    "repeat_mode": "Повтор",
    "shuffle_default": "Перемешивание по умолчанию",
    "stop_after_each": "Останавливать после каждого файла",
    "on_finish": "По окончании",
    "seek_precise": "Точный поиск",
    "scrub_thumbnail": "Миниатюры при перемотке",
    "enable_video": "Включить видео",
    "start_fullscreen": "Запускать в полноэкранном режиме",
    "disable_screensaver": "Отключать скринсейвер",
    "hue": "Оттенок",
    "brightness": "Яркость",
    "contrast": "Контраст",
    "saturation": "Насыщенность",
    "gamma": "Гамма",
    "deinterlace": "Деинтерлейс",
    "deinterlace_mode": "Режим деинтерлейса",
    "aspect_ratio": "Соотношение сторон",
    "crop": "Обрезка",
    "scale_to_window": "Масштаб под окно",
    "zoom": "Масштаб",
    "video_position": "Положение видео",
    "snapshot_dir": "Папка снимков",
    "snapshot_prefix": "Префикс снимков",
    "snapshot_format": "Формат снимков",
    "enable_audio": "Включить звук",
    "audio_device": "Аудиоустройство",
    "speaker_setup": "Конфигурация колонок",
    "max_volume": "Максимальная громкость",
    "startup_volume_mode": "Громкость при запуске",
    "startup_volume_pct": "Всегда начинать с:",
    "volume_step": "Шаг громкости",
    "normalize": "Нормализация громкости",
    "equalizer_enabled": "Эквалайзер",
    "audio_language": "Предпочтительный язык аудио",
    "audio_delay": "Задержка аудио",
    "spu_language": "Предпочтительный язык субтитров",
    "spu_delay": "Задержка субтитров",
    "spu_autoload": "Автозагрузка субтитров",
    "spu_font": "Шрифт субтитров",
    "spu_font_size": "Размер шрифта",
    "spu_color": "Цвет субтитров",
    "spu_opacity": "Непрозрачность",
    "hw_accel_mode": "Аппаратное ускорение",
    "reduce_motion": "Уменьшить анимацию",
    "accent_index": "Цвет акцента",
    "resume_playback": "Запоминать позицию",
    "hw_accel": "Аппаратное декодирование",
}

pairs: list[tuple[str, str]] = []
for line in lines:
    if not line.strip():
        continue
    key, label = line.split("\t", 1)
    pairs.append((key, label))


def emit(path: Path, locale: str) -> None:
    body = []
    for key, label in pairs:
        val = RU.get(key, label) if locale == "ru" else label
        esc = val.replace("\\", "\\\\").replace('"', '\\"')
        body.append(f'  {key}: "{esc}",')
    path.write_text(
        "/** Auto-generated from settings/catalog.ts — re-run scripts/gen-pref-i18n.py */\n"
        f"export const prefLabels{locale.capitalize()} = {{\n"
        + "\n".join(body)
        + "\n} as const;\n",
        encoding="utf-8",
    )
    print(f"wrote {path} ({len(pairs)} keys)")


emit(ROOT / "src" / "i18n" / "prefLabels.en.ts", "en")
emit(ROOT / "src" / "i18n" / "prefLabels.ru.ts", "ru")
