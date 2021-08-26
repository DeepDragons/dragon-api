use std::collections::HashMap;
use crate::state::*;

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
pub async fn create_app_state() -> AppState {
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
    let mut market_id_price_order: HashMap<String, (String, String)> = HashMap::with_capacity(market_len);
    let mut market_id_list: Vec<String> = Vec::with_capacity(market_len);
    let mut market_owned_id: HashMap<String, Vec<String>> = HashMap::new();
    // TODO Add with_capacity
    let mut all_id_owner = HashMap::new();
    for i in market_resp.result.orderbook.into_values() {
        let (owner, price, id, order_id) = (
            i.arguments[0].clone(),
            i.arguments[1].clone(),
            i.arguments[2].clone(),
            i.arguments[3].clone(),
        );
        all_id_owner.insert(id.clone(), owner.clone());
        market_id_price_order.insert(id.clone(), (price, order_id));
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
    AppState {
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
        market_id_price_order,
        market_owned_id,
    }
}
