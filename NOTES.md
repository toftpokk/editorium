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
