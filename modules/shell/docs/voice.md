# Kitokey's voice

How Kitokey speaks to its users, in English and in French. Every sentence it says is in
[`src/words.rs`](../src/words.rs) (`AGENTS.md`); this page is what a sentence there is written
to. It comes from the copy review of 2026-11-13 (D94, cycle 10): its principles, its lexicon and
its templates. Each example is given in both languages, English first.

The reader is not technical. They are often busy, sometimes in the middle of a call, sometimes
worried — a recording that stops, an error.

## Principles

| # | Rule | Example (EN / FR) | Source |
|---|---|---|---|
| R1 | **Short and direct.** As few words as the meaning allows. One idea a sentence. | "Recording stopped" / « Enregistrement arrêté » | GNOME HIG, *Writing style* |
| R2 | **The user's words, not the system's.** No jargon, no internal term (lexicon below). | "Others", not "Remote" / « Les autres », pas « À distance » | GNOME HIG; Google, *Tone* |
| R3 | **Active voice, a clear subject.** Kitokey, or the thing, does the action. | "Kitokey could not open the folder." / « Kitokey n’a pas pu ouvrir le dossier. » | Mailchimp, *Voice and tone*; Google, *Voice* |
| R4 | **Buttons and menu entries are verbs** that say what will happen. An ellipsis only when another step follows. | "Transcribe", "Choose…" / « Transcrire », « Choisir… » | GNOME HIG (ellipsis); Apple HIG; Material, *UX writing* |
| R5 | **An error says what happened, then what to do**, never blaming the user, no technical detail — that stays behind *Developer* (`shown_detail`). | "Kitokey could not open the folder. Open it from your file manager." / « Impossible d’ouvrir le dossier. Ouvrez-le depuis votre gestionnaire de fichiers. » | Google, *Writing helpful error messages* |
| R6 | **One voice, a tone that adapts.** Calm, clear, reassuring. In an error or a worry, reassure first — what goes on, what is not lost —, then the gesture. No humour in an error. | "The recording goes on; nothing is lost." / « L’enregistrement continue, rien n’est perdu. » | Mailchimp |
| R7 | **Privacy is said precisely**: what leaves, to whom, what stays. Never vague, never played down. | "The text of “Weekly sync” is about to leave your computer" / « Le texte de « Point hebdo » va quitter votre ordinateur » | The product's promise: nothing crosses the network without an agreement |
| R8 | **Sentence case everywhere in English** — titles, buttons, menus —, as Google, Material and Windows do; French never capitalises a title. | "Models to download" / « Modèles à télécharger » | Google style; GNOME HIG prefers title case, a deviation taken for one case on three systems |
| R9 | **One thing, one name, everywhere.** A notification that cites a menu entry cites it word for word. | "Transcribe the last recording" in the menu and in the notification / « Transcrire le dernier enregistrement » dans les deux | GNOME HIG; Apple HIG |
| R10 | **Accessibility.** An `aria-label` or a tooltip names the action, not the icon. A status reads on its own, without what surrounds it. | "Hide the sidebar" / « Masquer la barre latérale » | Mailchimp, *Writing for accessibility*; Apple HIG |

## French

- **« Vous ».** Software for work in French says *vous* (Apple, Google, Microsoft, Zoom, Notion);
  *tu* belongs to leisure apps. Kitokey is about work meetings, sometimes with clients, and the
  user pastes its announcement into the call's chat.
- **The infinitive on buttons and menu entries** (« Démarrer un enregistrement »), **the imperative
  in an instruction** (« Vérifiez… », « Réessayez »).
- **Typography (Imprimerie nationale)**: U+00A0 before « : » and inside « » ; U+202F before « ; ? ! »
  and before « % ». The typographic apostrophe ’. One character for « … ». « » to cite a menu entry.
- **No avoidable anglicism**: « clé d’API », not « token »; « transcription », never « transcript ».
- **No participle that assumes a gender**: « une notification vous préviendra », not « vous serez
  prévenu ».

## English

- **"Summary"** is the interface's word for what the summary step writes, where it says "minutes"
  today (D94; Zoom's *meeting summary*, Otter's *summary*). The interface only: `minutes.md`, the
  `minutes` contracts, the code's names and the charter's vocabulary keep "minutes".
- **The typographic apostrophe ’** and quotation marks “ ”, never `'` or `"`.

## Lexicon

| Concept | English | French | Refused |
|---|---|---|---|
| What was said, as text | transcript | transcription | « transcript », « segment » (FR) |
| What the AI writes of it | summary | compte rendu | "minutes" for it (EN); « CR » |
| A third party online | (online) service | service (en ligne) | "provider" (EN); « fournisseur », « hôte » (FR) |
| The AI's engine | model — in Settings and *Advanced* only | modèle | "ggml", quantisation, "engine" |
| The secret | API key | clé d’API | "key" alone, "token" |
| Where keys are kept | system keychain | trousseau du système | "keyring" (EN) |
| What the computer plays | meeting audio / others | le son de la réunion / les autres | "remote", "what the machine plays" |
| The user's microphone | your microphone | votre micro | "Here" |
| The icon by the clock | the Kitokey icon | l’icône Kitokey | "tray", « barre système » |
| A meeting's folder | the meeting’s folder | le dossier de la réunion | "session", "session's folder" |
| Offline | on this computer, offline | sur cet ordinateur, sans connexion | "with no network" |
| Timestamps | timestamps | horodatage | "times of each segment" |
| Deleted, recoverable | moved to the trash | mis à la corbeille | "freed", "erased" |
| Addressing the user | you | vous | « tu » (FR) |

## Templates

- **An error**: *what did not work. What is still safe. The gesture.*
  "Kitokey could not open the folder. Open it from your file manager." /
  « Impossible d’ouvrir le dossier. Ouvrez-le depuis votre gestionnaire de fichiers. »
- **An error completing a heading** (`failure`): it starts in lower case after the heading's
  « : » and stands as a clause of its own. "…: try again." / « … : réessayez. »
- **A notification**: a title read alone — at most 50 characters, no full stop —, then a body of
  one or two sentences. "Transcript ready" / « Transcription prête ».
- **A list's label** (a chip in a meeting's row): one or two words, at most 18 characters.
  "Waiting" / « En attente ».
- **Placeholders** `{}` and `{name}`: never removed, never repeated, the same in both languages,
  kept in their order when there are several.

## A sentence's comment

Each sentence of `words.rs` carries a comment that lets a writer — a person or a model — write it
without reading `ui/window.js`: on the field of `Window` or `Menu` that holds it, or above the arm
that says it. It gives four things, in this order, each after its word, two lines as a rule:

```rust
/// A meeting's row while its summary is written. Shows: the list, as a list label. When: the
/// summary step runs for this meeting. At most: 18 characters. Placeholders: none.
pub chip_making: &'static str,
```

`At most:` is a number of characters where a limit is held — a notification's title, 50; a list's label, 18 —; `about N characters` for the review's estimates, guidance as Android's and Xcode's string comments give translators, not a gate; or `no limit` for a sentence that wraps. `Placeholders:` names
each placeholder and what fills it — `{} the folder's path` —, or `none`. In a function, a comment
covers the sentences after it, up to the next such comment or the end of the function: one above
the English arm covers the French one too.

## What the tests hold

`tests/words.rs` reads every sentence against this page: each rule a test can tell apart — the
comment, the placeholders, the apostrophe, the limits, the refused words, a notification's title —
with what is still wrong today, a number or a list that only shrinks as the rewrite goes, one pull
request an area (D94). The rest of this page — the tone, the gesture after the error, the terms a
test cannot tell apart — is the review's.
