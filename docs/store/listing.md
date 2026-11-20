# Kitokey — the Microsoft Store's listing

The words Partner Center is given, kept here (D57, [PDR-0004](../pdr/0004-windows-through-the-microsoft-store.md)).
They say [PDR-0001](../pdr/0001-wholeheard-open-on-linux-paid-on-macos-and-windows.md)'s promise,
and claim only what the product does today. The trial's terms are
[PDR-0005](../pdr/0005-on-windows-the-trial-ends-without-locking-the-meetings.md)'s, said as
[Store Policies](https://learn.microsoft.com/en-us/windows/apps/publish/store-policies) §10.8.4
asks. Privacy policy's URL:
<https://github.com/NapkinStack/kitokey/blob/main/docs/privacy.md>.

## Screenshots

Made by the window's e2e, scenario CH `store_shots` (D103), from meetings invented for it — nothing of a real
one —, in French then in English, light, at 1920 × 1080: Microsoft's page asks « 1366 x 768 pixels or larger »,
a `.png` of at most 50 MB, shown « in order », each with a caption of at most 200 characters
([*App screenshots, images, and trailers for MSIX app*](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/screenshots-and-images),
read on 2026-11-20, D103's section of [cycle 11](../project/cycles/11-kitokey-on-sale-and-out-of-the-app.md#d103)).
The run writes them to `store/<en|fr>/<n>-<what>.png` in its screenshots' folder, and checks their count, order and
size; they are uploaded for each language in this order.

| # | File | Caption (English) | Légende (français) |
|---|---|---|---|
| 1 | `1-list.png` | Your meetings by week, each with its minutes, open beside the list. | Vos réunions par semaine, chacune avec son compte rendu, ouverte à côté de la liste. |
| 2 | `2-meeting.png` | Minutes written on your computer — highlights, actions by who and by when, key points — beside a transcript you can listen to. | Un compte rendu écrit sur votre ordinateur — points saillants, actions par qui et pour quand, points clés — à côté d'une transcription à réécouter. |
| 3 | `3-recording.png` | Recording any call from the window, or from a discreet bar above your other windows. | L'enregistrement de tout appel, depuis la fenêtre ou une barre discrète au-dessus de vos autres fenêtres. |
| 4 | `4-settings.png` | Transcription and minutes on this computer by default; a service only if you choose one, with your own key. | Transcription et compte rendu sur cet ordinateur par défaut ; un service seulement si vous le choisissez, avec votre clé. |

## English

**Name.** Kitokey

**Short description.** Be in the meeting. The minutes write themselves, on your computer.

**Description.**
Stay in the conversation: Kitokey records your call, transcribes it and writes its minutes —
highlights, actions, key points. No bot joins your meeting, and no account is asked for.

Your meetings stay on your computer. Transcript and minutes are made there by default. If you
prefer a service you trust, you choose it, with your own key, and you see where a meeting goes
before anything is sent. A meeting you say confidential never leaves.

You tell the other participants that you are recording: Kitokey shows them nothing, and reminds
you to.

**Free trial.** Try Kitokey for free, for 14 days and 10 meetings, whichever comes last. Settings
show what is left. Once the trial is over, you buy Kitokey to record new meetings. Every meeting
you made stays yours: you can still read it, play it, correct it and copy its minutes.

**What it downloads, before you buy.** To write minutes on your computer, Kitokey uses Ollama, a free
program you install from ollama.com: Kitokey never installs it. To transcribe on your computer, it downloads
its speech model, Whisper large-v3-turbo (874 MB), from huggingface.co when you click to get it. Choose a
service instead, and neither is needed.

**Features.**
- Records your microphone and your computer's sound, in any call application
- No bot in the call, no account
- Transcript and minutes on your computer by default
- Actions listed by who, what and by when
- Listen again from any line; correct a line of the transcript
- Minutes copied in one click, to paste where you write
- A confidential meeting stays on this computer

## Français

**Nom.** Kitokey

**Description courte.** Soyez dans la réunion. Le compte rendu s'écrit tout seul, sur votre
ordinateur.

**Description.**
Restez dans la conversation : Kitokey enregistre votre appel, le transcrit et en écrit le compte
rendu — points saillants, actions, points clés. Aucun robot ne rejoint votre réunion, et aucun
compte n'est demandé.

Vos réunions restent sur votre ordinateur. La transcription et le compte rendu s'y font par défaut.
Si vous préférez un service en qui vous avez confiance, vous le choisissez, avec votre propre clé,
et vous voyez où part une réunion avant tout envoi. Une réunion dite confidentielle ne part jamais.

C'est à vous de prévenir les autres participants que vous enregistrez : Kitokey ne leur montre
rien, et vous le rappelle.

**Essai gratuit.** Essayez Kitokey gratuitement pendant 14 jours et 10 réunions, jusqu'à ce que les
deux soient atteints. Les Réglages indiquent ce qu'il en reste. Une fois l'essai terminé, vous
achetez Kitokey pour enregistrer de nouvelles réunions. Toutes vos réunions restent à vous : vous
pouvez toujours les lire, les écouter, les corriger et copier leur compte rendu.

**Ce qu'il télécharge, avant l'achat.** Pour écrire le compte rendu sur votre ordinateur, Kitokey
utilise Ollama, un programme gratuit que vous installez depuis ollama.com : Kitokey ne l'installe jamais.
Pour transcrire sur votre ordinateur, il télécharge son modèle de parole, Whisper large-v3-turbo (874 Mo),
depuis huggingface.co quand vous cliquez pour l'obtenir. Choisissez plutôt un service, et ni l'un ni l'autre
n'est nécessaire.

**Fonctionnalités.**
- Enregistre votre micro et le son de votre ordinateur, dans toute application d'appel
- Pas de robot dans l'appel, pas de compte
- Transcription et compte rendu sur votre ordinateur par défaut
- Actions rangées par qui, quoi et pour quand
- Réécoutez depuis une ligne ; corrigez une ligne de la transcription
- Compte rendu copié en un clic, à coller là où vous écrivez
- Une réunion confidentielle reste sur cet ordinateur

## What holds each promise

Not for Partner Center: for whoever reads this listing before it is submitted (D103). Each promise of the text
above points to what holds it in the product — a scenario of the shell's window e2e (`scripts/e2e_window.py`,
by its letter and function), a module's tests, or a decision. A promise nothing holds is no promise to make.

| Promise | Held by |
|---|---|
| Records your microphone and your computer's sound, in any call application | `capture`: `tests/session_audio_v2.rs`, `tests/call.rs`; contract `session-audio` |
| No bot in the call, no account | Kitokey records the computer's own sound and joins nothing; it has no account or server ([PDR-0001](../pdr/0001-wholeheard-open-on-linux-paid-on-macos-and-windows.md), [privacy policy](../privacy.md)) |
| Transcript and minutes on your computer by default | AH `context_gaps` (« in private mode the context goes to Whisper and Ollama on this computer alone »); Y `first_run` (D58) |
| A service only if you choose it, with your own key; where a meeting goes seen before it is sent | Z `service_named` (its host agreed to first); CB `key_kept_to_its_host` (#964); AG `corrections_gated` |
| A confidential meeting never leaves | AA `confidential_kept`, AI `confidential_every_way` (D60) |
| Kitokey reminds you to tell the others you record | AM `announced` (D50, D90) |
| Actions listed by who, what and by when | AK `actions_rows` (D46) |
| Listen again from any line; correct a line of the transcript | R, in `main()`'s sections (D26); AB `transcript_corrected` (D62) |
| Minutes copied in one click, to paste where you write | AJ `minutes_copied` (D45, D109) |
| A free trial of 14 days and 10 meetings, what is left shown in Settings | AW `trial_left` (D83, [PDR-0005](../pdr/0005-on-windows-the-trial-ends-without-locking-the-meetings.md)) |
| After the trial, every meeting still read, played, corrected and copied | AV `trial_over` (D83) |
| Ollama never installed by Kitokey; the speech model downloaded on a click | V `ollama_pulled` (D37, on the user's click); `model::DEFAULT` (874 MB, `huggingface.co`) |
| The screenshots: four a language, at Microsoft's size | CH `store_shots` (D103) |

