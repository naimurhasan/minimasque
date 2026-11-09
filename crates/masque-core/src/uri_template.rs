use crate::{Error, percent};

pub fn expand(template: &str, vars: &[(&str, &str)]) -> Result<String, Error> {
    expand_with(template, vars, percent::encode)
}

pub fn expand_with(
    template: &str,
    vars: &[(&str, &str)],
    encode: fn(&str) -> String,
) -> Result<String, Error> {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let close = rest[open..].find('}').ok_or(Error::InvalidTemplate)? + open;
        let name = &rest[open + 1..close];
        let (_, value) = vars
            .iter()
            .find(|(var, _)| *var == name)
            .ok_or(Error::InvalidTemplate)?;
        out.push_str(&encode(value));
        rest = &rest[close + 1..];
    }
    out.push_str(rest);
    Ok(out)
}

pub fn captures<'a>(template: &'a str, input: &'a str) -> Option<Vec<(&'a str, &'a str)>> {
    let mut found = Vec::new();
    let mut rest = template;
    let mut remaining = input;
    while let Some(open) = rest.find('{') {
        remaining = remaining.strip_prefix(&rest[..open])?;
        let close = rest[open..].find('}')? + open;
        let name = &rest[open + 1..close];
        rest = &rest[close + 1..];
        let literal = &rest[..rest.find('{').unwrap_or(rest.len())];
        let end = if literal.is_empty() {
            remaining.len()
        } else {
            remaining.find(literal)?
        };
        found.push((name, &remaining[..end]));
        remaining = &remaining[end..];
    }
    (remaining == rest).then_some(found)
}
