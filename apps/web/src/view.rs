use crate::{
    api::{StudioContext, StudioHttp},
    editor::{Editor, Intent, Status},
    import::ManuscriptImport,
    messages,
};
use cantos_api::{ApiError, ErrorCode, FieldIssue};
use leptos::{prelude::*, task::spawn_local};

pub(crate) fn operation_id() -> Result<String, ApiError> {
    let error = || ApiError {
        code: ErrorCode::Unavailable,
        current_revision: None,
        issues: vec![],
    };
    let window = web_sys::window().ok_or_else(error)?;
    let crypto = window.crypto().map_err(|_| error())?;
    let mut bytes = [0u8; 16];
    crypto
        .get_random_values_with_u8_array(&mut bytes)
        .map_err(|_| error())?;
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    let hex = bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    Ok(format!(
        "{}-{}-{}-{}-{}",
        &hex[..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..]
    ))
}

fn dispatch_save(
    state: RwSignal<Editor>,
    issues: RwSignal<Vec<FieldIssue>>,
    api: StudioHttp,
    intent: Intent,
) {
    spawn_local(async move {
        match api.save(&intent.script, &intent.request).await {
            Ok(response) => {
                let _ = state.try_update(|editor| *editor = editor.saved(&intent, response));
            }
            Err(error) => {
                let _ = issues.try_set(error.issues.clone());
                let _ = state.try_update(|editor| *editor = editor.failed(&error));
            }
        }
    });
}

#[component]
pub fn Studio() -> impl IntoView {
    provide_context(StudioContext { api: StudioHttp });
    let api = expect_context::<StudioContext>().api;
    let state = RwSignal::new(Editor::default());
    let issues = RwSignal::new(Vec::<FieldIssue>::new());
    let token = RwSignal::new(String::new());
    // Safari can clear autocomplete=off fields while retaining WASM state in bfcache.
    // Reapply controlled values on return; never let the visible ID differ from the save target.
    let restored =
        leptos::leptos_dom::helpers::window_event_listener_untyped("pageshow", move |_| {
            let _ = state.try_update(|_| ());
            let _ = token.try_update(|_| ());
        });
    on_cleanup(move || restored.remove());

    let english = RwSignal::new(false);
    let theme = RwSignal::new(String::from("system"));
    let copy = move || messages::copy(english.get());
    let blocked = move || state.with(|state| state.busy || state.pending.is_some());
    let save_blocked = move || blocked() || state.with(|editor| editor.actor.is_empty());

    view! {
        <div class="studio-shell" data-theme=move || theme.get()>
        <main lang=move || if english.get() { "en" } else { "vi-VN" }>
            <header>
                <h1>{move || copy().title}</h1>
                <button type="button" on:click=move |_| english.update(|en| *en = !*en)>
                    {move || if english.get() { "Tiếng Việt" } else { "English" }}
                </button>
                <label for="theme-select">{move || copy().theme}</label>
                <select id="theme-select" prop:value=move || theme.get() on:change=move |event| theme.set(event_target_value(&event))>
                    <option value="system">{move || copy().system}</option>
                    <option value="light">{move || copy().light}</option>
                    <option value="dark">{move || copy().dark}</option>
                </select>
            </header>
            <p class="mode">{move || copy().mode}</p>
            <form on:submit=move |event| {
                event.prevent_default();
                if state.with_untracked(|editor| editor.busy) { return; }
                let submitted = token.get_untracked();
                state.update(|editor| { editor.busy = true; editor.status = Status::Loading; });
                spawn_local(async move {
                    match api.login(submitted).await {
                        Ok(response) => {
                            let _ = token.try_set(String::new());

                            let _ = state.try_update(|editor| { editor.busy = false; editor.actor = response.actor_id; editor.status = if editor.pending.as_ref().is_some_and(|intent| intent.actor != editor.actor) { Status::Forbidden } else { Status::SignedIn }; });
                        },
                        Err(error) => { let _ = state.try_update(|editor| *editor = editor.session_failed(&error)); }
                    }
                });
            }>
                <label for="session-token">{move || copy().token}</label>
                <div class="toolbar">
                    <input id="session-token" type="password" autocomplete="off" prop:value=move || token.get()
                        on:input=move |event| token.set(event_target_value(&event)) />
                    <button type="submit" aria-disabled=move || state.with(|e|e.busy)>{move || copy().login}</button>
                </div>
            </form>
            <p>{move || copy().actor}<output>{move || state.with(|editor| if editor.actor.is_empty() { copy().unknown.to_owned() } else { editor.actor.clone() })}</output></p>
            <ManuscriptImport actor=Signal::derive(move || state.with(|editor| editor.actor.clone())) english=english />
            <section aria-labelledby="draft-label">
                <label for="script-id">{move || copy().script}</label>
                <input id="script-id" autocomplete="off" spellcheck="false" disabled=blocked
                    prop:value=move || state.with(|editor| editor.script.clone())
                    on:input=move |event| state.update(|editor| *editor = editor.select_script(event_target_value(&event))) />
                <div class="toolbar">
                    <button type="button" aria-disabled=blocked on:click=move |_| {
                        if let Some(reading) = state.get_untracked().start_read() {
                            let script = reading.script.clone(); let ticket = reading.ticket;
                            state.set(reading); issues.set(vec![]);
                            spawn_local(async move {
                                match api.read(&script).await {
                                    Ok(response) => { let _ = state.try_update(|editor| *editor = editor.loaded(ticket,response)); },
                                    Err(error) => { let _ = state.try_update(|editor| *editor = editor.failed(&error)); }
                                }
                            });
                        }
                    }>{move || copy().read}</button>
                    <span>{move || copy().base}<output id="base-revision">{move || state.with(|editor|editor.base)}</output></span>
                </div>
                <p id="draft-help">{move || copy().help}</p>
                <label id="draft-label" for="script-draft">{move || copy().draft}</label>
                <textarea id="script-draft" lang="vi-VN" spellcheck="false" aria-describedby="draft-help"
                    prop:value=move || state.with(|editor| editor.draft.clone())
                    on:input=move |event| state.update(|editor| *editor = editor.edit(event_target_value(&event))) />
                <div class="toolbar">
                    <button class="primary" type="button" aria-disabled=save_blocked aria-describedby="request-reason" on:click=move |_| {
                        if save_blocked() { return; }
                        match operation_id() {
                            Ok(operation) => if let Some((saving,intent)) = state.get_untracked().start_save(operation) {
                                state.set(saving); issues.set(vec![]); dispatch_save(state,issues,api,intent);
                            },
                            Err(error) => state.update(|editor| *editor = editor.failed(&error)),
                        }
                    }>{move || copy().save}</button>
                    <Show when=move || state.with(|editor| editor.pending.is_some() && !editor.busy)>
                        <button type="button" aria-disabled=move || state.with(|editor| editor.pending.as_ref().is_some_and(|intent| intent.actor != editor.actor)) on:click=move |_| {
                            if let Some((saving,intent)) = state.get_untracked().retry() {
                                state.set(saving); issues.set(vec![]); dispatch_save(state,issues,api,intent);
                            }
                        }>{move || copy().retry}</button>
                    </Show>
                </div>
                <p id="request-reason">{move || state.with(|editor| if editor.busy { copy().busy } else if editor.pending.is_some() { copy().pending } else if editor.actor.is_empty() { copy().sign_in_first } else { "" })}</p>
            </section>
            <p id="status" role="status" aria-live="polite">{move || messages::status(state.with(|editor|editor.status),english.get())}</p>
            <Show when=move || !issues.get().is_empty()>
                <h2>{move || copy().validation}</h2>
                <ul>{move || issues.get().into_iter().map(|issue| view! { <li><code>{issue.path}</code>" · "{issue.rule}</li> }).collect_view()}</ul>
            </Show>
            <Show when=move || state.with(|editor|editor.remote.is_some())>
                <section aria-labelledby="stored-label">
                    <h2 id="stored-label">{move || copy().preview}</h2>
                    <p class="identity">{move || state.with(|editor| editor.remote.as_ref().map(|remote| format!("{} / {} · {} · {}",remote.script_id,remote.revision,remote.accepted_by,remote.accepted_at)).unwrap_or_default())}</p>
                    <pre lang="vi-VN">{move || state.with(|editor| editor.remote.as_ref().map(|remote|remote.script_json.clone()).unwrap_or_default())}</pre>
                    <button type="button" aria-disabled=blocked on:click=move |_| state.update(|editor| *editor = editor.rebase())>{move || copy().rebase}</button>
                </section>
            </Show>
        </main>
        </div>
    }
}
