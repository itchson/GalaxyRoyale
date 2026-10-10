# Contextual dialogue

`DialoguePanel` supplies semantic controls for a real site conversation. It does not accept quests or award rewards itself. Root integration supplies the current site description and invokes `Galaxy.interact_site()` from the action callback.

```ts
const dialogue = new DialoguePanel({
  onOpen: pauseForConversation,
  onClose: restoreConversationPause,
});
dialogue.show({
  speaker: site.name,
  text: site.description,
  portrait: family,
  actionLabel: site.status === "ready" ? "CLAIM RELIC" : "ACCEPT EVENT",
  onAction: interactWithCurrentSite,
});
```

`show()` refreshes an open conversation without calling `onOpen` again. `close()` invokes `onClose` once. `opened` reports the current state; `dispose()` closes and removes listeners and DOM. Open and close callbacks are synchronous. The owner must preserve the prior pause state, honor visibility/recovery, and keep the console and dialogue mutually exclusive. Closing a conversation must not resume a game that was paused for another reason.

The root is `#dialogue-screen`; `.dialogue-card` contains `.dialogue-portrait[data-portrait]`, `#dialogue-title`, `#dialogue-text`, `#dialogue-action`, and `#dialogue-close`. Portrait values are bounded integers 0–7. Speaker, text, and action labels are bounded plain text (64, 256, and 40 characters), never HTML. The pixel bridge should observe the screen and include the card and portrait node; the semantic card becomes transparent when `body.pixel-ready` is present.

The optional action is hidden when no callback exists. An action can return a promise; its button is disabled while it runs. The panel remains open so the owner can refresh its real completed state. A thrown action error is shown as bounded plain text. A revision guard prevents late action results from changing a closed or newer conversation. Escape closes, Tab stays within available actions, and gameplay keyboard/pointer events are intercepted while open.

The card uses a 680×110 logical maximum, a 48×64 portrait, safe-area padding, and a smaller landscape layout. Text can scroll if it exceeds its available area. Pixel UI must clip text to `#dialogue-text` when rendering that scroll area. Browser layout, actual portrait art, and integration pause/resume behavior still require visual validation.
