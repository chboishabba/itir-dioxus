#![cfg(feature = "production-data")]

use sensiblaw_pg_source_store::{
    load_database_config, load_legal_controversy_matter, load_legal_controversy_residual,
    load_legal_proof_obligation, load_legal_proposition_fibre, load_typed_response_edge,
    DisagreementKind, EpistemicStatus, LegalControversyMatter, LegalControversyResidual,
    LegalProofObligation, LegalPropositionFibre, ResponseMode, ReverseLegalProofSearch,
    TypedResponseEdge,
};

pub trait ControversyPersonaProjection {
    fn matter_ref(&self) -> &str;
    fn controversy_ref(&self) -> &str;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientControversyProjection {
    pub matter_ref: String,
    pub controversy_ref: String,
    pub proposition_fibre_refs: Vec<String>,
    pub reviewed_fibre_refs: Vec<String>,
    pub candidate_fibre_refs: Vec<String>,
    pub normative_order_refs: Vec<String>,
    pub response_modes: Vec<ResponseMode>,
    pub unresolved_questions: Vec<String>,
    pub creates_semantic_authority: bool,
    pub claim_truth_promoted: bool,
}

impl ControversyPersonaProjection for ClientControversyProjection {
    fn matter_ref(&self) -> &str { &self.matter_ref }
    fn controversy_ref(&self) -> &str { &self.controversy_ref }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SolicitorControversyProjection {
    pub matter_ref: String,
    pub controversy_ref: String,
    pub proposition_fibre_refs: Vec<String>,
    pub response_modes: Vec<ResponseMode>,
    pub residual_kinds: Vec<DisagreementKind>,
    pub open_obligation_refs: Vec<String>,
    pub requested_discriminators: Vec<String>,
    pub target_evidence_queries: Vec<String>,
    pub normative_order_refs: Vec<String>,
    pub creates_semantic_authority: bool,
    pub claim_truth_promoted: bool,
}

impl ControversyPersonaProjection for SolicitorControversyProjection {
    fn matter_ref(&self) -> &str { &self.matter_ref }
    fn controversy_ref(&self) -> &str { &self.controversy_ref }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CourtControversyProjection {
    pub matter_ref: String,
    pub controversy_ref: String,
    pub common_ground_refs: Vec<String>,
    pub disputed_occurrence_refs: Vec<String>,
    pub disputed_characterisation_refs: Vec<String>,
    pub disputed_causation_refs: Vec<String>,
    pub evidence_reliability_refs: Vec<String>,
    pub legal_consequence_refs: Vec<String>,
    pub normative_order_mismatch_refs: Vec<String>,
    pub disagreement_kinds: Vec<DisagreementKind>,
    pub open_question_refs: Vec<String>,
    pub determines_credibility: bool,
    pub determines_ultimate_fact: bool,
    pub assigns_normative_weight: bool,
    pub enters_final_judgment: bool,
}

impl ControversyPersonaProjection for CourtControversyProjection {
    fn matter_ref(&self) -> &str { &self.matter_ref }
    fn controversy_ref(&self) -> &str { &self.controversy_ref }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatterControversyWorkspace {
    pub matter: LegalControversyMatter,
    pub fibres: Vec<LegalPropositionFibre>,
    pub responses: Vec<TypedResponseEdge>,
    pub residuals: Vec<LegalControversyResidual>,
    pub obligations: Vec<LegalProofObligation>,
    pub reverse_search: Option<ReverseLegalProofSearch>,
    pub client: ClientControversyProjection,
    pub solicitor: SolicitorControversyProjection,
    pub court: CourtControversyProjection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControversyProjectionInput {
    pub matter: LegalControversyMatter,
    pub fibres: Vec<LegalPropositionFibre>,
    pub responses: Vec<TypedResponseEdge>,
    pub residuals: Vec<LegalControversyResidual>,
    pub obligations: Vec<LegalProofObligation>,
    pub reverse_search: Option<ReverseLegalProofSearch>,
}

pub fn project_controversy_personas(
    mut input: ControversyProjectionInput,
) -> Result<MatterControversyWorkspace, String> {
    if input.matter.matter_ref.trim().is_empty() || input.matter.controversy_ref.trim().is_empty() {
        return Err("persisted controversy is missing matter identity".into());
    }
    if input.fibres.iter().any(|f| !input.matter.proposition_fibre_refs.contains(&f.fibre_ref)) {
        return Err("controversy fibre is not a member of the persisted matter".into());
    }
    if input.responses.iter().any(|r| r.controversy_ref != input.matter.controversy_ref)
        || input.residuals.iter().any(|r| r.controversy_ref != input.matter.controversy_ref)
        || input.obligations.iter().any(|o| o.controversy_ref != input.matter.controversy_ref)
    {
        return Err("controversy child belongs to a different persisted matter".into());
    }

    input.fibres.sort_by(|a, b| a.fibre_ref.cmp(&b.fibre_ref));
    input.responses.sort_by(|a, b| a.response_ref.cmp(&b.response_ref));
    input.residuals.sort_by(|a, b| a.residual_ref.cmp(&b.residual_ref));
    input.obligations.sort_by(|a, b| a.obligation_ref.cmp(&b.obligation_ref));

    let mut normative_order_refs = input.fibres.iter().map(|f| f.normative_order_ref.clone()).collect::<Vec<_>>();
    normative_order_refs.sort();
    normative_order_refs.dedup();
    let proposition_fibre_refs = input.fibres.iter().map(|f| f.fibre_ref.clone()).collect::<Vec<_>>();
    let reviewed_fibre_refs = input.fibres.iter().filter(|f| f.reviewed_evidence_ref.is_some()).map(|f| f.fibre_ref.clone()).collect();
    let candidate_fibre_refs = input.fibres.iter().filter(|f| f.reviewed_evidence_ref.is_none()).map(|f| f.fibre_ref.clone()).collect();
    let response_modes = input.responses.iter().map(|r| r.mode).collect::<Vec<_>>();
    let unresolved_questions = input.residuals.iter().map(|r| r.unresolved_question.clone()).collect::<Vec<_>>();
    let residual_kinds = input.residuals.iter().map(|r| r.kind).collect::<Vec<_>>();
    let open_obligation_refs = input.obligations.iter().filter(|o| o.is_open).map(|o| o.obligation_ref.clone()).collect::<Vec<_>>();
    let requested_discriminators = input.residuals.iter().filter_map(|r| r.requested_discriminator.clone()).collect::<Vec<_>>();
    let target_evidence_queries = input.residuals.iter().filter_map(|r| r.target_evidence_query.clone()).collect::<Vec<_>>();

    let common_ground_refs = input.fibres.iter().filter(|f| f.epistemic_status == EpistemicStatus::Admitted).map(|f| f.fibre_ref.clone()).collect::<Vec<_>>();
    let mut disputed_occurrence_refs = Vec::new();
    let mut disputed_characterisation_refs = Vec::new();
    let mut disputed_causation_refs = Vec::new();
    let mut evidence_reliability_refs = Vec::new();
    for response in &input.responses {
        match response.mode {
            ResponseMode::DenyOccurrence => disputed_occurrence_refs.push(response.response_ref.clone()),
            ResponseMode::AdmitOccurrenceDisputeCharacterisation | ResponseMode::AdmitConductAddContext => disputed_characterisation_refs.push(response.response_ref.clone()),
            ResponseMode::DisputeCausation => disputed_causation_refs.push(response.response_ref.clone()),
            ResponseMode::ChallengeEvidenceReliability => evidence_reliability_refs.push(response.response_ref.clone()),
            ResponseMode::OfferAlternativeEvent | ResponseMode::AdmitProposition => {}
        }
    }
    let legal_consequence_refs = input.residuals.iter().filter(|r| r.kind == DisagreementKind::LegalConsequence).map(|r| r.residual_ref.clone()).collect();
    let normative_order_mismatch_refs = input.residuals.iter().filter(|r| r.kind == DisagreementKind::NormativeOrderMismatch).map(|r| r.residual_ref.clone()).collect();
    let open_question_refs = input.residuals.iter().map(|r| r.residual_ref.clone()).collect::<Vec<_>>();

    let matter_ref = input.matter.matter_ref.clone();
    let controversy_ref = input.matter.controversy_ref.clone();
    let client = ClientControversyProjection {
        matter_ref: matter_ref.clone(), controversy_ref: controversy_ref.clone(),
        proposition_fibre_refs: proposition_fibre_refs.clone(), reviewed_fibre_refs,
        candidate_fibre_refs, normative_order_refs: normative_order_refs.clone(),
        response_modes: response_modes.clone(), unresolved_questions,
        creates_semantic_authority: false, claim_truth_promoted: false,
    };
    let solicitor = SolicitorControversyProjection {
        matter_ref: matter_ref.clone(), controversy_ref: controversy_ref.clone(),
        proposition_fibre_refs, response_modes, residual_kinds: residual_kinds.clone(),
        open_obligation_refs, requested_discriminators, target_evidence_queries,
        normative_order_refs, creates_semantic_authority: false, claim_truth_promoted: false,
    };
    let court = CourtControversyProjection {
        matter_ref, controversy_ref, common_ground_refs, disputed_occurrence_refs,
        disputed_characterisation_refs, disputed_causation_refs, evidence_reliability_refs,
        legal_consequence_refs, normative_order_mismatch_refs,
        disagreement_kinds: residual_kinds, open_question_refs,
        determines_credibility: false, determines_ultimate_fact: false,
        assigns_normative_weight: false, enters_final_judgment: false,
    };

    Ok(MatterControversyWorkspace {
        matter: input.matter, fibres: input.fibres, responses: input.responses,
        residuals: input.residuals, obligations: input.obligations,
        reverse_search: input.reverse_search, client, solicitor, court,
    })
}

pub fn load_persisted_controversy_workspace(
    controversy_ref: &str,
    expected_matter_ref: &str,
) -> Result<MatterControversyWorkspace, String> {
    if controversy_ref.trim().is_empty() || expected_matter_ref.trim().is_empty() {
        return Err("controversy_ref and expected matter_ref are required".into());
    }
    let config = load_database_config(None).map_err(|error| error.to_string())?;
    let matter = load_legal_controversy_matter(&config, controversy_ref).map_err(|error| error.to_string())?;
    if matter.matter_ref != expected_matter_ref {
        return Err("persisted controversy belongs to a different Matter".into());
    }
    let mut fibres = Vec::new();
    for reference in &matter.proposition_fibre_refs {
        fibres.push(load_legal_proposition_fibre(&config, reference).map_err(|error| error.to_string())?);
    }
    let mut responses = Vec::new();
    for reference in &matter.response_refs {
        responses.push(load_typed_response_edge(&config, reference).map_err(|error| error.to_string())?);
    }
    let mut residuals = Vec::new();
    for reference in &matter.residual_refs {
        residuals.push(load_legal_controversy_residual(&config, reference).map_err(|error| error.to_string())?);
    }
    let mut obligations = Vec::new();
    for reference in &matter.obligation_refs {
        obligations.push(load_legal_proof_obligation(&config, reference).map_err(|error| error.to_string())?);
    }
    project_controversy_personas(ControversyProjectionInput {
        matter, fibres, responses, residuals, obligations, reverse_search: None,
    })
}

impl MatterControversyWorkspace {
    #[doc(hidden)]
    pub fn contract_fixture_for_test(
        matter_ref: &str,
        controversy_ref: &str,
        response_mode: ResponseMode,
        disagreement_kind: DisagreementKind,
        normative_orders: [&str; 2],
    ) -> Self {
        let applicant = LegalPropositionFibre {
            fibre_ref: "applicant".into(), claim_ref: "claim:applicant".into(),
            party: sensiblaw_pg_source_store::PartyRole::Applicant,
            legal_role: sensiblaw_pg_source_store::LegalRole::LegalProposition,
            epistemic_status: EpistemicStatus::Supported, evidence_kind_ref: "source_text".into(),
            source_reference: "span:applicant".into(), reviewed_evidence_ref: Some("reviewed:applicant".into()),
            normative_order_ref: normative_orders[0].into(), temporal_reference: "temporal:unknown".into(),
            relation_reference: "proposition:applicant".into(), candidate_only: true,
            creates_semantic_authority: false, creates_legal_authority: false,
            applicability_promoted: false, claim_truth_promoted: false,
        };
        let respondent = LegalPropositionFibre {
            fibre_ref: "respondent".into(), claim_ref: "claim:respondent".into(),
            party: sensiblaw_pg_source_store::PartyRole::Respondent,
            legal_role: sensiblaw_pg_source_store::LegalRole::LegalProposition,
            epistemic_status: EpistemicStatus::Admitted, evidence_kind_ref: "source_text".into(),
            source_reference: "span:respondent".into(), reviewed_evidence_ref: Some("reviewed:respondent".into()),
            normative_order_ref: normative_orders[1].into(), temporal_reference: "temporal:unknown".into(),
            relation_reference: "proposition:respondent".into(), candidate_only: true,
            creates_semantic_authority: false, creates_legal_authority: false,
            applicability_promoted: false, claim_truth_promoted: false,
        };
        let matter = LegalControversyMatter {
            controversy_ref: controversy_ref.into(), matter_ref: matter_ref.into(), root_fibre_ref: "applicant".into(),
            proposition_fibre_refs: vec!["applicant".into(), "respondent".into()], response_refs: vec!["response:1".into()],
            residual_refs: vec!["residual:1".into()], obligation_refs: vec![], candidate_only: true,
            creates_semantic_authority: false, creates_legal_authority: false, applicability_promoted: false, claim_truth_promoted: false,
        };
        let response = TypedResponseEdge {
            response_ref: "response:1".into(), controversy_ref: controversy_ref.into(), target_fibre_ref: "applicant".into(),
            response_fibre_ref: "respondent".into(), mode: response_mode, response_reference: "reviewed:respondent".into(),
            candidate_only: true, creates_semantic_authority: false, creates_legal_authority: false,
            applicability_promoted: false, claim_truth_promoted: false,
        };
        let residual = LegalControversyResidual {
            residual_ref: "residual:1".into(), controversy_ref: controversy_ref.into(), kind: disagreement_kind,
            applicant_fibre_ref: "applicant".into(), respondent_fibre_ref: "respondent".into(), residual_level_ref: "residual:open".into(),
            unresolved_question: "unresolved".into(), requested_discriminator: Some("discriminator".into()),
            target_evidence_query: Some("query".into()), candidate_only: true, creates_semantic_authority: false,
            creates_legal_authority: false, applicability_promoted: false, claim_truth_promoted: false,
        };
        project_controversy_personas(ControversyProjectionInput {
            matter, fibres: vec![applicant, respondent], responses: vec![response], residuals: vec![residual], obligations: vec![], reverse_search: None,
        }).expect("contract fixture is internally consistent")
    }
}
