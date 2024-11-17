use std::collections::HashMap;

pub fn fold_summaries(actions: &[Action]) -> HashMap<String, f64> {
    let mut summaries = HashMap::new();
    
    for action in actions {
        match action {
            Action::Login { duration, .. } => {
                *summaries.entry("login_total_time".to_string())
                    .or_insert(0.0) += duration;
            }
            Action::Logout { duration, .. } => {
                *summaries.entry("logout_total_time".to_string())
                    .or_insert(0.0) += duration;
            }
            _ => {} // Ignore other action types
        }
    }

    summaries
}
