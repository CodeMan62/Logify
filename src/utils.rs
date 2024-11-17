use crate::parser::LogEntry;

pub fn pair_log_entries<'a>(
    logs1: impl Iterator<Item = &'a LogEntry>,
    logs2: impl Iterator<Item = &'a LogEntry>
) -> impl Iterator<Item = (&'a LogEntry, &'a LogEntry)> {
    logs1.zip(logs2)
}

pub fn pair_with_ids<T>(ids: impl Iterator<Item = String>, items: impl Iterator<Item = T>) -> impl Iterator<Item = (String, T)> {
    ids.zip(items)
}

pub fn pair_log_sources_with_messages<'a>(
    logs: impl Iterator<Item = &'a LogEntry>
) -> impl Iterator<Item = (Option<String>, String)> {
    logs.map(|log| (log.source().clone(), log.message().to_string()))
}
