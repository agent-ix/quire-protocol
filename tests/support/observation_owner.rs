// SPDX-License-Identifier: AGPL-3.0-or-later
//! Shared strict QObs owner fixtures for protocol integration tests.

use agent_ix_baseline_producer::StaticProducerBundle;
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

const ORDER_KIND: &str = "ix://agent-ix/commerce/type/Order";

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

pub struct OrderingViews {
    pub subject: QualifiedSubject,
    pub observations: Vec<authority::observation::View>,
    pub positions: authority::position::View,
}

fn id(value: impl Into<String>) -> Identity {
    Identity::new(value)
}

fn digest(value: u8) -> Digest {
    Digest::new([value; 32])
}

fn producer() -> quire_observation::AdmittedStaticBundle {
    StaticProducerBundle::admit_json(include_bytes!("../fixtures/fcd-static-bundle-1.2.json"))
        .expect("shared FCD static bundle is admitted")
}

/// Admit the exact static Producer bundle shared by integration fixtures.
pub fn admitted_producer() -> quire_observation::AdmittedStaticBundle {
    producer()
}

fn subject(producer: &quire_observation::AdmittedStaticBundle, tag: &str) -> QualifiedSubject {
    QualifiedSubject::new(
        producer,
        SubjectKind::new(ORDER_KIND.as_bytes().to_vec()).expect("non-empty order kind"),
        SubjectIdentity::new(format!("order:{tag}").into_bytes())
            .expect("non-empty order identity"),
    )
}

/// Construct one complete authority-qualified QObs runtime subject.
pub fn qualified_subject(
    producer: &quire_observation::AdmittedStaticBundle,
    tag: &str,
) -> QualifiedSubject {
    subject(producer, tag)
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

pub fn ordering_views(tag: &str, variant: &str, order: [usize; 3]) -> OrderingViews {
    let producer = producer();
    let subject = subject(&producer, tag);
    let mut request = AdmissionRequest {
        package: PackageSelection {
            format: NATIVE_LINKED_PACKAGE_FORMAT.to_owned(),
            identity: id(format!("package:{tag}:ordering:{variant}")),
            revision: id("1"),
            digest: digest(7),
        },
        producer,
        binding: ObservationBinding {
            identity: id(format!("binding:{tag}:ordering")),
            source_identity: id(format!("source:{tag}:ordering")),
            schema_identity: id("schema:ordering/v1"),
            signal_identity: id("signal:ordering"),
            trigger_identity: id("trigger:ordering"),
            unit: id("position"),
            subject_kind: SubjectKind::new(ORDER_KIND.as_bytes().to_vec())
                .expect("non-empty order kind"),
            required: true,
        },
        expected_subject: subject.clone(),
        relationships: vec![],
        required_relationships: vec![],
        scope: ScopeSelection {
            population_identity: id("unsealed-ordering-population"),
            membership_rule_identity: id("unsealed-ordering-membership"),
            membership_digest: digest(0),
            membership_document: vec![],
            required_member_identities: (0..3)
                .map(|index| id(format!("member:{tag}:{variant}:{index}")))
                .collect(),
            observation_sources: vec![id(format!("source:{tag}:ordering"))],
            completeness_dependencies: vec![id(format!("completeness:{tag}:ordering"))],
            progress_dependencies: vec![id(format!("progress:{tag}:ordering"))],
            clock_identity: id(format!("clock:{tag}:ordering")),
            clock_revision: id("1"),
            membership_complete: true,
            closure_identity: Some(id(format!("closure:{tag}:ordering"))),
            closure_digest: Some(digest(8)),
            kind: ScopeKind::Window {
                window_identity: id(format!("window:{tag}:ordering")),
            },
            range: ClockRange::Timestamp {
                start_nanos: 0,
                end_nanos: 40,
            },
            members: (0..3)
                .map(|index| Member {
                    object_identity: id(format!("member:{tag}:{variant}:{index}")),
                    record_identity: id(format!("unsealed-ordering-record:{index}")),
                    anchor: Anchor::TimestampNanos(10 + i128::from(index)),
                })
                .collect(),
        },
        records: (0..3)
            .map(|index| AdmittedRecord {
                identity: id(format!("unsealed-ordering-record:{index}")),
                binding_identity: id(format!("binding:{tag}:ordering")),
                source_identity: id(format!("source:{tag}:ordering")),
                schema_identity: id("schema:ordering/v1"),
                subject: subject.clone(),
                signal_identity: id("signal:ordering"),
                trigger_identity: id("trigger:ordering"),
                unit: id("position"),
                value: ValueState::Present {
                    value_type: id("text"),
                    canonical_value: format!("{tag}:{variant}:{index}"),
                },
                visibility: Visibility::External,
                anchor: Anchor::TimestampNanos(10 + i128::from(index)),
                event_time_nanos: 10 + i128::from(index),
                ingestion_time_nanos: 20 + i128::from(index),
                causal_relationship_identity: None,
                clock_identity: id(format!("clock:{tag}:ordering")),
                clock_revision: id("1"),
                clock_uncertainty_nanos: 1,
            })
            .collect(),
        limits: ResourceLimits {
            max_records: 3,
            max_members: 3,
            max_relationships: 0,
            max_required_relationships: 0,
        },
    };
    for (record, member) in request.records.iter_mut().zip(&mut request.scope.members) {
        let identity =
            authority::observation::record_identity(record, ObservationLimits::owner_max())
                .expect("ordering record identity");
        record.identity = identity.clone();
        member.record_identity = identity;
    }
    authority::population::assign_request_identities(&mut request, ObservationLimits::owner_max())
        .expect("ordering population identities");
    let qualified = match admit(request) {
        AdmissionOutcome::Available { observation } => observation,
        other => panic!("ordering qualification failed: {other:?}"),
    };
    let owner = AuthoritySelection {
        definition_identity: id(format!("definition:{tag}:ordering")),
        definition_revision: id("1"),
        definition_digest: digest(9),
    };
    let selected_subject = SubjectSelection {
        scope_identity: id(format!("window:{tag}:ordering")),
        population_identity: qualified.scope().population_identity.clone(),
    };
    let context = Context::new(
        History::batch(&qualified),
        &owner,
        &selected_subject,
        1,
        None,
    );
    let observations = qualified
        .records()
        .iter()
        .map(|record| {
            let selection = authority::observation::Selection::new(
                record.identity.clone(),
                authority::observation::CutoffSelection::new(
                    id(format!("clock:{tag}:ordering")),
                    id("1"),
                    40,
                    authority::observation::CutoffRule::IngestionTimeAtOrBefore,
                    id("1"),
                ),
            );
            let document =
                authority::observation::derive(context, &selection, ObservationLimits::owner_max())
                    .expect("ordering observation document");
            authority::observation::read(
                document.bytes(),
                context,
                &selection,
                ObservationLimits::owner_max(),
            )
            .expect("ordering observation view")
        })
        .collect();
    let position_selection = authority::position::Selection::new(
        id(format!("ledger:{tag}:{variant}")),
        id(format!("clock:{tag}:ordering")),
        id("1"),
        order
            .into_iter()
            .enumerate()
            .map(|(position, record)| {
                authority::position::Position::new(
                    u64::try_from(position).expect("three positions"),
                    qualified.records()[record].identity.clone(),
                )
            })
            .collect(),
    );
    let document =
        authority::position::derive(context, &position_selection, ObservationLimits::owner_max())
            .expect("ordering position document");
    let positions = authority::position::read(
        document.bytes(),
        context,
        &position_selection,
        ObservationLimits::owner_max(),
    )
    .expect("ordering position view");
    OrderingViews {
        subject,
        observations,
        positions,
    }
}

fn population_view(context: Context<'_>) -> authority::population::View {
    let document = authority::population::derive(context, ObservationLimits::owner_max())
        .expect("population document");
    authority::population::read(document.bytes(), context, ObservationLimits::owner_max())
        .expect("population view")
}

fn qualified(tag: &str, valuation_present: bool) -> Box<QualifiedObservation> {
    let producer = producer();
    let subject = subject(&producer, tag);
    let mut request = AdmissionRequest {
        package: PackageSelection {
            format: NATIVE_LINKED_PACKAGE_FORMAT.to_owned(),
            identity: id(format!("package:{tag}")),
            revision: id("1"),
            digest: digest(1),
        },
        producer,
        binding: ObservationBinding {
            identity: id(format!("binding:{tag}")),
            source_identity: id(format!("source:{tag}")),
            schema_identity: id(format!("schema:{tag}")),
            signal_identity: id(format!("signal:{tag}")),
            trigger_identity: id(format!("trigger:{tag}")),
            unit: id("unit"),
            subject_kind: SubjectKind::new(ORDER_KIND.as_bytes().to_vec())
                .expect("non-empty order kind"),
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
