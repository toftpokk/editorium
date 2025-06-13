# Learning notes

## Iced Event Flow
1. Event -> Widget.on_event

2. Widget.on_event -> update Widget.state -> Captured -> shell(Message)

3. Message -> app.update -> app.internal_state

4. app.internal_state -> Task<operation(id)>

5. Task<operation(id)> -> find id in tree -> Widget(id).operate

6. operation::Operation -> update Widget.state -> END

## Other Events

- Event Ignored -> app.subscription -> Message

- Timed app.subscription -> Message

## How nvim detects file encodings
https://github.com/neovim/neovim/blob/8b9500c886bdb72620e331d430e166ad7d9c12f8/src/nvim/fileio.c#L162
- TLDR:
  - default set of file encodings to try:
    - ucs-bom, utf-8, latin1
  - check decode each one, if all fails, use utf-8
- from trial and error
  - text is written in utf-8, then use iconv to convert to final encoding
  - if cannot convert, error
- uchardet
  - software to detect character encoding using frequency analysis
  - returns most likely encoding, and confidence level
