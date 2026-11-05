//! Kitokey's licence, as the shell sees it, and the trial's gate before a recording (PDR-0005,
//! D83). The gate is the same on every system; only reading the Store's licence, and buying
//! there, is Windows' own.

use std::path::Path;

use crate::trial;

/// The window the Store's dialogs belong to: on Windows, its handle, as Microsoft requires of a
/// desktop application before any reading or purchase (*In-app purchases and trials*); none when
/// Kitokey shows in the tray alone. Elsewhere unused.
pub type Owner = Option<isize>;

/// What the licence is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Licence {
    Bought,
    Trial,
    /// The Store answers a licence that is not active.
    Inactive,
    /// Not installed from the Store: the zip, a package signed for a run, Linux, macOS.
    NotFromStore,
}

/// The licence now, read anew at each call: one bought while Kitokey is open is bought at the next
/// recording. An error is a licence that could not be read.
///
/// With `KITOKEY_LICENCE` naming a file, in a build for trying Kitokey only, the file's word
/// stands in for it — `bought`, `trial`, `inactive` or `none`, anything else not read —: the
/// window's end-to-end scenario holds the gate on Linux. A build made for release has no such
/// reading.
///
/// On Windows, the Store's own answer (`StoreContext.GetAppLicenseAsync`): a process Windows did
/// not start from a package — the zip — is not from the Store, asked nothing; a package answers
/// what the Store says of it. Elsewhere, not from the Store.
pub fn read(owner: Owner) -> Result<Licence, String> {
    #[cfg(debug_assertions)]
    if let Some(file) = std::env::var_os("KITOKEY_LICENCE") {
        return stand_in(Path::new(&file));
    }
    #[cfg(windows)]
    return store::licence(owner);
    #[cfg(not(windows))]
    {
        let _ = owner;
        Ok(Licence::NotFromStore)
    }
}

/// What asking the Store for the purchase came to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Purchase {
    /// The Store's dialog answered: bought, already bought, or closed by the user.
    Answered,
    /// The Store's dialog failed, or never started (Microsoft Q&A 24011): its page for Kitokey,
    /// by this address, is what « Acheter » opens in its place.
    Page(String),
    /// Nothing to buy from: not from the Store, or the stand-in licence, whose purchase is the
    /// test's own (`KITOKEY_LICENCE`).
    Nothing,
}

/// Kitokey's own Store ID, the product's (`9NBLGGH4R315`), as the Store's licence names it — none
/// outside the Store. Never written in the code: it is the identity the decider reserves in
/// Partner Center, which only the Store knows. Read, like the licence, off the window's thread.
pub fn store_id(owner: Owner) -> Result<Option<String>, String> {
    #[cfg(debug_assertions)]
    if std::env::var_os("KITOKEY_LICENCE").is_some() {
        return Ok(None);
    }
    #[cfg(windows)]
    return store::id(owner);
    #[cfg(not(windows))]
    {
        let _ = owner;
        Ok(None)
    }
}

/// Asks the Store to buy Kitokey, by its Store ID, its dialog owned by the window: to be called on
/// the thread of that window, as the Store requires (Microsoft Q&A 24011). Returns at once; `done`
/// is told what came of it, from another thread. Elsewhere than Windows, nothing to buy from.
pub fn purchase(owner: Owner, id: &str, done: impl FnOnce(Purchase) + Send + 'static) {
    #[cfg(windows)]
    store::purchase(owner, id, done);
    #[cfg(not(windows))]
    {
        let _ = (owner, id);
        done(Purchase::Nothing);
    }
}

/// The Store's page for a product, by its Store ID: the address Microsoft recommends.
pub fn page(id: &str) -> String {
    format!("ms-windows-store://pdp/?ProductId={id}")
}

#[cfg(debug_assertions)]
fn stand_in(file: &Path) -> Result<Licence, String> {
    match std::fs::read_to_string(file).map_err(|error| error.to_string())?.trim() {
        "bought" => Ok(Licence::Bought),
        "trial" => Ok(Licence::Trial),
        "inactive" => Ok(Licence::Inactive),
        "none" => Ok(Licence::NotFromStore),
        other => Err(format!("the stand-in licence says {other:?}")),
    }
}

/// Whether a recording starting now is refused: on a trial licence whose trial is over, and on
/// nothing else. A licence not read, not active or not from the Store records, as before the trial
/// existed: a recording is never refused on a doubt.
pub fn refused(licence: &Result<Licence, String>, meetings: &Path, now: i64) -> bool {
    matches!(licence, Ok(Licence::Trial)) && trial::over(meetings, now)
}

/// What Settings say is left of the trial: on a trial licence whose trial is not over, and on
/// nothing else. Over, the refusal says the rest; none either when it cannot be told.
pub fn left(licence: &Result<Licence, String>, meetings: &Path, now: i64) -> Option<trial::Left> {
    if !matches!(licence, Ok(Licence::Trial)) {
        return None;
    }
    trial::left(meetings, now).filter(|left| !left.over())
}

/// The Store, through `Windows.Services.Store` (Windows 10, version 1607 and later).
#[cfg(windows)]
mod store {
    use std::{sync::mpsc, thread, time::Duration};

    use windows::{
        ApplicationModel::Package,
        Services::Store::{StoreAppLicense, StoreContext, StorePurchaseStatus},
        Win32::{Foundation::HWND, UI::Shell::IInitializeWithWindow},
        core::{HRESULT, HSTRING, Interface},
    };

    use super::{Licence, Owner, Purchase, page};

    /// `APPMODEL_ERROR_NO_PACKAGE`: the process has no package's identity.
    const NO_PACKAGE: HRESULT = HRESULT::from_win32(15_700);

    /// How long the Store is waited for, a recording waiting on it: past this, a licence that
    /// could not be read, which records.
    const WAIT: Duration = Duration::from_secs(5);

    /// The Store's context, given the window its dialogs belong to before anything is asked.
    fn context(owner: Owner) -> windows::core::Result<StoreContext> {
        let context = StoreContext::GetDefault()?;
        if let Some(window) = owner {
            // SAFETY: a window's handle, given by the toolkit for a window that is open.
            unsafe { context.cast::<IInitializeWithWindow>()?.Initialize(HWND(window as *mut _))? };
        }
        Ok(context)
    }

    /// Whether Windows started Kitokey from a package; a doubt is an error.
    fn packaged() -> windows::core::Result<bool> {
        match Package::Current() {
            Ok(_) => Ok(true),
            Err(error) if error.code() == NO_PACKAGE => Ok(false),
            Err(error) => Err(error),
        }
    }

    /// The Store's licence of this package, none outside a package; asked on a thread of its own
    /// — never the window's, whose messages a wait would hold —, waited for at most `WAIT`.
    fn app_licence(owner: Owner) -> Result<Option<StoreAppLicense>, String> {
        let (sent, received) = mpsc::channel();
        thread::spawn(move || {
            let read = packaged().and_then(|packaged| packaged.then(|| context(owner)?.GetAppLicenseAsync()?.join()).transpose());
            let _ = sent.send(read);
        });
        received
            .recv_timeout(WAIT)
            .map_err(|_| format!("the Store did not answer within {} s", WAIT.as_secs()))?
            .map_err(|error| error.to_string())
    }

    pub fn licence(owner: Owner) -> Result<Licence, String> {
        let Some(licence) = app_licence(owner)? else {
            return Ok(Licence::NotFromStore);
        };
        let read = || -> windows::core::Result<Licence> {
            Ok(if !licence.IsActive()? {
                Licence::Inactive
            } else if licence.IsTrial()? {
                Licence::Trial
            } else {
                Licence::Bought
            })
        };
        read().map_err(|error| error.to_string())
    }

    pub fn id(owner: Owner) -> Result<Option<String>, String> {
        let Some(licence) = app_licence(owner)? else {
            return Ok(None);
        };
        // A SKU's Store ID is the product's, then `/` and the SKU's own four characters.
        let sku = licence.SkuStoreId().map_err(|error| error.to_string())?.to_string();
        let product = sku.split('/').next().unwrap_or_default();
        Ok((!product.is_empty()).then(|| product.to_owned()))
    }

    pub fn purchase(owner: Owner, id: &str, done: impl FnOnce(Purchase) + Send + 'static) {
        let fallback = page(id);
        let asked = context(owner).and_then(|context| context.RequestPurchaseAsync(&HSTRING::from(id)));
        let operation = match asked {
            Ok(operation) => operation,
            Err(error) => {
                tracing::warn!(detail = %error, "shell.purchase_failed");
                return done(Purchase::Page(fallback));
            }
        };
        let when = operation.when(move |result| {
            let answered = result.and_then(|result| Ok((result.Status()?, result.ExtendedError()?)));
            done(match answered {
                Ok((StorePurchaseStatus::Succeeded | StorePurchaseStatus::AlreadyPurchased, _)) => Purchase::Answered,
                // Closed by the user: their choice, nothing opened in its place.
                Ok((StorePurchaseStatus::NotPurchased, error)) if error.is_ok() => Purchase::Answered,
                Ok((status, error)) => {
                    tracing::warn!(status = status.0, detail = %error, "shell.purchase_failed");
                    Purchase::Page(fallback)
                }
                Err(error) => {
                    tracing::warn!(detail = %error, "shell.purchase_failed");
                    Purchase::Page(fallback)
                }
            });
        });
        // `when` only fails if the operation cannot take a handler: then `done` was dropped, and
        // the window's wait ends without an answer.
        if let Err(error) = when {
            tracing::warn!(detail = %error, "shell.purchase_failed");
        }
    }
}
