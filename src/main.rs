extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod datastruct;
use datastruct::*;
/*
{
    AppState, BreedItem, Item, MainState, /* MarketItem, */ OkResponse, OrderState, Page, Pagination, Resp, ShortItem, WaitState,
    BATTLESTATE, BREEDSTATE, DEFAULT_API_URL, MAINSTATE, MARKETSTATE, RI, URL,
}; */
use std::collections::HashMap;
use tide::{http::headers::HeaderValue, Request, Response, StatusCode};

#[tokio::main]
async fn main() -> tide::Result<()> {
    dotenv::dotenv().ok();
    let text = do_request(BREEDSTATE).await;
    let breed_resp: Resp<WaitState<BreedItem>> = serde_json::from_str(&text).expect("breed state");
    let mut breed_id_list: Vec<String> = breed_resp.result.waiting_list.keys().cloned().collect();
    let mut breed_owned_id: HashMap<String, Vec<String>> = HashMap::new();
    let mut breed_id_price = HashMap::with_capacity(breed_id_list.len());
    for (id, breed_item) in breed_resp.result.waiting_list {
        breed_id_price.insert(id.clone(), breed_item.arguments[0].clone());
        match breed_owned_id.get_mut(&breed_item.arguments[1]) {
            Some(x) => x.push(id),
            None => {
                breed_owned_id.insert(breed_item.arguments[1].clone(), vec![id]);
            }
        }
    }
    let text = do_request(MARKETSTATE).await;
    let market_resp: Resp<OrderState> = serde_json::from_str(&text).expect("market state");
    let market_len = market_resp.result.orderbook.len();
    let mut market_id_price: HashMap<String, String> = HashMap::with_capacity(market_len);
    let mut market_id_list: Vec<String> = Vec::with_capacity(market_len);
    let mut market_owned_id: HashMap<String, Vec<String>> = HashMap::new();
    // TODO Add with_capacity
    let mut all_id_owner = HashMap::new();
    for i in market_resp.result.orderbook.into_values() {
        let (owner, price, id) = (
            i.arguments[0].clone(),
            i.arguments[1].clone(),
            i.arguments[2].clone(),
        );
        all_id_owner.insert(id.clone(), owner.clone());
        market_id_price.insert(id.clone(), price);
        market_id_list.push(id.clone());
        match market_owned_id.get_mut(&owner) {
            Some(x) => x.push(id),
            None => {
                market_owned_id.insert(owner, vec![id]);
            }
        };
    }
    let text = do_request(MAINSTATE).await;
    // TODO error handling
    let main_resp: Resp<MainState> = serde_json::from_str(&text).expect("main state");
    let text = do_request(BATTLESTATE).await;
    // TODO error handling
    let battle_resp: Resp<WaitState<String>> = serde_json::from_str(&text).expect("battle state");
    drop(text);
    let mut battle_id_list: Vec<String> = battle_resp.result.waiting_list.keys().cloned().collect();
    let mut battle_owned_id: HashMap<String, Vec<String>> = HashMap::new();
    for id in battle_resp.result.waiting_list.keys() {
        // TODO Error handling
        let owner = main_resp.result.token_owners.get(id).unwrap();
        match battle_owned_id.get_mut(owner) {
            Some(x) => x.push(id.to_string()),
            None => {
                battle_owned_id.insert(owner.to_string(), vec![id.to_string()]);
            }
        }
    }
    for (id, owner) in &main_resp.result.token_owners {
        match all_id_owner.get(id) {
            None => {
                all_id_owner.insert(id.to_string(), owner.to_string());
            }
            Some(_) => {}
        }
    }
    // TODO error handling
    let parse_cmp =
        |a: &String, b: &String| a.parse::<u128>().unwrap().cmp(&b.parse::<u128>().unwrap());
    let mut all_owned_id = HashMap::with_capacity(main_resp.result.tokens_owner_stage.len());
    for (key, val) in &main_resp.result.tokens_owner_stage {
        let mut tokens: Vec<String> = val.keys().cloned().collect();
        if let Some(x) = market_owned_id.get_mut(key) {
            tokens.extend_from_slice(x);
        }
        tokens.sort_unstable_by(parse_cmp);
        all_owned_id.insert(key.to_string(), tokens);
    }
    let mut all_id_list: Vec<String> = main_resp.result.token_stage.keys().cloned().collect();
    all_id_list.sort_unstable_by(parse_cmp);
    market_id_list.sort_unstable_by(parse_cmp);
    battle_id_list.sort_unstable_by(parse_cmp);
    breed_id_list.sort_unstable_by(parse_cmp);
    let app_state = AppState {
        all_id_list,
        all_owned_id,
        all_id_owner,
        main_state: main_resp.result,
        battle_id_list,
        battle_id_price: battle_resp.result.waiting_list,
        battle_owned_id,
        breed_id_list,
        breed_id_price,
        breed_owned_id,
        market_id_list,
        market_id_price,
        market_owned_id,
    };
    let api_url = match std::env::var("API_URL") {
        Ok(val) => val,
        Err(_) => String::from(DEFAULT_API_URL),
    };
    println!("Dragons backend is starting on {}", api_url);
    let mut app = tide::with_state(app_state);
    #[cfg(debug_assertions)]
    {
        println!("Debug mode, CORS allow \"*\"");
        let cors_debug = tide::security::CorsMiddleware::new()
            .allow_methods("GET".parse::<HeaderValue>().unwrap())
            .allow_origin(tide::security::Origin::from("*"))
            .allow_credentials(false);
        app.with(cors_debug);
    }
    app.at("/api/v1/dragons").get(get_dragons);
    app.at("/api/v1/dragons/:id").get(get_dragon_by_id);
    app.at("/api/v1/market").get(get_from_market);
    app.at("/api/v1/battle").get(get_from_battle);
    app.at("/api/v1/breed").get(get_from_breed);
    app.listen(api_url).await?;
    Ok(())
}
async fn do_request(body: &'static str) -> String {
    let client = reqwest::Client::new();
    let response = client
        .post(URL)
        .body(body)
        .send()
        .await
        // TODO error handling
        .expect("Post request failed");
    // TODO check server error code and error handling
    let text = response.text().await.expect("1");
    // TODO remove debug println and text var...
    if text.len() < 3000 {
        println!("{}", text);
    }
    text
}
// GET /api/v1/dragons/:id
async fn get_dragon_by_id(req: Request<AppState>) -> tide::Result {
    let str_id = req.param("id")?;
    let app_state = req.state();
    match app_state.main_state.token_stage.get(str_id) {
        Some(_) => {
            let page: Page = Page {
                limit: 1,
                offset: 0,
                owner: "".to_string(),
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
async fn get_from_battle(req: Request<AppState>) -> tide::Result {
    let app_state = req.state();
    get_data(&app_state.battle_id_list, &app_state.battle_owned_id, &req)
}
// GET /api/v1/breed
async fn get_from_breed(req: Request<AppState>) -> tide::Result {
    let app_state = req.state();
    get_data(&app_state.breed_id_list, &app_state.breed_owned_id, &req)
}
// GET /api/v1/market
async fn get_from_market(req: Request<AppState>) -> tide::Result {
    let app_state = req.state();
    get_data(&app_state.market_id_list, &app_state.market_owned_id, &req)
}
// GET /api/v1/dragons [?limit=1&offset=1&owner=0x...]
async fn get_dragons(req: Request<AppState>) -> tide::Result {
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
                return Ok(create_error(
                    StatusCode::NotFound,
                    &format!("Owner {} is not found.", page.owner),
                ));
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
    if let Some(x) = app_s.market_id_price.get(str_id) {
        result.push((3, x)); // 3 is eq Market
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
