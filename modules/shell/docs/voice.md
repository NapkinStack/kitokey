# Kitokey's voice

How Kitokey speaks to its users, in English and in French. Every sentence it says is in
[`src/words.rs`](../src/words.rs) (`AGENTS.md`); this page is what a sentence there is written
to. Its rules, its message patterns and its terminology come from the microcopy audit of
2026-11-26 (`kitokey-team/audit-2026-11-26/messages.md`, §2, §3 and §5; D124, cycle 12), which
follows the decider's choices of that day: « moteur » and « format », plain words, no AI
typography (PDR-0008). They replace the copy review of 2026-11-13 (D94, cycle 10,
[`docs/project/reviews/2026-11-13-copy.md`](../../../docs/project/reviews/2026-11-13-copy.md)), whose
comment form, typography and limits stay. Each example is given in French, then in English.

Kitokey sounds like a calm colleague who takes notes for you. It speaks in short sentences, in
the second person, and celebrates briefly. The reader is not technical. They are often busy,
sometimes in the middle of a call, sometimes worried: a recording that stops, an error.

## Rules

| # | Rule | Do | Don't | Source |
|---|---|---|---|---|
| V1 | **Say the outcome, not the machinery.** | « Transcrire (300 Mo, une seule fois) » | « Télécharger le modèle (300 ko, 127.0.0.1:43567) et transcrire » | GNOME HIG, *Writing style*; Apple HIG, *Writing* |
| V2 | **One idea a sentence**, two sentences at most, about 90 characters. | « Plus rapide. Seul le texte part, jamais le son. » | three ideas in 185 characters | GNOME HIG; Google, *Concise* |
| V3 | **No AI typography.** No em dash (U+2014), no colon then a reveal, no arrow, no decorative « … ». The « … » stays on a menu entry or a button that opens a dialog, and on an action in progress. | « Votre micro est silencieux depuis 2 min. L’enregistrement continue. » | « Votre micro : rien depuis 2 min. » | GNOME HIG and Apple HIG (the ellipsis); the decider, 2026-11-26 |
| V4 | **When the user may worry, say what is safe first**, then the gesture. | « L’audio est intact. Réessayez. » | « Une erreur inattendue l’a interrompu. » | Google, *Writing helpful error messages* |
| V5 | **Kitokey or the thing is the subject.** Never the user's fault. | « Cette adresse ne fonctionne pas. Copiez-la depuis Slack. » | « Non enregistré : l’adresse doit commencer par https:// » | Mailchimp, *Voice and tone* |
| V6 | **One word per concept** (Terminology). A new word costs a decision. | « moteur », « format », « canal » | « modèle », « destination », « webhook », « Mode connecté » | GNOME HIG; Apple HIG |
| V7 | **Buttons: an infinitive and an object**, three words at most. | « Ajouter un canal », « Je choisirai plus tard » | « Plus tard, vos réunions attendront le modèle » | Material, *UX writing* |
| V8 | **Privacy is said exactly**: what leaves, for where, what stays. | « Seul le texte part vers Mistral. Le son reste ici. » | the rule hidden in a paragraph | The product's promise: nothing crosses the network without an agreement |
| V9 | **Name a third-party product only when the user must act on it**, once, in the help line. | « Ouvrez Ollama, puis réessayez. » | « Ollama · {} » on a summary's row | Granola, Otter: the engine is never named on a note |
| V10 | **Warm and light in success, plain in errors**, never a joke in an error. | « C’est prêt. Lancez votre premier enregistrement. » | « C’est fait : tout est dans la corbeille. » | Mailchimp |

**Raw data stays under « Avancé ».** A host, a port, a path, an engine's own id (`whisper-large-v3`,
`llama3.2:3b`) and a size under 1 Mo are the machinery V1 keeps out of a sentence. They are shown
only under « Avancé », « Détails » or the developer mode, where the person who asked for them reads
them. The same holds for « Mo » and « Go »: a size is rounded to what a person says.

A menu path cited in a sentence is written with « › », as the menus show it (« Refaire › Avancé »);
it is a separator, not an arrow, and the entry it cites loses its « … » (Apple Style Guide, *menu
names*).

## Message patterns

- **Error.** *What happened, Kitokey or the thing as the subject. What is safe.* Then one button.
  Two sentences, no technical detail: the detail stays behind the developer mode (`shown_detail`).
  « La transcription n’a pas abouti. L’audio est intact. » [Réessayer] /
  "The transcript didn’t finish. The audio is intact." [Try again]
- **An error completing a heading** (`failure`): it starts in lower case after the heading's
  « : » and stands as a clause of its own. « … : réessayez. » / "…: try again."
- **Progress.** *A noun or a verb in progress · n % or about n min*, drawn as a progress element.
  The text alone when no number is known.
  « Transcription · environ 3 min » ; « Téléchargement · 42 % » /
  "Transcribing · about 3 min"; "Downloading · 42%"
- **Success.** *The object, ready*, then one sentence or none.
  « Compte rendu prêt. » ; « Envoyé dans #produit » / "Summary ready."; "Sent to #product"
- **Empty state.** *Nothing yet, warmly. What it will hold, or how to start.* Then one action.
  « Aucun canal pour l’instant. Ajoutez Slack ou Discord pour envoyer un compte rendu en un clic. »
  [Ajouter un canal] / "No channel yet. Add Slack or Discord to send a summary in one click."
  [Add a channel]
- **Destructive confirmation.** A title that asks, with the verb. One line on what goes and
  whether it comes back. The confirm button repeats the verb.
  « Mettre cette réunion à la corbeille ? » / « L’audio, la transcription et le compte rendu. Vous
  pourrez la restaurer. » [Annuler] [Mettre à la corbeille] /
  "Move this meeting to the trash?" / "The audio, the transcript and the summary. You can restore
  it." [Cancel] [Move to trash]
- **Privacy notice.** *What leaves, for whom. What stays here.* Then the agreement as a verb.
  « Seul le texte de « Point hebdo » part vers Mistral. Le son reste ici. » [Envoyer et rédiger]
  [Rédiger sur cet ordinateur] / "Only the text of “Weekly sync” goes to Mistral. The sound stays
  here." [Send and write] [Write on this computer]
- **A notification**: a title read alone, at most 50 characters, no full stop, then a body of one
  or two sentences. « Transcription prête » / "Transcript ready".
- **A list's label** (a chip in a meeting's row): one or two words, at most 18 characters.
  « En attente » / "Waiting".
- **Placeholders** `{}` and `{name}`: never removed, never repeated, the same in both languages,
  kept in their order when there are several.

## Terminology

| Concept | French | English | Never |
|---|---|---|---|
| Meeting, the saved thing | réunion | meeting | session; « appel » except for the live call |
| Recording, the act and the button | enregistrement | recording | capture |
| Sound kept on disk | l’audio | audio | « le son » for the file (« le son » is the live signal: micro, son de la réunion) |
| What was said, as text | transcription | transcript | « transcript » (FR), « segment » |
| What the AI writes of it | compte rendu | summary | « CR », « résumé »; "minutes" in the interface |
| The AI that transcribes or writes | moteur (de transcription, de rédaction) | engine (transcription, summary) | « modèle », "model", Whisper, an engine's raw id |
| The shape of a summary | format | format | « modèle », "template", « structure » |
| Slack, Discord, a webhook | canal | channel | « destination », « webhook » as a noun, « adresse du webhook » |
| Sending to a channel | envoyer | send | « partager » (that is the menu), « publier » |
| Writing a file | exporter | export | « enregistrer un fichier », « télécharger » |
| Per meeting, nothing leaves | confidentielle | confidential | « Mode privé », « privé », « secret » |
| Deleted, recoverable | corbeille, « mettre à la corbeille » | trash, “move to trash” | « supprimer », « effacer », « libérer » when it is the trash |
| A third party online | service en ligne | online service | « fournisseur », "provider", « API » |
| What the user tells of a meeting | contexte | context | « prompt », « notes » |
| The secret | clé d’API | API key | "key" alone, « token » |
| Where keys are kept | trousseau du système | system keychain | "keyring" |
| The icon by the clock | l’icône Kitokey | the Kitokey icon | "tray", « barre système » |
| Addressing the user | vous | you | « tu » |

## French

- **« Vous ».** Software for work in French says *vous* (Apple, Google, Microsoft, Zoom, Notion);
  *tu* belongs to leisure apps. Kitokey is about work meetings, sometimes with clients, and the
  user pastes its announcement into the call's chat.
- **The infinitive on buttons and menu entries** (« Démarrer un enregistrement »), **the imperative
  in an instruction** (« Vérifiez », « Réessayez »).
- **Typography (Imprimerie nationale)**: U+00A0 before « : » and inside « » ; U+202F before « ; ? ! »
  and before « % ». The typographic apostrophe ’. One character for « … ». « » to cite a menu entry.
- **No avoidable anglicism**: « clé d’API », not « token »; « transcription », never « transcript ».
- **No participle that assumes a gender**: « une notification vous préviendra », not « vous serez
  prévenu ».

## English

- **Sentence case everywhere**, titles, buttons and menus, as Google, Material and Windows do.
- **"Summary"** is the interface's word for what the summary step writes (D94; Zoom's *meeting
  summary*, Otter's *summary*). The interface only: `minutes.md`, the `minutes` contracts, the
  code's names and the charter's vocabulary keep "minutes".
- **The typographic apostrophe ’** and quotation marks “ ”, never `'` or `"`.

## A sentence's comment

Each sentence of `words.rs` carries a comment that lets a writer, a person or a model, write it
without reading `ui/window.js`: on the field of `Window` or `Menu` that holds it, or above the arm
that says it. It gives four things, in this order, each after its word, two lines as a rule:

```rust
/// A meeting's row while its summary is written. Shows: the list, as a list label. When: the
/// summary step runs for this meeting. At most: 18 characters. Placeholders: none.
pub chip_making: &'static str,
```

`At most:` is a number of characters where a limit is held (a notification's title, 50; a list's
label, 18); `about N characters` for the review's estimates, guidance as Android's and Xcode's
string comments give translators, not a gate; or `no limit` for a sentence that wraps.
`Placeholders:` names each placeholder and what fills it (`{} the folder's path`), or `none`. In a
function, a comment covers the sentences after it, up to the next such comment or the end of the
function: one above the English arm covers the French one too.

## What the tests hold

`tests/words.rs` reads every sentence against this page: each rule a test can tell apart. The
comment, the placeholders, the apostrophe, the limits, the refused words, a notification's title;
and since D124 the rules a sweep can tell apart: no em dash, no arrow, no « … » nor « ... » outside
an action in progress or an entry that opens a dialog, no « modèle » nor "model", no host, port,
path, engine's raw id or size under 1 Mo.

What is still wrong today is listed in `tests/words_still_wrong.txt`, one `rule key` a line: the
list only shrinks, as each screen's pull request rewrites its own words (cycle 12, D121 to D127).
What is right to keep is listed in `tests/words.rs`, `KEPT`, one sentence a line with its reason:
an action in progress, an entry that opens a dialog, a raw value under « Avancé », « Détails » or
the developer mode. Never a pattern: each one by its key.

A test cannot read what fills a placeholder: an engine's id given as `{}` is the review's to see.
The rest of this page (the tone, the gesture after the error, the terms a test cannot tell apart)
is the review's.

## Changing a sentence

A sentence changed in `words.rs` changes every test that reads it: `tests/words.rs`, and the window's e2e
(`scripts/e2e_window.py`), which waits for the sentences the window shows. Before a pull request, the e2e is
searched from its own string literals, not from the sentence: each literal read as Python reads it (a
` ` or a ` ` is the character itself), then kept when it is part of the old sentence and not of the new
one. A search for the whole sentence misses a check written with escapes, and one that tests a fragment
(« absent », « n’a pas pu être gardé »); #950's first run failed 34 checks so. A check that fails on the new
sentence says the old one's meaning: when the new sentence drops it (where a download comes from, where a
meeting's words go, V8), the sentence is wrong, not the check.

A sentence with a placeholder is searched a second time, by each of its fixed parts, in the whole e2e,
`main()` included, which only the attest runs. A check that glues a value to a part both sentences share
(« 0 n’est pas un nombre de jours »), or composes the sentence with its reason in an f-string
(« Le réglage n’a pas été enregistré : {…} »), is no part of the old sentence alone, and the first search
lets it through; #950 and P7 each had two. A check that reads `.text` reads a ` ` as a space, a
` ` as itself: it waits for « « 0 » » with plain spaces.

A hit of either search inside `main()` is read and settled one by one: a partial run never reaches
`main()`, so a green partial says nothing of it (#950's attest failed so, on « compte rendu »).
