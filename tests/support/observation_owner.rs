// SPDX-License-Identifier: AGPL-3.0-or-later
//! Shared strict QObs owner fixtures for protocol integration tests.

use quire_observation::authority::{
    self, AuthoritySelection, Context, History, Limits as ObservationLimits, OpenClosed,
    SubjectSelection, TemporalBoundary,
};
use quire_observation::{
    admit, AdmissionOutcome, AdmissionRequest, AdmittedRecord, Anchor, ClockRange, Digest,
    Identity, Member, ObservationBinding, PackageSelection, QualifiedObservation, QualifiedSubject,
    ResourceLimits, ScopeKind, ScopeSelection, SubjectIdentity, SubjectKind, ValueState,
    Visibility, NATIVE_LINKED_PACKAGE_FORMAT,
};

#[path = "producer_owner.rs"]
mod producer_owner;

pub struct ScopeViews {
    pub population: authority::population::View,
    pub observation: authority::observation::View,
    pub progress_open: authority::progress::View,
    pub progress_closed: authority::progress::View,
    pub closure_open: authority::closure::View,
    pub closure_closed: authority::closure::View,
    pub completeness_complete: authority::completeness::View,
    pub completeness_incomplete: authority::completeness::View,
    pub completeness_contradicted: authority::completeness::View,
}

fn id(value: impl Into<String>) -> Identity {
    Identity::new(value)
}

fn digest(value: u8) -> Digest {
    Digest::new([value; 32])
}

pub fn scope_views(tag: &str) -> ScopeViews {
    scope_views_with_value(tag, true)
}

pub fn scope_views_with_value(tag: &str, valuation_present: bool) -> ScopeViews {
    let qualified = qualified(tag, valuation_present);
    let owner = AuthoritySelection {
        definition_identity: id(format!("definition:{tag}")),
        definition_revision: id("1"),
        definition_digest: digest(9),
    };
    let subject = SubjectSelection {
        scope_identity: id(format!("window:{tag}")),
        population_identity: qualified.scope().population_identity.clone(),
    };
    let context = Context::new(History::batch(&qualified), &owner, &subject, 1, None);
    let population = population_view(context);
    let observation = observation_view(context, &qualified, tag);
    let progress_open = progress_view(context, OpenClosed::Open, tag);
    let progress_closed = progress_view(context, OpenClosed::Closed, tag);
    let closure_open = closure_view(context, OpenClosed::Open, tag);
    let closure_closed = closure_view(context, OpenClosed::Closed, tag);
    let completeness_complete = completeness_view(
        context,
        &qualified,
        authority::completeness::FactStatus::Available,
        true,
        tag,
    );
    let completeness_incomplete = completeness_view(
        context,
        &qualified,
        authority::completeness::FactStatus::Incomplete,
        false,
        tag,
    );
    let completeness_contradicted = completeness_view(
        context,
        &qualified,
        authority::completeness::FactStatus::Contradicted,
        true,
        tag,
    );
    ScopeViews {
        population,
        observation,
        progress_open,
        progress_closed,
        closure_open,
        closure_closed,
        completeness_complete,
        completeness_incomplete,
        completeness_contradicted,
    }
}

fn population_view(context: Context<'_>) -> authority::population::View {
    let document = authority::population::derive(context, ObservationLimits::owner_max())
        .expect("population document");
    authority::population::read(document.bytes(), context, ObservationLimits::owner_max())
        .expect("population view")
}

fn qualified(tag: &str, valuation_present: bool) -> Box<QualifiedObservation> {
    let producer = producer_owner::admitted(tag);
    let subject_kind =
        SubjectKind::new(producer_owner::SUBJECT_KIND).expect("fixture subject kind");
    let subject = QualifiedSubject::new(
        &producer,
        subject_kind.clone(),
        SubjectIdentity::new(format!("order:{tag}").into_bytes()).expect("fixture subject"),
    );
    let mut request = AdmissionRequest {
        package: PackageSelection {
            format: NATIVE_LINKED_PACKAGE_FORMAT.to_owned(),
            identity: id(format!("package:{tag}")),
            revision: id("1"),
            digest: digest(1),
        },
        binding: ObservationBinding {
            identity: id(format!("binding:{tag}")),
            source_identity: id(format!("source:{tag}")),
            schema_identity: id(format!("schema:{tag}")),
            signal_identity: id(format!("signal:{tag}")),
            trigger_identity: id(format!("trigger:{tag}")),
            unit: id("unit"),
            subject_kind,
            required: valuation_present,
        },
        expected_subject: subject.clone(),
        relationships: vec![],
        required_relationships: vec![],
        scope: ScopeSelection {
            population_identity: id("unsealed-population"),
            membership_rule_identity: id(format!("membership:{tag}")),
            membership_digest: digest(4),
            membership_document: vec![],
            required_member_identities: vec![id(format!("member:{tag}"))],
            observation_sources: vec![id(format!("source:{tag}"))],
            completeness_dependencies: vec![id(format!("completeness:{tag}"))],
            progress_dependencies: vec![id(format!("progress:{tag}"))],
            clock_identity: id(format!("clock:{tag}")),
            clock_revision: id("1"),
            membership_complete: true,
            closure_identity: Some(id(format!("closure:{tag}"))),
            closure_digest: Some(digest(5)),
            kind: ScopeKind::Window {
                window_identity: id(format!("window:{tag}")),
            },
            range: ClockRange::Timestamp {
                start_nanos: 0,
                end_nanos: 30,
            },
            members: vec![Member {
                object_identity: id(format!("member:{tag}")),
                record_identity: id("unsealed-record"),
                anchor: Anchor::TimestampNanos(10),
            }],
        },
        records: vec![AdmittedRecord {
            identity: id("unsealed-record"),
            binding_identity: id(format!("binding:{tag}")),
            source_identity: id(format!("source:{tag}")),
            schema_identity: id(format!("schema:{tag}")),
            subject,
            signal_identity: id(format!("signal:{tag}")),
            trigger_identity: id(format!("trigger:{tag}")),
            unit: id("unit"),
            value: if valuation_present {
                ValueState::Present {
                    value_type: id("text"),
                    canonical_value: tag.to_owned(),
                }
            } else {
                ValueState::Missing
            },
            visibility: Visibility::External,
            anchor: Anchor::TimestampNanos(10),
            event_time_nanos: 10,
            ingestion_time_nanos: 11,
            causal_relationship_identity: None,
            clock_identity: id(format!("clock:{tag}")),
            clock_revision: id("1"),
            clock_uncertainty_nanos: 1,
        }],
        limits: ResourceLimits {
            max_records: 1,
            max_members: 1,
            max_relationships: 0,
            max_required_relationships: 0,
        },
        producer,
    };
    let record_identity = authority::observation::record_identity(
        &request.records[0],
        ObservationLimits::owner_max(),
    )
    .expect("record identity");
    request.records[0].identity = record_identity.clone();
    request.scope.members[0].record_identity = record_identity;
    authority::population::assign_request_identities(&mut request, ObservationLimits::owner_max())
        .expect("population identities");
    match admit(request) {
        AdmissionOutcome::Available { observation } => observation,
        other => panic!("qualification failed: {other:?}"),
    }
}

fn boundary(state: OpenClosed) -> TemporalBoundary {
    TemporalBoundary::TimestampedEvent {
        lower_nanos: 0,
        upper_inclusive_nanos: 29,
        carrier_end_exclusive_nanos: 30,
        watermark_nanos: if state == OpenClosed::Closed { 30 } else { 20 },
    }
}

fn cutoff(tag: &str) -> authority::observation::CutoffSelection {
    authority::observation::CutoffSelection::new(
        id(format!("clock:{tag}")),
        id("1"),
        40,
        authority::observation::CutoffRule::IngestionTimeAtOrBefore,
        id("1"),
    )
}

fn observation_view(
    context: Context<'_>,
    qualified: &QualifiedObservation,
    tag: &str,
) -> authority::observation::View {
    let selection = authority::observation::Selection::new(
        qualified.records()[0].identity.clone(),
        cutoff(tag),
    );
    let document =
        authority::observation::derive(context, &selection, ObservationLimits::owner_max())
            .expect("observation document");
    authority::observation::read(
        document.bytes(),
        context,
        &selection,
        ObservationLimits::owner_max(),
    )
    .expect("observation view")
}

fn progress_selection(state: OpenClosed, tag: &str) -> authority::progress::Selection {
    authority::progress::Selection::new(
        authority::clock::Selection::new(id(format!("clock:{tag}")), id("1")),
        vec![id(format!("source:{tag}"))],
        boundary(state),
        state,
        id(format!("trigger:{tag}")),
        cutoff(tag),
        id(format!("restoration:{tag}")),
    )
}

fn progress_view(context: Context<'_>, state: OpenClosed, tag: &str) -> authority::progress::View {
    let selection = progress_selection(state, tag);
    let document = authority::progress::derive(context, &selection, ObservationLimits::owner_max())
        .expect("progress document");
    authority::progress::read(
        document.bytes(),
        context,
        &selection,
        ObservationLimits::owner_max(),
    )
    .expect("progress view")
}

fn closure_selection(state: OpenClosed, tag: &str) -> authority::closure::Selection {
    authority::closure::Selection::new(
        id(format!("clock:{tag}")),
        id("1"),
        vec![id(format!("source:{tag}"))],
        boundary(state),
        state,
    )
}

fn closure_view(context: Context<'_>, state: OpenClosed, tag: &str) -> authority::closure::View {
    let selection = closure_selection(state, tag);
    let document = authority::closure::derive(context, &selection, ObservationLimits::owner_max())
        .expect("closure document");
    authority::closure::read(
        document.bytes(),
        context,
        &selection,
        ObservationLimits::owner_max(),
    )
    .expect("closure view")
}

fn completeness_view(
    context: Context<'_>,
    qualified: &QualifiedObservation,
    status: authority::completeness::FactStatus,
    observed: bool,
    tag: &str,
) -> authority::completeness::View {
    let selection = authority::completeness::Selection::new(
        id(format!("boundary:{tag}")),
        vec![authority::completeness::Fact::new(
            id(format!("member:{tag}")),
            observed.then(|| qualified.records()[0].identity.clone()),
            status,
        )],
    );
    let document =
        authority::completeness::derive(context, &selection, ObservationLimits::owner_max())
            .expect("completeness document");
    authority::completeness::read(
        document.bytes(),
        context,
        &selection,
        ObservationLimits::owner_max(),
    )
    .expect("completeness view")
}
