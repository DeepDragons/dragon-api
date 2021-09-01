use std::collections::HashMap;
use tide::{Request, Response, StatusCode};
use crate::state::{AppState, RI};
use crate::web_api::{Item, OkResponse, Page, Pagination};

// GET /api/v1/dragons/:id
pub async fn get_dragon_by_id(req: Request<AppState>) -> tide::Result {
    let str_id = req.param("id")?;
    let app_state = req.state();
    match app_state.main_state.token_stage.get(str_id) {
        Some(_) => {
            let mut page = Page::default();
            page.limit = 1;
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
    get_data(&app_state.market_id_list, &app_state.market_owned_id, &req)
}

// GET /api/v1/dragons [?limit=1&offset=1&owner=0x...]
pub async fn get_dragons(req: Request<AppState>) -> tide::Result {
    let app_state = req.state();
    get_data(&app_state.all_id_list, &app_state.all_owned_id, &req)
}
fn get_data(
    all_tokens: &[String],
    owned_id: &HashMap<String, Vec<String>>,
    req: &Request<AppState>,
) -> tide::Result {
    let page: Page = req.query()?;
    if page.limit == 0 {
        return Ok(create_error(
            StatusCode::BadRequest,
            "Limit cannot be zero.",
        ));
    }
    let app_state = req.state();
    if page.owner.is_empty() {
        let real_end = all_tokens.len();
        match calc_indexes(&page, real_end) {
            Some((start, end)) => {
                let items = collect_items(&all_tokens[start..end], app_state)?;
                Ok(create_response(items, &page, real_end)?.into())
            }
            None => Ok(create_error(StatusCode::BadRequest, "Offset is too big.")),
        }
    } else {
        let tokens = match owned_id.get(&page.owner) {
            Some(result) => result,
            None => {
                return Ok(create_response(vec![], &page, 0)?.into())
            }
        };
        let real_end = tokens.len();
        match calc_indexes(&page, real_end) {
            Some((start, end)) => {
                let items = collect_items(&tokens[start..end], app_state)?;
                Ok(create_response(items, &page, real_end)?.into())
            }
            None => Ok(create_error(StatusCode::BadRequest, "Offset is too big.")),
        }
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
    let gen_image = get_element(&ms.token_gen_image, str_id)?;
    Ok(Item {
        id: str_id,
        owner: get_element(&app_s.all_id_owner, str_id)?,
        url: get_element(&ms.token_uris, str_id)?,
        gen_image,
        gen_fight: get_element(&ms.token_gen_battle, str_id)?,
        stage: get_element(&ms.token_stage, str_id)?
            .parse()
            .map_err(|e| tide::Error::new(StatusCode::InternalServerError, e))?,
        rarity: calc_rarity(gen_image)?,
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
    if let Some(x) = app_s.market_id_price_order.get(str_id) {
        result.push((3, &x.0)); // 3 is eq Market with price
        result.push((4, &x.1)); // 4 is eq Market with order_id
    }
    result
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
// https://github.com/DeepDragons/dragon-zil/blob/master/src/mixins/utils.js#L50
// most of visual gens have 2 parts - type (0-9) and color(0-4)
// head have 1 digit
// claws have 1 digit
// Color Scheme have 3 digits
// MutagenImutable have 3 digits
// e.g. 777 03 03 43 31 14 33 44 11 73 1 4 110 158
//      777 01 64 02 94 03 04 40 24 11 4 1 076 065
// Aura-12   Horns-11   Scales-10   Spots-9   Tail-8   Wings-7
// Spins-6   Body-5   Eyes-4   Head-3   Claws-2   Color Scheme-1   MutagenImutable-0
/* https://github.com/DeepDragons/dragon-zil/blob/master/src/mixins/utils.js#L1
 * None      0
 * Common    1
 * Uncommon  2
 * Rare      3
 * Mythical  4
 * Legendary 5
 * Immortal  6
 * Arcana    7
 * Ancient   8
 */
fn calc_rarity(gens: &str) -> Result<u8, tide::Error> {
    let gen_to_index = |a, b| {
        gens[a..b]
            .parse::<usize>()
            .map_err(|e| tide::Error::new(StatusCode::InternalServerError, e))
    };
    // https://github.com/DeepDragons/dragon-zil/blob/master/src/mixins/utils.js#L372
    let rarity_sum = RI.aura[gen_to_index(3, 4)?]
        + RI.horns[gen_to_index(5, 6)?]
        + RI.scales[gen_to_index(7, 8)?]
        + RI.spots[gen_to_index(9, 10)?]
        + RI.tail[gen_to_index(11, 12)?]
        + RI.wings[gen_to_index(13, 14)?]
        + RI.body[gen_to_index(17, 18)?]
        + RI.eyes[gen_to_index(19, 20)?]
        + RI.head[gen_to_index(21, 22)?];
    Ok(match rarity_sum {
        0..=15 => 0,  //TODO check it?
        16..=23 => 1, // Uncommon
        24..=31 => 2, // Rare
        32..=39 => 3, // Mythical
        40..=47 => 4, // Legendary
        48..=55 => 5, // Imortal
        56..=63 => 6, // Arcana
        _ => 7,       // Ancient
    })
}
