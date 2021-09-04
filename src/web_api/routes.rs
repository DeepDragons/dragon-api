use crate::state::AppState;
use crate::web_api::{Item, OkResponse, Page, Pagination};
use std::collections::HashMap;
use tide::{Request, Response, StatusCode};

// GET /api/v1/dragons/:id
pub async fn get_dragon_by_id(req: Request<AppState>) -> tide::Result {
    let str_id = req.param("id")?;
    let app_state = req.state();
    match app_state.main_state.token_stage.get(str_id) {
        Some(_) => {
            let page = Page {
                limit: 1,
                ..Default::default()
            };
            Ok(create_response(vec![create_item(str_id, app_state)?], &page, 1)?.into())
        }
        None => Ok(create_error(
            StatusCode::NotFound,
            &format!("Id {} is not found.", str_id),
        )),
    }
}

// GET /api/v1/battle
pub async fn get_from_battle(req: Request<AppState>) -> tide::Result {
    let app_state = req.state();
    get_data(&app_state.battle_id_list, &app_state.battle_owned_id, &req)
}

// GET /api/v1/breed
pub async fn get_from_breed(req: Request<AppState>) -> tide::Result {
    let app_state = req.state();
    get_data(&app_state.breed_id_list, &app_state.breed_owned_id, &req)
}

// GET /api/v1/market
pub async fn get_from_market(req: Request<AppState>) -> tide::Result {
    let app_state = req.state();
    let page: Page = req.query()?;
    if page.owner.is_empty() {
        let mut tokens = Vec::with_capacity(app_state.market_id_list.len());
        let m_prices = &app_state.market_id_price;
        // if we have some filters
        if page.stage != 255 || page.start_price != 0 || page.end_price != u64::MAX {
            for str_id in &app_state.market_id_list {
                let price = get_price(str_id, m_prices)?;
                let stage = get_element(&app_state.main_state.token_stage, str_id)?
                    .parse::<u8>()
                    .map_err(|e| tide::Error::new(StatusCode::InternalServerError, e))?;
                if price >= u128::from(page.start_price)
                    && price <= u128::from(page.end_price)
                    && (page.stage == 255 || page.stage == stage)
                {
                    tokens.push(str_id.clone());
                }
            }
        } else if page.sort != 1 && page.sort != 2 && page.sort != 3 {
            // if we don't have filters and we have default sort by id
            return create_dragons(&app_state.market_id_list, &req);
        } else {
            // if we don't have filters and we have some sort
            tokens = app_state.market_id_list.clone();
        }
        match page.sort {
            1 => {
                let some_cmp = |a: &String, b: &String| {
                    app_state
                        .all_id_rarity
                        .get(a)
                        .unwrap_or(&0)
                        .cmp(app_state.all_id_rarity.get(b).unwrap_or(&0))
                };
                // TODO check sort function
                tokens.sort_unstable_by(some_cmp);
            }
            2 => {
                let some_cmp = |a: &String, b: &String| {
                    app_state
                        .all_id_strength
                        .get(a)
                        .unwrap_or(&0)
                        .cmp(app_state.all_id_strength.get(b).unwrap_or(&0))
                };
                // TODO check sort function
                tokens.sort_unstable_by(some_cmp);
            }
            3 => {
                let some_cmp = |a: &String, b: &String| {
                    get_price(a, m_prices)
                        .unwrap_or(0)
                        .cmp(&get_price(b, m_prices).unwrap_or(0))
                };
                // TODO check sort function
                tokens.sort_unstable_by(some_cmp);
            }
            _ => {}
        }
        create_dragons(&tokens, &req)
    } else {
        let tokens = match app_state.market_owned_id.get(&page.owner) {
            Some(result) => result,
            None => return Ok(create_response(vec![], &page, 0)?.into()),
        };
        create_dragons(tokens, &req)
    }
}

// GET /api/v1/dragons [?limit=1&offset=1&owner=0x...]
pub async fn get_dragons(req: Request<AppState>) -> tide::Result {
    let app_state = req.state();
    get_data(&app_state.all_id_list, &app_state.all_owned_id, &req)
}
fn get_price(str_id: &str, h_m: &HashMap<String, String>) -> Result<u128, tide::Error> {
    h_m.get(str_id)
        .ok_or_else(internal_error)?
        .parse::<u128>()
        .map_err(|e| tide::Error::new(StatusCode::InternalServerError, e))
}
fn get_data(
    all_tokens: &[String],
    owned_id: &HashMap<String, Vec<String>>,
    req: &Request<AppState>,
) -> tide::Result {
    let page: Page = req.query()?;
    if page.owner.is_empty() {
        create_dragons(all_tokens, req)
    } else {
        let tokens = match owned_id.get(&page.owner) {
            Some(result) => result,
            None => return Ok(create_response(vec![], &page, 0)?.into()),
        };
        create_dragons(tokens, req)
    }
}
fn create_dragons(tokens: &[String], req: &Request<AppState>) -> tide::Result {
    let page: Page = req.query()?;
    if page.limit == 0 {
        return Ok(create_error(
            StatusCode::BadRequest,
            "Limit cannot be zero.",
        ));
    }
    let app_state = req.state();
    let real_end = tokens.len();
    match calc_indexes(&page, real_end) {
        Some((start, end)) => {
            let items = collect_items(&tokens[start..end], app_state)?;
            Ok(create_response(items, &page, real_end)?.into())
        }
        None => Ok(create_error(StatusCode::BadRequest, "Offset is too big.")),
    }
}
fn create_error(code: tide::StatusCode, err_text: &str) -> tide::Response {
    let mut response = Response::new(code);
    response.set_body(format!(
        "{{\"success\":false,\"error\":{{\"code\":{},\"message\":\"{}\"}}}}",
        code, err_text
    ));
    response
}
fn calc_indexes(page: &Page, real_end: usize) -> Option<(usize, usize)> {
    let start = page.offset * page.limit;
    if real_end == 0 {
        return Some((0, 0));
    }
    if start >= real_end {
        return None;
    }
    Some((start, std::cmp::min(start + page.limit, real_end)))
}
fn collect_items<'a>(
    tokens: &'a [String],
    app_s: &'a AppState,
) -> Result<Vec<Item<'a>>, tide::Error> {
    let mut items = Vec::with_capacity(tokens.len());
    for str_id in tokens {
        items.push(create_item(str_id, app_s)?);
    }
    Ok(items)
}
fn create_item<'a>(str_id: &'a str, app_s: &'a AppState) -> Result<Item<'a>, tide::Error> {
    let ms = &app_s.main_state;
    Ok(Item {
        id: str_id,
        owner: get_element(&app_s.all_id_owner, str_id)?,
        url: get_element(&ms.token_uris, str_id)?,
        gen_image: get_element(&ms.token_gen_image, str_id)?,
        gen_fight: get_element(&ms.token_gen_battle, str_id)?,
        stage: get_element(&ms.token_stage, str_id)?
            .parse()
            .map_err(|e| tide::Error::new(StatusCode::InternalServerError, e))?,
        rarity: *get_element(&app_s.all_id_rarity, str_id)?,
        // TODO add real statistics
        fight_win: 666,
        fight_lose: 13,
        actions: collect_actions(str_id, app_s),
        // TODO write true parents
        parents: [].to_vec(),
        // TODO write true children
        children: [].to_vec(),
    })
}
fn collect_actions<'a>(str_id: &str, app_s: &'a AppState) -> Vec<(u8, &'a str)> {
    let mut result: Vec<(u8, &str)> = Vec::with_capacity(3);
    if let Some(x) = app_s.battle_id_price.get(str_id) {
        result.push((1, x)); // 1 is eq Battle
    }
    if let Some(x) = app_s.breed_id_price.get(str_id) {
        result.push((2, x)); // 2 is eq Breed
    }
    if let Some(x) = app_s.market_id_price.get(str_id) {
        result.push((3, x)); // 3 is eq Market with price
    }
    if let Some(x) = app_s.market_id_order.get(str_id) {
        result.push((4, x)); // 4 is eq Market with order_id
    }
    result
}
fn get_element<'a, T>(h_m: &'a HashMap<String, T>, str_id: &str) -> Result<&'a T, tide::Error> {
    h_m.get(str_id).ok_or_else(internal_error)
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
