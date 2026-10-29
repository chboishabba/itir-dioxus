use dioxus::prelude::*;

#[derive(Clone, PartialEq, Debug)]
pub struct DeliverableReceipt {
    pub label: String,
    pub status: String,
    pub pr_ref: Option<String>,
    pub proof_hash: Option<String>,
    pub sequence: u32,
}

#[derive(Clone, PartialEq, Debug)]
pub struct TopicGoal {
    pub goal_ref: String,
    pub label: String,
    pub blocked: bool,
    pub blocker_refs: Vec<String>,
    pub deliverables: Vec<DeliverableReceipt>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct TopicNode {
    pub topic_ref: String,
    pub label: String,
    pub continuous_alpha: f64,
    pub goals: Vec<TopicGoal>,
}

#[component]
pub fn TopicExplorer(topics: Vec<TopicNode>) -> Element {
    rsx! {
        section { class: "topic-explorer",
            h2 { "Topic Explorer" }
            p { "Main topics → topic goals → deliverables timeline. Receipts are projected as supplied." }
            for topic in topics {
                details { key: "{topic.topic_ref}",
                    summary {
                        strong { "{topic.label}" }
                        span { " · α={topic.continuous_alpha:.3}" }
                    }
                    ul {
                        for goal in topic.goals {
                            li { key: "{goal.goal_ref}",
                                details {
                                    summary {
                                        "{goal.label}"
                                        if goal.blocked {
                                            span { " · blocked by {goal.blocker_refs.len()} frontier item(s)" }
                                        }
                                    }
                                    ol {
                                        for item in ordered_deliverables(&goal.deliverables) {
                                            li { key: "{item.sequence}-{item.label}",
                                                strong { "{item.label}" }
                                                span { " · {item.status}" }
                                                if let Some(pr_ref) = &item.pr_ref {
                                                    code { " · PR {pr_ref}" }
                                                }
                                                if let Some(hash) = &item.proof_hash {
                                                    code { " · proof {hash}" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn ordered_deliverables(items: &[DeliverableReceipt]) -> Vec<DeliverableReceipt> {
    let mut rows = items.to_vec();
    rows.sort_by_key(|row| (row.sequence, row.label.clone()));
    rows
}
