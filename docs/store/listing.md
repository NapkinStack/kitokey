# Kitokey — the Microsoft Store's listing

The words Partner Center is given, kept here (D57, [PDR-0004](../pdr/0004-windows-through-the-microsoft-store.md)).
They say [PDR-0001](../pdr/0001-wholeheard-open-on-linux-paid-on-macos-and-windows.md)'s promise,
and claim only what the product does today. The trial's terms are
[PDR-0005](../pdr/0005-on-windows-the-trial-ends-without-locking-the-meetings.md)'s, said as
[Store Policies](https://learn.microsoft.com/en-us/windows/apps/publish/store-policies) §10.8.4
asks. Privacy policy's URL, each listing its language's: <https://kitokey.com/en/privacy/> and
<https://kitokey.com/fr/privacy/>, the site's own copy of docs/privacy.md (D114, open since 2026-11-28). Support page's URL:
<https://github.com/NapkinStack/kitokey/blob/main/docs/support.md>.

## Screenshots

Made by the window's e2e, scenario CH `store_shots` (D103), from meetings invented for it — nothing of a real
one —, in French then in English, light, at 1920 × 1080: Microsoft's page asks « 1366 x 768 pixels or larger »,
a `.png` of at most 50 MB, shown « in order », each with a caption of at most 200 characters
([*App screenshots, images, and trailers for MSIX app*](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/screenshots-and-images),
read on 2026-11-20, D103's section of [cycle 11](../project/cycles/11-kitokey-on-sale-and-out-of-the-app.md#d103)).
The run writes them to `store/<en|fr>/<n>-<what>.png` in its screenshots' folder, and checks their count, order and
size; they are uploaded for each language in this order. The order puts the benefit first — Microsoft's first
image « especially representative and appealing » —, and each caption says what the buyer gets, not the
screen's name (the coordinator's ruling, 2026-11-20). Nothing is added to the images: no logo, no text. What
matters stays in their top two-thirds, the bottom third being where the Store may lay its own text.

The four shots of D103's criterion 1, as the decider amended it (2026-11-21 ~03:00): meetings a prospect who
runs back-to-back calls recognises at a glance. Their captions, the decider's « ok, on valide comme tu proposes »
(2026-11-21 ~03:30), [#1073](https://github.com/NapkinStack/kitokey/pull/1073). Shot 2 is a meeting while it
records, its time large and both sides' levels (#1076, #1112, #1126); its caption, the coordinator's acceptance
(2026-11-22), for the decider.

| # | File | Caption (English) | Légende (français) |
|---|---|---|---|
| 1 | `1-meeting.png` | No bot joins your calls: Kitokey records on your computer, then writes the minutes — decisions, actions, who does what. | Aucun robot ne rejoint vos appels : Kitokey enregistre sur votre ordinateur, puis rédige le compte rendu — décisions, actions, qui fait quoi. |
| 2 | `2-recording.png` | Records any call, in any app: your voice and the others', each heard on its own. You see it recording — the time and both levels. | Enregistre n'importe quel appel, dans n'importe quelle application : votre voix et celle des autres, chacune à part. Vous le voyez enregistrer — le temps et les deux niveaux. |
| 3 | `3-list.png` | Every meeting with a client, in one search. | Toutes vos réunions avec un client, en une recherche. |
| 4 | `4-settings.png` | By default, transcription runs on your computer: nothing is sent. An online service stays your choice. | Par défaut, la transcription se fait sur votre ordinateur : rien n'est envoyé. Un service en ligne reste votre choix. |

## English

**Name.** Kitokey

**Short description.** Be in the meeting. The minutes write themselves, on your computer.

**Description.**
Stay in the conversation: Kitokey records your call, transcribes it and writes its minutes —
highlights, actions, key points. The minutes are written by an AI language model, on your computer by
default. No bot joins your meeting, and no account is asked for.

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
its speech model, Whisper large-v3-turbo (874 MB), from huggingface.co when you click to get it; Ollama
downloads its model for the minutes, Qwen3 8b (about 5 GB), on your click too. Choose a service instead, and
neither is needed.

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
rendu — points saillants, actions, points clés. Le compte rendu est rédigé par un modèle de langage
(IA), sur votre ordinateur par défaut. Aucun robot ne rejoint votre réunion, et aucun compte n'est
demandé.

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
depuis huggingface.co quand vous cliquez pour l'obtenir ; Ollama télécharge son modèle pour le compte rendu,
Qwen3 8b (environ 5 Go), sur votre clic aussi. Choisissez plutôt un service, et ni l'un ni l'autre n'est
nécessaire.

**Fonctionnalités.**
- Enregistre votre micro et le son de votre ordinateur, dans toute application d'appel
- Pas de robot dans l'appel, pas de compte
- Transcription et compte rendu sur votre ordinateur par défaut
- Actions rangées par qui, quoi et pour quand
- Réécoutez depuis une ligne ; corrigez une ligne de la transcription
- Compte rendu copié en un clic, à coller là où vous écrivez
- Une réunion confidentielle reste sur cet ordinateur

## Partner Center — the submission's other fields

Decided on 2026-11-21 by the coordinator as store-listing lead, or by the decider where it says so; Microsoft's
pages read the same day (ines, D103). Each field is entered in Partner Center, not in the package.

| Field | Value | Why, and its source |
|---|---|---|
| Listing languages | English (`en-us`), French (`fr-fr`) | The package's languages (`modules/install/windows/AppxManifest.xml.in`): one listing each |
| Category | Productivity, no subcategory | [Categories](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/categories-and-subcategories): Productivity has none; « Specifying a category is required » |
| Privacy policy | <https://kitokey.com/en/privacy/> (English), <https://kitokey.com/fr/privacy/> (French): the site's copy of docs/privacy.md (D114) | Required: a desktop app « inherently ha[s] access to Personal Information », and Kitokey records the microphone ([Store Policies](https://learn.microsoft.com/en-us/windows/apps/publish/store-policies) 10.5.1) |
| Website | <https://github.com/NapkinStack/kitokey> | Optional; « a page on your own website, not your app's web listing in the Store » ([support info](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/support-info)) |
| Support contact | <https://github.com/NapkinStack/kitokey/blob/main/docs/support.md>, which gives `support@kitokey.com` | The decider: « Une page de support »; the address, « Ok on valide support@kitokey.com » (2026-11-21 ~12:05), the same for support and for data requests (`privacy.md`); the mailbox's reception confirmed before submission |
| System requirements | None declared yet | Optional ([system requirements](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/system-requirements)). Memory and disk come from D105's measure of the release build, once run; the package already asks Windows 10, version 2004 (10.0.19041) or later |
| « This product has been tested to meet accessibility guidelines » | Not checked | D106 is a first pass by the keyboard, not an audit; checked when a sheet holds it |
| « This product incorporates generative AI features » | Checked | The coordinator, 2026-11-21: an LLM writes the minutes. [Store Policies](https://learn.microsoft.com/en-us/windows/apps/publish/store-policies) 11.16, *Live Generative AI Content*: the use is disclosed in the description (above), and users can report a minutes' content to `support@kitokey.com` ([support](../support.md)) ([product declarations](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/product-declarations)) |
| « … make purchases, but does not use the Microsoft Store commerce system » | Not checked | The Store sells the licence (`licence.rs`, `StoreContext`) |
| Alternate drives; OneDrive backups | Left as Partner Center checks them | Nothing in the product depends on either |
| Age rating (IARC) | An app, not a game. Users interact with each other: no. Location shared: no. Personal information shared with third parties: **yes** — only with a service the user chooses, with their own key. Unrestricted web access: no. Purchases in the app: none | Honesty first (the coordinator). The questionnaire's questions follow the category answered first ([age ratings](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/age-ratings)); Microsoft shares « your publisher display name and email address with IARC » |
| Copyright | © 2026 c12c | The decider (company c12c, trade name NapkinStack); 200 characters at most |
| Licence terms | Microsoft's standard application licence terms | The decider. The Store's binary is distributed by its copyright holder under them while the source stays GPL with a CLA ([PDR-0001](../pdr/0001-wholeheard-open-on-linux-paid-on-macos-and-windows.md)); marc checks no third-party dependency forbids it |
| Version | 1.0.0 | The decider |
| Free trial | Unlimited, in the Store | Kitokey's own trial counts days and meetings, which the Store's cannot (the decider, 2026-11-20, cycle 11) |
| Price, markets, release date | Owed by the decider | [Price and availability](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/price-and-availability) |
| Visibility | First a **private audience**: the decider and the three users; public only once [its test](private-audience-test.md) has heard both tracks of a real call on Windows (the coordinator, 2026-11-21) | A smoke test of install, licence and trial from the real Store. Each person given by the email of a *personal* Microsoft account ([visibility](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/visibility-options)) — entered in Partner Center, never here |
| Seller details (EU) | c12c's, in the account | A trader under the Digital Services Act verifies the company (a DUNS number or a business document) and gives customer contact details ([company verification](https://learn.microsoft.com/en-us/windows/apps/publish/store-business-verification-reqs)): the decider, in Partner Center |
| Payout and tax profiles | Owed by the decider | Needed before a paid product is submitted |

**Search terms** — seven at most, no pricing term (Store Policies 10.1.3):

| English | Français |
|---|---|
| meeting minutes · meeting notes · transcription · call recorder · offline transcription · meeting summary · action items | compte rendu · notes de réunion · transcription · enregistreur d'appel · transcription hors ligne · résumé de réunion · actions de réunion |

**Restricted capability** — Partner Center asks how each one is used ([capability
declarations](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/app-capability-declarations)):

> `runFullTrust`: Kitokey is a desktop application (Tauri, Win32). It records the microphone and the computer's
> own sound during a call the user starts, transcribes it with a local speech model, and keeps
> each meeting as files in the user's Documents folder. These need a full-trust process.

**Notes for certification**:

> Kitokey records a meeting from its window or its bar: « Start recording », then « Stop ». No account is needed.
> The transcript needs a speech model, offered at first start and downloaded only on a click (874 MB,
> huggingface.co). The minutes need Ollama, a separate free program Kitokey never installs: without it, the
> transcript is made and the window says the minutes wait. The trial is Kitokey's own — 14 days and 10
> meetings, whichever comes last —, shown in Settings; the Store's trial is set to « Unlimited » and the Store
> sells the licence. Kitokey starts with Windows only when the user turns it on in Settings.

## What holds each promise

Not for Partner Center: for whoever reads this listing before it is submitted (D103). Each promise of the text
above points to what holds it in the product — a scenario of the shell's window e2e (`scripts/e2e_window.py`,
by its letter and function), a module's tests, or a decision. A promise nothing holds is no promise to make.

| Promise | Held by |
|---|---|
| Records your microphone and your computer's sound, in any call application | `capture`: `tests/session_audio_v2.rs`, `tests/call.rs` (Linux's graph); contract `session-audio`. **On Windows**: the default input and the default output's sound through cpal, with no automated test — held by the [private audience's test](private-audience-test.md), step 6, a real call recorded from the Store's install, both tracks heard (#482), before Kitokey goes public (the coordinator, 2026-11-21) |
| No bot in the call, no account | Kitokey records the computer's own sound and joins nothing; it has no account or server ([PDR-0001](../pdr/0001-wholeheard-open-on-linux-paid-on-macos-and-windows.md), [privacy policy](../privacy.md)) |
| Transcript and minutes on your computer by default | AH `context_gaps` (« in private mode the context goes to Whisper and Ollama on this computer alone »); Y `first_run` (D58) |
| A service only if you choose it, with your own key; where a meeting goes seen before it is sent | Z `service_named` (its host agreed to first); CB `key_kept_to_its_host` (#964); AG `corrections_gated` |
| A confidential meeting never leaves | AA `confidential_kept`, AI `confidential_every_way` (D60) |
| Kitokey reminds you to tell the others you record | AM `announced` (D50, D90) |
| Actions listed by who, what and by when | AK `actions_rows` (D46) |
| Listen again from any line; correct a line of the transcript | R, in `main()`'s sections (D26); AB `transcript_corrected` (D62) |
| Minutes copied in one click, to paste where you write | AJ `minutes_copied` (D45, D109) |
| A free trial of 14 days and 10 meetings, what is left shown in Settings | AW `trial_left` (D83, [PDR-0005](../pdr/0005-on-windows-the-trial-ends-without-locking-the-meetings.md)); `tests/trial.rs`; the real Store's licence by `modules/install/windows/trial.ps1` alone, on a person's Windows, and the private audience's first install |
| After the trial, every meeting still read, played, corrected and copied | AV `trial_over` (D83) |
| Ollama never installed by Kitokey; the speech model downloaded on a click | V `ollama_pulled` (D37, on the user's click); `model::DEFAULT` (874 MB, `huggingface.co`) |
| The screenshots: four a language, at Microsoft's size | CH `store_shots` (D103) |

