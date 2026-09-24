#![cfg(feature = "production-data")]

use std::{fs, path::Path};

use serde::Deserialize;
use sensiblaw_core::matter_context::{
    ContextProjectionCoordinate, DisclosureBoundary, KnowledgeCutMembership,
    MatterConsumerRole, MatterContext, MatterPurpose,
};
use sensiblaw_reader_model::{
    KnowledgeTimelineEntry, MatterAcceptanceRoleCoordinate,
    MatterAcceptanceSemanticRole,
};

use super::matter::GenericMatterRequest;

pub const MATTER_SCOPE_SCHEMA: &str = "sensiblaw.matter-scope.v0_1";

#[derive(Debug, Clone, Deserialize)]
pub struct MatterScopeManifest {
    pub schema: String,
    pub matter_ref: String,
    #[serde(default)]
    pub event_refs: Vec<String>,
    #[serde(default)]
    pub operational_dates: Vec<String>,
    pub context: MatterScopeContext,
    #[serde(default)]
    pub visibility: Vec<MatterScopeVisibility>,
    #[serde(default)]
    pub knowledge_timeline: Vec<MatterScopeKnowledge>,
    #[serde(default)]
    pub legal_proof_refs: Vec<String>,
    #[serde(default)]
    pub research_refs: Vec<String>,
    #[serde(default)]
    pub work_product_refs: Vec<String>,
    #[serde(default)]
    pub handoff_refs: Vec<String>,
    #[serde(default)]
    pub no_event_refs: Vec<String>,
    #[serde(default)]
    pub acceptance_roles: Vec<MatterScopeAcceptanceRole>,
    #[serde(default)]
    pub procedural_significance_review_refs: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MatterScopeContext {
    pub purpose: String,
    pub role: String,
    pub disclosure_boundary: String,
    #[serde(default)]
    pub knowledge_time_cut_ref: Option<String>,
    #[serde(default)]
    pub sealed_refs: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MatterScopeVisibility {
    pub semantic_ref: String,
    #[serde(default)]
    pub allowed_roles: Vec<String>,
    #[serde(default)]
    pub allowed_purposes: Vec<String>,
    #[serde(default = "known_at_cut")]
    pub knowledge_membership: String,
    #[serde(default = "default_true")]
    pub explicitly_selected: bool,
    #[serde(default)]
    pub sealed: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MatterScopeKnowledge {
    pub semantic_ref: String,
    pub source_revision_ref: String,
    pub knowledge_time_ref: String,
    #[serde(default = "known_at_cut")]
    pub knowledge_membership: String,
    #[serde(default)]
    pub source_role_ref: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MatterScopeAcceptanceRole {
    pub semantic_ref: String,
    pub role: String,
    pub review_ref: String,
}

fn default_true() -> bool {
    true
}

fn known_at_cut() -> String {
    "known-at-cut".into()
}

pub fn load_matter_scope_manifest(
    path: impl AsRef<Path>,
) -> Result<GenericMatterRequest, String> {
    let raw = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let manifest: MatterScopeManifest =
        serde_json::from_str(&raw).map_err(|error| error.to_string())?;
    manifest.into_request()
}

impl MatterScopeManifest {
    pub fn into_request(self) -> Result<GenericMatterRequest, String> {
        if self.schema != MATTER_SCOPE_SCHEMA {
            return Err(format!(
                "matter scope schema mismatch: expected {MATTER_SCOPE_SCHEMA}, got {}",
                self.schema
            ));
        }
        if self.matter_ref.trim().is_empty() {
            return Err("matter_ref is required".into());
        }

        let role = parse_role(&self.context.role)?;
        let purpose = parse_purpose(&self.context.purpose)?;
        let disclosure_boundary =
            parse_disclosure_boundary(&self.context.disclosure_boundary)?;

        let context = MatterContext {
            matter_ref: self.matter_ref.clone(),
            purpose_ref: purpose,
            active_consumer_role: role,
            disclosure_boundary,
            knowledge_time_cut_ref: self.context.knowledge_time_cut_ref,
            sealed_refs: self.context.sealed_refs,
            minimum_necessary: true,
            purpose_limited: true,
            access_logged: true,
            revocable: true,
            mutates_canonical_world: false,
            creates_semantic_authority: false,
            claim_truth_promoted: false,
        };

        let context_coordinates = self
            .visibility
            .into_iter()
            .map(|item| {
                Ok(ContextProjectionCoordinate {
                    semantic_ref: item.semantic_ref,
                    matter_ref: self.matter_ref.clone(),
                    allowed_roles: item
                        .allowed_roles
                        .iter()
                        .map(|value| parse_role(value))
                        .collect::<Result<Vec<_>, _>>()?,
                    allowed_purposes: item
                        .allowed_purposes
                        .iter()
                        .map(|value| parse_purpose(value))
                        .collect::<Result<Vec<_>, _>>()?,
                    knowledge_membership:
                        parse_knowledge_membership(&item.knowledge_membership)?,
                    explicitly_selected: item.explicitly_selected,
                    sealed: item.sealed,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;

        let knowledge_timeline = self
            .knowledge_timeline
            .into_iter()
            .map(|entry| {
                Ok(KnowledgeTimelineEntry {
                    semantic_ref: entry.semantic_ref,
                    source_revision_ref: entry.source_revision_ref,
                    knowledge_time_ref: entry.knowledge_time_ref,
                    knowledge_membership:
                        parse_knowledge_membership(&entry.knowledge_membership)?,
                    source_role_ref: entry.source_role_ref,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;

        let acceptance_roles = self
            .acceptance_roles
            .into_iter()
            .map(|coordinate| {
                if coordinate.semantic_ref.trim().is_empty()
                    || coordinate.review_ref.trim().is_empty()
                {
                    return Err(
                        "acceptance role requires semantic_ref and review_ref".into()
                    );
                }
                Ok(MatterAcceptanceRoleCoordinate {
                    semantic_ref: coordinate.semantic_ref,
                    role: parse_acceptance_role(&coordinate.role)?,
                    review_ref: coordinate.review_ref,
                    candidate_only: true,
                    creates_semantic_authority: false,
                    claim_truth_promoted: false,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;

        Ok(GenericMatterRequest {
            matter_ref: self.matter_ref,
            event_refs: self.event_refs,
            operational_dates: self.operational_dates,
            context,
            context_coordinates,
            knowledge_timeline,
            legal_proof_refs: self.legal_proof_refs,
            research_refs: self.research_refs,
            work_product_refs: self.work_product_refs,
            handoff_refs: self.handoff_refs,
            no_event_refs: self.no_event_refs,
            acceptance_roles,
            procedural_significance_review_refs:
                self.procedural_significance_review_refs,
        })
    }
}

fn parse_role(value: &str) -> Result<MatterConsumerRole, String> {
    match value {
        "protected-subject" => Ok(MatterConsumerRole::ProtectedSubject),
        "support-operator" => Ok(MatterConsumerRole::SupportOperator),
        "clinician" => Ok(MatterConsumerRole::Clinician),
        "advocate" => Ok(MatterConsumerRole::Advocate),
        "lawyer" => Ok(MatterConsumerRole::Lawyer),
        "journalist" => Ok(MatterConsumerRole::Journalist),
        "public-official" => Ok(MatterConsumerRole::PublicOfficial),
        "public-audience" => Ok(MatterConsumerRole::PublicAudience),
        "regulator" => Ok(MatterConsumerRole::Regulator),
        "researcher" => Ok(MatterConsumerRole::Researcher),
        "other" => Ok(MatterConsumerRole::Other),
        other => Err(format!("unknown Matter consumer role: {other}")),
    }
}

fn parse_purpose(value: &str) -> Result<MatterPurpose, String> {
    match value {
        "personal-review" => Ok(MatterPurpose::PersonalReview),
        "care-planning" => Ok(MatterPurpose::CarePlanning),
        "legal-advocacy" => Ok(MatterPurpose::LegalAdvocacy),
        "affidavit-preparation" => Ok(MatterPurpose::AffidavitPreparation),
        "legal-research" => Ok(MatterPurpose::LegalResearch),
        "journalistic-verification" => Ok(MatterPurpose::JournalisticVerification),
        "public-administration-review" => Ok(MatterPurpose::PublicAdministrationReview),
        "regulatory-review" => Ok(MatterPurpose::RegulatoryReview),
        "research-publication" => Ok(MatterPurpose::ResearchPublication),
        "handoff-preparation" => Ok(MatterPurpose::HandoffPreparation),
        "other" => Ok(MatterPurpose::Other),
        other => Err(format!("unknown Matter purpose: {other}")),
    }
}

fn parse_disclosure_boundary(value: &str) -> Result<DisclosureBoundary, String> {
    match value {
        "matter-internal" => Ok(DisclosureBoundary::MatterInternal),
        "role-scoped" => Ok(DisclosureBoundary::RoleScoped),
        "recipient-scoped" => Ok(DisclosureBoundary::RecipientScoped),
        "metadata-only" => Ok(DisclosureBoundary::MetadataOnly),
        "public-projection" => Ok(DisclosureBoundary::PublicProjection),
        "protected-disclosure" => Ok(DisclosureBoundary::ProtectedDisclosure),
        other => Err(format!("unknown disclosure boundary: {other}")),
    }
}

fn parse_acceptance_role(
    value: &str,
) -> Result<MatterAcceptanceSemanticRole, String> {
    match value {
        "party-assertion" => Ok(MatterAcceptanceSemanticRole::PartyAssertion),
        "procedural-outcome" => Ok(MatterAcceptanceSemanticRole::ProceduralOutcome),
        "later-annotation" => Ok(MatterAcceptanceSemanticRole::LaterAnnotation),
        other => Err(format!("unknown Matter acceptance role: {other}")),
    }
}

fn parse_knowledge_membership(
    value: &str,
) -> Result<KnowledgeCutMembership, String> {
    match value {
        "known-at-cut" => Ok(KnowledgeCutMembership::KnownAtCut),
        "known-after-cut" => Ok(KnowledgeCutMembership::KnownAfterCut),
        "unknown-at-cut" => Ok(KnowledgeCutMembership::UnknownAtCut),
        "not-applicable" => Ok(KnowledgeCutMembership::NotApplicable),
        other => Err(format!("unknown knowledge-cut membership: {other}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_requires_explicit_known_after_cut_instead_of_inference() {
        let manifest = MatterScopeManifest {
            schema: MATTER_SCOPE_SCHEMA.into(),
            matter_ref: "matter:1".into(),
            event_refs: vec![],
            operational_dates: vec![],
            context: MatterScopeContext {
                purpose: "legal-advocacy".into(),
                role: "lawyer".into(),
                disclosure_boundary: "role-scoped".into(),
                knowledge_time_cut_ref: Some("cut:1".into()),
                sealed_refs: vec![],
            },
            visibility: vec![MatterScopeVisibility {
                semantic_ref: "source:later".into(),
                allowed_roles: vec!["lawyer".into()],
                allowed_purposes: vec!["legal-advocacy".into()],
                knowledge_membership: "known-after-cut".into(),
                explicitly_selected: true,
                sealed: false,
            }],
            knowledge_timeline: vec![],
            legal_proof_refs: vec![],
            research_refs: vec![],
            work_product_refs: vec![],
            handoff_refs: vec![],
            no_event_refs: vec![],
            acceptance_roles: vec![],
            procedural_significance_review_refs: vec![],
        };
        let request = manifest.into_request().unwrap();
        assert_eq!(
            request.context_coordinates[0].knowledge_membership,
            KnowledgeCutMembership::KnownAfterCut
        );
    }
}
