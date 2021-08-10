extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod datastruct;
use datastruct::{
    AppState, Item, OkResponse, Page, Pagination, Resp, ShortItem, State, DEFAULT_API_URL,
    GETCONTRACTSTATE, URL,
};
use std::collections::HashMap;
use tide::{Request, StatusCode};

#[tokio::main]
async fn main() -> tide::Result<()> {
    let client = reqwest::Client::new();
    let response = client
        .post(URL)
        .body(GETCONTRACTSTATE)
        .send()
        .await
        // TODO error handling
        .expect("Post request failed");
    // TODO check server error code and error handling
    let text = response.text().await.expect("1");
    // TODO remove debug println
    if text.len() < 3000 {
        println!("{}", text);
    }
    // TODO error handling
    let cur_state: Resp = serde_json::from_str(&text).expect("2");
    let mut owned_id = HashMap::with_capacity(cur_state.result.tokens_owner_stage.len());
    for (key, val) in &cur_state.result.tokens_owner_stage {
        let tokens = val.keys().cloned().collect();
        owned_id.insert(key.to_string(), tokens);
    }
    let app_state = AppState {
        id_list: cur_state.result.token_stage.keys().cloned().collect(),
        owned_id,
        contract_state: cur_state.result,
    };
    let api_url = match std::env::var("API_URL") {
        Ok(val) => val,
        Err(_) => String::from(DEFAULT_API_URL),
    };
    println!("Dragons backend is starting on {}", api_url);
    let mut app = tide::with_state(app_state);
    app.at("/api/v1/dragons").get(get_dragons);
    app.at("/api/v1/dragon/:id").get(get_dragon_by_id);
    app.listen(api_url).await?;
    Ok(())
}
// GET /api/v1/dragon/:id
async fn get_dragon_by_id(req: Request<AppState>) -> tide::Result {
    let str_id = req.param("id")?;
    let app_state = req.state();
    let cs = &app_state.contract_state;
    match cs.token_stage.get(str_id) {
        Some(_) => {
            let page: Page = Page {
                limit: 1,
                offset: 0,
                owner: "".to_string(),
            };
            Ok(create_response(vec![create_item(str_id, cs)?], &page, 1)?.into())
        }
        None => Err(tide::Error::from_str(
            StatusCode::NotFound,
            "id is not found",
        )),
    }
}
// GET /api/v1/dragons [?limit=1&offset=1&owner=0x...]
async fn get_dragons(req: Request<AppState>) -> tide::Result {
    let page: Page = req.query()?;
    if page.limit == 0 {
        return Err(tide::Error::from_str(
            StatusCode::BadRequest,
            "limit cannot be zero",
        ));
    }
    let app_state = req.state();
    let cs = &app_state.contract_state;
    if page.owner.is_empty() {
        let real_end = app_state.id_list.len();
        let (start, end) = calc_indexes(&page, real_end)?;
        let items = get_items(&app_state.id_list[start..end], cs)?;
        Ok(create_response(items, &page, real_end)?.into())
    } else {
        let tokens = match app_state.owned_id.get(&page.owner) {
            Some(result) => result,
            None => {
                return Err(tide::Error::from_str(
                    StatusCode::NotFound,
                    "owner is not found",
                ))
            }
        };
        let real_end = tokens.len();
        let (start, end) = calc_indexes(&page, real_end)?;
        let items = get_items(&tokens[start..end], cs)?;
        Ok(create_response(items, &page, real_end)?.into())
    }
}
fn calc_indexes(page: &Page, real_end: usize) -> Result<(usize, usize), tide::Error> {
    let start = page.offset * page.limit;
    if start >= real_end {
        return Err(tide::Error::from_str(
            StatusCode::BadRequest,
            "offset is too big",
        ));
    }
    Ok((start, std::cmp::min(start + page.limit, real_end)))
}
fn get_items<'a>(tokens: &[String], cs: &'a State) -> Result<Vec<Item<'a>>, tide::Error> {
    let mut items = Vec::with_capacity(tokens.len());
    for str_id in tokens {
        items.push(create_item(str_id, cs)?);
    }
    Ok(items)
}
fn create_item<'a>(str_id: &str, cs: &'a State) -> Result<Item<'a>, tide::Error> {
    Ok(Item {
        id: str_id
            .parse()
            .map_err(|e| tide::Error::new(StatusCode::InternalServerError, e))?,
        owner: get_element(&cs.token_owners, str_id)?,
        url: get_element(&cs.token_uris, str_id)?,
        gen_image: get_element(&cs.token_gen_image, str_id)?,
        gen_fight: get_element(&cs.token_gen_battle, str_id)?,
        stage: get_element(&cs.token_stage, str_id)?
            .parse()
            .map_err(|e| tide::Error::new(StatusCode::InternalServerError, e))?,
        // TODO add real statistics
        fight_win: 666,
        fight_lose: 13,
        // TODO write true parents
        parents: [].to_vec(),
        // TODO write true children
        children: [].to_vec(),
    })
}
fn get_element<'a>(h_m: &'a HashMap<String, String>, str_id: &str) -> Result<&'a str, tide::Error> {
    Ok(h_m.get(str_id).ok_or_else(internal_error)?)
}
fn create_response(items: Vec<Item>, page: &Page, records: usize) -> Result<String, tide::Error> {
    let cur_pag = Pagination {
        records,
        pages: (records + page.limit - 1) / page.limit,
        current_page: page.offset + 1,
        limit: page.limit,
    };
    let result = OkResponse {
        success: true,
        data: items,
        pagination: cur_pag,
    };
    serde_json::to_string(&result).map_err(|e| tide::Error::new(StatusCode::InternalServerError, e))
}
fn internal_error() -> tide::Error {
    tide::Error::from_str(StatusCode::InternalServerError, "HashMap::get() error")
}
