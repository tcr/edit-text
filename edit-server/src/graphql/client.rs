use extern::{
    failure::Error,
    oatie::{
        doc::*,
    },
    reqwest,
    serde_json,
};

pub fn get_single_page_graphql(input_id: &str) -> Option<Doc> {
    let client = reqwest::Client::new();
    let text = client.post("http://127.0.0.1:8003/graphql/")
        .json(&json!({
            "query": r#"

query ($id: String!) {
    page(id: $id) {
        doc
    
}

"#,
            "variables": {
                "id": input_id,
            },
        }))
        .send()
        .ok()?
        .text()
        .ok()?;
    
    let ret: ::serde_json::Value = serde_json::from_str(&text).ok()?;
    let node = ret.pointer("/data/page/doc")?;
    let ron = node.as_str()?.to_string();
    let body = ::ron::de::from_str(&ron).ok()?;
    Some(Doc(body))
}

pub fn graphql_request(
    query: &str,
    variables: &serde_json::Value,
) -> Result<serde_json::Value, Error> {
    let client = reqwest::Client::new();
    let text = client.post("http://127.0.0.1:8003/graphql/")
        .json(&json!({
            "query": query,
            "variables": variables,
        }))
        .send()?
        .text()?;
    
    // TODO handle /errors[...]
    Ok(serde_json::from_str(&text)?)
}

pub fn get_or_create_page_graphql(input_id: &str, doc: &Doc) -> Result<Doc, Error> {
    let ret = graphql_request(
        r#"

mutation ($id: String!, $default: String!) {
    getOrCreatePage(id: $id, default: $default) {
        doc
    }
}

"#,
        &json!({
            "id": input_id,
            "default": ::ron::ser::to_string(&doc.0).unwrap(),
        }),
    )?;

    // Extract the doc field.
    let doc_string = ret.pointer("/data/getOrCreatePage/doc")
        .ok_or(format_err!("unexpected json structure"))?
        .as_str().unwrap()
        .to_string();

    Ok(Doc(::ron::de::from_str(&doc_string)?))
}

pub fn create_page_graphql(input_id: &str, doc: &Doc) -> Option<Doc> {
    let client = reqwest::Client::new();
    let text = client.post("http://127.0.0.1:8003/graphql/")
        .json(&json!({
            "query": r#"

mutation ($id: String!, $doc: String!) {
    createPage(id: $id, doc: $doc) {
        doc
    }
}

"#,
            "variables": {
                "id": input_id,
                "doc": ::ron::ser::to_string(&doc.0).unwrap(),
            },
        }))
        .send()
        .ok()?
        .text()
        .ok()?;
    
    let ret: ::serde_json::Value = serde_json::from_str(&text).ok()?;
    let node = ret.pointer("/data/createPage/doc")?;
    let ron = node.as_str()?.to_string();
    let body = ::ron::de::from_str(&ron).ok()?;
    Some(Doc(body))
}
