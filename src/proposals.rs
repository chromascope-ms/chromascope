//! Human/agent proposals are immutable suggestions until explicitly resolved.
//! All numerical/review changes still execute through the shared engine.
use crate::{
    domain::{EngineError, Operation, Request, Result},
    engine::{Output, Response},
    jobs::JobControl,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Event {
    pub actor: String,
    pub reason: String,
    pub decision: String,
    pub unix_ms: u64,
}
/// The start, end and area of one integrated peak.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PeakBounds {
    pub start: Option<f64>,
    pub end: Option<f64>,
    pub area: Option<f64>,
}
/// New peak boundaries for one result of a batch quantification (a row of the Quantification
/// and QC table). The result lives in the window rather than in an engine response, so the
/// window applies the change when a person approves it, exactly as it applies a manual
/// boundary edit.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PeakEdit {
    /// Who made the proposal (the MCP server, on behalf of an AI client).
    pub proposer: String,
    /// `sha256:` plus the SHA-256 of the dataset's source file, when the window knew it.
    pub dataset_stable_id: Option<String>,
    /// Position in the results of the batch when the proposal was made.
    pub result_index: usize,
    pub sample: String,
    pub analyte: String,
    pub original: PeakBounds,
    pub proposed: PeakBounds,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Proposal {
    pub version: u32,
    pub id: uuid::Uuid,
    /// The engine operation approval runs. Absent for a peak-boundary proposal (see
    /// [`PeakEdit`]), which the window applies itself.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request: Option<Request>,
    /// Set for a proposal of new peak boundaries; exactly one of `request` and `peak_edit`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub peak_edit: Option<PeakEdit>,
    pub expected_project_revision: Option<u64>,
    pub before: Value,
    pub before_sha256: String,
    pub proposed: Value,
    pub reason: String,
    pub evidence: Vec<String>,
    pub events: Vec<Event>,
    pub applied: Option<Value>,
}
fn invalid(message: impl ToString) -> EngineError {
    EngineError::new("invalid_parameters", message)
}
impl Proposal {
    /// Exactly one of the engine request and the peak edit is present.
    pub fn check_shape(&self) -> Result<()> {
        match (&self.request, &self.peak_edit) {
            (Some(_), None) | (None, Some(_)) => Ok(()),
            _ => Err(invalid(
                "A proposal carries either an engine request or a peak edit",
            )),
        }
    }
    /// Who made the proposal.
    pub fn proposer(&self) -> &str {
        match (&self.request, &self.peak_edit) {
            (Some(request), _) => &request.actor,
            (None, Some(edit)) => &edit.proposer,
            _ => "unknown",
        }
    }
    /// The engine request, or the reason this proposal has none.
    pub fn engine_request(&self) -> Result<&Request> {
        self.request.as_ref().ok_or_else(|| {
            invalid("This proposal changes peak boundaries in a window; approve it in the window that holds the batch")
        })
    }
}
pub fn same_subject(value: &Value, before: &Value) -> bool {
    for key in ["batch_id", "report_id", "id"] {
        if before[key].is_string() {
            return value[key] == before[key];
        }
    }
    if before["raw"]["id"].is_string() {
        return value["raw"]["id"] == before["raw"]["id"];
    }
    if before["matrix"].is_object() && before["hypotheses"].is_array() {
        return value["matrix"] == before["matrix"] && value["hypotheses"].is_array();
    }
    false
}
pub fn find_current(value: &Value, before: &Value) -> Option<Value> {
    let mut found = vec![];
    fn visit(value: &Value, before: &Value, found: &mut Vec<Value>) {
        if value.is_object() && same_subject(value, before) {
            found.push(value.clone());
            return;
        }
        match value {
            Value::Array(values) => {
                for value in values {
                    visit(value, before, found)
                }
            }
            Value::Object(map) => {
                for value in map.values() {
                    visit(value, before, found)
                }
            }
            _ => {}
        }
    }
    visit(value, before, &mut found);
    found.into_iter().max_by_key(|value| {
        ["reviews", "revisions", "annotations"]
            .iter()
            .map(|key| value[*key].as_array().map_or(0, Vec::len))
            .sum::<usize>()
    })
}
pub fn before(operation: &Operation) -> Result<Value> {
    let value = match operation {
        Operation::ReviseChromatogram { analysis, .. } => serde_json::to_value(analysis),
        Operation::ReviewTargeted { batch, .. } => serde_json::to_value(batch),
        Operation::ReviewQc { report, .. } => serde_json::to_value(report),
        Operation::AnnotateSpectrum { report, .. } => serde_json::to_value(report),
        Operation::ReviewFeatureAnnotation { ledger, .. } => serde_json::to_value(ledger),
        _ => {
            return Err(invalid(
                "A review proposal must use a supported revision-bound review operation",
            ))
        }
    };
    value.map_err(invalid)
}
/// SHA-256 of a value's JSON text, as stored in `before_sha256`.
pub fn hash_value(value: &Value) -> Result<String> {
    hash(value)
}
fn check_reason(reason: &str, evidence: &[String]) -> Result<()> {
    if reason.trim().is_empty()
        || evidence.is_empty()
        || evidence.iter().any(|x| x.trim().is_empty())
    {
        return Err(invalid(
            "Proposal reason and evidence references are required",
        ));
    }
    Ok(())
}
/// A proposal of new peak boundaries for a batch quantification result. `before` is the
/// result as the window holds it (see the window's snapshot of a result) and `proposed` the
/// same with the new peak; approval is refused unless the result still equals `before`.
pub fn peak_edit_proposal(
    edit: PeakEdit,
    before: Value,
    proposed: Value,
    reason: String,
    evidence: Vec<String>,
) -> Result<Proposal> {
    check_reason(&reason, &evidence)?;
    if edit
        .proposed
        .start
        .zip(edit.proposed.end)
        .is_none_or(|(a, z)| !(a.is_finite() && z.is_finite() && a < z))
    {
        return Err(invalid("A proposal needs finite, increasing boundaries"));
    }
    Ok(Proposal {
        version: 1,
        id: uuid::Uuid::new_v4(),
        request: None,
        peak_edit: Some(edit),
        expected_project_revision: None,
        before_sha256: hash(&before)?,
        before,
        proposed,
        reason,
        evidence,
        events: vec![],
        applied: None,
    })
}
/// Records a person's decision on a peak-boundary proposal. Returns the updated proposal; the
/// caller applies the boundaries on approval. Refused when the result is not what it was when
/// the proposal was made, when the deciding actor is the proposer, or when a decision exists.
pub fn resolve_peak_edit(
    proposal: &Proposal,
    current: &Value,
    approve: bool,
    actor: &str,
    reason: &str,
) -> Result<Proposal> {
    let edit = proposal
        .peak_edit
        .as_ref()
        .ok_or_else(|| invalid("Not a peak-boundary proposal"))?;
    if proposal.version != 1 || actor.trim().is_empty() || reason.trim().is_empty() {
        return Err(invalid("Version 1, review actor and reason are required"));
    }
    if actor == edit.proposer {
        return Err(invalid("A proposal cannot be decided by its proposer"));
    }
    if !proposal.events.is_empty() {
        return Err(invalid(
            "Resolve a pending proposal once; prepare a new revision to change the decision",
        ));
    }
    if current != &proposal.before || hash(current)? != proposal.before_sha256 {
        return Err(EngineError::new(
            "stale_revision",
            "The result changed after the proposal was made; ask for a new proposal",
        ));
    }
    let mut updated = proposal.clone();
    if approve {
        updated.applied = Some(serde_json::json!({
            "start": edit.proposed.start,
            "end": edit.proposed.end,
            "area": edit.proposed.area,
            "approved_by": actor,
        }));
    }
    updated.events.push(Event {
        actor: actor.into(),
        reason: reason.into(),
        decision: if approve { "approved" } else { "rejected" }.into(),
        unix_ms: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64,
    });
    Ok(updated)
}
fn hash(value: &Value) -> Result<String> {
    Ok(format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(value).map_err(invalid)?)
    ))
}
pub fn reviewed_value(output: &Output) -> Result<Value> {
    match output {
        Output::TargetedQuantification { batch } => serde_json::to_value(batch),
        Output::QcReport { report } => serde_json::to_value(report),
        Output::SpectralSearch { report } => serde_json::to_value(report),
        Output::FeatureAnnotations { ledger } => serde_json::to_value(ledger),
        Output::ChromatographicProcessing { analyses, .. } => serde_json::to_value(
            analyses
                .first()
                .ok_or_else(|| invalid("Review output is empty"))?,
        ),
        _ => return Err(invalid("Not a supported reviewed result")),
    }
    .map_err(invalid)
}
fn semantic_review(value: &Value, original: &Value) -> Value {
    let mut value = value.clone();
    for key in ["reviews", "revisions", "annotations"] {
        let old = original[key].as_array().map_or(0, Vec::len);
        if let Some(events) = value[key].as_array_mut() {
            for event in events.iter_mut().skip(old) {
                if let Some(event) = event.as_object_mut() {
                    for field in ["id", "unix_ms", "actor"] {
                        event.remove(field);
                    }
                }
            }
        }
    }
    value
}
pub fn prepare(
    path: &std::path::Path,
    request: Request,
    revision: Option<u64>,
    reason: String,
    evidence: Vec<String>,
    control: &JobControl,
) -> Result<Proposal> {
    request.operation.ensure_available()?;
    check_reason(&reason, &evidence)?;
    if matches!(
        request.operation,
        Operation::ReviseChromatogram { preview: true, .. }
    ) {
        return Err(invalid(
            "A proposal must request a committed review; preparation itself does not apply it",
        ));
    }
    let original = before(&request.operation)?;
    let response = crate::engine::execute(path, request.clone(), control)?;
    Ok(Proposal {
        version: 1,
        id: uuid::Uuid::new_v4(),
        request: Some(request),
        peak_edit: None,
        expected_project_revision: revision,
        before_sha256: hash(&original)?,
        before: original,
        proposed: reviewed_value(&response.output)?,
        reason,
        evidence,
        events: vec![],
        applied: None,
    })
}
#[expect(
    clippy::too_many_arguments,
    reason = "Preserve the explicit revision-bound review API: source, evidence, revision, attributed decision and cancellation"
)]
pub fn resolve(
    path: &std::path::Path,
    proposal: &Proposal,
    current: &Value,
    revision: Option<u64>,
    approve: bool,
    actor: &str,
    reason: &str,
    control: &JobControl,
) -> Result<(Proposal, Option<Response>)> {
    let request = proposal.engine_request()?;
    request.operation.ensure_available()?;
    if proposal.version != 1 || actor.trim().is_empty() || reason.trim().is_empty() {
        return Err(invalid("Version 1, review actor and reason are required"));
    }
    if !proposal.events.is_empty() {
        return Err(invalid(
            "Resolve a pending proposal once; prepare a new revision to change the decision",
        ));
    }
    if proposal.expected_project_revision != revision
        || current != &proposal.before
        || hash(&before(&request.operation)?)? != proposal.before_sha256
        || hash(current)? != proposal.before_sha256
    {
        return Err(EngineError::new(
            "stale_revision",
            "Proposal evidence or project revision changed; prepare a new suggestion",
        ));
    }
    let mut updated = proposal.clone();
    let response = if approve {
        let mut request = request.clone();
        request.actor = actor.into();
        let response = crate::engine::execute(path, request, control)?;
        if semantic_review(&reviewed_value(&response.output)?, &proposal.before)
            != semantic_review(&proposal.proposed, &proposal.before)
        {
            return Err(invalid(
                "Proposal preview differs from the operation; prepare it again",
            ));
        }
        updated.applied = Some(serde_json::to_value(&response).map_err(invalid)?);
        Some(response)
    } else {
        None
    };
    updated.events.push(Event {
        actor: actor.into(),
        reason: reason.into(),
        decision: if approve { "approved" } else { "rejected" }.into(),
        unix_ms: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64,
    });
    Ok((updated, response))
}
/// Reversal is another reasoned review revision; old evidence and decisions survive.
pub fn reversal(proposal: &Proposal, actor: &str, reason: &str) -> Result<Request> {
    let response: Response = serde_json::from_value(
        proposal
            .applied
            .clone()
            .ok_or_else(|| invalid("Only applied proposals can be reversed"))?,
    )
    .map_err(invalid)?;
    let operation = match (&proposal.engine_request()?.operation, response.output) {
        (
            Operation::ReviewTargeted {
                batch,
                sample,
                target,
                ..
            },
            Output::TargetedQuantification { batch: current },
        ) => {
            let row = batch
                .results
                .iter()
                .find(|row| row.sample == *sample && row.target == *target)
                .ok_or_else(|| invalid("Original target row missing"))?;
            Operation::ReviewTargeted {
                expected_revision: current.reviews.len(),
                batch: current,
                sample: sample.clone(),
                target: target.clone(),
                accepted: row.state != crate::targeted::State::Rejected,
                reason: reason.into(),
            }
        }
        (
            Operation::ReviewQc {
                report, rule_id, ..
            },
            Output::QcReport { report: current },
        ) => {
            let acknowledged = report
                .reviews
                .iter()
                .rev()
                .find(|event| event.rule_id == *rule_id)
                .map(|event| event.acknowledged)
                .unwrap_or(false);
            Operation::ReviewQc {
                expected_revision: current.reviews.len(),
                report: current,
                rule_id: rule_id.clone(),
                reason: reason.into(),
                acknowledged,
            }
        }
        (
            Operation::ReviseChromatogram { analysis, .. },
            Output::ChromatographicProcessing { mut analyses, .. },
        ) => {
            let current = analyses
                .pop()
                .ok_or_else(|| invalid("Applied analysis missing"))?;
            Operation::ReviseChromatogram {
                expected_revision: current.revisions.len(),
                analysis: Box::new(current),
                correction: crate::chromatography::Correction::Restore {
                    revision: analysis.revisions.len(),
                    reason: reason.into(),
                },
                preview: false,
            }
        }
        (
            Operation::ReviewFeatureAnnotation {
                hypothesis_id,
                ledger: original,
                ..
            },
            Output::FeatureAnnotations { ledger },
        ) => Operation::ReviewFeatureAnnotation {
            expected_revision: ledger.hypotheses.len() + ledger.reviews.len(),
            ledger,
            hypothesis_id: *hypothesis_id,
            reason: reason.into(),
            decision: original
                .reviews
                .iter()
                .rev()
                .find(|r| r.hypothesis_id == *hypothesis_id)
                .map_or(crate::annotation::Decision::Reopen, |r| r.decision),
        },
        (
            Operation::AnnotateSpectrum {
                report: original, ..
            },
            Output::SpectralSearch { report },
        ) => Operation::AnnotateSpectrum {
            expected_revision: report.annotations.len(),
            report,
            annotation: original
                .annotations
                .last()
                .cloned()
                .map(|mut annotation| {
                    annotation.reason = reason.into();
                    annotation.actor = actor.into();
                    annotation
                })
                .unwrap_or(crate::spectral::Annotation {
                    id: uuid::Uuid::nil(),
                    actor: actor.into(),
                    reason: reason.into(),
                    unix_ms: 0,
                    candidate_accession: None,
                    label: "Reversed annotation".into(),
                    confidence: crate::spectral::Confidence::Unknown,
                    evidence: vec![],
                }),
        },
        _ => {
            return Err(invalid(
                "Applied proposal output does not match its review operation",
            ))
        }
    };
    Ok(Request {
        version: 1,
        operation_id: Default::default(),
        actor: actor.into(),
        operation,
    })
}
