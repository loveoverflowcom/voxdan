//! Leptos adaptation shell: every mutation is an explicit, immutable intent.
use super::{AdaptationIntent, AdaptationReview, Mutation};
use crate::{
    api::{StudioContext, StudioHttp},
    import::ExtractionPreview,
    messages,
    view::operation_id,
};
use cantos_api::{
    AdaptationCoverageDisposition, AdaptationFindingCode, AdaptationProviderMetadata,
    AdaptationProviderResponse, AdaptationRunResponse, AdaptationStatus, ErrorCode, ImportOutcome,
    ImportResponse,
};
use leptos::{prelude::*, task::spawn_local};
use std::sync::Arc;

fn dispatch(state: RwSignal<AdaptationReview>, api: StudioHttp, intent: AdaptationIntent) {
    spawn_local(async move {
        if let Mutation::Accept {
            run_id, request, ..
        } = &intent.mutation
        {
            let result = api.accept_adaptation(run_id, request).await;
            let _ = state.try_update(|review| {
                if review.pending.as_ref() == Some(&intent) {
                    *review = match result {
                        Ok(revision) => review.accepted_result(&intent, revision),
                        Err(error) => review.failed(&error),
                    };
                }
            });
            return;
        }
        let result = match &intent.mutation {
            Mutation::Start(request) => api.start_adaptation(request).await,
            Mutation::Cancel { run_id, request } => api.cancel_adaptation(run_id, request).await,
            Mutation::Accept { .. } => return,
        };
        let _ = state.try_update(|review| {
            if review.pending.as_ref() == Some(&intent) {
                *review = match result {
                    Ok(response) => review.mutated(&intent, response),
                    Err(error) => review.failed(&error),
                };
            }
        });
    });
}

fn read_source(state: RwSignal<AdaptationReview>, api: StudioHttp) {
    let Some(reading) = state.get_untracked().start_source_read() else {
        return;
    };
    let id = reading.source_id.clone();
    let actor = reading.actor.clone();
    let ticket = reading.ticket;
    state.set(reading);
    spawn_local(async move {
        let result = api.read_import(&id).await;
        let _ = state.try_update(|review| {
            *review = match result {
                Ok(source) => review.source_loaded(ticket, &actor, source),
                Err(error) => review.read_failed(ticket, &actor, &error),
            };
        });
    });
}

fn read_run(state: RwSignal<AdaptationReview>, api: StudioHttp) {
    let Some(reading) = state.get_untracked().start_read() else {
        return;
    };
    let id = reading.run_id.clone();
    let actor = reading.actor.clone();
    let ticket = reading.ticket;
    state.set(reading);
    spawn_local(async move {
        let result = api.read_adaptation(&id).await;
        let _ = state.try_update(|review| {
            *review = match result {
                Ok(run) => review.loaded(ticket, &actor, run),
                Err(error) => review.read_failed(ticket, &actor, &error),
            };
        });
    });
}

#[component]
pub fn ScriptAdaptation(actor: Signal<String>, english: RwSignal<bool>) -> impl IntoView {
    let state = RwSignal::new(AdaptationReview::default());
    let provider = RwSignal::new(None::<Result<AdaptationProviderResponse, ErrorCode>>);
    let provider_busy = RwSignal::new(false);
    let provider_ticket = RwSignal::new(0u64);
    let composing = RwSignal::new(false);
    let api = expect_context::<StudioContext>().api;
    let copy = move || messages::adaptation_copy(english.get());
    let signed_actor = Memo::new(move |_| actor.get());
    Effect::new(move |_| {
        state.update(|review| *review = review.signed_in_as(signed_actor.get()));
        provider.set(None);
        provider_busy.set(false);
        provider_ticket.update(|ticket| *ticket = ticket.saturating_add(1));
    });
    // Immutable source/run facts do not remount their reading panes when the JSON changes.
    let source = Memo::new(move |_| state.with(|review| review.source.clone()));
    let run = Memo::new(move |_| state.with(|review| review.run.clone()));
    let accepted = Memo::new(move |_| state.with(|review| review.accepted.clone()));
    let blocked = move || state.with(AdaptationReview::blocked);
    let inputs_blocked = move || blocked() || run.get().is_some();
    let start_blocked = move || {
        provider.with(|value| !matches!(value, Some(Ok(config)) if config.provider.is_some()))
            || state.with(|review| !review.can_start())
    };
    let accept_blocked = move || state.with(|review| !review.can_accept());

    view! {
        <section class="script-adaptation" aria-labelledby="adaptation-title">
            <h2 id="adaptation-title">{move || copy().title}</h2>
            <p id="adaptation-help" class="help">{move || copy().help}</p>
            <div class="adaptation-provider">
                <button type="button" aria-disabled=move || provider_busy.get() || actor.get().is_empty() aria-describedby="adaptation-provider-status" on:click=move |_| {
                    if provider_busy.get_untracked() || actor.get_untracked().is_empty() { return; }
                    let submitted_actor = actor.get_untracked();
                    provider_ticket.update(|ticket| *ticket = ticket.saturating_add(1));
                    let ticket = provider_ticket.get_untracked();
                    provider_busy.set(true);
                    spawn_local(async move {
                        let result = api.adaptation_provider().await.map_err(|error| error.code);
                        if provider_ticket.try_get_untracked() == Some(ticket) && actor.try_get_untracked().as_deref() == Some(submitted_actor.as_str()) {
                            let observed = result.as_ref().ok().and_then(|response| response.provider.clone());
                            let _ = state.try_update(|review| *review = review.provider_observed(observed));
                            let _ = provider.try_set(Some(result));
                            let _ = provider_busy.try_set(false);
                        }
                    });
                }>{move || copy().provider_check}</button>
                <p id="adaptation-provider-status" class="help" role="status" aria-live="polite">{move || {
                    if provider_busy.get() { copy().provider_loading } else { provider.with(|value| match value {
                        None => copy().provider_unknown,
                        Some(Ok(response)) if response.provider.is_some() => copy().provider_configured,
                        Some(Ok(_)) => copy().provider_absent,
                        Some(Err(_)) => copy().provider_failed,
                    }) }
                }}</p>
                {move || provider.get().and_then(Result::ok).and_then(|response| response.provider).map(|metadata| view! { <ProviderDetails metadata=metadata english=english /> })}
            </div>
            <form on:submit=move |event| {
                event.prevent_default();
                if start_blocked() { return; }
                match operation_id() {
                    Ok(operation) => if let Some((sending, intent)) = state.get_untracked().start_run(operation) {
                        state.set(sending); dispatch(state, api, intent);
                    },
                    Err(error) => state.update(|review| *review = review.failed(&error)),
                }
            }>
                <label for="adaptation-source-id">{move || copy().source_id}</label>
                <div class="toolbar">
                    <input id="adaptation-source-id" maxlength="36" disabled=inputs_blocked autocomplete="off" spellcheck="false"
                        prop:value=move || state.with(|review| review.source_id.clone())
                        on:input=move |event| state.update(|review| *review = review.edit_inputs(|review| review.source_id = event_target_value(&event))) />
                    <button type="button" aria-disabled=move || blocked() || actor.get().is_empty() aria-describedby="adaptation-reason" on:click=move |_| read_source(state, api)>{move || copy().source_open}</button>
                </div>
                <div class="adaptation-target">
                    <div><label for="adaptation-script-id">{move || copy().script_id}</label>
                        <input id="adaptation-script-id" maxlength="36" disabled=inputs_blocked autocomplete="off" spellcheck="false" aria-describedby="adaptation-base-help"
                            prop:value=move || state.with(|review| review.script_id.clone())
                            on:input=move |event| state.update(|review| *review = review.edit_inputs(|review| review.script_id = event_target_value(&event))) /></div>
                    <div><label for="adaptation-base">{move || copy().base}</label>
                        <input id="adaptation-base" type="number" min="0" step="1" disabled=inputs_blocked aria-describedby="adaptation-base-help"
                            prop:value=move || state.with(|review| review.expected_revision.clone())
                            on:input=move |event| state.update(|review| *review = review.edit_inputs(|review| review.expected_revision = event_target_value(&event))) /></div>
                </div>
                <p id="adaptation-base-help" class="help">{move || copy().base_help}</p>
                <label class="adaptation-confirm" for="adaptation-rights">
                    <input id="adaptation-rights" type="checkbox" disabled=move || inputs_blocked() || state.with(|review| review.provider.is_none())
                        prop:checked=move || state.with(|review| review.rights_authorization)
                        on:change=move |event| state.update(|review| *review = review.edit_inputs(|review| review.rights_authorization = event_target_checked(&event))) />
                    <span>{move || copy().rights}</span>
                </label>
                <div class="toolbar"><button class="primary" type="submit" aria-disabled=start_blocked aria-describedby="adaptation-reason">{move || copy().start}</button></div>
            </form>
            <p id="adaptation-reason" class="help">{move || state.with(|review| if review.pending.is_some() { copy().pending } else if start_blocked() && review.run.is_none() { copy().required } else { "" })}</p>
            <Show when=move || state.with(|review| review.pending.is_some())>
                <p class="identity">{move || copy().operation}<output>{move || state.with(|review| review.pending.as_ref().map(|intent| intent.operation_id().to_owned()).unwrap_or_default())}</output></p>
                <button type="button" aria-describedby="adaptation-reason" aria-disabled=move || state.with(|review| !review.can_retry()) on:click=move |_| {
                    if let Some((sending, intent)) = state.get_untracked().retry() { state.set(sending); dispatch(state, api, intent); }
                }>{move || copy().retry}</button>
            </Show>
            <form class="adaptation-reopen" on:submit=move |event| { event.prevent_default(); read_run(state, api); }>
                <label for="adaptation-run-id">{move || copy().run_id}</label>
                <div class="toolbar">
                    <input id="adaptation-run-id" maxlength="36" disabled=move || blocked() || state.with(|review| review.draft_changed) autocomplete="off" spellcheck="false"
                        prop:value=move || state.with(|review| review.run_id.clone())
                        on:input=move |event| state.update(|review| *review = review.select_run(event_target_value(&event))) />
                    <button type="submit" aria-disabled=move || state.with(|review| !review.can_read()) aria-describedby="adaptation-new-help">{move || copy().open}</button>
                </div>
            </form>
            <div class="toolbar">
                <button type="button" aria-disabled=move || state.with(|review| !review.can_read()) on:click=move |_| read_run(state, api)>{move || copy().refresh}</button>
                <button type="button" aria-disabled=move || state.with(|review| !review.can_cancel()) aria-describedby="adaptation-new-help" on:click=move |_| {
                    if let Ok(operation) = operation_id() {
                        if let Some((sending, intent)) = state.get_untracked().start_cancel(operation) { state.set(sending); dispatch(state, api, intent); }
                    }
                }>{move || copy().cancel}</button>
                <button type="button" aria-disabled=blocked aria-describedby="adaptation-new-help" on:click=move |_| state.update(|review| *review = review.new_run())>{move || copy().new_run}</button>
            </div>
            <p id="adaptation-new-help" class="help">{move || copy().new_help}</p>
            <p id="adaptation-status" class="status" role="status" aria-live="polite">{move || state.with(|review| messages::adaptation_status(&review.status, english.get()))}</p>
            <Show when=move || state.with(|review| !review.issues.is_empty())>
                <ul class="import-warnings">{move || state.with(|review| review.issues.clone()).into_iter().map(|issue| view! { <li><code>{issue.path}</code>" · "<code>{issue.rule}</code></li> }).collect_view()}</ul>
            </Show>
            {move || run.get().map(|run| view! { <RunDetails run=run english=english /> })}
            <div class="adaptation-comparison">
                <section aria-labelledby="adaptation-source-title">
                    <h3 id="adaptation-source-title">{move || copy().original}</h3>
                    <Show when=move || source.get().is_none()><p class="help">{move || copy().source_missing}</p></Show>
                    {move || source.get().map(|source| view! { <SourceComparison source=source english=english /> })}
                </section>
                <section aria-labelledby="adaptation-proposal-title">
                    <h3 id="adaptation-proposal-title">{move || copy().proposal}</h3>
                    <p id="adaptation-proposal-help" class="help">{move || copy().proposal_help}</p>
                    <Show when=move || run.get().is_some_and(|run| run.proposal.is_some())>
                        <label for="adaptation-proposal-json">{move || copy().edit}</label>
                        <textarea id="adaptation-proposal-json" lang="vi-VN" spellcheck="false" aria-describedby="adaptation-proposal-help adaptation-accept-reason"
                            readonly=move || run.get().is_none_or(|run| !matches!(run.status, AdaptationStatus::Succeeded | AdaptationStatus::Accepted)) || state.with(|review| review.draft_actor != review.actor)
                            prop:value=move || state.with(|review| if review.draft_actor == review.actor { review.draft.clone() } else { String::new() })
                            on:compositionstart=move |_| { composing.set(true); state.update(|review| *review = review.begin_composition()); }
                            on:compositionend=move |event| { composing.set(false); state.update(|review| *review = review.edit_draft(event_target_value(&event))); }
                            on:input=move |event| if !composing.get_untracked() { state.update(|review| *review = review.edit_draft(event_target_value(&event))); } />
                        <label class="adaptation-confirm" for="adaptation-reviewed">
                            <input id="adaptation-reviewed" type="checkbox" disabled=blocked prop:checked=move || state.with(|review| review.reviewed_findings)
                                on:change=move |event| state.update(|review| *review = review.review_findings(event_target_checked(&event))) />
                            <span>{move || copy().review}</span>
                        </label>
                        <button class="primary" type="button" aria-disabled=accept_blocked aria-describedby="adaptation-accept-reason adaptation-accept-help" on:click=move |_| {
                            if accept_blocked() { return; }
                            match operation_id() {
                                Ok(operation) => if let Some((sending, intent)) = state.get_untracked().start_accept(operation) { state.set(sending); dispatch(state, api, intent); },
                                Err(error) => state.update(|review| *review = review.failed(&error)),
                            }
                        }>{move || copy().accept}</button>
                    </Show>
                    <p id="adaptation-accept-reason" class="help">{move || if accept_blocked() { copy().accept_required } else { "" }}</p>
                    <p id="adaptation-accept-help" class="help">{move || copy().accept_help}</p>
                </section>
            </div>
            {move || accepted.get().map(|revision| {
                let revision = StoredValue::new(revision);
                view! {
                    <section aria-labelledby="adaptation-accepted-title">
                        <h3 id="adaptation-accepted-title">{move || copy().accepted}</h3>
                        <p class="identity">{revision.with_value(|revision| format!("{} / {} · {} · {}", revision.script_id, revision.revision, revision.accepted_by, revision.accepted_at))}</p>
                        <p class="identity"><code>{revision.with_value(|revision| revision.export_digest.clone())}</code></p>
                        <pre tabindex="0" lang="vi-VN">{revision.with_value(|revision| revision.script_json.clone())}</pre>
                        <button type="button" aria-disabled=blocked on:click=move |_| {
                            let Some(reading) = state.get_untracked().start_accepted_read() else { return; };
                            let actor = reading.actor.clone(); let ticket = reading.ticket;
                            let (script_id, version) = revision.with_value(|revision| (revision.script_id.clone(), revision.revision));
                            state.set(reading);
                            spawn_local(async move {
                                let result = api.read_revision(&script_id, version).await;
                                let _ = state.try_update(|review| *review = match result {
                                    Ok(revision) => review.accepted_loaded(ticket, &actor, revision),
                                    Err(error) => review.read_failed(ticket, &actor, &error),
                                });
                            });
                        }>{move || copy().reopen}</button>
                    </section>
                }
            })}
        </section>
    }
}

#[component]
fn ProviderDetails(metadata: AdaptationProviderMetadata, english: RwSignal<bool>) -> impl IntoView {
    let metadata = StoredValue::new(metadata);
    let copy = move || messages::adaptation_copy(english.get());
    view! {
        <dl class="source-identity">
            <dt>{move || copy().provider}</dt><dd>{metadata.with_value(|metadata| metadata.provider.clone())}</dd>
            <dt>{move || copy().model}</dt><dd>{metadata.with_value(|metadata| metadata.model.clone())}</dd>
            <dt>{move || messages::adaptation_local_model_label(english.get())}</dt><dd>
                {metadata.with_value(|metadata| metadata.local_model_digest.clone()).map(|digest| view! { <code>{digest}</code> })}
                <p class="help">{move || metadata.with_value(|metadata| messages::adaptation_local_model_evidence(metadata.local_model_digest.is_some(), english.get()))}</p>
            </dd>
            <dt>{move || copy().endpoint}</dt><dd>{metadata.with_value(|metadata| metadata.endpoint.clone())}</dd>
            <dt>{move || copy().prompt}</dt><dd>{metadata.with_value(|metadata| format!("{} / {}", metadata.prompt_version, metadata.contract_version))}</dd>
            <dt>{move || copy().config}</dt><dd><code>{metadata.with_value(|metadata| serde_json::to_string(&metadata.config).unwrap_or_default())}</code></dd>
        </dl>
    }
}

#[component]
fn RunDetails(run: Arc<AdaptationRunResponse>, english: RwSignal<bool>) -> impl IntoView {
    let run = StoredValue::new(run);
    let copy = move || messages::adaptation_copy(english.get());
    view! {
        <section aria-labelledby="adaptation-provenance-title">
            <h3 id="adaptation-provenance-title">{move || copy().provenance}</h3>
            <p class="status" role="status" aria-live="polite">{move || run.with_value(|run| messages::adaptation_run_status(run.status, english.get()))}</p>
            <dl class="source-identity">
                <dt>{move || copy().run_id}</dt><dd>{run.with_value(|run| run.id.clone())}</dd>
                <dt>{move || copy().operation}</dt><dd>{run.with_value(|run| run.request.operation_id.clone())}</dd>
                <dt>{move || copy().source_id}</dt><dd>{run.with_value(|run| run.request.source_id.clone())}</dd>
                <dt>{move || copy().checksum}</dt><dd><code>{run.with_value(|run| run.source_sha256.clone())}</code></dd>
                <dt>{move || copy().extraction}</dt><dd>{run.with_value(|run| run.extractor_version.clone())}</dd>
                <dt>{move || copy().script_id}</dt><dd>{run.with_value(|run| run.request.script_id.clone())}</dd>
                <dt>{move || copy().base}</dt><dd>{run.with_value(|run| run.request.expected_revision)}</dd>
                <dt>{move || copy().input_digest}</dt><dd>{move || run.with_value(|run| format!("{} / {}", run.input_content_digest.as_deref().unwrap_or(copy().unknown), run.input_export_digest.as_deref().unwrap_or(copy().unknown)))}</dd>
                <dt>{move || copy().records}</dt><dd>{run.with_value(|run| format!("{} / {}", run.generation_record_id, run.rights_record_id))}</dd>
                <dt>{move || copy().cost}</dt><dd>{move || run.with_value(|run| messages::adaptation_cost(run.cost.as_ref(), english.get()))}</dd>
                <dt>{move || copy().usage}</dt><dd>{move || run.with_value(|run| format!("{} / {}", run.usage.as_ref().and_then(|usage| usage.input_tokens).map(|value| value.to_string()).unwrap_or_else(|| copy().unknown.into()), run.usage.as_ref().and_then(|usage| usage.output_tokens).map(|value| value.to_string()).unwrap_or_else(|| copy().unknown.into())))}</dd>
            </dl>
            <ProviderDetails metadata=run.with_value(|run| run.provider.clone()) english=english />
            {run.with_value(|run| run.problem.clone()).map(|problem| view! {
                <h4>{move || copy().problem}</h4><p>{move || messages::adaptation_problem(&problem.code, english.get())}</p>
                <ul>{problem.issues.into_iter().map(|issue| view! { <li><code>{issue.path}</code>" · "<code>{issue.rule}</code></li> }).collect_view()}</ul>
            })}
            {run.with_value(|run| run.proposal.clone()).map(|proposal| {
                let no_findings = proposal.findings.is_empty();
                view! {
                    <h4>{move || copy().findings}</h4>
                    <Show when=move || no_findings><p>{move || copy().no_findings}</p></Show>
                    <ul class="import-warnings">{proposal.findings.into_iter().map(|finding| {
                        let finding = StoredValue::new(finding);
                        view! { <li><strong>{move || finding.with_value(|finding| messages::adaptation_finding(finding.code, english.get()))}</strong>
                            {finding.with_value(|finding| finding.block).map(|block| view! { <p>{move || copy().block}" "{block}</p> })}
                            <p>{move || finding.with_value(|finding| messages::adaptation_finding_explanation(finding.code, english.get()))}</p>
                            <Show when=move || finding.with_value(|finding| finding.code == AdaptationFindingCode::SourceWarning)>
                                <p>{move || finding.with_value(|finding| messages::import_diagnostic(&finding.detail, english.get()))}</p>
                                <small><code>{finding.with_value(|finding| finding.detail.clone())}</code></small>
                            </Show>
                            <Show when=move || finding.with_value(|finding| messages::adaptation_has_provider_detail(finding.code))>
                                <p class="help">{move || messages::adaptation_provider_detail(english.get())}</p>
                                <blockquote lang="vi-VN">{finding.with_value(|finding| finding.detail.clone())}</blockquote>
                            </Show>
                        </li> }
                    }).collect_view()}</ul>
                    <details><summary>{move || copy().coverage}</summary>
                        <p class="help">{move || copy().coverage_help}</p>
                        <ul class="adaptation-coverage">{proposal.coverage.into_iter().map(|coverage| {
                            let coverage = StoredValue::new(coverage);
                            view! { <li><strong>{move || copy().block}" "{coverage.with_value(|coverage| coverage.block)}</strong>" · "{move || coverage.with_value(|coverage| match coverage.disposition { AdaptationCoverageDisposition::Represented => copy().represented, AdaptationCoverageDisposition::Omitted => copy().omitted })}
                                <p class="identity"><code>{coverage.with_value(|coverage| coverage.dialogue_ids.join(", "))}</code></p>
                                {coverage.with_value(|coverage| coverage.reason.clone()).map(|reason| view! { <p lang="vi-VN">{reason}</p> })}
                            </li> }
                        }).collect_view()}</ul>
                    </details>
                }
            })}
        </section>
    }
}

#[component]
fn SourceComparison(source: Arc<ImportResponse>, english: RwSignal<bool>) -> impl IntoView {
    let api = expect_context::<StudioContext>().api;
    let download = api.original_import_path(&source.id).ok();
    let source = StoredValue::new(source);
    let copy = move || messages::adaptation_copy(english.get());
    let import_copy = move || messages::import_copy(english.get());
    view! {
        <dl class="source-identity">
            <dt>{move || import_copy().reference}</dt><dd>{source.with_value(|source| source.metadata.reference.clone())}</dd>
            <dt>{move || import_copy().rights_holder}</dt><dd>{move || source.with_value(|source| source.metadata.rights_holder.clone().unwrap_or_else(|| import_copy().absent.into()))}</dd>
            <dt>{move || import_copy().permission_evidence}</dt><dd>{move || source.with_value(|source| source.metadata.permission_evidence.clone().unwrap_or_else(|| import_copy().absent.into()))}</dd>
            <dt>{move || import_copy().usage_scope}</dt><dd>{move || source.with_value(|source| source.metadata.usage_scope.clone().unwrap_or_else(|| import_copy().absent.into()))}</dd>
            <dt>{move || copy().checksum}</dt><dd><code>{source.with_value(|source| source.sha256.clone())}</code></dd>
        </dl>
        {source.with_value(|source| source.original_text.clone()).map(|text| view! { <pre tabindex="0" lang="vi-VN">{text}</pre> })}
        <Show when=move || source.with_value(|source| source.original_text.is_none())><p>{move || copy().original_binary}</p></Show>
        {download.map(|path| view! { <a class="download-original" href=path download=source.with_value(|source| source.metadata.file_name.clone())>{move || copy().download}</a> })}
        {match source.with_value(|source| source.outcome.clone()) {
            ImportOutcome::Parsed { extraction } => view! { <ExtractionPreview extraction=extraction english=english namespace="adaptation" /> }.into_any(),
            ImportOutcome::Failed { error } => view! { <p>{move || messages::import_diagnostic(&error.code, english.get())}</p> }.into_any(),
        }}
    }
}
